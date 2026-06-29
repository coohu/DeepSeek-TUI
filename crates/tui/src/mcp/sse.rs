use std::collections::VecDeque;
use std::time::Duration;

use anyhow::{Context, Result};

use super::headers::{apply_safe_custom_headers, with_default_mcp_http_headers};
use super::{
    ERROR_BODY_PREVIEW_BYTES, McpHttpAuth, McpTransport, bounded_body_excerpt,
    find_sse_event_separator, is_mcp_stale_session_body, mask_url_secrets, sse_field_value,
};

pub(super) struct SseTransport {
    pub(super) client: reqwest::Client,
    pub(super) base_url: String,
    pub(super) auth: McpHttpAuth,
    pub(super) endpoint_url: Option<String>,
    pub(super) receiver: tokio::sync::mpsc::UnboundedReceiver<SseInbound>,
    pub(super) pending_messages: VecDeque<Vec<u8>>,
    #[allow(dead_code)]
    pub(super) sse_task: tokio::task::JoinHandle<()>,
}

pub(super) enum SseInbound {
    Endpoint(String),
    Message(Vec<u8>),
}

impl SseTransport {
    pub(super) async fn connect(
        client: reqwest::Client,
        url: String,
        auth: McpHttpAuth,
        cancel_token: tokio_util::sync::CancellationToken,
        endpoint_timeout: Duration,
    ) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let client_clone = client.clone();
        let url_clone = url.clone();
        let auth_clone = auth.clone();
        let wait_cancel_token = cancel_token.clone();

        let sse_task = tokio::spawn(async move {
            if cancel_token.is_cancelled() {
                return;
            }
            use futures_util::FutureExt;
            let result = std::panic::AssertUnwindSafe(Self::run_sse_loop(
                client_clone,
                url_clone,
                auth_clone,
                tx,
                cancel_token,
            ))
            .catch_unwind()
            .await;
            match result {
                Ok(res) => {
                    if let Err(e) = res {
                        tracing::error!("SSE loop error: {}", e);
                    }
                }
                Err(panic_err) => {
                    if let Some(msg) = panic_err.downcast_ref::<&str>() {
                        tracing::error!("SSE loop panicked: {}", msg);
                    } else if let Some(msg) = panic_err.downcast_ref::<String>() {
                        tracing::error!("SSE loop panicked: {}", msg);
                    } else {
                        tracing::error!("SSE loop panicked with unknown error");
                    }
                }
            }
        });

        let mut transport = Self {
            client,
            base_url: url,
            auth,
            endpoint_url: None,
            receiver: rx,
            pending_messages: VecDeque::new(),
            sse_task,
        };
        transport
            .wait_for_endpoint(&wait_cancel_token, endpoint_timeout)
            .await?;
        Ok(transport)
    }

    async fn run_sse_loop(
        client: reqwest::Client,
        url: String,
        auth: McpHttpAuth,
        tx: tokio::sync::mpsc::UnboundedSender<SseInbound>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<()> {
        let headers = auth.resolved_headers().await?;
        let response = apply_safe_custom_headers(
            with_default_mcp_http_headers(client.get(&url), false),
            &headers,
        )
        .send()
        .await
        .with_context(|| {
            format!(
                "MCP SSE connect failed (transport=http url={})",
                mask_url_secrets(&url),
            )
        })?;
        let status = response.status();
        if !status.is_success() {
            let body_excerpt = bounded_body_excerpt(response, ERROR_BODY_PREVIEW_BYTES).await;
            anyhow::bail!(
                "MCP SSE rejected (transport=http url={} status={}): {}",
                mask_url_secrets(&url),
                status,
                body_excerpt,
            );
        }

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        let mut buffer = String::new();

        loop {
            if cancel_token.is_cancelled() {
                tracing::debug!("SSE loop cancelled");
                break;
            }
            let item = tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::debug!("SSE loop shutting down");
                    break;
                }
                item = stream.next() => {
                    match item {
                        Some(i) => i,
                        None => break,
                    }
                }
            };
            let chunk = item?;
            let s = String::from_utf8_lossy(&chunk);
            buffer.push_str(&s);

            while let Some((pos, separator_len)) = find_sse_event_separator(&buffer) {
                let event_block = buffer[..pos].to_string();
                buffer = buffer[pos + separator_len..].to_string();

                let mut event_type = "message";
                let mut data = String::new();

                for line in event_block.lines() {
                    if let Some(value) = sse_field_value(line, "event:") {
                        event_type = value;
                    } else if let Some(value) = sse_field_value(line, "data:") {
                        if !data.is_empty() {
                            data.push('\n');
                        }
                        data.push_str(value);
                    }
                }

                match event_type {
                    "endpoint" => {
                        let _ = tx.send(SseInbound::Endpoint(data));
                    }
                    "message" if !data.trim().is_empty() => {
                        let _ = tx.send(SseInbound::Message(data.into_bytes()));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn wait_for_endpoint(
        &mut self,
        cancel_token: &tokio_util::sync::CancellationToken,
        endpoint_timeout: Duration,
    ) -> Result<()> {
        let timeout = tokio::time::sleep(endpoint_timeout);
        tokio::pin!(timeout);

        loop {
            let msg = tokio::select! {
                _ = cancel_token.cancelled() => {
                    anyhow::bail!("SSE transport cancelled before endpoint was discovered");
                }
                _ = &mut timeout => {
                    anyhow::bail!(
                        "SSE endpoint not received within {}ms",
                        endpoint_timeout.as_millis()
                    );
                }
                msg = self.receiver.recv() => {
                    msg.context("SSE transport closed before endpoint was discovered")?
                }
            };

            match msg {
                SseInbound::Endpoint(endpoint) => {
                    self.store_endpoint(&endpoint)?;
                    return Ok(());
                }
                SseInbound::Message(msg) => self.pending_messages.push_back(msg),
            }
        }
    }

    fn store_endpoint(&mut self, endpoint: &str) -> Result<()> {
        self.endpoint_url = Some(Self::resolve_endpoint_url(&self.base_url, endpoint)?);
        Ok(())
    }

    fn resolve_endpoint_url(base_url: &str, endpoint_url: &str) -> Result<String> {
        if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://") {
            return Ok(endpoint_url.to_string());
        }
        let base = reqwest::Url::parse(base_url)?;
        let joined = base.join(endpoint_url)?;
        Ok(joined.to_string())
    }
}

#[async_trait::async_trait]
impl McpTransport for SseTransport {
    async fn send(&mut self, msg: Vec<u8>) -> Result<()> {
        let endpoint = self
            .endpoint_url
            .as_ref()
            .context("SSE endpoint not yet discovered")?
            .clone();
        let headers = self.auth.resolved_headers().await?;
        let response = apply_safe_custom_headers(
            with_default_mcp_http_headers(self.client.post(&endpoint), true),
            &headers,
        )
        .body(msg)
        .send()
        .await
        .with_context(|| {
            format!(
                "MCP SSE POST send failed (transport=sse endpoint={})",
                mask_url_secrets(&endpoint)
            )
        })?;
        let status = response.status();
        if !status.is_success() {
            let body_excerpt = bounded_body_excerpt(response, ERROR_BODY_PREVIEW_BYTES).await;
            if is_mcp_stale_session_body(&body_excerpt) {
                anyhow::bail!(
                    "MCP session expired (transport=sse endpoint={} status={}): {}",
                    mask_url_secrets(&endpoint),
                    status,
                    body_excerpt
                );
            }
            anyhow::bail!(
                "MCP SSE POST rejected (transport=sse endpoint={} status={}): {}",
                mask_url_secrets(&endpoint),
                status,
                body_excerpt
            );
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>> {
        loop {
            if let Some(msg) = self.pending_messages.pop_front() {
                return Ok(msg);
            }

            match self.receiver.recv().await.context("SSE transport closed")? {
                SseInbound::Endpoint(endpoint) => {
                    self.store_endpoint(&endpoint)?;
                }
                SseInbound::Message(msg) => return Ok(msg),
            }
        }
    }
}

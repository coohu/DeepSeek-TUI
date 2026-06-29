# ACP Registry Submission Prep

Prepared for #3192. The external registry submission is now open as
`agentclientprotocol/registry#411`.

## Upstream Registry Requirements

Checked against `agentclientprotocol/registry` on 2026-06-27:

- New entries live in a directory whose name matches the `id` field.
- Each entry needs `agent.json` plus a required `icon.svg`.
- `agent.json` requires `id`, `name`, `version`, `description`, and at least one
  `distribution` method.
- Supported distribution methods are `binary`, `npx`, and `uvx`.
- Package and binary versions must match the entry version, and `latest` is not
  allowed.
- Binary platform ids are `darwin-aarch64`, `darwin-x86_64`, `linux-aarch64`,
  `linux-x86_64`, `windows-aarch64`, and `windows-x86_64`.
- Icons must be 16x16 SVG, square, monochrome, and use `currentColor`.
- Registry CI runs an auth check: `initialize` must return at least one
  `authMethods` entry with `type: "agent"` or `type: "terminal"`.

Sources for the external PR author:

- https://github.com/agentclientprotocol/registry
- https://github.com/agentclientprotocol/registry/blob/main/FORMAT.md
- https://github.com/agentclientprotocol/registry/blob/main/CONTRIBUTING.md
- https://github.com/agentclientprotocol/registry/blob/main/AUTHENTICATION.md
- https://github.com/agentclientprotocol/registry/blob/main/agent.schema.json

## Local ACP Readiness Audit

DeepSeek already exposes ACP through `deepseek serve --acp`.

Implemented locally:

- `crates/tui/src/main.rs` accepts `serve --acp` and dispatches to the ACP
  server.
- `crates/tui/src/acp_server.rs` implements JSON-RPC 2.0 over newline-delimited
  stdio.
- `initialize` advertises:
  - `agentInfo.name = "deepseek"`
  - `agentInfo.title = "deepseek"`
  - `agentInfo.version = env!("CARGO_PKG_VERSION")`
  - `promptCapabilities.embeddedContext = true`
  - `loadSession = false`
  - `mcpCapabilities.http = false`
  - `mcpCapabilities.sse = false`
  - `authMethods` with terminal auth: `auth set --provider <provider>`
- `session/new` creates an in-memory session with a cwd.
- `session/prompt` accepts string prompts plus text/resource/resource_link
  blocks and routes through the configured DeepSeek client.
- `session/prompt` **streams**: each provider text delta is emitted as a
  `session/update` agent_message_chunk as it arrives, then the prompt returns
  `stopReason: "end_turn"` (instead of buffering the whole turn and sending one
  chunk at the end).
- The stream is consumed concurrently with the input reader, so a
  `session/cancel` for the same session interrupts the turn mid-stream and the
  prompt returns `stopReason: "cancelled"`; dropping the stream aborts the
  underlying provider connection. A no-prompt `session/cancel` stays an
  idempotent `null` no-op. The turn is single-flight: another request arriving
  mid-turn gets a clear "prompt in progress" error instead of being silently
  dropped.

Known limitations to state clearly:

- The adapter is baseline ACP, not the full interactive TUI/runtime surface.
- Streaming covers text deltas only; thinking/tool/server-tool deltas are not
  surfaced over ACP (ACP baseline here is text-only, `tools: None`).
- ACP does not expose shell tools, file-write tools, checkpoint replay, session
  loading, or the HTTP/SSE runtime API.
- Registry submission should be gated on a local run of the upstream registry
  auth-check before opening the external PR. That check passed locally before
  `agentclientprotocol/registry#411` was opened.

The submitted registry PR uses the `npx` distribution because
`deepseek@0.8.65` is already published and the npm wrapper handles platform
selection, checksums, mirrors, and glibc preflight.

## External Registry Files

Create this directory in `agentclientprotocol/registry`:

```text
deepseek/
  agent.json
  icon.svg
```

Use a concrete published version. Do not use `@latest`.

### `deepseek/agent.json`

```json
{
  "id": "deepseek",
  "name": "DeepSeek",
  "version": "0.8.65",
  "description": "Provider-agnostic terminal coding agent with first-class DeepSeek support.",
  "repository": "https://github.com/coohu/deepseek-tui",
  "website": "https://github.com/coohu/deepseek-tui/blob/main/docs/RUNTIME_API.md#acp-stdio-adapter-deepseek-serve---acp",
  "authors": ["Hunter Bown"],
  "license": "MIT",
  "distribution": {
    "npx": {
      "package": "deepseek@0.8.65",
      "args": ["serve", "--acp"]
    }
  }
}
```

### `deepseek/icon.svg`

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" fill="none">
  <path d="M2 9.5c0-3.3 2.7-6 6-6h4.5v2H8a4 4 0 0 0-4 4v.5h7.5a2.5 2.5 0 0 0 2.4-1.8l.6-2.2H16l-.7 2.7A4 4 0 0 1 11.5 12H4.2A3 3 0 0 1 2 9.5Z" fill="currentColor"/>
  <path d="M5 7h1.5v1.5H5V7Zm3 0h1.5v1.5H8V7Z" fill="currentColor"/>
</svg>
```

## External PR Draft

Title:

```text
Add DeepSeek ACP agent
```

Body:

```text
Adds DeepSeek to the ACP registry.

DeepSeek is a provider-agnostic terminal coding agent with first-class
DeepSeek support. The submitted distribution uses the published npm package and
runs `deepseek serve --acp`.

Local readiness checked in coohu/deepseek-tui:
- ACP stdio adapter exists at `deepseek serve --acp`.
- `initialize` returns terminal auth via `auth set --provider <provider>`.
- `session/new`, `session/prompt`, and `session/cancel` are implemented.
- `session/prompt` streams provider text deltas as `session/update` chunks.
- The adapter is intentionally baseline: no ACP shell/file tools, no session
  load, and no full runtime API through ACP.

Version: 0.8.65
```

## Pre-Submission Checklist

- Confirm `deepseek@0.8.65` is published to npm: done on 2026-06-27.
- Run the upstream registry validator: done on 2026-06-27 with
  `python3 .github/workflows/verify_agents.py --auth-check --agent deepseek --verbose`;
  result was `Auth OK: deepseek-terminal-auth(terminal)`.
- Verify `npx -y deepseek@0.8.65 serve --acp` returns `authMethods` from
  `initialize`: done on 2026-06-27.
- Keep the external PR body explicit that ACP support is baseline and does not
  imply the full TUI/runtime API is available inside ACP: done in
  `agentclientprotocol/registry#411`.

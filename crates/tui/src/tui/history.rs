//! TUI rendering helpers for chat history and tool output.

use std::path::{Path, PathBuf};
use std::time::Instant;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

use crate::deepseek_theme::active_theme;
use crate::models::{ContentBlock, Message};
use crate::palette;
use crate::tools::plan::PlanSnapshot;
use crate::tools::review::ReviewOutput;
use crate::tui::app::TranscriptSpacing;
use crate::tui::diff_render;
use crate::tui::markdown_render;
use crate::tui::ui_text::{CopyLineSeparator, truncate_line_to_width};

mod archived_context;
mod checklist;
mod plan;
mod tool_run;

use archived_context::{parse_archived_context, render_archived_context};
use checklist::{
    is_checklist_tool_name, parse_checklist_snapshot, parse_update_prefix, render_checklist_card,
    render_checklist_change_card,
};

#[cfg(test)]
use checklist::{ChecklistChange, ChecklistItemSnapshot, ChecklistSnapshot};

pub use plan::PlanUpdateCell;
pub use tool_run::{
    ToolRun, ToolRunActivitySummary, detect_tool_runs, detect_tool_runs_from_slices,
    tool_run_summary,
};

// === Constants ===

use std::process::Command;
const TOOL_COMMAND_LINE_LIMIT: usize = 3;
const TOOL_OUTPUT_LINE_LIMIT: usize = 6;
const TOOL_TEXT_LIMIT: usize = 300;
const TOOL_HEADER_SUMMARY_LIMIT: usize = 56;
const TOOL_OUTPUT_HEAD_LINES: usize = 2;
const TOOL_OUTPUT_TAIL_LINES: usize = 2;
const TOOL_RUNNING_SYMBOLS: [&str; 10] = [
    "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}", "\u{2827}",
    "\u{2807}", "\u{280F}",
];
/// Per-glyph cadence: 100 ms — visibly alive without exceeding the redraw cap.
const TOOL_STATUS_SYMBOL_MS: u64 = 100;
/// Visual marker for the user role at the start of their message line. Solid
/// vertical bar — no animation; user input is a finished thing.
const USER_GLYPH: &str = "\u{258E}"; // ▎
/// Visual marker for the assistant role. Solid bullet that pulses at 2s
/// cycle while the response is streaming, holds full brightness when idle.
const ASSISTANT_GLYPH: &str = "\u{25CF}"; // ●
/// Transcript body left rail. Solid 1/8 block (`▏`) followed by a space —
/// used as a visual left-margin anchor for continuation lines, tool-card
/// detail rows, and affordance lines. Dimmed so it guides the eye without
/// competing with content.
const TRANSCRIPT_RAIL: &str = "\u{258F} "; // ▏ + space
/// Reasoning header opener. Replaces the spinner glyph on thinking cells —
/// reasoning is a slow exhale, not a tool spin.
const REASONING_OPENER: &str = "\u{2026}"; // …
/// Reasoning body left rail. Dashed (`╎`) instead of the solid `▏` block to
/// visually separate reasoning from message body and tool output.
const REASONING_RAIL: &str = "\u{254E} "; // ╎ + space
/// Trailing-line cursor on streaming reasoning. Anchored to the live colour
/// so the user sees where new tokens land.
const REASONING_CURSOR: &str = "\u{258E}"; // ▎
const TOOL_CARD_SUMMARY_LINES: usize = 4;
const THINKING_SUMMARY_LINE_LIMIT: usize = 4;
const THINKING_COMPLETED_PREVIEW_LINE_LIMIT: usize = 6;
const THINKING_STREAMING_PREVIEW_LINE_LIMIT: usize = 8;
const TOOL_DONE_SYMBOL: &str = "•";
const TOOL_FAILED_SYMBOL: &str = "•";

/// Render mode controlling whether tool/thinking cells render their compact
/// "live" form (with caps and collapsed reasoning) or their full transcript
/// form (uncapped, suitable for the pager / clipboard / message export).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Live in-stream view: thinking is collapsed to a summary, tool output is
    /// truncated with a "Alt+V for details" affordance.
    Live,
    /// Full transcript view: every line of reasoning and tool output is
    /// emitted, no caps, no affordance.
    Transcript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThinkingVisualState {
    Live,
    Done,
    Idle,
}

// === History Cells ===

/// Renderable history cell for user/assistant/system entries.
#[derive(Debug, Clone)]
pub enum HistoryCell {
    User {
        content: String,
    },
    Assistant {
        content: String,
        streaming: bool,
    },
    System {
        content: String,
    },
    /// Categorized engine-error cell. Severity drives the label glyph + color
    /// (red for `Error`/`Critical`, amber for `Warning`, dim for `Info`) so
    /// the user can prioritize at a glance.
    Error {
        message: String,
        severity: crate::error_taxonomy::ErrorSeverity,
    },
    Thinking {
        content: String,
        streaming: bool,
        duration_secs: Option<f32>,
    },
    /// An `<archived_context>` seam block produced by the Flash seam manager
    /// (issue #159). Rendered dimmed/italic with a level + range label so
    /// the user can see at a glance where context seams exist.
    ArchivedContext {
        /// Seam level (1, 2, 3, or 0 for cycle-level).
        level: u8,
        /// Message range covered (e.g. "msg 0-128").
        range: String,
        /// Token estimate string (e.g. "~2500").
        tokens: String,
        /// Density label (e.g. "~2,500 tokens").
        density: String,
        /// Model that produced the summary.
        model: String,
        /// RFC 3339 timestamp.
        timestamp: String,
        /// The summary text content.
        summary: String,
    },
    Tool(ToolCell),
    /// Live in-transcript card for sub-agent activity (issue #128). Owns
    /// either a single `DelegateCard` or a multi-worker `FanoutCard`; the
    /// UI re-binds it from the mailbox stream as envelopes arrive.
    SubAgent(SubAgentCell),
}

/// In-transcript sub-agent cell — either a single delegate or a fanout.
/// State mutates over the turn as mailbox envelopes are drained.
#[derive(Debug, Clone)]
pub enum SubAgentCell {
    Delegate(crate::tui::widgets::agent_card::DelegateCard),
    Fanout(crate::tui::widgets::agent_card::FanoutCard),
}

impl SubAgentCell {
    pub fn lines(&self, width: u16) -> Vec<Line<'static>> {
        match self {
            SubAgentCell::Delegate(card) => card.render_lines(width),
            SubAgentCell::Fanout(card) => card.render_lines(width),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscriptRenderOptions {
    pub show_thinking: bool,
    pub verbose: bool,
    pub show_tool_details: bool,
    pub calm_mode: bool,
    pub low_motion: bool,
    pub spacing: TranscriptSpacing,
}

pub(crate) struct RenderedTranscriptLine {
    pub line: Line<'static>,
    pub copy_prefix_width: usize,
    pub copy_separator_after: CopyLineSeparator,
}

impl Default for TranscriptRenderOptions {
    fn default() -> Self {
        Self {
            show_thinking: true,
            verbose: false,
            show_tool_details: true,
            calm_mode: false,
            low_motion: false,
            spacing: TranscriptSpacing::Comfortable,
        }
    }
}

impl HistoryCell {
    /// Render the cell into a set of terminal lines.
    ///
    /// This is the live-display path used by widgets that don't already pass
    /// `TranscriptRenderOptions`. Tool output is capped, but thinking is shown
    /// in full because callers using bare `lines()` historically expected the
    /// uncollapsed body. For the in-stream transcript view prefer
    /// `lines_with_options`; for the pager / clipboard prefer
    /// `transcript_lines`.
    pub fn lines(&self, width: u16) -> Vec<Line<'static>> {
        match self {
            HistoryCell::User { content } => render_user_message(content, width),
            HistoryCell::Assistant { content, streaming } => render_message(
                ASSISTANT_GLYPH,
                assistant_label_style_for(*streaming, /*low_motion*/ false),
                message_body_style(),
                content,
                width,
            ),
            HistoryCell::System { content } => {
                if is_cycle_boundary(content) {
                    render_cycle_boundary(content, width)
                } else {
                    render_message(
                        "Note",
                        system_label_style(),
                        system_body_style(),
                        content,
                        width,
                    )
                }
            }
            HistoryCell::Error { message, severity } => {
                // Error messages are machine-generated and should not be run
                // through markdown rendering, which would mangle env-var names
                // containing underscores (e.g. DEEPSEEK_ALLOW_INSECURE_HTTP
                // would lose its underscores as italic markers).
                let label = error_label_text(*severity);
                let label_style = error_label_style(*severity);
                let body_style = error_body_style(*severity);
                let prefix_width = UnicodeWidthStr::width(label);
                let content_width = width.saturating_sub(2 + prefix_width as u16).max(1);
                let mut lines = wrap_plain_line(message, body_style, content_width);
                // Add the label prefix to the first line
                if let Some(first) = lines.get_mut(0) {
                    first.spans.insert(0, Span::raw(" "));
                    first.spans.insert(0, Span::styled(label, label_style));
                }
                // Continuation rail for subsequent lines
                let rail = format!("{}{}", '\u{258F}', " ".repeat(prefix_width));
                let rail_style = Style::default().fg(palette::TEXT_DIM);
                for line in lines.iter_mut().skip(1) {
                    line.spans.insert(0, Span::styled(rail.clone(), rail_style));
                }
                lines
            }
            HistoryCell::Thinking {
                content,
                streaming,
                duration_secs,
            } => render_thinking(content, width, *streaming, *duration_secs, false, false),
            HistoryCell::Tool(cell) => cell.lines_with_motion(width, false),
            HistoryCell::SubAgent(cell) => cell.lines(width),
            HistoryCell::ArchivedContext { .. } => render_archived_context(self, width, false),
        }
    }

    pub fn lines_with_options(
        &self,
        width: u16,
        options: TranscriptRenderOptions,
    ) -> Vec<Line<'static>> {
        self.lines_with_options_folded(width, options, false)
    }

    /// Render with an explicit per-cell fold override for thinking cells.
    ///
    /// Uses XOR with the `verbose` flag so that pressing Space toggles
    /// the collapsed state *relative* to the global setting:
    /// - verbose off (default): thinking is collapsed; Space unfolds it
    /// - verbose on: thinking is expanded; Space folds it
    pub fn lines_with_options_folded(
        &self,
        width: u16,
        options: TranscriptRenderOptions,
        folded: bool,
    ) -> Vec<Line<'static>> {
        match self {
            HistoryCell::Thinking {
                streaming,
                duration_secs,
                ..
            } if !options.show_thinking => {
                if *streaming {
                    render_hidden_thinking_activity(width, *duration_secs, options.low_motion)
                } else {
                    Vec::new()
                }
            }
            HistoryCell::Thinking {
                content,
                streaming,
                duration_secs,
            } => render_thinking(
                content,
                width,
                *streaming,
                *duration_secs,
                folded ^ !options.verbose,
                options.low_motion,
            ),
            HistoryCell::Tool(cell) if !options.show_tool_details => {
                let mut lines = cell.lines_with_motion(width, options.low_motion);
                if lines.len() > 2 {
                    lines.truncate(2);
                    lines.push(details_affordance_line(
                        "details hidden",
                        Style::default().fg(palette::TEXT_MUTED).italic(),
                    ));
                }
                lines
            }
            HistoryCell::Tool(cell) if options.calm_mode => {
                let mut lines = cell.lines_with_motion(width, options.low_motion);
                if lines.len() > TOOL_CARD_SUMMARY_LINES {
                    lines.truncate(TOOL_CARD_SUMMARY_LINES);
                    lines.push(details_affordance_line(
                        "details hidden",
                        Style::default().fg(palette::TEXT_MUTED).italic(),
                    ));
                }
                lines
            }
            HistoryCell::Tool(cell) => cell.lines_with_motion(width, options.low_motion),
            HistoryCell::User { content } => render_user_message(content, width),
            HistoryCell::Assistant { content, streaming } => render_message(
                ASSISTANT_GLYPH,
                assistant_label_style_for(*streaming, options.low_motion),
                message_body_style(),
                content,
                width,
            ),
            HistoryCell::System { .. } | HistoryCell::Error { .. } => self.lines(width),
            HistoryCell::SubAgent(cell) => cell.lines(width),
            HistoryCell::ArchivedContext { .. } => {
                render_archived_context(self, width, options.low_motion)
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn lines_with_copy_metadata(
        &self,
        width: u16,
        options: TranscriptRenderOptions,
    ) -> Vec<RenderedTranscriptLine> {
        self.lines_with_copy_metadata_folded(width, options, false)
    }

    pub(crate) fn lines_with_copy_metadata_folded(
        &self,
        width: u16,
        options: TranscriptRenderOptions,
        folded: bool,
    ) -> Vec<RenderedTranscriptLine> {
        match self {
            HistoryCell::User { content } => {
                hard_break_copy_lines(render_user_message(content, width))
            }
            HistoryCell::Assistant { content, streaming } => render_message_with_copy_metadata(
                ASSISTANT_GLYPH,
                assistant_label_style_for(*streaming, options.low_motion),
                message_body_style(),
                content,
                width,
            ),
            HistoryCell::System { content } if !is_cycle_boundary(content) => {
                render_message_with_copy_metadata(
                    "Note",
                    system_label_style(),
                    system_body_style(),
                    content,
                    width,
                )
            }
            _ => hard_break_copy_lines(self.lines_with_options_folded(width, options, folded)),
        }
    }

    /// Render the cell in transcript mode: full content, no caps, no
    /// "Alt+V for details" affordances.
    ///
    /// Use this for full-detail pagers, clipboard exports, and any
    /// surface that wants the complete body rather than the live summary.
    /// For most variants (User / Assistant / System) this matches `lines()`;
    /// `Thinking` and `Tool` are where the live and transcript surfaces
    /// diverge.
    pub fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        match self {
            HistoryCell::User { content } => render_plain_message(
                USER_GLYPH,
                user_label_style(),
                user_body_style(),
                content,
                width,
            ),
            HistoryCell::Assistant { content, streaming } => render_message(
                ASSISTANT_GLYPH,
                // Pager / clipboard surface — pin the glyph at full
                // brightness so a screenshot reads the same as a live frame.
                assistant_label_style_for(*streaming, /*low_motion*/ true),
                message_body_style(),
                content,
                width,
            ),
            HistoryCell::System { .. } | HistoryCell::Error { .. } => self.lines(width),
            HistoryCell::Thinking {
                content,
                streaming,
                duration_secs,
            } => render_thinking(
                content,
                width,
                *streaming,
                *duration_secs,
                /*collapsed*/ false,
                /*low_motion*/ false,
            ),
            HistoryCell::Tool(cell) => cell.transcript_lines(width),
            HistoryCell::SubAgent(cell) => cell.lines(width),
            HistoryCell::ArchivedContext { .. } => render_archived_context(self, width, true),
        }
    }

    /// Whether this cell is the continuation of a streaming assistant message.
    #[must_use]
    pub fn is_stream_continuation(&self) -> bool {
        matches!(
            self,
            HistoryCell::Assistant {
                streaming: true,
                ..
            }
        )
    }

    #[must_use]
    pub fn is_conversational(&self) -> bool {
        matches!(
            self,
            HistoryCell::User { .. } | HistoryCell::Assistant { .. } | HistoryCell::Thinking { .. }
        )
    }
}

/// Convert a message into history cells for rendering.
#[must_use]
pub fn history_cells_from_message(msg: &Message) -> Vec<HistoryCell> {
    let mut cells = Vec::new();

    for block in &msg.content {
        match block {
            ContentBlock::Text { text, .. } => {
                // Check if this is an `<archived_context>` block.
                if msg.role == "assistant"
                    && let Some(archived) = parse_archived_context(text)
                {
                    cells.push(archived);
                    continue;
                }
                match msg.role.as_str() {
                    "user" => {
                        if let Some(HistoryCell::User { content }) = cells.last_mut() {
                            if !content.is_empty() {
                                content.push('\n');
                            }
                            content.push_str(text);
                        } else {
                            cells.push(HistoryCell::User {
                                content: text.clone(),
                            });
                        }
                    }
                    "assistant" => {
                        if let Some(HistoryCell::Assistant { content, .. }) = cells.last_mut() {
                            if !content.is_empty() {
                                content.push('\n');
                            }
                            content.push_str(text);
                        } else {
                            cells.push(HistoryCell::Assistant {
                                content: text.clone(),
                                streaming: false,
                            });
                        }
                    }
                    "system" => {
                        if let Some(HistoryCell::System { content }) = cells.last_mut() {
                            if !content.is_empty() {
                                content.push('\n');
                            }
                            content.push_str(text);
                        } else {
                            cells.push(HistoryCell::System {
                                content: text.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            ContentBlock::Thinking { thinking, .. } => {
                if let Some(HistoryCell::Thinking { content, .. }) = cells.last_mut() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(thinking);
                } else {
                    cells.push(HistoryCell::Thinking {
                        content: thinking.clone(),
                        streaming: false,
                        duration_secs: None,
                    });
                }
            }
            ContentBlock::ToolUse { name, input, .. } if name == "update_plan" => {
                cells.push(HistoryCell::Tool(ToolCell::PlanUpdate(PlanUpdateCell {
                    snapshot: PlanSnapshot::from_tool_input(input),
                    status: ToolStatus::Success,
                })));
            }
            _ => {}
        }
    }

    cells
}

// === Tool Cells ===

/// Variants describing a tool result cell.
#[derive(Debug, Clone)]
pub enum ToolCell {
    Exec(ExecCell),
    Exploring(ExploringCell),
    PlanUpdate(PlanUpdateCell),
    PatchSummary(PatchSummaryCell),
    Review(ReviewCell),
    DiffPreview(DiffPreviewCell),
    Mcp(McpToolCell),
    ViewImage(ViewImageCell),
    WebSearch(WebSearchCell),
    Generic(GenericToolCell),
}

impl ToolCell {
    /// Status for cells that have a concrete lifecycle state.
    pub fn status(&self) -> Option<ToolStatus> {
        match self {
            ToolCell::Exec(cell) => Some(cell.status),
            ToolCell::Exploring(cell) => {
                let has_running = cell
                    .entries
                    .iter()
                    .any(|entry| entry.status == ToolStatus::Running);
                let has_failed = cell
                    .entries
                    .iter()
                    .any(|entry| entry.status == ToolStatus::Failed);
                Some(if has_running {
                    ToolStatus::Running
                } else if has_failed {
                    ToolStatus::Failed
                } else {
                    ToolStatus::Success
                })
            }
            ToolCell::PlanUpdate(cell) => Some(cell.status),
            ToolCell::PatchSummary(cell) => Some(cell.status),
            ToolCell::Review(cell) => Some(cell.status),
            ToolCell::Mcp(cell) => Some(cell.status),
            ToolCell::WebSearch(cell) => Some(cell.status),
            ToolCell::Generic(cell) => Some(cell.status),
            ToolCell::DiffPreview(_) | ToolCell::ViewImage(_) => Some(ToolStatus::Success),
        }
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status() == Some(ToolStatus::Success)
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.status() == Some(ToolStatus::Running)
    }

    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.status() == Some(ToolStatus::Failed)
    }

    /// Whether this cell should stay visible even inside a dense tool run.
    #[must_use]
    pub fn is_collapsible_guard(&self) -> bool {
        self.is_running()
            || self.is_failed()
            || matches!(
                self,
                ToolCell::Exec(_)
                    | ToolCell::PatchSummary(_)
                    | ToolCell::Review(_)
                    | ToolCell::DiffPreview(_)
                    | ToolCell::PlanUpdate(_)
            )
            || matches!(self, ToolCell::Generic(cell) if tool_run::generic_tool_name_is_collapse_guard(&cell.name) || cell.is_diff)
    }

    /// Render the tool cell into lines.
    pub fn lines(&self, width: u16) -> Vec<Line<'static>> {
        self.lines_with_motion(width, false)
    }

    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        self.render(width, low_motion, RenderMode::Live)
    }

    /// Full-content rendering for the pager / clipboard. Tool output that
    /// would be capped + suffixed with "Alt+V for details" in the live view
    /// is emitted in full here.
    pub fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.render(width, /*low_motion*/ false, RenderMode::Transcript)
    }

    fn render(&self, width: u16, low_motion: bool, mode: RenderMode) -> Vec<Line<'static>> {
        match self {
            ToolCell::Exec(cell) => cell.render(width, low_motion, mode),
            ToolCell::Exploring(cell) => cell.lines_with_motion(width, low_motion),
            ToolCell::PlanUpdate(cell) => cell.lines_with_motion(width, low_motion),
            ToolCell::PatchSummary(cell) => cell.render(width, low_motion, mode),
            ToolCell::Review(cell) => cell.render(width, low_motion, mode),
            ToolCell::DiffPreview(cell) => cell.lines_with_motion(width, low_motion),
            ToolCell::Mcp(cell) => cell.render(width, low_motion, mode),
            ToolCell::ViewImage(cell) => cell.lines_with_motion(width, low_motion),
            ToolCell::WebSearch(cell) => cell.lines_with_motion(width, low_motion),
            ToolCell::Generic(cell) => cell.lines_with_mode(width, low_motion, mode),
        }
    }
}

/// Overall status for a tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Success,
    Hydrated,
    Failed,
}

/// Shell command execution rendering data.
#[derive(Debug, Clone)]
pub struct ExecCell {
    pub command: String,
    pub status: ToolStatus,
    pub output: Option<String>,
    pub live_output: Option<String>,
    pub shell_task_id: Option<String>,
    pub owner_agent_id: Option<String>,
    pub owner_agent_name: Option<String>,
    pub started_at: Option<Instant>,
    pub duration_ms: Option<u64>,
    pub source: ExecSource,
    pub interaction: Option<String>,
    /// Cached output summary — avoids re-parsing JSON every frame.
    pub output_summary: Option<String>,
}

impl ExecCell {
    /// Render the execution cell into lines (live view, capped output).
    #[cfg(test)]
    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        self.render(width, low_motion, RenderMode::Live)
    }

    pub(super) fn render(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let command_summary = command_header_summary(&self.command);
        let header_summary = self
            .interaction
            .as_deref()
            .or(Some(command_summary.as_str()));
        lines.push(render_tool_header_with_summary(
            "Shell",
            header_summary,
            tool_status_label(self.status),
            self.status,
            self.started_at,
            low_motion,
        ));

        // A successful shell call is rarely worth its full body — collapse it
        // to the single header line in live mode. The bottom shell strip owns
        // live/background detail, failures stay fully verbose so errors remain
        // visible, and Transcript mode keeps everything for the pager/clipboard.
        if mode == RenderMode::Live && self.status == ToolStatus::Success {
            if let Some(duration_ms) = self.duration_ms
                && duration_ms >= 1000
            {
                let seconds = f64::from(u32::try_from(duration_ms).unwrap_or(u32::MAX)) / 1000.0;
                lines.extend(render_compact_kv(
                    "time",
                    &format!("{seconds:.2}s"),
                    Style::default().fg(palette::TEXT_DIM),
                    width,
                ));
            }
            return wrap_card_rail(lines);
        }

        if self.status == ToolStatus::Success && self.source == ExecSource::User {
            lines.extend(render_compact_kv(
                "source",
                "started by you",
                Style::default().fg(palette::TEXT_MUTED),
                width,
            ));
        }

        if let Some(owner) = self
            .owner_agent_name
            .as_deref()
            .or(self.owner_agent_id.as_deref())
        {
            lines.extend(render_compact_kv(
                "owner",
                owner,
                Style::default().fg(palette::TEXT_MUTED),
                width,
            ));
        }

        if let Some(interaction) = self.interaction.as_ref() {
            lines.extend(wrap_plain_line(
                &format!("  {interaction}"),
                Style::default().fg(palette::TEXT_MUTED),
                width,
            ));
        } else {
            lines.extend(render_command_mode(&self.command, width, mode));
        }

        if self.interaction.is_none() {
            if let Some(output) = self.output.as_ref().or(self.live_output.as_ref()) {
                lines.extend(render_exec_output_mode(
                    output,
                    width,
                    TOOL_OUTPUT_LINE_LIMIT,
                    mode,
                ));
            } else if self.status == ToolStatus::Running && self.source == ExecSource::Assistant {
                lines.extend(wrap_plain_line(
                    "  Ctrl+B backgrounds this command.",
                    Style::default().fg(palette::TEXT_MUTED),
                    width,
                ));
            } else if self.status != ToolStatus::Running && mode == RenderMode::Transcript {
                // #3031: Suppress "(no output)" in compact/Live mode;
                // the success header is enough signal. Transcript still
                // records it for exports/clipboard/pager.
                lines.push(Line::from(Span::styled(
                    "  (no output)",
                    Style::default().fg(palette::TEXT_MUTED).italic(),
                )));
            }
        }

        if let Some(duration_ms) = self.duration_ms {
            // #3031: Suppress sub-second timing in compact mode.
            // Transcript mode always shows exact timing.
            if mode == RenderMode::Transcript || duration_ms >= 1000 {
                let seconds = f64::from(u32::try_from(duration_ms).unwrap_or(u32::MAX)) / 1000.0;
                lines.extend(render_compact_kv(
                    "time",
                    &format!("{seconds:.2}s"),
                    Style::default().fg(palette::TEXT_DIM),
                    width,
                ));
            }
        }

        wrap_card_rail(lines)
    }
}

/// Source of a shell command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecSource {
    User,
    Assistant,
}

/// Aggregate cell for tool exploration runs.
#[derive(Debug, Clone)]
pub struct ExploringCell {
    pub entries: Vec<ExploringEntry>,
}

impl ExploringCell {
    /// Render the exploring cell into lines.
    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let all_done = self
            .entries
            .iter()
            .all(|entry| entry.status != ToolStatus::Running);
        let any_hydrated = self
            .entries
            .iter()
            .any(|entry| entry.status == ToolStatus::Hydrated);
        let status = if all_done {
            if any_hydrated {
                ToolStatus::Hydrated
            } else {
                ToolStatus::Success
            }
        } else {
            ToolStatus::Running
        };
        let header_summary = exploring_header_summary(&self.entries);
        lines.push(render_tool_header_with_summary(
            "Workspace",
            header_summary.as_deref(),
            if all_done {
                tool_status_label(status)
            } else {
                "running"
            },
            status,
            None,
            low_motion,
        ));

        // Dot-grid status strip — one glyph per entry, showing parallel
        // fanout at a glance: ●=done ◐=running ✕=failed.
        if self.entries.len() > 1 {
            let (done, running, failed) =
                self.entries
                    .iter()
                    .fold((0usize, 0usize, 0usize), |(d, r, f), e| match e.status {
                        ToolStatus::Success | ToolStatus::Hydrated => (d + 1, r, f),
                        ToolStatus::Running => (d, r + 1, f),
                        ToolStatus::Failed => (d, r, f + 1),
                    });
            let dots: String = self
                .entries
                .iter()
                .map(|e| match e.status {
                    ToolStatus::Success | ToolStatus::Hydrated => "\u{25CF}",
                    ToolStatus::Running => "\u{25D0}",
                    ToolStatus::Failed => "\u{2715}",
                })
                .collect();
            let counts = format!(
                "{done} done, {running} running{}",
                if failed > 0 {
                    format!(", {failed} failed")
                } else {
                    String::new()
                },
            );
            lines.push(Line::styled(
                format!("  {dots}  {counts}"),
                Style::default().fg(palette::DEEPSEEK_SKY),
            ));
        }

        for entry in &self.entries {
            let prefix = match entry.status {
                ToolStatus::Running => "live",
                ToolStatus::Success => "done",
                ToolStatus::Hydrated => "loaded",
                ToolStatus::Failed => "issue",
            };
            lines.extend(render_compact_kv(
                prefix,
                &entry.label,
                tool_value_style(),
                width,
            ));
        }
        lines
    }

    /// Insert a new entry and return its index.
    #[must_use]
    pub fn insert_entry(&mut self, entry: ExploringEntry) -> usize {
        self.entries.push(entry);
        self.entries.len().saturating_sub(1)
    }
}

/// Single entry for exploring tool output.
#[derive(Debug, Clone)]
pub struct ExploringEntry {
    pub label: String,
    pub status: ToolStatus,
}

/// Cell for patch summaries emitted by the patch tool.
#[derive(Debug, Clone)]
pub struct PatchSummaryCell {
    pub path: String,
    pub summary: String,
    pub status: ToolStatus,
    pub error: Option<String>,
}

impl PatchSummaryCell {
    pub(super) fn render(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(render_tool_header_with_summary(
            "Patch",
            Some(&self.path),
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        ));
        lines.extend(render_compact_kv(
            "file",
            &self.path,
            tool_value_style(),
            width,
        ));
        lines.extend(render_tool_output_mode(
            &self.summary,
            width,
            TOOL_COMMAND_LINE_LIMIT,
            mode,
        ));
        if let Some(error) = self.error.as_ref() {
            lines.extend(render_tool_output_mode(
                error,
                width,
                TOOL_COMMAND_LINE_LIMIT,
                mode,
            ));
        }
        lines
    }
}

/// Cell for structured review output.
#[derive(Debug, Clone)]
pub struct ReviewCell {
    pub target: String,
    pub status: ToolStatus,
    pub output: Option<ReviewOutput>,
    pub error: Option<String>,
}

impl ReviewCell {
    pub(super) fn render(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(render_tool_header(
            "Review",
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        ));

        if !self.target.trim().is_empty() {
            lines.extend(render_compact_kv(
                "target",
                self.target.trim(),
                tool_value_style(),
                width,
            ));
        }

        if self.status == ToolStatus::Running {
            return lines;
        }

        if let Some(error) = self.error.as_ref() {
            lines.extend(render_tool_output_mode(
                error,
                width,
                TOOL_COMMAND_LINE_LIMIT,
                mode,
            ));
            return lines;
        }

        let Some(output) = self.output.as_ref() else {
            return lines;
        };

        if !output.summary.trim().is_empty() {
            lines.extend(wrap_plain_line(
                &format!("Summary: {}", output.summary.trim()),
                Style::default().fg(palette::TEXT_PRIMARY),
                width,
            ));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Issues",
            Style::default()
                .fg(palette::WHALE_ACCENT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )));
        if output.issues.is_empty() {
            lines.extend(wrap_plain_line(
                "  (none)",
                Style::default().fg(palette::TEXT_MUTED),
                width,
            ));
        } else {
            for issue in &output.issues {
                let severity = issue.severity.trim().to_ascii_lowercase();
                let color = review_severity_color(&severity);
                let location = format_review_location(issue.path.as_ref(), issue.line);
                let label = if location.is_empty() {
                    format!("  - [{}] {}", severity, issue.title.trim())
                } else {
                    format!("  - [{}] {} ({})", severity, issue.title.trim(), location)
                };
                lines.extend(wrap_plain_line(&label, Style::default().fg(color), width));
                if !issue.description.trim().is_empty() {
                    lines.extend(wrap_plain_line(
                        &format!("    {}", issue.description.trim()),
                        Style::default().fg(palette::TEXT_MUTED),
                        width,
                    ));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Suggestions",
            Style::default()
                .fg(palette::WHALE_ACCENT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )));
        if output.suggestions.is_empty() {
            lines.extend(wrap_plain_line(
                "  (none)",
                Style::default().fg(palette::TEXT_MUTED),
                width,
            ));
        } else {
            for suggestion in &output.suggestions {
                let location = format_review_location(suggestion.path.as_ref(), suggestion.line);
                let label = if location.is_empty() {
                    format!("  - {}", suggestion.suggestion.trim())
                } else {
                    format!("  - {} ({})", suggestion.suggestion.trim(), location)
                };
                lines.extend(wrap_plain_line(
                    &label,
                    Style::default().fg(palette::TEXT_PRIMARY),
                    width,
                ));
            }
        }

        if !output.overall_assessment.trim().is_empty() {
            lines.push(Line::from(""));
            lines.extend(wrap_plain_line(
                &format!("Overall: {}", output.overall_assessment.trim()),
                Style::default().fg(palette::TEXT_PRIMARY),
                width,
            ));
        }

        lines
    }
}

/// Cell for showing a diff preview before applying changes.
#[derive(Debug, Clone)]
pub struct DiffPreviewCell {
    pub title: String,
    pub diff: String,
}

impl DiffPreviewCell {
    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let diff_summary = diff_render::diff_summary_label(&self.diff);
        lines.push(render_tool_header_with_summary(
            "Diff",
            diff_summary.as_deref(),
            "done",
            ToolStatus::Success,
            None,
            low_motion,
        ));
        lines.extend(render_compact_kv(
            "title",
            &self.title,
            tool_value_style(),
            width,
        ));
        lines.extend(diff_render::render_diff(&self.diff, width));
        lines
    }
}

/// Cell representing an MCP tool execution.
#[derive(Debug, Clone)]
pub struct McpToolCell {
    pub tool: String,
    pub status: ToolStatus,
    pub content: Option<String>,
    pub is_image: bool,
}

impl McpToolCell {
    pub(super) fn render(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(render_tool_header_with_summary(
            "Tool",
            Some(&self.tool),
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        ));
        lines.extend(render_compact_kv(
            "name",
            &self.tool,
            tool_value_style(),
            width,
        ));

        if self.is_image {
            lines.extend(render_compact_kv(
                "result",
                "image",
                tool_value_style(),
                width,
            ));
        }

        if let Some(content) = self.content.as_ref() {
            lines.extend(render_tool_output_mode(
                content,
                width,
                TOOL_COMMAND_LINE_LIMIT,
                mode,
            ));
        }
        lines
    }
}

/// Cell for image view actions.
#[derive(Debug, Clone)]
pub struct ViewImageCell {
    pub path: PathBuf,
}

impl ViewImageCell {
    /// Render the image view cell into lines.
    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        let path = self.path.display().to_string();
        let mut lines = vec![render_tool_header_with_summary(
            "Image",
            Some(&path),
            "done",
            ToolStatus::Success,
            None,
            low_motion,
        )];
        lines.extend(render_compact_kv("path", &path, tool_value_style(), width));
        lines
    }
}

/// Cell for web search tool output.
#[derive(Debug, Clone)]
pub struct WebSearchCell {
    pub query: String,
    pub status: ToolStatus,
    pub summary: Option<String>,
}

impl WebSearchCell {
    /// Render the web search cell into lines.
    pub fn lines_with_motion(&self, width: u16, low_motion: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(render_tool_header_with_summary(
            "Search",
            Some(&self.query),
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        ));
        lines.extend(render_compact_kv(
            "query",
            &self.query,
            tool_value_style(),
            width,
        ));
        if let Some(summary) = self.summary.as_ref() {
            lines.extend(render_compact_kv(
                "result",
                summary,
                tool_value_style(),
                width,
            ));
        }
        lines
    }
}

/// Generic cell for tool output when no specialized rendering exists.
#[derive(Debug, Clone)]
pub struct GenericToolCell {
    pub name: String,
    pub status: ToolStatus,
    pub input_summary: Option<String>,
    pub output: Option<String>,
    /// Optional list of per-child prompts. When populated (by any future
    /// fan-out tool), each prompt is shown on its own indented row instead
    /// of the inline `args:` summary. `None` for ordinary tools.
    pub prompts: Option<Vec<String>>,
    /// Filesystem path to the full output's spillover file (#422/#423).
    /// Set by the tool-routing layer when `ToolResult.metadata` carried a
    /// `spillover_path` field. The truncation affordance includes the
    /// path so the user can `read_file` it (or Cmd+click in
    /// OSC 8-aware terminals — the path renders as a hyperlink when
    /// `tui.osc8_links` is enabled).
    pub spillover_path: Option<std::path::PathBuf>,
    // --- Pre-computed render cache (populated once at cell creation) ---
    /// Cached output summary — avoids re-parsing JSON every frame.
    pub output_summary: Option<String>,
    /// Whether the output looks like a unified diff (cached after first check).
    pub is_diff: bool,
}

fn should_show_raw_tool_name(
    name: &str,
    family: crate::tui::widgets::tool_card::ToolFamily,
    mode: RenderMode,
) -> bool {
    matches!(mode, RenderMode::Transcript)
        || matches!(family, crate::tui::widgets::tool_card::ToolFamily::Generic)
        || name.starts_with("mcp_")
}

impl GenericToolCell {
    /// Render the generic tool cell into lines.
    ///
    /// `mode` controls multi-line output handling: `Live` caps at
    /// `TOOL_OUTPUT_LINE_LIMIT` rows with a "+N more" affordance;
    /// `Transcript` emits the full output.
    pub fn lines_with_mode(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Vec<Line<'static>> {
        if self.name == "activity_group" {
            return self.render_activity_group(width);
        }

        // Issue #241: when the underlying tool is a checklist/todo update and
        // the output is parseable, render a purpose-built progress card
        // instead of dumping the JSON into the generic tool block.
        if let Some(lines) = self.try_render_as_checklist(width, low_motion, mode) {
            return lines;
        }

        // Sub-agent launch already gets a dedicated `DelegateCard`
        // that owns the live action tree, status, and final summary. The
        // generic tool block for the same call duplicates that signal at
        // 3-4 lines per spawn — N parallel spawns multiply the noise. In
        // live mode, render one compact summary line and let the
        // DelegateCard be the source of truth. Transcript mode keeps the
        // full block so session replay remains complete.
        if matches!(mode, RenderMode::Live) && self.name == "agent" {
            return self.render_agent_compact(low_motion);
        }

        // Live mode stays calm: successful tool calls collapse to one header
        // line, and non-read in-flight tools do the same. Failures keep their
        // body visible because error output is the useful part.
        if matches!(mode, RenderMode::Live) {
            let family = crate::tui::widgets::tool_card::tool_family_for_name(&self.name);
            let is_read_family = matches!(
                family,
                crate::tui::widgets::tool_card::ToolFamily::Read
                    | crate::tui::widgets::tool_card::ToolFamily::Find
            );
            let should_collapse = self.status == ToolStatus::Success
                || (self.status != ToolStatus::Failed && !is_read_family);
            if should_collapse {
                let header_summary = crate::tui::widgets::tool_card::tool_header_summary_for_name(
                    &self.name,
                    self.input_summary.as_deref(),
                );
                return wrap_card_rail(vec![render_tool_header_with_family_and_summary(
                    family,
                    header_summary.as_deref(),
                    tool_status_label(self.status),
                    self.status,
                    None,
                    low_motion,
                )]);
            }
        }

        let mut lines = Vec::new();
        // Map the actual tool name (e.g. `agent`, `apply_patch`) to a
        // family rather than the catch-all `"Tool"` title — this is what
        // gives a `GenericToolCell` the right verb glyph (◐ delegate, ⋮⋮
        // fanout, etc.) instead of falling back to the neutral bullet.
        let family = crate::tui::widgets::tool_card::tool_family_for_name(&self.name);
        let header_summary = crate::tui::widgets::tool_card::tool_header_summary_for_name(
            &self.name,
            self.input_summary.as_deref(),
        );
        lines.push(render_tool_header_with_family_and_summary(
            family,
            header_summary.as_deref(),
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        ));
        if should_show_raw_tool_name(&self.name, family, mode) {
            lines.extend(render_compact_kv(
                "name",
                &self.name,
                tool_value_style(),
                width,
            ));
        }

        // Prefer per-prompt rows over the generic args summary when the tool
        // exposes a list of child prompts. One row per child with a `[i]`
        // index makes the fan-out legible without expanding JSON.
        let show_prompts = matches!(self.status, ToolStatus::Running) || self.output.is_none();
        if show_prompts
            && let Some(prompts) = self.prompts.as_ref()
            && !prompts.is_empty()
        {
            for (idx, prompt) in prompts.iter().enumerate() {
                let label = if idx == 0 { "prompts" } else { "" };
                let value = format!("[{idx}] {}", truncate_text(prompt.trim(), 200));
                lines.extend(render_card_detail_line(
                    if label.is_empty() { None } else { Some(label) },
                    &value,
                    tool_value_style(),
                    width,
                ));
            }
        } else {
            let show_args = matches!(self.status, ToolStatus::Running) || self.output.is_none();
            if show_args && let Some(summary) = self.input_summary.as_ref() {
                lines.extend(render_compact_kv(
                    "args",
                    summary,
                    tool_value_style(),
                    width,
                ));
            }
        }

        if let Some(output) = self.output.as_ref() {
            if self.is_diff {
                let diff_summary = diff_render::diff_summary_label(output);
                lines.push(render_tool_header_with_summary(
                    "Diff",
                    diff_summary.as_deref(),
                    tool_status_label(self.status),
                    self.status,
                    None,
                    low_motion,
                ));
                lines.extend(diff_render::render_diff(output, width));
            } else {
                lines.extend(render_tool_output_mode(
                    output,
                    width,
                    TOOL_OUTPUT_LINE_LIMIT,
                    mode,
                ));
            }

            if matches!(mode, RenderMode::Live)
                && let Some(path) = self.spillover_path.as_ref()
            {
                lines.push(render_spillover_annotation(path, width));
            }
        }
        wrap_card_rail(lines)
    }

    /// Render `agent` as a single compact summary line for live mode. The
    /// companion `DelegateCard` already carries the
    /// live action tree, status, and final summary; this line is just
    /// the pointer that says "a spawn happened, here's the agent id".
    ///
    /// Output shape (header):
    ///   `◐ delegate · agent  agent-abc12  [running]`
    /// Falls back to a placeholder when the spawn is still pending and
    /// no agent id has been assigned yet.
    fn render_agent_compact(&self, low_motion: bool) -> Vec<Line<'static>> {
        let family = crate::tui::widgets::tool_card::ToolFamily::Delegate;
        let agent_id = self
            .output
            .as_deref()
            .and_then(extract_agent_id)
            .unwrap_or("…");
        vec![render_tool_header_with_family_and_summary(
            family,
            Some(agent_id),
            tool_status_label(self.status),
            self.status,
            None,
            low_motion,
        )]
    }

    fn render_activity_group(&self, width: u16) -> Vec<Line<'static>> {
        let summary = self.input_summary.as_deref().unwrap_or("Updated metadata");
        let budget = usize::from(width).max(1);
        vec![Line::from(Span::styled(
            truncate_text(summary, budget),
            Style::default().fg(palette::TEXT_MUTED),
        ))]
    }

    /// If this cell is a checklist/todo write/add/update and the output is
    /// parseable as a checklist snapshot, render a purpose-built checklist
    /// card instead of the generic `name: ... { json }` block (issue #241).
    fn try_render_as_checklist(
        &self,
        width: u16,
        low_motion: bool,
        mode: RenderMode,
    ) -> Option<Vec<Line<'static>>> {
        if !is_checklist_tool_name(&self.name) {
            return None;
        }
        let output = self.output.as_ref()?;
        let snapshot = parse_checklist_snapshot(output)?;

        // Concise update rendering (#403). When the tool emits an
        // "Updated todo #N to STATUS" prefix line — which `todo_update` /
        // `checklist_update` always do on a successful match — render
        // only the changed item plus a `M/N · pct%` summary instead of
        // dumping the full list every time. The full list is still
        // reachable via Alt+V on the tool detail record. This keeps the
        // transcript scannable in long sessions.
        if matches!(mode, RenderMode::Live)
            && let Some(change) = parse_update_prefix(output)
        {
            return Some(render_checklist_change_card(
                &self.name,
                self.status,
                &snapshot,
                &change,
                width,
                low_motion,
            ));
        }

        Some(render_checklist_card(
            &self.name,
            self.status,
            &snapshot,
            width,
            low_motion,
            mode,
        ))
    }
}

/// Render the inline annotation for a tool cell whose full output was
/// spilled to disk (#422 + #423). Produces a one-line muted hint:
///
/// ```text
///   full output: /Users/you/.deepseek/tool_outputs/call-abc12.txt
/// ```
///
/// Path is plain text on this branch; the OSC 8 hyperlink-wrap that
/// makes it Cmd+click-openable lives on the OSC 8 branch (PR #515)
/// and merges in once both PRs land on `main`. The clipboard /
/// selection path already strips OSC 8 there, so a future enhancement
/// stays backward-compatible.
fn render_spillover_annotation(path: &std::path::Path, width: u16) -> Line<'static> {
    let display = path.display().to_string();
    let prefix = "  full output: ";
    let budget = usize::from(width).saturating_sub(prefix.len()).max(8);
    let truncated = truncate_text(&display, budget);
    Line::from(vec![
        Span::styled(prefix, Style::default().fg(palette::TEXT_MUTED)),
        Span::styled(truncated, Style::default().fg(palette::TEXT_MUTED).italic()),
    ])
}

/// Pull the `agent_id` field out of a sub-agent open tool output. The
/// tool emits structured JSON shaped like
/// `{"agent_id": "agent-abc12", "nickname": "...", "model": "..."}` so we
/// look for the `agent_id` key and return its string value.
///
/// Returns `None` for outputs we can't parse as JSON or that lack the
/// expected key — the caller falls back to a placeholder so a still-pending
/// spawn renders cleanly.
fn extract_agent_id(output: &str) -> Option<&str> {
    // Cheap, deterministic, no allocations: scan for the literal key.
    // Avoids dragging serde_json into a render hot path on every frame.
    let key = "\"agent_id\"";
    let key_idx = output.find(key)?;
    let rest = &output[key_idx + key.len()..];
    let colon = rest.find(':')?;
    let after_colon = rest[colon + 1..].trim_start();
    let after_colon = after_colon.strip_prefix('"')?;
    let end = after_colon.find('"')?;
    let id = &after_colon[..end];
    (!id.is_empty()).then_some(id)
}

/// Heuristic: does the output look like a unified diff? Returns true when
/// the output contains at least one hunk header (`@@`) or a `diff --git`
/// line, which are reliable markers of unified diff content (#380).
pub(crate) fn output_looks_like_diff(output: &str) -> bool {
    let mut lines = output.lines();
    // Check first 5 lines for diff markers
    for _ in 0..5 {
        let Some(line) = lines.next() else { break };
        let trimmed = line.trim();
        if trimmed.starts_with("@@") || trimmed.starts_with("diff --git") {
            return true;
        }
    }
    false
}

fn summarize_string_value(text: &str, max_len: usize, count_only: bool) -> String {
    let trimmed = text.trim();
    let len = trimmed.chars().count();
    if count_only || len > max_len {
        return format!("<{len} chars>");
    }
    truncate_text(trimmed, max_len)
}

fn summarize_inline_value(value: &Value, max_len: usize, count_only: bool) -> String {
    match value {
        Value::String(s) => summarize_string_value(s, max_len, count_only),
        Value::Array(items) => format!("<{} items>", items.len()),
        Value::Object(map) => format!("<{} keys>", map.len()),
        Value::Bool(b) => b.to_string(),
        Value::Number(num) => num.to_string(),
        Value::Null => "null".to_string(),
    }
}

#[must_use]
pub fn summarize_tool_args(input: &Value) -> Option<String> {
    let obj = input.as_object()?;
    if obj.is_empty() {
        return None;
    }

    let mut parts = Vec::new();

    if let Some(value) = obj.get("path") {
        parts.push(format!(
            "path: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("command") {
        parts.push(format!(
            "command: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("query") {
        parts.push(format!(
            "query: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("prompt") {
        parts.push(format!(
            "prompt: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("text") {
        parts.push(format!(
            "text: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("pattern") {
        parts.push(format!(
            "pattern: {}",
            summarize_inline_value(value, 80, false)
        ));
    }
    if let Some(value) = obj.get("model") {
        parts.push(format!(
            "model: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("profile") {
        parts.push(format!(
            "profile: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("level") {
        parts.push(format!(
            "level: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("file_id") {
        parts.push(format!(
            "file_id: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("task_id") {
        parts.push(format!(
            "task_id: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("voice_id") {
        parts.push(format!(
            "voice_id: {}",
            summarize_inline_value(value, 40, false)
        ));
    }
    if let Some(value) = obj.get("content") {
        parts.push(format!(
            "content: {}",
            summarize_inline_value(value, 0, true)
        ));
    }

    if parts.is_empty()
        && let Some((key, value)) = obj.iter().next()
    {
        return Some(format!(
            "{}: {}",
            key,
            summarize_inline_value(value, 80, false)
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

#[must_use]
pub fn summarize_tool_output(output: &str) -> String {
    if let Ok(json) = serde_json::from_str::<Value>(output) {
        if let Some(obj) = json.as_object() {
            if let Some(error) = obj.get("error").or(obj.get("status_msg")) {
                return format!("Error: {}", summarize_inline_value(error, 120, false));
            }

            let mut parts = Vec::new();

            if let Some(status) = obj.get("status").and_then(|v| v.as_str()) {
                parts.push(format!("status: {status}"));
            }
            if let Some(message) = obj.get("message").and_then(|v| v.as_str()) {
                parts.push(truncate_text(message, TOOL_TEXT_LIMIT));
            }
            if let Some(task_id) = obj.get("task_id").and_then(|v| v.as_str()) {
                parts.push(format!("task_id: {task_id}"));
            }
            if let Some(file_id) = obj.get("file_id").and_then(|v| v.as_str()) {
                parts.push(format!("file_id: {file_id}"));
            }
            if let Some(url) = obj
                .get("file_url")
                .or_else(|| obj.get("url"))
                .and_then(|v| v.as_str())
            {
                parts.push(format!("url: {}", truncate_text(url, 120)));
            }
            if let Some(data) = obj.get("data") {
                parts.push(format!("data: {}", summarize_inline_value(data, 80, true)));
            }

            if !parts.is_empty() {
                return parts.join(" | ");
            }

            if let Some(content) = obj
                .get("content")
                .or(obj.get("result"))
                .or(obj.get("output"))
            {
                return summarize_inline_value(content, TOOL_TEXT_LIMIT, false);
            }
        }

        return summarize_inline_value(&json, TOOL_TEXT_LIMIT, true);
    }

    truncate_text(output, TOOL_TEXT_LIMIT)
}

// === MCP Output Summaries ===

/// Summary information extracted from an MCP tool output payload.
pub struct McpOutputSummary {
    pub content: Option<String>,
    pub is_image: bool,
    pub is_error: Option<bool>,
}

/// Summarize raw MCP output into UI-friendly content.
#[must_use]
pub fn summarize_mcp_output(output: &str) -> McpOutputSummary {
    if let Ok(json) = serde_json::from_str::<Value>(output) {
        let is_error = json
            .get("isError")
            .and_then(serde_json::Value::as_bool)
            .or_else(|| json.get("is_error").and_then(serde_json::Value::as_bool));

        if let Some(blocks) = json.get("content").and_then(|v| v.as_array()) {
            let mut lines = Vec::new();
            let mut is_image = false;

            for block in blocks {
                let block_type = block
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                match block_type {
                    "text" => {
                        let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        if !text.is_empty() {
                            lines.push(format!("- text: {}", truncate_text(text, 200)));
                        }
                    }
                    "image" | "image_url" => {
                        is_image = true;
                        let url = block
                            .get("url")
                            .or_else(|| block.get("image_url"))
                            .and_then(|v| v.as_str());
                        if let Some(url) = url {
                            lines.push(format!("- image: {}", truncate_text(url, 200)));
                        } else {
                            lines.push("- image".to_string());
                        }
                    }
                    "resource" | "resource_link" => {
                        let uri = block
                            .get("uri")
                            .or_else(|| block.get("url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("<resource>");
                        lines.push(format!("- resource: {}", truncate_text(uri, 200)));
                    }
                    other => {
                        lines.push(format!("- {other} content"));
                    }
                }
            }

            return McpOutputSummary {
                content: if lines.is_empty() {
                    None
                } else {
                    Some(lines.join("\n"))
                },
                is_image,
                is_error,
            };
        }
    }

    McpOutputSummary {
        content: Some(summarize_tool_output(output)),
        is_image: output_is_image(output),
        is_error: None,
    }
}

#[must_use]
pub fn output_is_image(output: &str) -> bool {
    let lower = output.to_lowercase();

    [
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".tiff", ".ppm",
    ]
    .iter()
    .any(|ext| lower.contains(ext))
}

#[allow(dead_code)] // Kept for compatibility/tests; live view uses explicit summaries only.
#[must_use]
pub fn extract_reasoning_summary(text: &str) -> Option<String> {
    extract_explicit_reasoning_summary(text).or_else(|| {
        let fallback = text.trim();
        if fallback.is_empty() {
            None
        } else {
            Some(fallback.to_string())
        }
    })
}

fn extract_explicit_reasoning_summary(text: &str) -> Option<String> {
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("summary") {
            let mut summary = String::new();
            if let Some((_, rest)) = trimmed.split_once(':')
                && !rest.trim().is_empty()
            {
                summary.push_str(rest.trim());
                summary.push('\n');
            }
            while let Some(next) = lines.peek() {
                let next_trimmed = next.trim();
                if next_trimmed.is_empty() {
                    break;
                }
                if next_trimmed.starts_with('#') || next_trimmed.starts_with("**") {
                    break;
                }
                summary.push_str(next_trimmed);
                summary.push('\n');
                lines.next();
            }
            let summary = summary.trim().to_string();
            return if summary.is_empty() {
                None
            } else {
                Some(summary)
            };
        }
    }
    None
}

fn render_thinking(
    content: &str,
    width: u16,
    streaming: bool,
    duration_secs: Option<f32>,
    collapsed: bool,
    low_motion: bool,
) -> Vec<Line<'static>> {
    let state = thinking_visual_state(streaming, duration_secs);
    let style = thinking_style();
    // 12% reasoning surface tint over the app ink — the only deliberately
    // warm element in the transcript. Dropped on Ansi-16 terminals where the
    // tint would distort the named palette.
    let depth = cached_color_depth();
    let body_bg = palette::reasoning_surface_tint(depth);
    let body_style = match body_bg {
        Some(bg) => style.italic().bg(bg),
        None => style.italic(),
    };
    let mut lines = Vec::new();

    // Header: `…` opener (replaces the spinner; reasoning isn't a tool, it's
    // a slow exhale) followed by the reasoning label and live status.
    let mut header_spans = vec![
        Span::styled(
            format!("{REASONING_OPENER} "),
            Style::default().fg(thinking_state_accent(state)),
        ),
        Span::styled("reasoning", thinking_title_style()),
    ];
    header_spans.push(Span::styled(" ", Style::default()));
    header_spans.push(Span::styled(
        thinking_status_label(state),
        thinking_status_style(state),
    ));
    if let Some(dur) = duration_secs {
        header_spans.push(Span::styled(" · ", Style::default().fg(palette::TEXT_DIM)));
        header_spans.push(Span::styled(format!("{dur:.1}s"), thinking_meta_style()));
    }
    lines.push(Line::from(header_spans));

    let content_width = width.saturating_sub(3).max(1);
    let mut collapsed_without_explicit_summary = false;
    let body_text = if collapsed {
        if streaming {
            // #861 RC4 / #1324: during streaming we don't yet have a
            // completed reasoning block, so `extract_reasoning_summary`
            // is meaningless. Show the raw content and let the
            // truncation logic below keep the *last* `LIMIT` lines so
            // the user sees the model's most recent thinking instead of
            // staring at an empty placeholder.
            content.to_string()
        } else {
            match extract_explicit_reasoning_summary(content) {
                Some(summary) => summary,
                None => {
                    collapsed_without_explicit_summary = true;
                    content.to_string()
                }
            }
        }
    } else {
        content.to_string()
    };
    let mut rendered = if body_text.trim().is_empty() {
        Vec::new()
    } else {
        markdown_render::render_markdown(&body_text, content_width, body_style)
    };
    let mut truncated = false;
    let line_limit = if streaming {
        THINKING_STREAMING_PREVIEW_LINE_LIMIT
    } else if collapsed_without_explicit_summary {
        THINKING_COMPLETED_PREVIEW_LINE_LIMIT
    } else {
        THINKING_SUMMARY_LINE_LIMIT
    };
    if collapsed && rendered.len() > line_limit {
        if streaming {
            // Drop the *head* during streaming so the visible window
            // tracks the live cursor at the bottom.
            let drop = rendered.len() - line_limit;
            rendered.drain(0..drop);
        } else {
            rendered.truncate(line_limit);
        }
        truncated = true;
    }

    let rail_style = Style::default().fg(thinking_state_accent(state));
    let cursor_style = Style::default().fg(palette::ACCENT_REASONING_LIVE);

    if rendered.is_empty() && streaming {
        let mut spans = vec![Span::styled(REASONING_RAIL.to_string(), rail_style)];
        spans.push(Span::styled("reasoning...", body_style.italic()));
        if !low_motion {
            spans.push(Span::styled(format!(" {REASONING_CURSOR}"), cursor_style));
        }
        lines.push(Line::from(spans));
    }

    let last_idx = rendered.len().saturating_sub(1);
    for (idx, line) in rendered.into_iter().enumerate() {
        let mut spans = vec![Span::styled(REASONING_RAIL.to_string(), rail_style)];
        spans.extend(line.spans);
        // Trailing cursor on the very last body line while streaming —
        // signals "still generating" without churning every line.
        if streaming && !low_motion && idx == last_idx {
            spans.push(Span::styled(format!(" {REASONING_CURSOR}"), cursor_style));
        }
        lines.push(Line::from(spans));
    }

    let needs_affordance = collapsed
        && if streaming {
            // #861 RC4 / #1324: during streaming, surface the affordance
            // whenever any head lines have been clipped so the user
            // knows there's more above and how to reach it.
            truncated
        } else {
            truncated || body_text.trim() != content.trim()
        };
    if needs_affordance {
        let label = if streaming {
            "More reasoning in Ctrl+O"
        } else {
            "Space to expand · Full reasoning in Ctrl+O"
        };
        lines.push(Line::from(vec![
            Span::styled(REASONING_RAIL.to_string(), rail_style),
            Span::styled(label, Style::default().fg(palette::TEXT_MUTED).italic()),
        ]));
    }

    lines
}

fn render_hidden_thinking_activity(
    width: u16,
    duration_secs: Option<f32>,
    low_motion: bool,
) -> Vec<Line<'static>> {
    let state = ThinkingVisualState::Live;
    let rail_style = Style::default().fg(thinking_state_accent(state));
    let body_style = thinking_style().italic();
    let content_width = width.saturating_sub(3).max(1) as usize;

    let mut header_spans = vec![
        Span::styled(
            format!("{REASONING_OPENER} "),
            Style::default().fg(thinking_state_accent(state)),
        ),
        Span::styled("reasoning", thinking_title_style()),
        Span::styled(" ", Style::default()),
        Span::styled(thinking_status_label(state), thinking_status_style(state)),
    ];
    if let Some(dur) = duration_secs {
        header_spans.push(Span::styled(" · ", Style::default().fg(palette::TEXT_DIM)));
        header_spans.push(Span::styled(format!("{dur:.1}s"), thinking_meta_style()));
    }

    let mut body =
        truncate_line_to_width("reasoning hidden; model is still working", content_width);
    if !low_motion {
        body.push(' ');
        body.push_str(REASONING_CURSOR);
    }

    vec![
        Line::from(header_spans),
        Line::from(vec![
            Span::styled(REASONING_RAIL.to_string(), rail_style),
            Span::styled(body, body_style),
        ]),
    ]
}

fn render_message(
    prefix: &str,
    label_style: Style,
    body_style: Style,
    content: &str,
    width: u16,
) -> Vec<Line<'static>> {
    render_message_with_copy_metadata(prefix, label_style, body_style, content, width)
        .into_iter()
        .map(|rendered| rendered.line)
        .collect()
}

fn render_message_with_copy_metadata(
    prefix: &str,
    label_style: Style,
    body_style: Style,
    content: &str,
    width: u16,
) -> Vec<RenderedTranscriptLine> {
    // An assistant cell whose content is entirely whitespace (e.g. a stray
    // newline streamed between reasoning and a tool call) would otherwise
    // render as a bare, orphaned role glyph floating on its own line — the
    // "blue dots with nothing after them" artifact. Render nothing so the
    // transcript doesn't accumulate empty markers. Real prose, including
    // messages that merely start with blank lines, still renders normally.
    if prefix == ASSISTANT_GLYPH && content.trim().is_empty() {
        return Vec::new();
    }
    let prefix_width = UnicodeWidthStr::width(prefix);
    let prefix_width_u16 = u16::try_from(prefix_width.saturating_add(2)).unwrap_or(u16::MAX);
    let content_width = usize::from(width.saturating_sub(prefix_width_u16).max(1));
    let mut lines = Vec::new();
    let rendered =
        markdown_render::render_markdown_tagged(content, content_width as u16, body_style);
    for (idx, rendered_line) in rendered.into_iter().enumerate() {
        let line = if idx == 0 {
            let mut spans = Vec::new();
            if !prefix.is_empty() {
                spans.push(Span::styled(
                    prefix.to_string(),
                    label_style.add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(" "));
            }
            spans.extend(rendered_line.line.spans);
            Line::from(spans)
        } else {
            let indent = if prefix.is_empty() {
                String::new()
            } else if rendered_line.is_code {
                " ".repeat(prefix_width + 1)
            } else {
                let mut s = String::with_capacity(prefix_width + 1);
                s.push('\u{258F}');
                s.extend(std::iter::repeat_n(' ', prefix_width));
                s
            };
            let rail_style = Style::default().fg(palette::TEXT_DIM);
            let mut spans = vec![Span::styled(indent, rail_style)];
            spans.extend(rendered_line.line.spans);
            Line::from(spans)
        };
        lines.push(RenderedTranscriptLine {
            line,
            copy_prefix_width: rendered_line.copy_prefix_width
                + history_copy_prefix_width(prefix, prefix_width, rendered_line.is_code, idx),
            copy_separator_after: rendered_line.copy_separator_after,
        });
    }
    if lines.is_empty() {
        lines.push(RenderedTranscriptLine {
            line: Line::from(""),
            copy_prefix_width: 0,
            copy_separator_after: CopyLineSeparator::Newline,
        });
    }
    lines
}

fn history_copy_prefix_width(
    prefix: &str,
    prefix_width: usize,
    is_code: bool,
    line_index: usize,
) -> usize {
    if line_index > 0 && is_code && !prefix.is_empty() {
        prefix_width + 1
    } else {
        0
    }
}

fn hard_break_copy_lines(lines: Vec<Line<'static>>) -> Vec<RenderedTranscriptLine> {
    lines
        .into_iter()
        .map(|line| RenderedTranscriptLine {
            line,
            copy_prefix_width: 0,
            copy_separator_after: CopyLineSeparator::Newline,
        })
        .collect()
}

/// Render a plain-text user message: split on newlines, word-wrap each line,
/// preserve leading whitespace. No markdown interpretation (headings, lists,
/// code blocks, etc. are rendered as literal text).
fn render_plain_message(
    prefix: &str,
    label_style: Style,
    body_style: Style,
    content: &str,
    width: u16,
) -> Vec<Line<'static>> {
    let prefix_width = UnicodeWidthStr::width(prefix);
    let prefix_width_u16 = u16::try_from(prefix_width.saturating_add(2)).unwrap_or(u16::MAX);
    let content_width = width.saturating_sub(prefix_width_u16).max(1);
    let rendered = markdown_render::render_plain_text(content, content_width, body_style);
    let mut lines = Vec::with_capacity(rendered.len());

    for (idx, line) in rendered.into_iter().enumerate() {
        if idx == 0 {
            let mut spans = Vec::new();
            if !prefix.is_empty() {
                spans.push(Span::styled(
                    prefix.to_string(),
                    label_style.add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(" "));
            }
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        } else {
            let indent = if prefix.is_empty() {
                String::new()
            } else {
                let mut s = String::with_capacity(prefix_width + 1);
                s.push('\u{258F}');
                s.extend(std::iter::repeat_n(' ', prefix_width));
                s
            };
            let rail_style = Style::default().fg(palette::TEXT_DIM);
            let mut spans = vec![Span::styled(indent, rail_style)];
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    lines
}

fn render_user_message(content: &str, width: u16) -> Vec<Line<'static>> {
    render_plain_message(
        USER_GLYPH,
        user_label_style(),
        user_body_style(),
        content,
        width,
    )
    .into_iter()
    .map(|line| apply_user_message_highlight(line, width))
    .collect()
}

fn apply_user_message_highlight(mut line: Line<'static>, width: u16) -> Line<'static> {
    let bg = palette::SURFACE_ELEVATED;
    line.style = line.style.bg(bg);

    let target_width = usize::from(width);
    let line_width = line.width();
    if line_width < target_width {
        line.spans.push(Span::styled(
            " ".repeat(target_width - line_width),
            Style::default().bg(bg),
        ));
    }

    line
}

fn render_command_mode(command: &str, width: u16, mode: RenderMode) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let cap = match mode {
        RenderMode::Live => TOOL_COMMAND_LINE_LIMIT,
        RenderMode::Transcript => usize::MAX,
    };
    for (count, chunk) in wrap_text(command, width.saturating_sub(4).max(1) as usize)
        .into_iter()
        .enumerate()
    {
        if count >= cap {
            lines.push(details_affordance_line(
                "command clipped",
                Style::default().fg(palette::TEXT_MUTED),
            ));
            break;
        }
        lines.extend(render_card_detail_line(
            if count == 0 { Some("command") } else { None },
            chunk.as_str(),
            tool_value_style(),
            width,
        ));
    }
    lines
}

fn command_header_summary(command: &str) -> String {
    command
        .lines()
        .next()
        .unwrap_or(command)
        .trim_start_matches("$ ")
        .trim()
        .to_string()
}

fn exploring_header_summary(entries: &[ExploringEntry]) -> Option<String> {
    match entries {
        [] => None,
        [entry] => Some(entry.label.clone()),
        entries => Some(format!("{} items", entries.len())),
    }
}

fn render_compact_kv(label: &str, value: &str, style: Style, width: u16) -> Vec<Line<'static>> {
    render_card_detail_line(Some(label.trim_end_matches(':')), value, style, width)
}

/// Wrap rendered tool-card lines with card-rail glyphs (╭ │ ╰).
/// First non-empty line gets `╭`, middle lines get `│`, last line gets `╰`.
/// Single-line cards get a single `─` prefix.
fn wrap_card_rail(mut lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let n = lines.len();
    if n == 0 {
        return lines;
    }
    if n == 1 {
        lines[0].spans.insert(0, Span::raw("─ "));
        return lines;
    }
    for (i, line) in lines.iter_mut().enumerate() {
        let rail = if i == 0 {
            "\u{256D} " // ╭
        } else if i == n - 1 {
            "\u{2570} " // ╰
        } else {
            "\u{2502} " // │
        };
        line.spans.insert(0, Span::raw(rail));
    }
    lines
}

fn render_tool_output_mode(
    output: &str,
    width: u16,
    line_limit: usize,
    mode: RenderMode,
) -> Vec<Line<'static>> {
    render_preserved_output_mode(output, width, line_limit, mode, "result")
}

fn review_severity_color(severity: &str) -> Color {
    match severity {
        "error" => palette::STATUS_ERROR,
        "warning" => palette::STATUS_WARNING,
        _ => palette::STATUS_INFO,
    }
}

fn format_review_location(path: Option<&String>, line: Option<u32>) -> String {
    let path = path.map(|p| p.trim().to_string()).filter(|p| !p.is_empty());
    match (path, line) {
        (Some(path), Some(line)) => format!("{path}:{line}"),
        (Some(path), None) => path,
        (None, Some(line)) => format!("line {line}"),
        (None, None) => String::new(),
    }
}

fn render_exec_output_mode(
    output: &str,
    width: u16,
    line_limit: usize,
    mode: RenderMode,
) -> Vec<Line<'static>> {
    render_preserved_output_mode(output, width, line_limit, mode, "output")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRow {
    pub text: String,
    pub intact: bool,
}

fn render_preserved_output_mode(
    output: &str,
    width: u16,
    line_limit: usize,
    mode: RenderMode,
    first_label: &str,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if output.trim().is_empty() {
        // #3031: In compact/Live mode, suppress "(no output)" — the tool
        // header already carries the success/failure status.  Transcript
        // mode still records it for exports/clipboard/pager.
        if mode == RenderMode::Transcript {
            lines.push(Line::from(Span::styled(
                "  (no output)",
                Style::default().fg(palette::TEXT_MUTED).italic(),
            )));
        }
        return lines;
    }

    let content_hash = crate::tui::output_rows_cache::hash_str(output);
    let all_lines = crate::tui::output_rows_cache::get_or_compute_rows(output, width, || {
        output_rows(output, width)
    });

    if matches!(mode, RenderMode::Transcript) {
        // Full-content path: emit every wrapped line with no head/tail split,
        // no "+N more" affordance.
        for (idx, row) in all_lines.iter().enumerate() {
            render_output_row(
                &mut lines,
                if idx == 0 { Some(first_label) } else { None },
                row,
                width,
            );
        }
        return lines;
    }

    let selected = crate::tui::output_rows_cache::get_or_compute_indices(
        content_hash,
        width,
        line_limit,
        || selected_output_indices(&all_lines, line_limit),
    );
    let mut previous: Option<usize> = None;
    for (rendered_idx, idx) in selected.iter().copied().enumerate() {
        if let Some(prev) = previous {
            let omitted = idx.saturating_sub(prev + 1);
            if omitted > 0 {
                lines.push(details_affordance_line(
                    &format!("{omitted} lines omitted"),
                    Style::default().fg(palette::TEXT_MUTED),
                ));
            }
        }

        let row = &all_lines[idx];
        render_output_row(
            &mut lines,
            if rendered_idx == 0 {
                Some(first_label)
            } else {
                None
            },
            row,
            width,
        );
        previous = Some(idx);
    }

    lines
}

fn output_rows(output: &str, width: u16) -> Vec<OutputRow> {
    let wrap_width = width.saturating_sub(4).max(1) as usize;
    let mut rows = Vec::new();
    let mut sanitized = String::with_capacity(output.len());
    for line in output.lines() {
        sanitized.clear();
        crate::tui::osc8::strip_ansi_into(line, &mut sanitized);
        let intact = is_path_or_url_like(&sanitized);
        if intact {
            rows.push(OutputRow {
                text: sanitized.clone(),
                intact: true,
            });
        } else {
            for wrapped in wrap_text(&sanitized, wrap_width) {
                rows.push(OutputRow {
                    text: wrapped,
                    intact: false,
                });
            }
        }
    }
    if rows.is_empty() {
        rows.push(OutputRow {
            text: String::new(),
            intact: false,
        });
    }
    rows
}

fn selected_output_indices(rows: &[OutputRow], line_limit: usize) -> Vec<usize> {
    let total = rows.len();
    if total <= line_limit || line_limit == 0 {
        return (0..total).collect();
    }

    let head = TOOL_OUTPUT_HEAD_LINES.min(line_limit).min(total);
    let tail = TOOL_OUTPUT_TAIL_LINES
        .min(line_limit.saturating_sub(head))
        .min(total.saturating_sub(head));
    let mut selected = std::collections::BTreeSet::new();
    selected.extend(0..head);
    selected.extend(total.saturating_sub(tail)..total);

    let budget = line_limit.saturating_sub(selected.len());
    if budget > 0 {
        let mut important: Vec<(usize, usize)> = rows
            .iter()
            .enumerate()
            .skip(head)
            .take(total.saturating_sub(head + tail))
            .filter_map(|(idx, row)| output_importance_rank(&row.text).map(|rank| (idx, rank)))
            .collect();
        important.sort_by_key(|(idx, rank)| (*rank, *idx));
        for (idx, _) in important.into_iter().take(budget) {
            selected.insert(idx);
        }
    }

    selected.into_iter().collect()
}

fn output_importance_rank(line: &str) -> Option<usize> {
    let lower = line.to_ascii_lowercase();
    if [
        "error",
        "failed",
        "failure",
        "fatal",
        "panic",
        "exception",
        "traceback",
        "denied",
        "not found",
        "no such file",
        "cannot",
        "can't",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return Some(0);
    }
    if lower.contains("warning") || lower.contains("warn") {
        return Some(1);
    }
    if is_path_or_url_like(line) {
        return Some(2);
    }
    None
}

fn is_path_or_url_like(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.contains("://") || trimmed.starts_with("file:") {
        return true;
    }
    let has_separator = trimmed.contains('/') || trimmed.contains('\\');
    let has_extension = trimmed
        .split_whitespace()
        .any(|part| part.rsplit_once('.').is_some_and(|(_, ext)| ext.len() <= 8));
    has_separator && has_extension
}

/// Detect whether a system message is a cycle-boundary announcement
/// (e.g. `─── cycle 0 → 1  (briefing: 2500 tokens) ───`).
fn is_cycle_boundary(content: &str) -> bool {
    content.contains("cycle")
}

/// Render a cycle-boundary system message with distinct visual styling (#395):
/// full-width line with primary accent text and bold weight, plus a thin
/// horizontal rule above for visual separation.
fn render_cycle_boundary(content: &str, width: u16) -> Vec<Line<'static>> {
    let style = Style::default()
        .fg(palette::WHALE_ACCENT_PRIMARY)
        .add_modifier(Modifier::BOLD);
    let rule_style = Style::default().fg(palette::TEXT_DIM);
    let content_width = usize::from(width.saturating_sub(2).max(1));
    let mut lines = Vec::new();
    // Thin horizontal rule above for visual separation
    if width >= 4 {
        let rule = "\u{2500}".repeat(content_width);
        lines.push(Line::from(Span::styled(format!("  {rule}"), rule_style)));
    }
    // Cycle boundary text — just the content, full-width
    let rendered =
        crate::tui::markdown_render::render_markdown(content, content_width as u16, style);
    for line in rendered {
        let mut spans = vec![Span::raw("  ")];
        spans.extend(line.spans);
        lines.push(Line::from(spans));
    }
    if lines.len() == 1 && width >= 4 {
        // Only the rule was added (unlikely), but add at least a spacer
        lines.push(Line::from(""));
    }
    lines
}

/// Detect whether a line contains a `path:line` pattern that could be
/// opened by `try_open_file_at_line`. Returns a distinctive style
/// (underline + blue) when the pattern matches, or `None` otherwise.
/// The style is applied over the existing value style so the line
/// remains readable.
fn file_line_style(text: &str) -> Option<Style> {
    let trimmed = text.trim();
    if let Some((before, after)) = trimmed.rsplit_once(':')
        && !before.is_empty()
        && after.chars().all(|c| c.is_ascii_digit())
        && looks_like_file_path(before)
    {
        Some(
            Style::default()
                .fg(palette::DEEPSEEK_SKY)
                .add_modifier(Modifier::UNDERLINED),
        )
    } else {
        None
    }
}

/// Apply inline diff highlighting to a single text line.
///
/// Returns the appropriate style for the line based on its prefix:
/// - Lines starting with `+` (after trimming) => `palette::DIFF_ADDED` (green)
/// - Lines starting with `-` (after trimming) => `palette::STATUS_ERROR` (red)
/// - Lines starting with `@@` => `palette::DEEPSEEK_SKY` (cyan/blue)
/// - All other lines => None (use default style)
fn diff_line_style(text: &str) -> Option<Style> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("@@") {
        Some(Style::default().fg(palette::WHALE_ACCENT_PRIMARY))
    } else if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
        Some(Style::default().fg(palette::DIFF_ADDED))
    } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
        Some(Style::default().fg(palette::STATUS_ERROR))
    } else {
        None
    }
}

fn render_output_row(
    lines: &mut Vec<Line<'static>>,
    label: Option<&str>,
    row: &OutputRow,
    width: u16,
) {
    // #374: apply file:line highlighting when the row text contains
    // a `path:line` pattern. Diff style takes precedence (colored
    // prefix lines should stay colored), but if no diff style matched,
    // check for a file:line pattern and highlight it distinctively.
    let diff_style = diff_line_style(&row.text);
    let file_style = file_line_style(&row.text);
    let value_style = diff_style.or(file_style).unwrap_or_else(tool_value_style);
    if row.intact {
        lines.push(render_card_detail_line_single(
            label,
            &row.text,
            value_style,
        ));
    } else {
        lines.extend(render_card_detail_line(
            label,
            &row.text,
            value_style,
            width,
        ));
    }
}

fn wrap_plain_line(line: &str, style: Style, width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for part in wrap_text(line, width.max(1) as usize) {
        lines.push(Line::from(Span::styled(part, style)));
    }
    lines
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        let tentative = if current.is_empty() {
            ch.to_string()
        } else {
            let mut t = current.clone();
            t.push(ch);
            t
        };

        if UnicodeWidthStr::width(tentative.as_str()) > width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }

        current.push(ch);
    }

    lines.push(current);

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn status_symbol(started_at: Option<Instant>, status: ToolStatus, low_motion: bool) -> String {
    match status {
        ToolStatus::Running => {
            if low_motion {
                return TOOL_RUNNING_SYMBOLS[0].to_string();
            }
            let elapsed_ms = started_at.map_or_else(
                || {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |duration| duration.as_millis())
                },
                |t| t.elapsed().as_millis(),
            );
            let cycle = u128::from(TOOL_STATUS_SYMBOL_MS);
            let idx = elapsed_ms
                .checked_div(cycle)
                .map_or(0, |d| d % (TOOL_RUNNING_SYMBOLS.len() as u128));
            TOOL_RUNNING_SYMBOLS[usize::try_from(idx).unwrap_or_default()].to_string()
        }
        ToolStatus::Success | ToolStatus::Hydrated => TOOL_DONE_SYMBOL.to_string(),
        ToolStatus::Failed => TOOL_FAILED_SYMBOL.to_string(),
    }
}

fn details_affordance_line(text: &str, style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            TRANSCRIPT_RAIL.to_string(),
            Style::default().fg(palette::TEXT_DIM),
        ),
        Span::styled(text.to_string(), style),
    ])
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars().take(max_len.saturating_sub(3)) {
        out.push(ch);
    }
    out.push_str("...");
    out
}

fn user_label_style() -> Style {
    Style::default().fg(palette::USER_BODY)
}

fn user_body_style() -> Style {
    Style::default().fg(palette::USER_BODY)
}

/// Style for the assistant glyph (`●`). When the cell is streaming and
/// motion is allowed, the foreground pulses on a 2s cycle between 30% and
/// 100% brightness — the only deliberately animated element in a calm
/// transcript. When idle (or low_motion is on) it sits at the full DeepSeek
/// sky color so finished turns read as solid rather than dim.
fn assistant_label_style_for(streaming: bool, low_motion: bool) -> Style {
    let color = if streaming && !low_motion {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        palette::pulse_brightness(palette::DEEPSEEK_SKY, now_ms)
    } else {
        palette::DEEPSEEK_SKY
    };
    Style::default().fg(color)
}

fn system_label_style() -> Style {
    Style::default().fg(palette::TEXT_DIM)
}

fn message_body_style() -> Style {
    Style::default().fg(palette::TEXT_PRIMARY)
}

fn system_body_style() -> Style {
    Style::default().fg(palette::TEXT_MUTED).italic()
}

/// Label glyph for an error cell. `Critical`/`Error` get the loudest marker;
/// `Warning` is softer; `Info` is neutral. Kept as ASCII so it survives any
/// terminal font fallback.
fn error_label_text(severity: crate::error_taxonomy::ErrorSeverity) -> &'static str {
    match severity {
        crate::error_taxonomy::ErrorSeverity::Critical
        | crate::error_taxonomy::ErrorSeverity::Error => "Error",
        crate::error_taxonomy::ErrorSeverity::Warning => "Warn",
        crate::error_taxonomy::ErrorSeverity::Info => "Info",
    }
}

/// Label color for an error cell — drives the leading rail glyph.
fn error_label_style(severity: crate::error_taxonomy::ErrorSeverity) -> Style {
    let color = match severity {
        crate::error_taxonomy::ErrorSeverity::Critical
        | crate::error_taxonomy::ErrorSeverity::Error => palette::STATUS_ERROR,
        crate::error_taxonomy::ErrorSeverity::Warning => palette::STATUS_WARNING,
        crate::error_taxonomy::ErrorSeverity::Info => palette::TEXT_DIM,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

/// Body color for an error cell — softer than the label so the rail draws
/// the eye but the prose stays readable.
fn error_body_style(severity: crate::error_taxonomy::ErrorSeverity) -> Style {
    let color = match severity {
        crate::error_taxonomy::ErrorSeverity::Critical
        | crate::error_taxonomy::ErrorSeverity::Error => palette::STATUS_ERROR,
        crate::error_taxonomy::ErrorSeverity::Warning => palette::STATUS_WARNING,
        crate::error_taxonomy::ErrorSeverity::Info => palette::TEXT_MUTED,
    };
    Style::default().fg(color)
}

fn thinking_style() -> Style {
    Style::default().fg(palette::TEXT_REASONING)
}

fn render_tool_header(
    title: &str,
    state: &str,
    status: ToolStatus,
    started_at: Option<Instant>,
    low_motion: bool,
) -> Line<'static> {
    let family = crate::tui::widgets::tool_card::tool_family_for_title(title);
    render_tool_header_with_family(family, state, status, started_at, low_motion)
}

fn render_tool_header_with_summary(
    title: &str,
    summary: Option<&str>,
    state: &str,
    status: ToolStatus,
    started_at: Option<Instant>,
    low_motion: bool,
) -> Line<'static> {
    let family = crate::tui::widgets::tool_card::tool_family_for_title(title);
    render_tool_header_with_family_and_summary(
        family, summary, state, status, started_at, low_motion,
    )
}

/// Render a tool-card header with an explicit verb family. Lets callers
/// (e.g. `GenericToolCell`) bypass the legacy title→family mapping when
/// they already know the actual tool name.
fn render_tool_header_with_family(
    family: crate::tui::widgets::tool_card::ToolFamily,
    state: &str,
    status: ToolStatus,
    started_at: Option<Instant>,
    low_motion: bool,
) -> Line<'static> {
    render_tool_header_with_family_and_summary(family, None, state, status, started_at, low_motion)
}

fn render_tool_header_with_family_and_summary(
    family: crate::tui::widgets::tool_card::ToolFamily,
    summary: Option<&str>,
    state: &str,
    status: ToolStatus,
    started_at: Option<Instant>,
    low_motion: bool,
) -> Line<'static> {
    // For long-running tools, append elapsed seconds so the user can see the
    // call isn't stuck. Threshold matches the eye's "did this hang?" reflex
    // — under 3s we stay quiet so quick reads/greps don't visually churn.
    let state_owned: String = if state == "running"
        && status == ToolStatus::Running
        && let Some(started) = started_at
    {
        running_status_label_with_elapsed(started.elapsed().as_secs())
    } else {
        state.to_string()
    };

    let glyph = crate::tui::widgets::tool_card::family_glyph(family);
    let verb = crate::tui::widgets::tool_card::family_label(family);

    let mut spans = vec![
        Span::styled(
            format!("{} ", status_symbol(started_at, status, low_motion)),
            Style::default().fg(tool_state_color(status)),
        ),
        Span::styled(
            format!("{glyph} "),
            Style::default().fg(tool_state_color(status)),
        ),
        Span::styled(verb.to_string(), tool_title_style()),
        Span::styled(" ", Style::default()),
        Span::styled(state_owned, tool_status_style(status)),
    ];

    if let Some(summary) = summary.and_then(normalize_header_summary) {
        spans.push(Span::styled(" · ", Style::default().fg(palette::TEXT_DIM)));
        spans.push(Span::styled(
            truncate_text(&summary, TOOL_HEADER_SUMMARY_LIMIT),
            Style::default().fg(palette::TEXT_MUTED),
        ));
    }

    Line::from(spans)
}

fn normalize_header_summary(summary: &str) -> Option<String> {
    let normalized = summary
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Build the "running" label with an elapsed-seconds badge for long-running
/// tools. Below 3s the badge is suppressed to avoid visual churn for tools
/// that resolve in milliseconds; at 3s and beyond the badge appears and ticks
/// every second the tool stays in flight.
pub(crate) fn running_status_label_with_elapsed(elapsed_secs: u64) -> String {
    if elapsed_secs < 3 {
        "running".to_string()
    } else {
        format!("running ({elapsed_secs}s)")
    }
}

fn render_card_detail_line(
    label: Option<&str>,
    value: &str,
    value_style: Style,
    width: u16,
) -> Vec<Line<'static>> {
    let label_text = label.map(|text| format!("{text}:"));
    let prefix_width = UnicodeWidthStr::width(TRANSCRIPT_RAIL)
        + label_text.as_deref().map_or(0, UnicodeWidthStr::width)
        + usize::from(label.is_some());
    let content_width = usize::from(width).saturating_sub(prefix_width).max(1);

    let mut lines = Vec::new();
    for (idx, part) in wrap_text(value, content_width).into_iter().enumerate() {
        let mut spans = vec![Span::styled(
            TRANSCRIPT_RAIL.to_string(),
            Style::default().fg(palette::TEXT_DIM),
        )];
        if idx == 0 {
            if let Some(label_text) = label_text.as_deref() {
                spans.push(Span::styled(
                    label_text.to_string(),
                    tool_detail_label_style(),
                ));
                spans.push(Span::raw(" "));
            }
        } else if let Some(label_text) = label_text.as_deref() {
            spans.push(Span::raw(
                " ".repeat(UnicodeWidthStr::width(label_text) + 1),
            ));
        }
        spans.push(Span::styled(part, value_style));
        lines.push(Line::from(spans));
    }
    lines
}

fn render_card_detail_line_single(
    label: Option<&str>,
    value: &str,
    value_style: Style,
) -> Line<'static> {
    let label_text = label.map(|text| format!("{text}:"));
    let mut spans = vec![Span::styled(
        TRANSCRIPT_RAIL.to_string(),
        Style::default().fg(palette::TEXT_DIM),
    )];
    if let Some(label_text) = label_text {
        spans.push(Span::styled(label_text, tool_detail_label_style()));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(value.to_string(), value_style));
    Line::from(spans)
}

fn tool_title_style() -> Style {
    active_theme().tool_title_style()
}

fn tool_status_style(status: ToolStatus) -> Style {
    active_theme().tool_status_style(status)
}

fn tool_detail_label_style() -> Style {
    active_theme().tool_label_style()
}

fn tool_state_color(status: ToolStatus) -> Color {
    active_theme().tool_status_color(status)
}

fn tool_status_label(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::Running => "running",
        ToolStatus::Success => "done",
        ToolStatus::Hydrated => "tool loaded - retry required",
        ToolStatus::Failed => "issue",
    }
}

fn tool_value_style() -> Style {
    active_theme().tool_value_style()
}

fn thinking_visual_state(streaming: bool, duration_secs: Option<f32>) -> ThinkingVisualState {
    if streaming {
        ThinkingVisualState::Live
    } else if duration_secs.is_some() {
        ThinkingVisualState::Done
    } else {
        ThinkingVisualState::Idle
    }
}

fn thinking_status_label(state: ThinkingVisualState) -> &'static str {
    match state {
        ThinkingVisualState::Live => "live",
        ThinkingVisualState::Done => "done",
        ThinkingVisualState::Idle => "idle",
    }
}

fn thinking_title_style() -> Style {
    Style::default()
        .fg(palette::TEXT_SOFT)
        .add_modifier(Modifier::BOLD)
}

fn thinking_status_style(state: ThinkingVisualState) -> Style {
    Style::default().fg(match state {
        ThinkingVisualState::Live => palette::ACCENT_REASONING_LIVE,
        ThinkingVisualState::Done => palette::TEXT_DIM,
        ThinkingVisualState::Idle => palette::TEXT_DIM,
    })
}

fn thinking_meta_style() -> Style {
    Style::default().fg(palette::TEXT_DIM)
}

fn thinking_state_accent(state: ThinkingVisualState) -> Color {
    match state {
        ThinkingVisualState::Live => palette::ACCENT_REASONING_LIVE,
        ThinkingVisualState::Done => palette::TEXT_DIM,
        ThinkingVisualState::Idle => palette::TEXT_DIM,
    }
}

// === Cached colour depth ===

/// Once-initialised colour depth for the terminal session. Avoids re-reading
/// `COLORTERM` / `TERM` env vars on every frame.
static COLOR_DEPTH: std::sync::OnceLock<palette::ColorDepth> = std::sync::OnceLock::new();

fn cached_color_depth() -> palette::ColorDepth {
    *COLOR_DEPTH.get_or_init(palette::ColorDepth::detect)
}

/// Parse `path:line` patterns from `text` and open the file at the given line
/// in the user's preferred editor (`$VISUAL` / `$EDITOR` / `vim`).
///
/// Scans lines of `text` for patterns like `src/main.rs:42`. Resolves the path
/// relative to `workspace` (if not absolute) and opens the editor. Returns
/// `true` if at least one file was opened successfully.
pub fn try_open_file_at_line(text: &str, workspace: &Path) -> bool {
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("EDITOR")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| "vim".to_string());

    let mut any_opened = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some((before, after)) = trimmed.rsplit_once(':')
            && after.chars().all(|c| c.is_ascii_digit())
        {
            let line_num: u32 = after.parse().unwrap_or(1);
            let path_str = before.trim();
            if !path_str.is_empty() && looks_like_file_path(path_str) {
                let abs_path = if Path::new(path_str).is_absolute() {
                    PathBuf::from(path_str)
                } else {
                    workspace.join(path_str)
                };
                if abs_path.is_file()
                    && Command::new(&editor)
                        .arg(format!("+{line_num}"))
                        .arg(&abs_path)
                        .spawn()
                        .is_ok()
                {
                    any_opened = true;
                }
            }
        }
    }
    any_opened
}

/// Heuristic check whether a string looks like a file path (contains a
/// directory separator or a known source file extension).
fn looks_like_file_path(s: &str) -> bool {
    if s.contains('/') || s.contains('\\') {
        return true;
    }
    // Check for a known file extension
    if let Some((_, ext)) = s.rsplit_once('.') {
        let ext = ext.trim();
        matches!(
            ext,
            "rs" | "toml"
                | "md"
                | "sh"
                | "py"
                | "js"
                | "ts"
                | "json"
                | "yaml"
                | "yml"
                | "css"
                | "html"
                | "go"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "java"
                | "kt"
                | "swift"
                | "rb"
                | "php"
                | "lua"
                | "zig"
                | "mod"
                | "sum"
                | "lock"
                | "txt"
                | "ini"
                | "cfg"
                | "conf"
                | "env"
                | "gitignore"
                | "dockerfile"
                | "sql"
                | "r"
                | "ex"
                | "exs"
                | "vue"
                | "svelte"
                | "tsx"
                | "jsx"
                | "scss"
                | "sass"
                | "less"
                | "gradle"
                | "properties"
                | "xml"
                | "proto"
                | "nix"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests;

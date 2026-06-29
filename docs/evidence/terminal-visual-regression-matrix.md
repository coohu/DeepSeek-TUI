# Terminal Visual Regression Matrix

Issue: #3487

This matrix tracks deterministic terminal UI fixtures that should stay legible without provider or network access. It is intentionally focused on objective failures: unreadable contrast, broken borders, clipped key labels, missing panes, replacement characters, and long-row truncation.

## Gate Commands

```sh
cargo test -p deepseek-tui --test palette_audit --locked
cargo test -p deepseek-tui --bin deepseek-tui --locked visual_matrix -- --nocapture
cargo test -p deepseek-tui --bin deepseek-tui --locked selected_provider_row_uses_strong_highlight -- --nocapture
cargo test -p deepseek-tui --bin deepseek-tui --locked config_view_selected_row_uses_muted_selection_highlight -- --nocapture
```

## Matrix

| Surface | Widths | Fixture | Guardrails |
|---------|--------|---------|------------|
| Palette contrast | dark, light | `crates/tui/tests/palette_audit.rs::contrast_guardrails_for_key_ui_pairs` | body, muted, warning, error, selected row, elevated row, and light-palette text pairs meet 4.5:1 contrast |
| `/model` provider selector | narrow-ish, medium | `provider_picker::tests::small_list_render_keeps_selected_provider_visible_after_down_navigation`, `selected_provider_row_uses_strong_highlight` | selected provider remains visible after scroll, selection background is continuous and avoids bright accent backgrounds |
| `/sessions` selector | 72x20, 120x28 | `session_picker::tests::session_picker_visual_matrix_covers_narrow_and_medium_rendering`, `session_picker_selected_row_renders_readable_selection_contrast` | both panes render, borders survive, long CJK titles truncate with ellipsis, no replacement characters, selected row stays visible and keeps readable contrast |
| Settings/config modal | 60x18, 100x24 | `views::tests::localized_config_view_renders_at_narrow_width`, `config_view_selected_row_uses_muted_selection_highlight`, `config_view_keeps_scope_column_aligned_for_long_keys` | localized title survives narrow width, selected row uses muted highlight, long labels and CJK scope column remain aligned |
| Sidebar hotbar/task rows | sidebar unit widths | `sidebar::tests::hotbar_panel_lines_keep_two_fixed_rows_and_hover_status`, `hotbar_panel_slots_handle_empty_partial_and_unknown_config` | fixed rows do not resize, empty/unknown slots render explicit states |
| Transcript/live overlay | 40x10, 48x10, 60x16 | `live_transcript::tests::backtrack_preview_opens_near_latest_user_not_transcript_start`, `cache_reuses_unchanged_cells_across_renders` | overlay renders without provider access, recent turns stay visible, unchanged cells reuse wrap cache |

## Deferred Rows

- Full sub-agent/Fleet progress overlay screenshots remain under #3480 because the current tests assert model rows and live fanout membership but do not yet render a complete narrow terminal shell.
- Approval modal destructive-review semantics are tracked under #3466; visual checks should be added there once the permission copy is finalized.
- Hotbar end-to-end source coverage is tracked under #3401 after MCP, skill, and plugin source adapters land.

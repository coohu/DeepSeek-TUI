# Prompt Mode Matrix

This note records the v0.8.66 prompt-mode audit for #2958. The current
contract is:

- `crates/tui/src/prompts/constitution.md` is the single shared base prompt.
- `crates/tui/src/prompts/modes/*.md` are mode deltas only.
- `crates/tui/src/prompts/approvals/*.md` describe approval policy overlays.
- Plan write blocking is enforced by runtime policy and tool registry setup,
  not by trusting prompt prose.

## Mode Matrix

| Mode | Runtime authority | Shell policy | Approval surface | Model-visible delta |
| --- | --- | --- | --- | --- |
| Agent | User's durable Agent baseline | From Agent baseline | From Agent baseline | Autonomous execution, approval batching, checklist discipline, session longevity |
| Plan | Read-only investigation | None | Suggest UI, with writes blocked by mode/runtime | Investigation first, `update_plan` as handoff, no shell/code execution |
| YOLO | Full authority | Full | Auto | Auto-approved execution plus destructive-action caution |

Runtime anchors:

- `base_policy_for_mode` in `crates/tui/src/tui/app.rs` derives the live
  permission mirrors for Plan, Agent, and YOLO.
- `sandbox_policy_for_mode` and `shell_policy_for_mode` in
  `crates/tui/src/core/engine/tool_setup.rs` keep Plan in a read-only sandbox
  and remove shell tools from Plan turns.
- `Engine::send_user_shell_command` still rejects `exec_shell` directly when
  the active mode is Plan.

## Size Snapshot

Measured on 2026-06-25 with `wc -l -w -c`. Estimated tokens use the existing
conservative rule in `compaction::estimate_text_tokens_conservative`
(`chars / 3`, rounded up); these are not provider-tokenizer exact counts.

| Layer | Lines | Words | Bytes | Est. tokens |
| --- | ---: | ---: | ---: | ---: |
| `constitution.md` | 667 | 5728 | 37301 | 12434 |
| `modes/agent.md` | 33 | 314 | 2059 | 687 |
| `modes/plan.md` | 21 | 216 | 1389 | 463 |
| `modes/yolo.md` | 13 | 120 | 775 | 259 |
| `approvals/suggest.md` | 12 | 131 | 843 | 281 |
| `approvals/never.md` | 12 | 162 | 1022 | 341 |
| `approvals/auto.md` | 11 | 127 | 806 | 269 |

The important audit result is that mode prompts stay under 6 percent of the
base prompt by word count, and approval overlays stay separate. Future prompt
changes should update this table when they intentionally change the contract.

## v0.8.56 Comparison

Measured against the `v0.8.56` tag with the same `wc -l -w -c` command and
the same `chars / 3` conservative token estimate. This comparison covers the
static composed prompt layers that are directly relevant to #2958:

- base prompt layer
- active mode delta
- default approval overlay for that mode (`suggest` for Agent and Plan, `auto`
  for YOLO)

It does not include runtime project context, skills, memory, handoff blocks, or
other per-session dynamic sections.

| Mode | Ref | Static layers | Words | Bytes | Est. tokens | Delta vs v0.8.56 |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Agent | `v0.8.56` | `base.md` + `modes/agent.md` + `approvals/suggest.md` | 4706 | 32063 | 10688 | baseline |
| Agent | current | `constitution.md` + `modes/agent.md` + `approvals/suggest.md` | 6173 | 40203 | 13401 | +2713 |
| Plan | `v0.8.56` | `base.md` + `modes/plan.md` + `approvals/suggest.md` | 4582 | 31208 | 10403 | baseline |
| Plan | current | `constitution.md` + `modes/plan.md` + `approvals/suggest.md` | 6075 | 39533 | 13178 | +2775 |
| YOLO | `v0.8.56` | `base.md` + `modes/yolo.md` + `approvals/auto.md` | 4515 | 30786 | 10262 | baseline |
| YOLO | current | `constitution.md` + `modes/yolo.md` + `approvals/auto.md` | 5975 | 38882 | 12961 | +2699 |

Token-savings result: **0 estimated tokens saved** in the audited static layers
relative to `v0.8.56`. The current Constitution is larger than the old
`base.md`, so this slice records an audit and separation win rather than a size
win. Future prompt slimming should target the shared Constitution or move
stable explanatory text behind cheaper dynamic diagnostics before claiming
Codex-parity token savings.

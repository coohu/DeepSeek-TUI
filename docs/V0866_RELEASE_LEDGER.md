# v0.8.66 Release Ledger

This ledger records the live v0.8.66 release lane as of 2026-06-27. It is a
release-prep artifact, not a tag, GitHub Release, npm publish, or version bump.

## Current Release Candidate

- Main candidate: `5e9f7662f` (`Merge pull request #3706 ...`).
- Workspace/npm version currently remains `0.8.65`; do not bump to `0.8.66`
  without explicit maintainer approval.
- Latest main CI, Nightly, CodeQL, CNB sync, and auto-close workflows were green
  at `5e9f7662f` when this ledger was written.
- `deepseek@0.8.65` is published on npm and was used for ACP registry
  validation.

## Landed v0.8.66 Highlights

- ACP editor adapter hardening: `session/cancel` can abort an in-flight prompt,
  `session/prompt` streams provider text deltas through `session/update`, and
  the external ACP registry submission is open as
  `agentclientprotocol/registry#411` (#3192).
- Token/cache/context discipline: the `scorecard` command provides offline
  token/cache/cost metrics and baseline regression checks (#3388), exec
  stream-json metadata exposes `input_analysis` and
  `visible_final_answer_chars` (#2956, #2957), shell-only benchmark surface
  handling is hardened (#2954), and the default Constitution prompt is slimmer
  with a regression guard (#2953).
- Fleet/WhaleFlow follow-through: Fleet remains the durable sub-agent
  configuration layer while WhaleFlow describes launch plans that select and
  monitor Fleet slots. Launch-shape validation keeps the default workflow under
  100 agents and 5 recursive rings.
- Release and CI hardening: required-check placeholder jobs keep light PRs
  mergeable, stale issue cleanup is reactivated with conservative labels, and
  registry/provider docs were corrected.
- Community PR intake: OpenModel, WeCom Bridge docs/runtime, ask-rules config
  view, provider context-window overrides, direct URL fallback hints, and
  command-registry validation have landed with contributor credit preserved.

## Issue State For Release Completion

### Can Be Considered v0.8.66 Done After This Ledger

- #3388: the release-gate measurement machinery is in-tree. Follow-up benchmark
  runs and long-lived trend storage should continue under #2962 and the
  individual token/cache evidence issues rather than blocking the patch release.
- #3192: DeepSeek-side ACP readiness is validated and the external registry PR
  is open. Keep the issue open only until the external registry accepts or
  rejects `agentclientprotocol/registry#411`.

### Move Out Of v0.8.66

These are meaningful, but too broad or externally gated for this patch:

- #3541 native desktop/Rust companion: v0.9.0 or later.
- #3495 Moraine memory backend: keep moving, but full acceptance requires
  Moraine-side adapter/upstreaming plus `moraine up` + MCP recall verification.
- #3480 TUI information architecture: product epic, not a single release fix.
- #3389/#3399 Hotbar MVP/source adapters: partial foundations landed; remaining
  source-adapter and wizard work belongs in the next UI lane.
- #3205/#2300 Fleet automatic loadout selection: model reference and Fleet
  substrate landed, but automatic capability-based selection still needs the
  refined Fleet/model-lab implementation.
- #2984 OpenAI Codex/ChatGPT OAuth: requires live-account verification and
  usage/quota proof.
- #2024/#2023 RLM routing/log workbench: architecture follow-up.
- #3089 stale backlog cleanup: automation is active; manual consolidation can
  continue after release.

### Keep As Evidence/Needs-Data Follow-Ups

Do not close these from code inspection alone:

- #2953, #2954, #2956, #2957: code/instrumentation landed; paired benchmark
  rows remain follow-up evidence.
- #2962: recurring benchmark trend storage is a larger automation lane.
- #743, #1120, #1177, #1747, #1818, #1732: user reports remain valuable, but
  need current-version session data, request snapshots, or same-task benchmark
  pairs before final closure.

## Release Actions Not Taken

- No `0.8.66` version bump.
- No tag.
- No npm publish.
- No GitHub Release.
- No release artifact push.

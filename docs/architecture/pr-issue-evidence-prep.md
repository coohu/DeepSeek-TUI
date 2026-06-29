# EPIC Evidence Preparation

## EPIC-002 Closure Evidence (Final — Phase 8 complete; ready for PR)

**Epic:** EPIC-002 — Command Single Responsibility Extraction
**Related EPIC:** [#2870](https://github.com/coohu/deepseek-tui/issues/2870)
**Related issues:** [#2791](https://github.com/coohu/deepseek-tui/issues/2791),
[#2851](https://github.com/coohu/deepseek-tui/pull/2851),
[#2887](https://github.com/coohu/deepseek-tui/pull/2887)

This section records final EPIC-002 closure evidence verified during Phase 8
(final checkpoint). All evidence below was collected on the current working
tree by running the documented commands.

### PR References

- Layer 4 (FEAT-006): Core, config, session, and debug command extraction
- Layer 4.1 (FEAT-007): Project, memory, skills, utility, and plugins extraction
- Layer 4.2 (FEAT-008): Registry cleanup, documentation, and full validation

### Acceptance Evidence

| AT ID | Check | Result |
|-------|-------|--------|
| AT-001 | `cargo test -p deepseek-tui acceptance` (epic_acceptance_harness + eval_harness) | ✅ 2 passed (0 failed) |
| AT-002 | `every_registered_command_dispatches_to_a_handler` | ✅ Passed (part of 489 command tests) |
| AT-003 | `every_command_alias_dispatches_to_a_handler` | ✅ Passed (part of 489 command tests) |
| AT-004 | Help/palette/completion surface tests (included in 489 command tests) | ✅ Passed |
| AT-005 | `dispatch_prefers_user_command_over_builtin_with_same_name` | ✅ Passed |
| AT-006 | `hidden_user_commands_still_dispatch_directly` | ✅ Passed |
| AT-007 | `unknown_command_suggests_nearest_match` | ✅ Passed |
| AT-008 | `command_registry_has_unique_names_and_aliases` | ✅ Passed (0 duplicate names/aliases) |
| AT-009 | `command_ownership_contract_is_enforced` | ✅ Passed (9 groups, layered ownership) |
| AT-010 | Cleanup inventory — no undocumented migration paths | ✅ Verified (all items permanent exceptions or absent) |
| AT-011 | Final closure matrix (this document) | ✅ Complete |

### Permanent Exceptions

| Exception | Rationale |
|-----------|-----------|
| Config group-local metadata | Config `mod.rs` keeps 11 `CommandInfo` statics and dispatch — permanent structure, not cleanup scope |
| Debug group-local metadata | Debug `mod.rs` keeps 11 `CommandInfo` statics and dispatch — permanent structure, not cleanup scope |
| `/jihua`, `/zidong` | Chinese-language back-compat aliases for `/mode` — predate group-owned registry |
| `/slop`, `/canzha` | Typed-only aliases for `/debt` — predate group-owned registry |
| `/set`, `/deepseek` migration hints | Retired commands, direct typed guidance only, excluded from registry/completion |
| `$skill` prefix | Non-slash compatibility syntax, predates EPIC-002 |
| Skill-name fallback | Slash commands fall back to skill dispatch after built-ins and user commands |
| `command_runs_directly()` palette list | UI policy decision, not registry metadata |
| Public re-export bridge paths | Long-standing public API compatibility |
| User-command compatibility loaders | `.deepseek`, `.claude`, `.cursor` directories — user-command scope, not built-in cleanup |
| `#[allow(clippy::module_inception)]` | Intentional structure for same-named group and child modules |

### Validation

- `cargo fmt --all -- --check` — clean
- `cargo check -p deepseek-tui` — clean (no errors, no warnings)
- `cargo test -p deepseek-tui commands::` — 489 passed (0 failed)
- `cargo test -p deepseek-tui acceptance` — 2 passed (epic_acceptance_harness: 1 scenario, 3 steps; eval_harness: 1 test)
- `cargo test --workspace` — 5344 passed, 1 failed (known flaky: `run_verifiers_background_starts_shell_jobs_and_returns_task_ids`; passes in isolation — pre-existing papercut, not a FEAT-008 regression), 2 ignored
- `git diff --check` — clean (both repos)
- Orphaned file check — no orphaned `.rs` files

## FEAT-008 PR Summary Draft

**Title:** Layer 4.2: Registry cleanup, docs, and full validation (FEAT-008)

```markdown
Refs #2870.

## Summary

FEAT-008 completes EPIC-002 (Command Single Responsibility Extraction) by
removing transition-only command scaffolding, validating command and alias
uniqueness, updating source-verified command architecture documentation, and
preparing auditable EPIC closure evidence. This is Layer 4.2 (the final cleanup
and validation layer).

## Changes

- No temporary adapters, duplicate command lists, or migration-only dispatch
  paths remain — all §3.2 inventory items confirmed as permanent exceptions or
  not present after Phase 3 source verification.
- Command registration ownership follows the final layered model:
  top-level group registration → group-owned command modules → command-level
  metadata and behavior.
- Architecture documentation (`docs/architecture/command-dispatch.md`) updated
  to reflect the finalized dispatch flow and permanent exceptions.
- PR/issue evidence document (`docs/architecture/pr-issue-evidence-prep.md`)
  prepared for EPIC-002 closure.

## Gherkin / Acceptance Coverage

- `tests/epic_acceptance_harness.rs` — 1 scenario, 3 steps (AT-001)
- `tests/core_session_command_extraction.rs` — 1 scenario, 4 steps (AT-002/003)
- `tests/eval_smoke_acceptance.rs` — 1 scenario, 4 steps (not AT-004 evidence)
- `tests/plugin_e2e_acceptance.rs` — 4 tests (AT-002/003/004 coverage)
- AT-008: `command_registry_has_unique_names_and_aliases` — enforced by test
- AT-009: `command_ownership_contract_is_enforced` — enforced by test
- AT-010: cleanup inventory verified — no undocumented migration paths

## Validation

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | Clean |
| `cargo check -p deepseek-tui` | Clean (0 errors, 0 warnings) |
| `cargo test -p deepseek-tui commands::` | 489 passed, 0 failed |
| `cargo test -p deepseek-tui acceptance` | 2 passed (epic_acceptance_harness: 1, eval_harness: 1) |
| `cargo test --workspace` | 5344 passed, 1 known-flaky (verifier parallel contention; passes in isolation), 2 ignored |
| `git diff --check` | Clean (both repos) |
| Orphaned file check | No orphaned `.rs` files |
| `git status --porcelain` | Clean (DeepSeek repo) |

Paulo Aboim Pinto
```

---

## EPIC-001 Hunter Replay Evidence

**Target branch:** `hunter/0.8.62-glm-subagents`
**Replay branch:** `feat/replay-epic-001-on-hunter`
**Related EPIC:** [#2870](https://github.com/coohu/deepseek-tui/issues/2870)
**Related issue:** [#2791](https://github.com/coohu/deepseek-tui/issues/2791)

This section records the working PR/issue evidence checklist for replaying
EPIC-001 FEAT-001, FEAT-002, and FEAT-003 onto the Hunter branch.

## Replay Scope

| Feature | Hunter replay decision |
|---------|------------------------|
| FEAT-001 | No raw cherry-pick. Hunter already contains the newer group-owned command tree and trait-backed registry. |
| FEAT-002 | Replayed semantically as `user_registry.rs`, wired into dispatch, palette, and slash completion. Adapted to keep newer Hunter command-state reset behavior. |
| FEAT-003 | Replayed as public architecture and PR/issue evidence docs for the Hunter target. Old release-branch validation claims were not copied. |

## PR Summary Draft

```markdown
## Summary

Replays the completed EPIC-001 command-boundary work onto
`hunter/0.8.62-glm-subagents`.

## Changes

- Keep Hunter's existing trait-backed built-in command registry and nested
  group-owned command tree as the FEAT-001 result.
- Add a dedicated `UserCommandRegistry` boundary for markdown user commands.
- Route user command dispatch, command palette entries, and slash completion
  through the registry.
- Preserve Hunter's newer command-state reset behavior when a user command
  starts, including todos and plan state.
- Preserve empty `allowed-tools` semantics: an explicit empty value blocks all
  tools.
- Add public architecture and PR/issue evidence docs for the Hunter target.

## Validation

- `cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo check -p deepseek-tui`
- `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui commands::`
- `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui command_palette`
- `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui slash_completion`
- `git diff --check`
```

## Issue #2870 Comment Draft

```markdown
EPIC-001 has been replayed onto the Hunter target as a semantic replay rather
than raw cherry-picks.

- FEAT-001: represented by Hunter's current trait-backed registry and
  group-owned command tree.
- FEAT-002: replayed as the user-command registry boundary, adapted to preserve
  current Hunter behavior.
- FEAT-003: replayed as public architecture and evidence docs for the Hunter
  target.

Validation evidence is included in the PR body.

Paulo Aboim Pinto
```

## Validation Results

Record live results here before opening or updating the PR.

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | Pass |
| `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo check -p deepseek-tui` | Pass |
| `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui commands::` | Pass: 456 command tests |
| `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui command_palette` | Pass: 18 tests |
| `CARGO_TARGET_DIR=/tmp/deepseek-hunter-target cargo test -p deepseek-tui slash_completion` | Pass: 17 tests |
| `git diff --check` | Pass |

# EPIC Evidence Preparation

## EPIC-002 Closure Evidence (Draft — Phase 6 In Progress)

**Epic:** EPIC-002 — Command Single Responsibility Extraction
**Related EPIC:** [#2870](https://github.com/Hmbown/CodeWhale/issues/2870)
**Related issues:** [#2791](https://github.com/Hmbown/CodeWhale/issues/2791),
[#2851](https://github.com/Hmbown/CodeWhale/pull/2851),
[#2887](https://github.com/Hmbown/CodeWhale/pull/2887)

This section records draft EPIC-002 closure evidence prepared during Phase 6.
Layer 4.4 (FEAT-008) is currently in Phase 6 code review. Evidence below is
Phase 6 current-state — final pass/fail markers replace these placeholders only
after the Phase 8 final gate.

### PR References

- Layer 4.0 (FEAT-004): Command extraction contract and baseline
- Layer 4.1 (FEAT-005): Core and session command extraction
- Layer 4.2 (FEAT-006): Config and debug command extraction
- Layer 4.3 (FEAT-007): Project, memory, skills, and utility extraction
- Layer 4.4 (FEAT-008): Registry cleanup, documentation, and full validation

### Acceptance Evidence

| AT ID | Check | Result |
|-------|-------|--------|
| AT-001 | `cargo test -p codewhale-tui --test epic_acceptance_harness` | ⬜ Draft (Phase 6 current evidence) |
| AT-002 | `every_registered_command_dispatches_to_a_handler` | ⬜ Draft (Phase 6 current evidence) |
| AT-003 | `every_command_alias_dispatches_to_a_handler` | ⬜ Draft (Phase 6 current evidence) |
| AT-004 | Help/palette/completion surface tests | ⬜ Draft (Phase 6 current evidence) |
| AT-005 | `dispatch_prefers_user_command_over_builtin_with_same_name` | ⬜ Draft (Phase 6 current evidence) |
| AT-006 | `hidden_user_commands_still_dispatch_directly` | ⬜ Draft (Phase 6 current evidence) |
| AT-007 | `unknown_command_suggests_nearest_match` | ⬜ Draft (Phase 6 current evidence) |
| AT-008 | `command_registry_has_unique_names_and_aliases` | ⬜ Draft (Phase 6 current evidence) |
| AT-009 | `command_ownership_contract_is_enforced` | ⬜ Draft (Phase 6 current evidence) |
| AT-010 | Cleanup inventory — no undocumented migration paths | ⬜ Draft (Phase 6 current evidence) |
| AT-011 | Final closure matrix | ⬜ Draft (Phase 6 current evidence — subject to Phase 8 final gate) |

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

- `cargo fmt --all -- --check`
- `cargo check -p codewhale-tui`
- `cargo test -p codewhale-tui commands::tests:: -- --test-threads=1` (60 passed)
- `cargo test -p codewhale-tui command_palette -- --test-threads=1` (21 passed)
- `cargo test -p codewhale-tui slash_completion -- --test-threads=1` (18 passed)
- `cargo test -p codewhale-tui user_registry -- --test-threads=1` (18 passed)
- `cargo test -p codewhale-tui --test epic_acceptance_harness` (1 passed)
- `cargo test -p codewhale-tui --test eval_smoke_acceptance -- --test-threads=1` (1 passed) — eval smoke, not AT-004 command-surface evidence
- `cargo test -p codewhale-tui --test core_session_command_extraction -- --test-threads=1` (1 passed)
- `cargo test -p codewhale-tui --test plugin_e2e_acceptance -- --test-threads=1` (4 passed)
- `git diff --check`

---

## EPIC-001 Hunter Replay Evidence

**Target branch:** `hunter/0.8.62-glm-subagents`
**Replay branch:** `feat/replay-epic-001-on-hunter`
**Related EPIC:** [#2870](https://github.com/Hmbown/CodeWhale/issues/2870)
**Related issue:** [#2791](https://github.com/Hmbown/CodeWhale/issues/2791)

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
- `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo check -p codewhale-tui`
- `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui commands::`
- `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui command_palette`
- `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui slash_completion`
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
| `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo check -p codewhale-tui` | Pass |
| `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui commands::` | Pass: 456 command tests |
| `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui command_palette` | Pass: 18 tests |
| `CARGO_TARGET_DIR=/tmp/codewhale-hunter-target cargo test -p codewhale-tui slash_completion` | Pass: 17 tests |
| `git diff --check` | Pass |

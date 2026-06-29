# Command Dispatch Architecture

**Target branch:** `main`
**Related EPIC:** [#2870](https://github.com/coohu/deepseek-tui/issues/2870)
**Related issue:** [#2791](https://github.com/coohu/deepseek-tui/issues/2791)
**EPIC-002 (Command Single Responsibility Extraction):** Layer 4.x (FEAT-006 through FEAT-008)

This document records the command-dispatch ownership model after the
command-boundary replay landed on `main`, updated through EPIC-002 (command
single responsibility extraction). It reflects the final layered ownership:
top-level group registration, group-owned command registration, and
command-level ownership of metadata and behavior. It is the public reference for the
module boundaries, dispatch precedence, and permanent exceptions that remain
after the command-boundary refactor.

## Dispatch Flow

`commands::execute()` owns the slash-command dispatch gate. The order is
intentional:

| Step | Source | Behavior |
|------|--------|----------|
| 0 | `$skill` compatibility | `$name` is resolved as `/skill name` before slash parsing. |
| 1 | User commands | `user_registry::try_dispatch()` checks workspace and global markdown commands first, so user commands can shadow built-ins. |
| 2 | Permanent compatibility aliases | `/jihua` and `/zidong` route through config mode dispatch; `/slop` and `/canzha` dispatch directly to `/debt`. All predate the group-owned registry and bypass the built-in `CommandRegistry`. |
| 3 | Built-in registry | `CommandRegistry` resolves group-owned built-in commands by canonical name or alias. |
| 4 | Legacy migration hints | Retired commands such as `/set` and `/deepseek` return targeted replacement guidance. |
| 5 | Skills fallback | If no command matches, a skill with the same name may run before unknown-command suggestions are shown. |

## Module Boundaries

| Module | Responsibility |
|--------|----------------|
| `crates/tui/src/commands/mod.rs` | Central dispatch gate, registry initialization, public command lookup helpers, and unknown-command suggestions. |
| `crates/tui/src/commands/traits.rs` | Built-in command metadata, trait-backed command objects, command groups, and registry lookup. |
| `crates/tui/src/commands/groups/` | Group-owned built-in command areas. Each group owns its command metadata and handlers. |
| `crates/tui/src/commands/user_registry.rs` | User-command registry boundary: markdown metadata, aliases, hidden entries, validation errors, dispatch state resets, and shadowing behavior. |
| `crates/tui/src/commands/user_commands.rs` | Lower-level file scanning, frontmatter parsing, allowed-tools parsing, and template substitution used by the registry. |
| `crates/tui/src/tui/command_palette.rs` | Palette entries for built-ins and visible user commands, with user commands shadowing built-ins. |
| `crates/tui/src/tui/widgets/mod.rs` | Slash completion, user-command metadata display, and alias-shadowing behavior. |

## Built-In Command Groups

| Group | Scope |
|-------|-------|
| `core` | Help, model/provider selection, queue, hooks, subagents, links, feedback, voice, and core navigation. |
| `config` | Config, settings, status surfaces, mode, theme, trust, logout, and related settings commands. |
| `debug` | Token/cost introspection, cache, system/context, diff/edit, undo, and retry. |
| `memory` | Persistent memory and notes. |
| `plugins` | Plugin discovery, listing, and per-plugin metadata detail display. |
| `project` | Project initialization, sharing, LSP, and goal/hunt commands. |
| `session` | Rename, save, fork/new/load sessions, compaction, purge, relay, and export. |
| `skills` | Skill listing, execution, review, and restore. |
| `utility` | Attachments, tasks/jobs, MCP, and network. |

## User Commands

User commands are markdown files loaded from these locations in precedence
order:

1. `<workspace>/.deepseek/commands/`
2. `<workspace>/.deepseek/commands/`
3. `<workspace>/.claude/commands/`
4. `<workspace>/.cursor/commands/`
5. `~/.deepseek/commands/`
6. `~/.deepseek/commands/`

Supported frontmatter fields:

| Field | Meaning |
|-------|---------|
| `description` | Work objective and UI description. |
| `argument-hint` | Palette/completion hint for expected arguments. |
| `allowed-tools` | Restricts command execution tools. An explicit empty value blocks all tools. |
| `pausable` | Marks the command as pause/resume capable. |
| `alias` / `aliases` | Additional user-command names that can shadow built-in aliases. |
| `hidden` | Hides the command from palette/completion while allowing direct dispatch. |

Dispatch through `user_registry` resets stale command state before sending the
new command body: hunt objective fields, token/time counters, continuation
count, allowed tools, pause state, todos, and plan state.

## Permanent Exceptions

| Exception | Rationale |
|-----------|-----------|
| `/jihua`, `/zidong`, `/slop`, `/canzha` | Backward-compatible dispatch aliases that predate the group-owned registry. `/jihua` and `/zidong` route through config mode dispatch; `/slop` and `/canzha` dispatch directly to `/debt`. |
| `/set` and `/deepseek` migration hints | Retired commands kept only as direct typed guidance. They are excluded from registry and autocomplete. |
| `#[allow(clippy::module_inception)]` in matching group modules | Group directories intentionally contain same-named child modules such as `core/core.rs`. |
| `user_commands.rs` lower layer | The registry owns runtime behavior, while this module remains the shared filesystem and parser layer. |
| `#[cfg(test)]` helpers in `user_commands.rs` | Deferred test migration compatibility while registry-specific tests are added. |

## EPIC-002 Completion Status (Phase 8 complete; ready for PR)

EPIC-002 (Command Single Responsibility Extraction) extracted commands for
all 9 command groups through Layer 4.x sublayers. Layer 4.2 (FEAT-008) is
complete with final validation evidence recorded.

| Layer | FEAT | Title | Status |
|---|---|---|---|
| 4 | FEAT-006 | Core, Config, Session, and Debug Command Extraction | Complete |
| 4.1 | FEAT-007 | Project, Memory, Skills, Utility, and Plugins Extraction | Complete |
| 4.2 | FEAT-008 | Registry Cleanup, Documentation, and Full Validation | Complete |

### Current Evidence (Draft — subject to final verification)

## Replay Status (EPIC-001)

FEAT-001's group-owned built-in command direction is represented on `main` by
the newer trait-backed registry and nested group tree. FEAT-002 is replayed as
the dedicated user-command registry boundary. FEAT-003 is replayed as public
architecture and PR/issue evidence documentation, updated for the current
`main` target instead of the old `release/v0.8.60` branch.

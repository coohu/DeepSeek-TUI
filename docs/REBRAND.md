# Rebrand: DeepSeek TUI → CodeWhale

Starting with **v0.8.41**, this project ships under a new name: `deepseek`.

This document explains what changed, what didn't, and how to migrate. None of the
DeepSeek provider integration changed — only the local CLI / TUI brand.

## TL;DR

```bash
# 1. Uninstall the old wrapper or binaries.
npm uninstall -g       # or cargo uninstall -cli 
                                    # or brew uninstall 

# 2. Install under the new name.
npm install -g deepseek            # or cargo install deepseek-cli deepseek-tui --locked
                                    # or brew install  (Homebrew tap still
                                    #     uses the legacy name during the transition;
                                    #     it installs the new binaries underneath.)

# 3. Run with the new command.
deepseek doctor
deepseek
```

Your `~/.deepseek/config.toml`, `~/.deepseek/sessions/`, `~/.deepseek/skills/`,
`~/.deepseek/tasks/`, and `~/.deepseek/mcp.json` are untouched. Existing
`DEEPSEEK_*` environment variables continue to work.

## What got renamed

| Surface | Before | After |
|---|---|---|
| CLI dispatcher binary | `deepseek` | `deepseek` |
| TUI runtime binary | `` | `deepseek-tui` |
| npm wrapper package | `` | `deepseek` |
| Crates.io crates | `-cli` / `` / `deepseek-*` | `deepseek-cli` / `deepseek-tui` / `deepseek-*` |
| Release assets | `deepseek-<platform>` / `-<platform>` | `deepseek-<platform>` / `deepseek-tui-<platform>` |
| Checksum manifest | `deepseek-artifacts-sha256.txt` | `deepseek-artifacts-sha256.txt` |

## What did NOT change

Anything that targets the DeepSeek provider API stays exactly as it was:

- **Environment variables**: `DEEPSEEK_API_KEY`, `DEEPSEEK_BASE_URL`,
  `DEEPSEEK_MODEL`, `DEEPSEEK_PROVIDER`, `DEEPSEEK_PROFILE`, `DEEPSEEK_YOLO`,
  `DEEPSEEK_LOG_LEVEL`, plus the existing `DEEPSEEK_TUI_*` runtime knobs
  (`DEEPSEEK_TUI_BIN`, `DEEPSEEK_TUI_RELEASE_BASE_URL`, etc.). They're kept
  for backward compatibility; renaming them would break every shell rc on
  the planet.
- **Model IDs**: `deepseek-v4-pro`, `deepseek-v4-flash`, and the legacy
  aliases `deepseek-chat` and `deepseek-reasoner`.
- **Hosts**: `api.deepseek.com` (global) and `api.deepseeki.com` (China
  fallback).
- **Config directory**: `~/.deepseek/`. Renaming this would invalidate
  every existing install's saved API key, sessions, skills, MCP config,
  and audit log.
- **GitHub repository URL**: `https://github.com/Hmbown/CodeWhale`.
  The old `Hmbown/DeepSeek-TUI` URL redirects there during the transition.
- **Homebrew tap and formula** (`Hmbown/homebrew-`): still
  installs by the legacy name during the transition. The tap's formula
  will be flipped to the new names in a follow-up.
- **Docker image**: `ghcr.io/hmbown/deepseek`.

## Deprecation shims (through v0.8.x)

To keep existing shell aliases, scripts, and CI working through the rename,
v0.8.41 and later v0.8.x releases ship **deprecation shims**:

- A `deepseek` binary that prints a one-line warning to stderr and forwards
  argv to `deepseek`.
- A `` binary that does the same for `deepseek-tui`.
- An `npm` package at `@0.8.x` with no `bin` and a postinstall
  that prints a clear rename notice.

These shims will be removed in **v0.9.0**. Please migrate before then.

## Migrating in practice

### npm

```bash
npm uninstall -g 
npm install -g deepseek
```

### Cargo

```bash
cargo uninstall -cli  2>/dev/null || true
cargo install deepseek-cli deepseek-tui --locked
```

Or in a checkout:

```bash
cargo install --path crates/cli --locked --force
cargo install --path crates/tui --locked --force
```

### Homebrew

The tap formula still installs `` during the transition.
Existing `brew install ` invocations continue to work and land
the new binaries underneath the legacy formula name. The formula and tap
repo will follow up with their own rename.

### Manual / GitHub Releases

`v0.8.41` Releases attach **both** the canonical `deepseek-*` /
`deepseek-tui-*` assets and the legacy `deepseek-*` / `-*`
shim assets. Existing `deepseek update` invocations on v0.8.40 keep working;
they land you on the deprecation shim, which then prompts the install of
`deepseek`.

A second checksum manifest, `deepseek-artifacts-sha256.txt`, is attached as
an alias of `deepseek-artifacts-sha256.txt` so v0.8.40's hardcoded lookup
still verifies.

## Why the name change

CodeWhale is a shorter, terminal-friendlier handle for the same terminal
coding agent and the longer-term product direction: a DeepSeek-first agentic
terminal for open source and open-weight coding models. The project name,
command names, package names, release assets, Docker image, and CNB mirror move
to CodeWhale; the official DeepSeek provider, model IDs, env vars, and
`~/.deepseek/` config surface remain first-class.

## Reporting issues with the rename

If your install broke during the migration, please open an issue at
<https://github.com/Hmbown/CodeWhale/issues> and include:

- The output of `deepseek --version` (or `deepseek --version` if you're
  still on the shim).
- Which install path you used (npm, cargo, brew, manual).
- The exact command you ran and the full error output.

We'll prioritize migration regressions.

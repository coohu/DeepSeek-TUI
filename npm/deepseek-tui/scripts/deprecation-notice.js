#!/usr/bin/env node

const notice = [
  "",
  "  ╭───────────────────────────────────────────────────────────────────╮",
  "  │                                                                   │",
  "  │   has been renamed to `deepseek`.                    │",
  "  │                                                                   │",
  "  │  Please uninstall this package and install deepseek instead:     │",
  "  │                                                                   │",
  "  │    npm uninstall -g                                   │",
  "  │    npm install -g deepseek                                       │",
  "  │                                                                   │",
  "  │  deepseek ships the same `deepseek` and `deepseek-tui`         │",
  "  │  binaries plus deprecation shims under the old names. See:        │",
  "  │  https://github.com/Hmbown/CodeWhale/blob/main/docs/REBRAND.md │",
  "  │                                                                   │",
  "  ╰───────────────────────────────────────────────────────────────────╯",
  "",
].join("\n");

process.stderr.write(notice);

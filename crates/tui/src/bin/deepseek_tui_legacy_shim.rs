//! Legacy `` alias.
//!
//! Forwards argv to the `deepseek-tui` runtime and prints a one-line
//! deprecation notice to stderr on each invocation. This binary exists
//! for one release cycle to give existing installs a smooth path to the
//! new name; it will be removed in v0.9.0. See `docs/REBRAND.md` for the
//! full migration story.

use std::env;
use std::process::Command;

fn main() {
    eprintln!(
        "warning: `` is deprecated; run `deepseek-tui` (or `deepseek`) instead. \
         This alias will be removed in v0.9.0."
    );
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let status = match Command::new("deepseek-tui").args(&args).status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "error: failed to spawn `deepseek-tui`: {e}. Is it on PATH? \
                 Install with `cargo install deepseek-tui` or via npm/Homebrew."
            );
            std::process::exit(127);
        }
    };
    std::process::exit(status.code().unwrap_or(1));
}

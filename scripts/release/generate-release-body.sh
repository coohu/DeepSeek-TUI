#!/usr/bin/env bash
# Generate the GitHub Release body for a tag.
#
# Usage: generate-release-body.sh <vX.Y.Z> [path/to/CHANGELOG.md]
#
# The install/verify sections are static; the release notes and contributor
# credits come from the CHANGELOG section for the version, so they can never
# drift the way a hand-edited workflow body does.
set -euo pipefail

tag="${1:?usage: $0 <vX.Y.Z> [CHANGELOG.md]}"
changelog="${2:-CHANGELOG.md}"
version="${tag#v}"

section="$(awk -v version="${version}" '
  index($0, "## [" version "]") == 1 { in_section = 1; next }
  in_section && /^## \[/ { exit }
  in_section { print }
' "${changelog}")"

cat <<EOF
> **DeepSeek** is the canonical project, command, npm package, and
> release-asset name. The legacy npm package \`deepseek-tui\` is
> deprecated and receives no further releases. Users coming from
> v0.8.x legacy \`deepseek\` / \`deepseek-tui\` names should migrate
> with \`docs/REBRAND.md\`.

## Install

### Recommended — npm (one command, both binaries)

\`\`\`bash
npm install -g deepseek
\`\`\`

The wrapper downloads the matched runtime binaries from this Release and places them in the same directory.

### Docker / GHCR

\`\`\`bash
docker run --rm -it \\
  -e DEEPSEEK_API_KEY="\$DEEPSEEK_API_KEY" \\
  -v ~/.deepseek:/home/deepseek/.deepseek \\
  ghcr.io/hmbown/deepseek:${tag}
\`\`\`

The image ships the \`deepseek\` dispatcher, \`codew\` shim, and \`deepseek-tui\` runtime. The \`latest\` tag is also updated on release.

### Cargo (Linux / macOS)

\`\`\`bash
cargo install deepseek-cli deepseek-tui --locked
\`\`\`

Both crates are required — \`deepseek-cli\` produces the \`deepseek\` dispatcher and \`codew\` shim, while \`deepseek-tui\` produces the interactive runtime that the dispatcher delegates to. Installing only one crate will fail at runtime with a \`MISSING_COMPANION_BINARY\` error.

### Manual download — platform archives (recommended)

Each archive below contains the \`deepseek\` dispatcher, \`codew\` shim, and \`deepseek-tui\` runtime, plus an install script:

| Platform | Archive | Install script |
|---|---|---|
| Linux x64 | \`deepseek-linux-x64.tar.gz\` | \`install.sh\` |
| Linux ARM64 | \`deepseek-linux-arm64.tar.gz\` | \`install.sh\` |
| Linux RISC-V | \`deepseek-linux-riscv64.tar.gz\` | \`install.sh\` |
| macOS x64 | \`deepseek-macos-x64.tar.gz\` | \`install.sh\` |
| macOS ARM | \`deepseek-macos-arm64.tar.gz\` | \`install.sh\` |
| Windows x64 (installer) | \`CodeWhaleSetup.exe\` | NSIS setup |
| Windows x64 | \`deepseek-windows-x64.zip\` | \`install.bat\` |
| Windows x64 (portable) | \`deepseek-windows-x64-portable.zip\` | — |

**Unix (Linux / macOS):**
\`\`\`bash
tar xzf deepseek-<platform>.tar.gz
cd deepseek-<platform>
./install.sh
\`\`\`

**Windows:**
- For the installer path, run \`CodeWhaleSetup.exe\`; it installs \`deepseek.exe\`, \`codew.exe\`, and \`deepseek-tui.exe\` under \`%LOCALAPPDATA%\\Programs\\DeepSeek\\bin\` and adds that directory to the current-user PATH.
- Extract \`deepseek-windows-x64.zip\`
- Run \`install.bat\` (copies to \`%USERPROFILE%\\bin\`)
- Add \`%USERPROFILE%\\bin\` to your PATH

The **portable** Windows archive skips the install script — extract and run from any directory. The NSIS installer is currently unsigned and may trigger Windows SmartScreen until a signing certificate is wired into the release pipeline.

Each platform also has **bare, unarchived** binaries attached below (\`deepseek-<platform>\`, \`codew-<platform>\`, and \`deepseek-tui-<platform>\`) — the npm wrapper and the in-app \`deepseek update\` download the matched runtime binaries, whereas the \`.tar.gz\` / \`.zip\` archives above are the recommended manual download and additionally bundle an install script. The legacy npm package \`deepseek-tui\` is deprecated and is not republished. For migration from v0.8.x legacy binary names, see \`docs/REBRAND.md\`.

### Verify (recommended)

Download the checksum manifests from this Release and verify:

\`\`\`bash
# Linux — archive bundles
sha256sum -c deepseek-bundles-sha256.txt

# Linux — individual binaries
sha256sum -c deepseek-artifacts-sha256.txt

# macOS
shasum -a 256 -c deepseek-bundles-sha256.txt
shasum -a 256 -c deepseek-artifacts-sha256.txt
\`\`\`

## What's in ${tag}
EOF

if [[ -n "${section}" ]]; then
  printf '%s\n' "${section}"
else
  printf '%s\n' "See the changelog link below for this release's notes."
fi

cat <<EOF

Contributor credits for this release live in the changelog entry above —
thank you to everyone whose reports, PRs, reviews, and reproductions shaped it.

See [CHANGELOG.md](https://github.com/coohu/deepseek-tui/blob/main/CHANGELOG.md) for full notes and [docs/CHANGELOG_ARCHIVE.md](https://github.com/coohu/deepseek-tui/blob/main/docs/CHANGELOG_ARCHIVE.md) for older releases.
EOF

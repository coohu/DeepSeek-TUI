"use client";

import { useEffect, useState } from "react";
import { InstallCodeBlock } from "./install-code-block";

type Arch = "macos-arm64" | "macos-x64" | "linux-x64" | "linux-arm64" | "linux-riscv64" | "windows-x64";

const SNIPPETS: Record<Arch, string> = {
  "macos-arm64": `curl -fsSL -O https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
curl -fsSL -o deepseek \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-macos-arm64
curl -fsSL -o deepseek-tui \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-tui-macos-arm64
grep -E ' (deepseek|deepseek-tui)-macos-arm64$' deepseek-artifacts-sha256.txt | shasum -a 256 -c -
chmod +x deepseek deepseek-tui
xattr -d com.apple.quarantine deepseek deepseek-tui 2>/dev/null || true
sudo mv deepseek deepseek-tui /usr/local/bin/`,
  "macos-x64": `curl -fsSL -O https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
curl -fsSL -o deepseek \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-macos-x64
curl -fsSL -o deepseek-tui \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-tui-macos-x64
grep -E ' (deepseek|deepseek-tui)-macos-x64$' deepseek-artifacts-sha256.txt | shasum -a 256 -c -
chmod +x deepseek deepseek-tui
xattr -d com.apple.quarantine deepseek deepseek-tui 2>/dev/null || true
sudo mv deepseek deepseek-tui /usr/local/bin/`,
  "linux-x64": `curl -fsSL -O https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
curl -fsSL -o deepseek \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-linux-x64
curl -fsSL -o deepseek-tui \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-tui-linux-x64
grep -E ' (deepseek|deepseek-tui)-linux-x64$' deepseek-artifacts-sha256.txt | sha256sum -c -
chmod +x deepseek deepseek-tui
sudo mv deepseek deepseek-tui /usr/local/bin/`,
  "linux-arm64": `curl -fsSL -O https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
curl -fsSL -o deepseek \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-linux-arm64
curl -fsSL -o deepseek-tui \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-tui-linux-arm64
grep -E ' (deepseek|deepseek-tui)-linux-arm64$' deepseek-artifacts-sha256.txt | sha256sum -c -
chmod +x deepseek deepseek-tui
sudo mv deepseek deepseek-tui /usr/local/bin/`,
  "linux-riscv64": `curl -fsSL -O https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
curl -fsSL -o deepseek \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-linux-riscv64
curl -fsSL -o deepseek-tui \\
  https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-tui-linux-riscv64
grep -E ' (deepseek|deepseek-tui)-linux-riscv64$' deepseek-artifacts-sha256.txt | sha256sum -c -
chmod +x deepseek deepseek-tui
sudo mv deepseek deepseek-tui /usr/local/bin/`,
  "windows-x64": `# PowerShell
$ErrorActionPreference = "Stop"
$dest = "$Env:USERPROFILE\\bin"
New-Item -ItemType Directory -Force $dest | Out-Null
$manifest = Invoke-WebRequest https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt

Invoke-WebRequest \`
  -Uri https://github.com/coohu//releases/latest/download/deepseek-windows-x64.exe \`
  -OutFile "$dest\\deepseek.exe"
Invoke-WebRequest \`
  -Uri https://github.com/coohu//releases/latest/download/-windows-x64.exe \`
  -OutFile "$dest\\.exe"

$expected = @{}
$manifest.Content -split "\`n" | ForEach-Object {
  $parts = $_.Trim() -split "\\s+"
  if ($parts.Length -ge 2) { $expected[$parts[1]] = $parts[0].ToUpperInvariant() }
}
if ((Get-FileHash "$dest\\deepseek.exe" -Algorithm SHA256).Hash -ne $expected["deepseek-windows-x64.exe"]) { throw "deepseek.exe checksum mismatch" }
if ((Get-FileHash "$dest\\deepseek-tui.exe" -Algorithm SHA256).Hash -ne $expected["deepseek-tui-windows-x64.exe"]) { throw "deepseek-tui.exe checksum mismatch" }

$Env:Path = "$dest;$Env:Path"`,
};

const VERIFY: Record<Arch, string> = {
  "macos-arm64": `grep -E ' (deepseek|deepseek-tui)-macos-arm64$' deepseek-artifacts-sha256.txt | shasum -a 256 -c -`,
  "macos-x64": `grep -E ' (deepseek|deepseek-tui)-macos-x64$' deepseek-artifacts-sha256.txt | shasum -a 256 -c -`,
  "linux-x64": `grep -E ' (deepseek|deepseek-tui)-linux-x64$' deepseek-artifacts-sha256.txt | sha256sum -c -`,
  "linux-arm64": `grep -E ' (deepseek|deepseek-tui)-linux-arm64$' deepseek-artifacts-sha256.txt | sha256sum -c -`,
  "linux-riscv64": `grep -E ' (deepseek|deepseek-tui)-linux-riscv64$' deepseek-artifacts-sha256.txt | sha256sum -c -`,
  "windows-x64": `# PowerShell
$manifest = Invoke-WebRequest https://github.com/coohu/deepseek-tui/releases/latest/download/deepseek-artifacts-sha256.txt
$expected = @{}
$manifest.Content -split "\`n" | ForEach-Object {
  $parts = $_.Trim() -split "\\s+"
  if ($parts.Length -ge 2) { $expected[$parts[1]] = $parts[0].ToUpperInvariant() }
}
if ((Get-FileHash "$Env:USERPROFILE\\bin\\deepseek.exe" -Algorithm SHA256).Hash -ne $expected["deepseek-windows-x64.exe"]) { throw "deepseek.exe checksum mismatch" }
if ((Get-FileHash "$Env:USERPROFILE\\bin\\deepseek-tui.exe" -Algorithm SHA256).Hash -ne $expected["deepseek-tui-windows-x64.exe"]) { throw "deepseek-tui.exe checksum mismatch" }`,
};

const LABELS: Record<Arch, string> = {
  "macos-arm64": "macOS · Apple Silicon",
  "macos-x64": "macOS · Intel",
  "linux-x64": "Linux · x64",
  "linux-arm64": "Linux · arm64",
  "linux-riscv64": "Linux · riscv64",
  "windows-x64": "Windows · x64",
};

function detect(): Arch {
  if (typeof navigator === "undefined") return "macos-arm64";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "windows-x64";
  if (ua.includes("linux")) {
    if (ua.includes("aarch64") || ua.includes("arm64")) return "linux-arm64";
    return "linux-x64";
  }
  return "macos-arm64";
}

interface Props {
  copyLabel?: string;
  copiedLabel?: string;
  verifyHeading?: string;
}

export function InstallBinary({ copyLabel, copiedLabel, verifyHeading = "Verify checksum" }: Props) {
  const [arch, setArch] = useState<Arch>("macos-arm64");

  useEffect(() => { setArch(detect()); }, []);

  return (
    <div>
      <div className="flex flex-wrap gap-0 mb-3 hairline-t hairline-b hairline-l hairline-r">
        {(Object.keys(SNIPPETS) as Arch[]).map((a, i) => (
          <button
            key={a}
            onClick={() => setArch(a)}
            className={`px-3 py-1.5 font-mono text-[0.7rem] tracking-wider transition-colors ${
              i > 0 ? "hairline-l" : ""
            } ${arch === a ? "bg-ink text-paper" : "bg-paper hover:bg-paper-deep"}`}
          >
            {LABELS[a]}
          </button>
        ))}
      </div>

      <InstallCodeBlock cmd={SNIPPETS[arch]} copyLabel={copyLabel} copiedLabel={copiedLabel} />

      <div className="mt-4">
        <div className="eyebrow mb-2">{verifyHeading}</div>
        <InstallCodeBlock cmd={VERIFY[arch]} copyLabel={copyLabel} copiedLabel={copiedLabel} />
      </div>
    </div>
  );
}

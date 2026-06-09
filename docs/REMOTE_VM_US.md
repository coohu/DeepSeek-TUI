# US Remote VM Quickstart

This is the default remote-phone setup for US-based users who want the least
cloud ceremony. Use this path unless you specifically need the Tencent/CNB/
Feishu workflow in `docs/TENCENT_CLOUD_REMOTE_FIRST.md`.

## Recommendation

Use a small persistent Ubuntu VPS/VM, not a serverless/container platform, for
the main DeepSeek host.

Good defaults:

- DigitalOcean Droplet: 2 vCPU / 4 GB RAM / 80 GB SSD, Ubuntu.
- AWS Lightsail: 2 vCPU / 4 GB RAM / 80 GB SSD, Ubuntu.

Minimum for a smoke test:

- 2 vCPU / 2 GB RAM / 60 GB SSD.

Better for Rust builds, subagents, and longer sessions:

- 4 vCPU / 8 GB RAM / 160 GB SSD.

## Why Not Railway First

Railway is fine for a tiny relay service, but the main DeepSeek runtime wants:

- a persistent checkout and worktrees
- shell access
- systemd or equivalent long-running service supervision
- predictable local disk paths
- direct SSH recovery when the agent or bridge is unhealthy

That maps more cleanly to a VM. Use Railway later only if you want a public web
status page or a small bridge relay in front of a VM-hosted runtime.

## Provider Choice

- Choose DigitalOcean if you want the simplest VPS control panel and predictable
  developer workflow.
- Choose AWS Lightsail if you already use AWS billing or want the AWS free-trial
  path while staying in a simple VPS product.
- Avoid raw EC2 for the first setup unless you already know AWS networking,
  IAM, security groups, and EBS.
- Avoid Lambda/ECR/Load Balancers for the first setup; they are not the
  persistent interactive host DeepSeek needs.

## Target Architecture

```text
Laptop
  -> git push / SSH

DigitalOcean or AWS Lightsail Ubuntu VM
  -> /opt/whalebro/deepseek
  -> /opt/whalebro/worktrees
  -> deepseek-runtime.service on 127.0.0.1:7878
  -> deepseek-telegram-bridge.service

Telegram phone DM
  -> Telegram Bot API long polling
  -> local runtime API with CODEWHALE_RUNTIME_TOKEN
```

The runtime API must stay on `127.0.0.1`. Telegram long polling does not need
an inbound public webhook port.

## Setup Shape

Create an Ubuntu VM with SSH-key login. Open SSH only. Then:

```bash
sudo apt-get update
sudo apt-get install -y git

export CODEWHALE_BRANCH=codex/v0.8.53
export CODEWHALE_REPO_URL=https://github.com/coohu/deepseek-tui.git

git clone --branch "$CODEWHALE_BRANCH" "$CODEWHALE_REPO_URL" /tmp/deepseek
cd /tmp/deepseek
sudo CODEWHALE_REPO_URL="$CODEWHALE_REPO_URL" \
  CODEWHALE_REPO_BRANCH="$CODEWHALE_BRANCH" \
  bash scripts/tencent-lighthouse/bootstrap-ubuntu.sh
```

The bootstrap script is named for the older Tencent runbook, but it now creates
DeepSeek-primary paths and env files:

- `/etc/deepseek/runtime.env`
- `/etc/deepseek/feishu-bridge.env`
- `/opt/whalebro`
- `/opt/deepseek`

Install Rust for the `deepseek` user, then build:

```bash
sudo -iu deepseek
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sed -n '1,120p' /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --profile minimal
. "$HOME/.cargo/env"
rustup default stable
cd /opt/whalebro/deepseek
cargo install --path crates/cli --locked --force
cargo install --path crates/tui --locked --force
exit
```

Install the Telegram bridge service:

```bash
cd /opt/whalebro/deepseek
sudo CODEWHALE_BRIDGE=telegram bash scripts/tencent-lighthouse/install-services.sh
```

Create a bot with Telegram's `@BotFather`, then edit:

```bash
sudoedit /etc/deepseek/runtime.env
sudoedit /etc/deepseek/telegram-bridge.env
```

Required values:

- `/etc/deepseek/runtime.env`
  - `CODEWHALE_RUNTIME_TOKEN`
  - `CODEWHALE_RUNTIME_PORT=7878`
  - `CODEWHALE_PROVIDER=<provider>`
  - provider API key such as `ARCEE_API_KEY`, `DEEPSEEK_API_KEY`, or
    `XIAOMI_MIMO_API_KEY`
- `/etc/deepseek/telegram-bridge.env`
  - `TELEGRAM_BOT_TOKEN`
  - `CODEWHALE_RUNTIME_TOKEN` matching runtime.env
  - `CODEWHALE_WORKSPACE=/opt/whalebro`

For first pairing, temporarily set `TELEGRAM_ALLOW_UNLISTED=true`, DM the bot
`/status`, copy the returned `chat_id` into `TELEGRAM_CHAT_ALLOWLIST`, then set
`TELEGRAM_ALLOW_UNLISTED=false`.

Validate and start:

```bash
sudo -u deepseek node /opt/deepseek/telegram-bridge/scripts/validate-config.mjs \
  --env /etc/deepseek/telegram-bridge.env \
  --runtime-env /etc/deepseek/runtime.env \
  --workspace-root /opt/whalebro \
  --check-filesystem

sudo systemctl start deepseek-runtime
sudo systemctl start deepseek-telegram-bridge
sudo CODEWHALE_BRIDGE=telegram bash /opt/whalebro/deepseek/scripts/tencent-lighthouse/doctor.sh
```

Useful logs:

```bash
sudo journalctl -u deepseek-runtime -f
sudo journalctl -u deepseek-telegram-bridge -f
```

## First Smoke Test

From Telegram:

1. Send `/status`.
2. Send `/menu` and confirm the tappable control panel appears.
3. Send `summarize git status in /opt/whalebro/deepseek`.
4. Send `/threads` and test a `Resume` button.
5. Start a prompt that requires shell approval, then test both approval buttons
   and the text fallback `/allow <approval_id>` / `/deny <approval_id>`.
6. Restart the VM and confirm both services come back.

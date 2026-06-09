#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run as root: sudo bash scripts/tencent-lighthouse/bootstrap-ubuntu.sh" >&2
  exit 1
fi

CODEWHALE_USER="${CODEWHALE_USER:-${DEEPSEEK_USER:-deepseek}}"
CODEWHALE_ROOT="${CODEWHALE_ROOT:-${DEEPSEEK_ROOT:-/opt/deepseek}}"
WHALEBRO_ROOT="${WHALEBRO_ROOT:-/opt/whalebro}"
REPO_URL="${CODEWHALE_REPO_URL:-${DEEPSEEK_REPO_URL:-https://github.com/coohu/deepseek-tui.git}}"
WHALEBRO_EXTRA_REPOS="${WHALEBRO_EXTRA_REPOS:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SOURCE_BRANCH="$(git -C "${SOURCE_ROOT}" branch --show-current 2>/dev/null || true)"
REPO_BRANCH="${CODEWHALE_REPO_BRANCH:-${DEEPSEEK_REPO_BRANCH:-${SOURCE_BRANCH:-main}}}"

apt-get update
apt-get install -y \
  ca-certificates \
  curl \
  git \
  iproute2 \
  openssh-client \
  build-essential \
  pkg-config \
  libssl-dev \
  nodejs \
  npm \
  rsync \
  tmux \
  fail2ban \
  ufw

node_major="$(node -p "Number(process.versions.node.split('.')[0])")"
if (( node_major < 18 )); then
  echo "Node.js 18+ is required for the phone bridges; install a newer Node.js before running install-services.sh." >&2
fi

if ! id -u "${CODEWHALE_USER}" >/dev/null 2>&1; then
  useradd --create-home --shell /bin/bash "${CODEWHALE_USER}"
fi

install -d -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" "${CODEWHALE_ROOT}"
install -d -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" "${CODEWHALE_ROOT}/bridge"
install -d -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" "${CODEWHALE_ROOT}/telegram-bridge"
install -d -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" "${WHALEBRO_ROOT}"
install -d -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" "${WHALEBRO_ROOT}/worktrees"
install -d -m 0750 -o root -g "${CODEWHALE_USER}" /etc/deepseek
install -d -m 0700 -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" /var/lib/deepseek-feishu-bridge
install -d -m 0700 -o "${CODEWHALE_USER}" -g "${CODEWHALE_USER}" /var/lib/deepseek-telegram-bridge

if [[ ! -d "${WHALEBRO_ROOT}/deepseek/.git" ]]; then
  sudo -u "${CODEWHALE_USER}" git clone --branch "${REPO_BRANCH}" "${REPO_URL}" "${WHALEBRO_ROOT}/deepseek"
fi

for repo_spec in ${WHALEBRO_EXTRA_REPOS}; do
  repo_name="${repo_spec%%=*}"
  repo_url="${repo_spec#*=}"
  if [[ -z "${repo_name}" || -z "${repo_url}" || "${repo_name}" == "${repo_url}" ]]; then
    echo "Skipping malformed WHALEBRO_EXTRA_REPOS entry: ${repo_spec}" >&2
    continue
  fi
  if [[ ! -d "${WHALEBRO_ROOT}/${repo_name}/.git" ]]; then
    sudo -u "${CODEWHALE_USER}" git clone "${repo_url}" "${WHALEBRO_ROOT}/${repo_name}" || {
      echo "Warning: failed to clone optional repo ${repo_name} from ${repo_url}" >&2
    }
  fi
done

if [[ ! -f /etc/deepseek/runtime.env ]]; then
  cat >/etc/deepseek/runtime.env <<'EOF'
CODEWHALE_RUNTIME_TOKEN=replace-with-long-random-token
CODEWHALE_RUNTIME_PORT=7878
CODEWHALE_RUNTIME_WORKERS=2
CODEWHALE_PROVIDER=deepseek
DEEPSEEK_API_KEY=replace-with-provider-key
RUST_LOG=info
EOF
  chown root:"${CODEWHALE_USER}" /etc/deepseek/runtime.env
  chmod 0640 /etc/deepseek/runtime.env
fi

if [[ ! -f /etc/deepseek/feishu-bridge.env ]]; then
  cat >/etc/deepseek/feishu-bridge.env <<'EOF'
FEISHU_APP_ID=cli_xxxxxxxxxxxxxxxx
FEISHU_APP_SECRET=replace-with-app-secret
FEISHU_DOMAIN=feishu
CODEWHALE_RUNTIME_URL=http://127.0.0.1:7878
CODEWHALE_RUNTIME_TOKEN=replace-with-same-token-as-runtime-env
CODEWHALE_WORKSPACE=/opt/whalebro
CODEWHALE_MODEL=auto
CODEWHALE_MODE=agent
CODEWHALE_ALLOW_SHELL=true
CODEWHALE_TRUST_MODE=false
CODEWHALE_AUTO_APPROVE=false
CODEWHALE_CHAT_ALLOWLIST=
CODEWHALE_ALLOW_UNLISTED=false
FEISHU_THREAD_MAP_PATH=/var/lib/deepseek-feishu-bridge/thread-map.json
FEISHU_ALLOW_GROUPS=false
FEISHU_REQUIRE_PREFIX_IN_GROUP=true
FEISHU_GROUP_PREFIX=/cw
FEISHU_MAX_REPLY_CHARS=3500
CODEWHALE_TURN_TIMEOUT_MS=900000
EOF
  chown root:"${CODEWHALE_USER}" /etc/deepseek/feishu-bridge.env
  chmod 0640 /etc/deepseek/feishu-bridge.env
fi

ufw allow OpenSSH
ufw --force enable

cat <<EOF

Base server setup complete.

Next:
1. Install Rust 1.88+ for ${CODEWHALE_USER}; rustup is the usual path.
2. Build/install both binaries:
   sudo -iu ${CODEWHALE_USER}
   cd ${WHALEBRO_ROOT}/deepseek
   cargo install --path crates/cli --locked --force
   cargo install --path crates/tui --locked --force
3. Copy integrations/feishu-bridge or integrations/telegram-bridge to ${CODEWHALE_ROOT} and run npm install.
4. Edit /etc/deepseek/runtime.env and the selected bridge env file.
5. Install systemd units with scripts/tencent-lighthouse/install-services.sh.
6. After the env files are edited and services are started, run:
   sudo bash scripts/tencent-lighthouse/doctor.sh

EOF

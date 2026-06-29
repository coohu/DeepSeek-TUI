#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run as root: sudo bash scripts/tencent-lighthouse/install-services.sh" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEPSEEK_USER="${DEEPSEEK_USER:-${DEEPSEEK_USER:-deepseek}}"
DEEPSEEK_ROOT="${DEEPSEEK_ROOT:-${DEEPSEEK_ROOT:-/opt/deepseek}}"
BRIDGE_KIND="${DEEPSEEK_BRIDGE:-${DEEPSEEK_BRIDGE:-feishu}}"

case "${BRIDGE_KIND}" in
  feishu|lark)
    BRIDGE_SRC="integrations/feishu-bridge"
    BRIDGE_DST="${DEEPSEEK_ROOT}/bridge"
    BRIDGE_UNIT="deepseek-feishu-bridge.service"
    BRIDGE_ENV="/etc/deepseek/feishu-bridge.env"
    BRIDGE_ENV_EXAMPLE="deploy/tencent-lighthouse/examples/feishu-bridge.env.example"
    BRIDGE_STATE_DIR="/var/lib/deepseek-feishu-bridge"
    VALIDATOR="integrations/feishu-bridge/scripts/validate-config.mjs"
    ;;
  telegram)
    BRIDGE_SRC="integrations/telegram-bridge"
    BRIDGE_DST="${DEEPSEEK_ROOT}/telegram-bridge"
    BRIDGE_UNIT="deepseek-telegram-bridge.service"
    BRIDGE_ENV="/etc/deepseek/telegram-bridge.env"
    BRIDGE_ENV_EXAMPLE="deploy/tencent-lighthouse/examples/telegram-bridge.env.example"
    BRIDGE_STATE_DIR="/var/lib/deepseek-telegram-bridge"
    VALIDATOR="integrations/telegram-bridge/scripts/validate-config.mjs"
    ;;
  *)
    echo "Unknown bridge '${BRIDGE_KIND}'. Use DEEPSEEK_BRIDGE=feishu or DEEPSEEK_BRIDGE=telegram." >&2
    exit 1
    ;;
esac

install -d -m 0750 -o root -g "${DEEPSEEK_USER}" /etc/deepseek
install -d -m 0700 -o "${DEEPSEEK_USER}" -g "${DEEPSEEK_USER}" "${BRIDGE_STATE_DIR}"
install -d -o "${DEEPSEEK_USER}" -g "${DEEPSEEK_USER}" "${BRIDGE_DST}"

if [[ ! -f /etc/deepseek/runtime.env && -f "${REPO_ROOT}/deploy/tencent-lighthouse/examples/runtime.env.example" ]]; then
  install -m 0640 -o root -g "${DEEPSEEK_USER}" \
    "${REPO_ROOT}/deploy/tencent-lighthouse/examples/runtime.env.example" \
    /etc/deepseek/runtime.env
fi

if [[ ! -f "${BRIDGE_ENV}" && -f "${REPO_ROOT}/${BRIDGE_ENV_EXAMPLE}" ]]; then
  install -m 0640 -o root -g "${DEEPSEEK_USER}" \
    "${REPO_ROOT}/${BRIDGE_ENV_EXAMPLE}" \
    "${BRIDGE_ENV}"
fi
rsync -a --delete \
  --exclude node_modules \
  "${REPO_ROOT}/${BRIDGE_SRC}/" \
  "${BRIDGE_DST}/"
chown -R "${DEEPSEEK_USER}:${DEEPSEEK_USER}" "${BRIDGE_DST}"

if [[ -f "${BRIDGE_DST}/package-lock.json" ]]; then
  sudo -u "${DEEPSEEK_USER}" npm --prefix "${BRIDGE_DST}" ci --omit=dev
else
  sudo -u "${DEEPSEEK_USER}" npm --prefix "${BRIDGE_DST}" install --omit=dev
fi

<<<<<<< HEAD
install -m 0644 "${REPO_ROOT}/deploy/tencent-lighthouse/systemd/deepseek-runtime.service" /etc/systemd/system/deepseek-runtime.service
install -m 0644 "${REPO_ROOT}/deploy/tencent-lighthouse/systemd/deepseek-feishu-bridge.service" /etc/systemd/system/deepseek-feishu-bridge.service

systemctl daemon-reload
systemctl enable deepseek-runtime deepseek-feishu-bridge
=======
install -m 0644 "${REPO_ROOT}/deploy/tencent-lighthouse/systemd/deepseek-runtime.service" /etc/systemd/system/deepseek-runtime.service
install -m 0644 "${REPO_ROOT}/deploy/tencent-lighthouse/systemd/${BRIDGE_UNIT}" "/etc/systemd/system/${BRIDGE_UNIT}"

systemctl daemon-reload
systemctl enable deepseek-runtime "${BRIDGE_UNIT}"
>>>>>>> 9463266cb1278cdb82ada338479f32d47f31a704

cat <<'EOF'
Services installed but not started.

Before starting, verify:
<<<<<<< HEAD
  /etc/deepseek/runtime.env
  /etc/deepseek/feishu-bridge.env
  sudo -u deepseek node /opt/deepseek/bridge/scripts/validate-config.mjs --env /etc/deepseek/feishu-bridge.env --runtime-env /etc/deepseek/runtime.env --workspace-root /opt/whalebro --check-filesystem

Then run:
  sudo systemctl start deepseek-runtime
  sudo systemctl start deepseek-feishu-bridge
  sudo bash /opt/whalebro/deepseek/scripts/tencent-lighthouse/doctor.sh
  sudo journalctl -u deepseek-feishu-bridge -f
=======
  /etc/deepseek/runtime.env
EOF
cat <<EOF
  ${BRIDGE_ENV}
  sudo -u ${DEEPSEEK_USER} node ${REPO_ROOT}/${VALIDATOR} --env ${BRIDGE_ENV} --runtime-env /etc/deepseek/runtime.env --workspace-root /opt/whalebro --check-filesystem
Then run:
  sudo systemctl start deepseek-runtime
  sudo systemctl start ${BRIDGE_UNIT}
  sudo DEEPSEEK_BRIDGE=${BRIDGE_KIND} bash /opt/whalebro/deepseek/scripts/tencent-lighthouse/doctor.sh
  sudo journalctl -u ${BRIDGE_UNIT} -f
>>>>>>> 9463266cb1278cdb82ada338479f32d47f31a704
EOF

#!/usr/bin/env bash
# Source into an interactive agent shell (tmux, ssh) to export the provider
# key and set defaults that systemd normally handles via EnvironmentFile=.
#
# Usage (as the deepseek user):
#   . /opt/whalebro/deepseek/scripts/remote-smoke/agent-session.sh
#   deepseek models           # should list deepseek-v4-pro
#   gh auth status             # should show the fine-grained PAT
#
# The runtime.env file is 0640 root:deepseek, readable by the deepseek user.
set -a
# shellcheck disable=SC1091
. /etc/deepseek/runtime.env
set +a
export DEEPSEEK_MODEL="${DEEPSEEK_MODEL:-deepseek-v4-pro}"

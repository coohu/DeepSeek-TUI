#!/bin/bash
# verify_task.sh <task_id> <docker_image>
# Runs the DeepSWE verifier inside the task's Docker container.
# Expects model.patch at /tmp/deep-swe-verify/<task_id>/model.patch
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <task_id> <docker_image>" >&2
  exit 64
fi

TASK_ID="$1"
IMAGE="$2"
TASKS_DIR="${DEEPSWE_TASKS_DIR:-/Volumes/VIXinSSD/whalebro/deepseek/deep-swe/tasks}"
WORK_BASE="${DEEPSWE_VERIFY_DIR:-/tmp/deep-swe-verify}"
WORK_DIR="$WORK_BASE/$TASK_ID"

mkdir -p "$WORK_DIR"
RESULT_FILE="$WORK_DIR/result.txt"
MODEL_PATCH="$WORK_DIR/model.patch"
TEST_PATCH="$TASKS_DIR/$TASK_ID/tests/test.patch"
TEST_SCRIPT="$TASKS_DIR/$TASK_ID/tests/test.sh"

for required in "$MODEL_PATCH" "$TEST_PATCH" "$TEST_SCRIPT"; do
  if [[ ! -f "$required" ]]; then
    echo "missing required file: $required" >&2
    exit 66
  fi
done

echo "[$TASK_ID] Pulling image..."
docker pull "$IMAGE" 2>&1 | tail -1

echo "[$TASK_ID] Running verifier..."
docker run --rm \
  --platform linux/amd64 \
  -v "$MODEL_PATCH:/model.patch:ro" \
  -v "$TEST_PATCH:/tests/test.patch:ro" \
  -v "$TEST_SCRIPT:/verify.sh:ro" \
  "$IMAGE" \
  bash -c '
    set -e
    mkdir -p /logs/verifier /logs/artifacts
    cd /app
    git apply --whitespace=nowarn /model.patch 2>/dev/null || { echo "PATCH_FAILED"; exit 2; }
    bash /verify.sh > /logs/verifier/output.txt 2>&1
    EC=$?
    if [ -f /logs/verifier/reward.txt ]; then
      REWARD=$(cat /logs/verifier/reward.txt)
      echo "REWARD=$REWARD"
    else
      # Extract from output
      if grep -q "New tests exit code: 0" /logs/verifier/output.txt && \
         grep -q "Baseline exit code: 0" /logs/verifier/output.txt; then
        echo "REWARD=1"
      else
        echo "REWARD=0"
      fi
    fi
    echo "---OUTPUT_TAIL---"
    tail -30 /logs/verifier/output.txt
  ' > "$RESULT_FILE" 2>&1

echo "[$TASK_ID] Done. Result:"
grep -E 'REWARD|FAILED|PATCH_FAILED|passed' "$RESULT_FILE" || true
echo ""

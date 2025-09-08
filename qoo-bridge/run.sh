#!/usr/bin/env bash
set -euo pipefail

# qoo-bridge launcher
# - Creates a venv (./.venv) and installs deps
# - Starts the FastAPI bridge on 0.0.0.0:4050
# - Optionally starts LiteLLM on 0.0.0.0:4000 if available

cd "$(dirname "$0")"

VENV_DIR=".venv"
PY=${PYTHON:-python3}
PIP_ARGS=(--upgrade pip)

if [[ ! -d "$VENV_DIR" ]]; then
  "$PY" -m venv "$VENV_DIR"
fi

source "$VENV_DIR/bin/activate"
python3 -m pip install -r requirements.txt >/dev/null

# Start bridge
if [[ -f bridge.pid ]] && kill -0 "$(cat bridge.pid)" 2>/dev/null; then
  echo "Bridge already running (pid $(cat bridge.pid))"
else
  nohup python3 bridge.py >/dev/null 2>&1 & echo $! > bridge.pid
  echo "Bridge started on :4050 (pid $(cat bridge.pid))"
fi

# Optional: LiteLLM (if litellm is installed)
if command -v litellm >/dev/null 2>&1; then
  if [[ -f litellm.pid ]] && kill -0 "$(cat litellm.pid)" 2>/dev/null; then
    echo "LiteLLM already running (pid $(cat litellm.pid))"
  else
    nohup litellm --host 0.0.0.0 --port 4000 --config "$(pwd)/litellm.yaml" >/dev/null 2>&1 & echo $! > litellm.pid
    echo "LiteLLM started on :4000 (pid $(cat litellm.pid))"
  fi
else
  echo "LiteLLM not installed; skipping. (pip install litellm)"
fi


#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"

# Extract ChatGPT access token from ~/.codex/auth.json
eval "$(python3 - <<'PY'
import json, os, sys
codex_home = os.environ.get("CODEX_HOME", os.path.expanduser("~/.codex"))
auth_path = os.path.join(codex_home, "auth.json")
try:
    with open(auth_path, "r", encoding="utf-8") as f:
        data = json.load(f)
    tokens = data.get("tokens") or {}
    access = tokens.get("access_token")
    account_id = tokens.get("account_id")
    if not access:
        print("echo 'ERROR: No access_token in ~/.codex/auth.json. Run `codex` and sign in with ChatGPT.' 1>&2")
        print("exit 2")
    else:
        print(f"export CHATGPT_ACCESS_TOKEN='{access}'")
        if isinstance(account_id, str) and account_id.strip():
            print(f"export CHATGPT_ACCOUNT_ID='{account_id}'")
except FileNotFoundError:
    print("echo 'ERROR: ~/.codex/auth.json not found. Run `codex` and sign in with ChatGPT.' 1>&2")
    print("exit 2")
PY
)"

# Create venv + install deps
VENV="$here/.venv"
if [ ! -d "$VENV" ]; then
  python3 -m venv "$VENV"
fi
source "$VENV/bin/activate"
pip -q install --upgrade pip >/dev/null
pip -q install -r "$here/requirements.txt" >/dev/null
pip -q install litellm[proxy] >/dev/null 2>&1 || pip -q install litellm >/dev/null 2>&1

# Start the bridge (0.0.0.0:4050)
BRIDGE_HOST="0.0.0.0" BRIDGE_PORT="4050" nohup python3 "$here/bridge.py" >"$here/bridge.log" 2>&1 & echo $! > "$here/bridge.pid"
sleep 1

# Start LiteLLM on 0.0.0.0:4000 pointing at the bridge
nohup litellm --host 0.0.0.0 --port 4000 --config "$here/litellm.yaml" >"$here/litellm.log" 2>&1 & echo $! > "$here/litellm.pid"

echo "LAN services started:"
echo "- Bridge (Responses API): http://0.0.0.0:4050/v1/responses"
echo "- LiteLLM (Chat API to bridge): http://0.0.0.0:4000/v1/chat/completions"
echo
echo "From another machine:"
echo "- Preferred (Responses): set base_url to http://<THIS_HOST_LAN_IP>:4050/v1 and wire_api=\"responses\""
echo "- LiteLLM (Chat, non-streaming): set base_url to http://<THIS_HOST_LAN_IP>:4000/v1 and wire_api=\"chat\""

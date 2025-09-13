#!/usr/bin/env bash
set -euo pipefail

# qoo: dev Codex CLI wrapper
# Uses a separate Codex home and points to the workspace build by default.

export CODEX_HOME="${CODEX_HOME:-$HOME/.qoo}"

DEV_BIN_DEFAULT="$HOME/Documents/Projects/bhd/qox/dist/codex-259636b0-mooded"
DEV_BIN="${CODEX_DEV_BIN:-$DEV_BIN_DEFAULT}"

if [[ ! -x "$DEV_BIN" ]]; then
  if command -v codex >/dev/null 2>&1; then
    DEV_BIN="$(command -v codex)"
  else
    echo "Error: dev codex binary not found at $DEV_BIN and no 'codex' in PATH" >&2
    exit 127
  fi
fi

# Support a non-interactive status mode via `qoo --status` (or `qoo status`).
# This maps to the codex TUI's special positional command "status" which
# prints a summary and exits immediately.
if [[ "${1-}" == "--status" || "${1-}" == "status" ]]; then
  # Non-interactive status printed by the wrapper to avoid launching the TUI.
  # Layout mirrors the TUI's /status but omits dynamic token counts.
  CWD_DISPLAY() {
    local p="$PWD"
    local home="$HOME"
    if [[ "$p" == "$home" ]]; then
      printf '~'
    elif [[ "$p" == "$home"/* ]]; then
      printf '~/%s' "${p#"$home/"}"
    else
      printf '%s' "$p"
    fi
  }

  # Resolve absolute prompt file paths relative to the dev build tree.
  DEV_DIR="$(dirname "$DEV_BIN")"
  RSDIR="$(realpath "$DEV_DIR/../..")"  # codex-rs root
  SYSTEM_PROMPT="$RSDIR/core/prompt.md"
  COMPACT_PROMPT="$RSDIR/core/src/prompt_for_compact_command.md"
  # qox carries multiple init prompts; prefer prompt_for_init_command.md
  if [[ -f "$RSDIR/tui/prompt_for_init_command.md" ]]; then
    INIT_PROMPT="$RSDIR/tui/prompt_for_init_command.md"
  elif [[ -f "$RSDIR/tui/prompt_for_init_command_custom.md" ]]; then
    INIT_PROMPT="$RSDIR/tui/prompt_for_init_command_custom.md"
  else
    INIT_PROMPT="$RSDIR/tui/prompt_for_init_command_v0.md"
  fi

  # Discover AGENTS.md files walking up to /.
  AGENTS_LIST=()
  dir="$PWD"
  while :; do
    if [[ -f "$dir/AGENTS.md" ]]; then AGENTS_LIST+=("$dir/AGENTS.md"); fi
    [[ "$dir" == "/" ]] && break
    dir="$(dirname "$dir")"
  done
  if ((${#AGENTS_LIST[@]}==0)); then
    AGENTS_SUMMARY="(none)"
  else
    AGENTS_SUMMARY="${AGENTS_LIST[*]}"
  fi

  # Pull model/provider/reasoning from $CODEX_HOME/config.toml if present.
  MODEL_NAME=""
  PROVIDER_NAME=""
  REASONING_EFFORT=""
  REASONING_SUMMARIES=""
  CFG_TOML="${CODEX_HOME}/config.toml"
  if [[ -f "$CFG_TOML" ]]; then
    PY_OUT="$(python3 - "$CFG_TOML" <<'PY'
import sys, tomllib
path=sys.argv[1]
with open(path,'rb') as f:
    cfg=tomllib.load(f)
def get(d,k,default=None):
    return d.get(k,default)
model=get(cfg,'model')
provider=get(cfg,'model_provider','openai')
eff=get(cfg,'model_reasoning_effort')
summ=get(cfg,'model_reasoning_summary')
print('\n'.join([
    model or '',
    provider or '',
    (eff or ''),
    (summ or ''),
]))
PY
    )" || true
    MODEL_NAME="$(printf '%s' "$PY_OUT" | sed -n '1p')"
    PROVIDER_NAME="$(printf '%s' "$PY_OUT" | sed -n '2p')"
    REASONING_EFFORT="$(printf '%s' "$PY_OUT" | sed -n '3p')"
    REASONING_SUMMARIES="$(printf '%s' "$PY_OUT" | sed -n '4p')"
  fi
  title_case(){
    local s="$1"; [[ -z "$s" ]] && { printf '%s' ""; return; }
    printf '%s%s' "${s:0:1^^}" "${s:1,,}"
  }
  PRETTY_PROVIDER="$PROVIDER_NAME"
  if [[ -n "$PRETTY_PROVIDER" && "${PRETTY_PROVIDER,,}" == "openai" ]]; then PRETTY_PROVIDER="OpenAI"; else PRETTY_PROVIDER="$(title_case "$PRETTY_PROVIDER")"; fi

  # Account info from $CODEX_HOME/auth.json (optional).
  ACCOUNT_BLOCK=""
  if [[ -f "$CODEX_HOME/auth.json" ]]; then
    PY_ACC="$(AUTH="$CODEX_HOME/auth.json" python3 - <<'PY'
import json,os
try:
  auth=os.environ.get('AUTH')
  with open(auth,'r') as f:
    a=json.load(f)
  tokens=a.get('tokens') if isinstance(a,dict) else None
  if tokens and isinstance(tokens,dict):
    idt=tokens.get('id_token') or {}
    email=(idt if isinstance(idt,dict) else {}).get('email')
    plan=(idt if isinstance(idt,dict) else {}).get('chatgpt_plan_type')
    print('1')
    print(email or '')
    print(plan or '')
  else:
    print('0')
except Exception:
  print('0')
PY
    )" || true
    if [[ "$(printf '%s' "$PY_ACC" | sed -n '1p')" == "1" ]]; then
      ACC_EMAIL="$(printf '%s' "$PY_ACC" | sed -n '2p')"
      ACC_PLAN="$(printf '%s' "$PY_ACC" | sed -n '3p')"
      ACCOUNT_BLOCK+=$'👤 Account
  • Signed in with ChatGPT
'
      [[ -n "$ACC_EMAIL" ]] && ACCOUNT_BLOCK+="  • Login: $ACC_EMAIL
"
      if [[ -n "$ACC_PLAN" ]]; then
        ACCOUNT_BLOCK+="  • Plan: $(title_case "$ACC_PLAN")
"
      fi
      ACCOUNT_BLOCK+=$'\n'
    fi
  fi

  # Print status
  printf '/status\n'
  printf '📂 Workspace\n'
  printf '  • Path: %s\n' "$(CWD_DISPLAY)"
  printf '  • Approval Mode: never\n'
  printf '  • Sandbox: danger-full-access\n'
  printf '  • AGENTS files: %s\n' "$AGENTS_SUMMARY"
  printf '  • System Prompt: %s\n' "$SYSTEM_PROMPT"
  printf '  • Init Prompt: %s\n' "$INIT_PROMPT"
  printf '  • Compact Prompt: %s\n' "$COMPACT_PROMPT"
  if [[ -n "$ACCOUNT_BLOCK" ]]; then printf '%s' "$ACCOUNT_BLOCK"; fi
  printf '🧠 Model\n'
  if [[ -n "$MODEL_NAME" ]]; then printf '  • Name: %s\n' "$MODEL_NAME"; fi
  if [[ -n "$PRETTY_PROVIDER" ]]; then printf '  • Provider: %s\n' "$PRETTY_PROVIDER"; fi
  if [[ -n "$REASONING_EFFORT" ]]; then printf '  • Reasoning Effort: %s\n' "$(title_case "$REASONING_EFFORT")"; fi
  if [[ -n "$REASONING_SUMMARIES" ]]; then printf '  • Reasoning Summaries: %s\n' "$(title_case "$REASONING_SUMMARIES")"; fi
  printf '\n📊 Token Usage\n'
  printf '  • Input: 0\n  • Output: 0\n  • Total: 0\n'
  exit 0
fi

exec "$DEV_BIN" \
  --ask-for-approval never \
  --sandbox danger-full-access \
  "$@"

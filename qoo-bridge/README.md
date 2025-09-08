qoo-bridge
===========

LAN-accessible bridge that forwards OpenAI Responses API-compatible requests to ChatGPT’s internal Codex endpoint using the logged-in session on this host. Useful for running Codex on a second machine without API keys.

Files
- bridge.py – FastAPI app exposing /v1/responses and a convenience /v1/chat/completions mapping.
- run.sh – Sets up a venv, installs deps, and starts the bridge on 0.0.0.0:4050 (and LiteLLM on 4000 if installed).
- requirements.txt – Python dependencies.
- ecosystem.config.js – PM2 ecosystem to keep the bridge running with retries and delayed restarts.
- examples/config.local_lan.example.toml – Example qoo profile config referencing this LAN bridge.

Quick start
1) Ensure this machine is signed into ChatGPT via Codex so ~/.codex/auth.json exists.
2) Start under PM2 from repo root:
   - pm2 start qoo-bridge/ecosystem.config.js --only qoo-bridge
   - pm2 save
   - sudo env PATH=$PATH:$(dirname $(which node)) pm2 startup systemd -u $(whoami) --hp $HOME
3) Or run directly:
   - cd qoo-bridge && ./run.sh

Endpoints
- Bridge: http://0.0.0.0:4050/v1/responses (SSE for streaming)
- Optional LiteLLM: http://0.0.0.0:4000/v1 (non-streaming chat wrapper)

Behavior
- Forces store=false and normalizes prompt_cache_key/session_id for multi-turn.
- Drops reasoning items and strips message ids when store=false to prevent rs_* reference errors.
- Maps model alias local_md -> gpt-5.
- Passes headers: Authorization (ChatGPT), OpenAI-Beta, originator, session_id, chatgpt-account-id.

Example qoo profile (examples/config.local_lan.example.toml)
Copy this into ~/.qoo/config.toml or merge into an existing file.


qoo-bridge (LAN ChatGPT Responses Bridge)

Purpose
- Expose a LAN-accessible proxy that uses the ChatGPT plan session from this host to call the ChatGPT Responses API. No OpenAI API keys are used by clients across the LAN; only this host needs to be signed into ChatGPT via Codex.
- Designed to be a standalone sub-repo and Git submodule, with all docs and scripts self-contained.

Features
- Forwards /v1/responses requests to https://chatgpt.com/backend-api/codex/responses using the ChatGPT access token from ~/.codex/auth.json on this host.
- SSE passthrough for streaming, header normalization (OpenAI-Beta, originator, session_id, chatgpt-account-id), and continuity fixes for store=false sessions.
- Optional LiteLLM front-end (configured via litellm.yaml) for a Chat Completions-shaped endpoint if desired; direct Responses is recommended.

Repo Layout
- bridge.py – FastAPI app exposing /v1/responses and a convenience /v1/chat/completions mapping.
- run.sh – Sets up a venv, installs deps, and starts the bridge on 0.0.0.0:4050 (and LiteLLM on 4000 if desired).
- requirements.txt – Python dependencies.
- litellm.yaml – Example LiteLLM routing that targets the bridge.

Quick Start
1) Sign in to ChatGPT via Codex on this host (writes ~/.codex/auth.json):
   - Run `codex` and log in with ChatGPT if prompted.

2) Start the bridge (first run creates a venv and installs deps):
   - cd qoo-bridge
   - ./run.sh
   - Bridge listens on http://0.0.0.0:4050/v1

3) Configure the client (another machine on LAN) to use the bridge via Codex profile:
   - ~/.codex/config.toml (or for dev wrapper ~/.qoo/config.toml):
     [model_providers.lan]
     name = "LAN ChatGPT Proxy"
     base_url = "http://<BRIDGE_HOST_LAN_IP>:4050/v1"
     wire_api = "responses"
     requires_openai_auth = false
     stream_idle_timeout_ms = 20000

     [profiles.local_lan]
     model = "local_md"            # alias mapped to gpt-5 by the bridge
     model_provider = "lan"
     approval_policy = "never"

4) Test:
   - qoo --profile local_lan e "say hello to me then run 1 shell cmd"

Continuity and 404 Fix
- Upstream rejects references to non-persisted items when store=false (e.g., rs_* reasoning items). The bridge normalizes inputs to:
  - Enforce store=false and ensure prompt_cache_key is set to session_id when missing.
  - Drop reasoning items from input and strip message ids when store=false.

LiteLLM (optional)
- run.sh also starts LiteLLM on 0.0.0.0:4000 using litellm.yaml if installed.
- Prefer direct /v1/responses for streaming; Chat Completions mapping is best-effort for non-streaming convenience.

Logs & Debugging
- Bridge logs: qoo-bridge/bridge.log
- Start with higher verbosity on the client if needed: RUST_LOG=trace RUST_BACKTRACE=full

Security
- No auth is enforced on the LAN endpoints. Restrict exposure to trusted networks and/or firewall to a specific CIDR.

Submodule Setup (make this a private sub-repo)
1) Create a private repository named qoo-bridge in your Git provider.
2) From the parent repo root, add it as a submodule:
   - git submodule add -b main <git_url_for_private_repo> qoo-bridge
   - git submodule update --init --recursive
3) Move existing contents into the submodule repo and commit from within qoo-bridge:
   - cd qoo-bridge
   - git add . && git commit -m "Initial import"
   - git push origin main

Notes
- This directory is self-contained; do not rely on files outside qoo-bridge.
- Local runtime artifacts (.venv, *.pid, *.log) are git-ignored at the repo root.


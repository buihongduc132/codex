Lan ChatGPT Proxy (ChatGPT-plan only)

Purpose
- Expose a local, LAN-accessible proxy that uses your existing ChatGPT (Plus/Pro/Team/…) session from this machine to call the ChatGPT Responses API. No API keys are used.
- Optionally place a LiteLLM proxy in front, so other machines can just hit a LiteLLM endpoint. The other machine does not authenticate and never reaches OpenAI directly.

What’s inside
- bridge.py – FastAPI app exposing an OpenAI-compatible endpoint at `/v1/responses`, forwarding to `https://chatgpt.com/backend-api/codex/responses` using the ChatGPT token from `~/.codex/auth.json`.
- litellm.yaml – LiteLLM config routing a model alias to the bridge (optional, only if you want the LiteLLM URL).
- run.sh – Helper to start the bridge and LiteLLM bound on 0.0.0.0 for LAN access.

Quick start
1) Ensure this machine is already logged into ChatGPT via Codex (auth.json present):
   - Run `codex` here and sign in with ChatGPT if not already done.
2) Start services (creates a venv and installs deps on first run):
   - `cd lan-chatgpt-proxy`
   - `./run.sh`
3) From another machine on the LAN, configure Codex to use Responses wire API pointing at this host:
   - Create `~/.codex/config.toml` with:
     model = "gpt-4o-mini"
     model_provider = "proxy"
     [model_providers.proxy]
     name = "LAN ChatGPT Proxy"
     base_url = "http://<THIS_HOST_LAN_IP>:4050/v1"
     wire_api = "responses"
     requires_openai_auth = false

   Notes:
   - This hits the bridge directly at `/v1/responses` (no LiteLLM). It avoids any API keys and does not contact OpenAI from the client machine.
   - If you specifically want to hit a LiteLLM URL instead, start `run.sh` and point the other machine to `http://<THIS_HOST_LAN_IP>:4000/v1` with `wire_api = "chat"` (experimental mapping). See below.

LiteLLM (optional)
- If you need a LiteLLM endpoint, `run.sh` also starts LiteLLM on `0.0.0.0:4000` using `litellm.yaml`.
- By default, LiteLLM forwards Chat Completions requests to the bridge’s `/v1/chat/completions` which maps to ChatGPT Responses upstream. This path is provided as a convenience; for best compatibility, prefer the direct Responses route shown above.

Security
- No auth is enforced on LAN endpoints – restrict to your trusted network.
- The bridge reads the ChatGPT access token from `~/.codex/auth.json` on this host. It does not share or export the token.


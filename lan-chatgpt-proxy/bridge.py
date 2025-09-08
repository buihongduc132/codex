#!/usr/bin/env python3
import asyncio
import json
import logging
import os
from typing import Any, Dict, Optional, Tuple

import httpx
from fastapi import FastAPI, Request, Response
from fastapi.responses import JSONResponse, StreamingResponse


CHATGPT_RESPONSES_URL = "https://chatgpt.com/backend-api/codex/responses"

# Basic file logger
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[logging.FileHandler("bridge.log"), logging.StreamHandler()]
)
log = logging.getLogger("bridge")


def _load_chatgpt_tokens() -> Tuple[Optional[str], Optional[str]]:
    # Read from ~/.codex/auth.json written by Codex login
    codex_home = os.environ.get("CODEX_HOME", os.path.expanduser("~/.codex"))
    auth_path = os.path.join(codex_home, "auth.json")
    try:
        with open(auth_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        tokens = data.get("tokens") or {}
        access = tokens.get("access_token")
        account_id = tokens.get("account_id") if isinstance(tokens.get("account_id"), str) else None
        if access and isinstance(access, str) and access.strip():
            return access, account_id
    except FileNotFoundError:
        pass
    except Exception as e:
        print(f"[bridge] failed reading auth.json: {e}")
    return None, None


def _make_auth_headers() -> Dict[str, str]:
    token_env = os.environ.get("CHATGPT_ACCESS_TOKEN")
    acc_env = os.environ.get("CHATGPT_ACCOUNT_ID")
    access_token, account_id_from_auth = _load_chatgpt_tokens()
    token = token_env or access_token
    if not token:
        raise RuntimeError(
            "No ChatGPT access token found. Ensure this machine is signed in via Codex (auth.json)."
        )
    headers = {
        "Authorization": f"Bearer {token}",
        "OpenAI-Beta": "responses=experimental",
        "User-Agent": "codex-lan-bridge/1.0",
        "Accept": "application/json",
        # Some upstreams check browser-like CORS headers. Provide stable values.
        "Origin": "https://chatgpt.com",
        "Referer": "https://chatgpt.com/",
    }
    # Optional – prefer explicit env override, otherwise use auth.json account_id
    account_id = acc_env or account_id_from_auth
    if account_id and account_id.strip():
        headers["chatgpt-account-id"] = account_id.strip()
    return headers


app = FastAPI(title="LAN ChatGPT Bridge", version="1.0")


# 1) Native Responses endpoint: forward as-is to ChatGPT backend
@app.post("/v1/responses")
async def responses_endpoint(req: Request) -> Response:
    body = await req.body()
    headers = _make_auth_headers()
    timeout = httpx.Timeout(300.0)
    async with httpx.AsyncClient(timeout=timeout) as client:
        # Preserve the request’s Accept header for streaming clients
        accept = req.headers.get("accept", "application/json")
        headers["Accept"] = accept
        # Preserve Content-Type for JSON forwarding
        content_type = req.headers.get("content-type")
        if content_type:
            headers["Content-Type"] = content_type
        # Pass-through important request headers when present
        for h in ("originator", "session_id", "user-agent", "chatgpt-account-id"):
            v = req.headers.get(h)
            if v:
                headers[h] = v

        # Ensure default originator & UA match Codex defaults when not provided by client
        if "originator" not in headers:
            headers["originator"] = "codex_cli_rs"
        headers["User-Agent"] = headers.get("user-agent") or "codex_cli_rs"

        log.info("/v1/responses from %s accept=%s stream=%s", req.client.host if req.client else "?", accept, accept.startswith("text/event-stream"))

        # If JSON and model alias used, rewrite model name. Also capture/normalize continuity hints for logs.
        payload = None
        prompt_cache_key = None
        model_before = None
        model_after = None
        store_flag = None
        input_len = None
        if content_type and content_type.startswith("application/json"):
            try:
                payload = json.loads(body.decode("utf-8"))
                model = payload.get("model")
                model_before = model
                if isinstance(model, str) and model == "local_md":
                    payload["model"] = "gpt-5"
                model_after = payload.get("model")
                prompt_cache_key = payload.get("prompt_cache_key")
                store_flag = payload.get("store")
                # Normalize continuity: if client omitted prompt_cache_key but provided session_id, fill it.
                if not prompt_cache_key and headers.get("session_id"):
                    payload["prompt_cache_key"] = headers.get("session_id")
                    prompt_cache_key = payload["prompt_cache_key"]
                # ChatGPT Codex backend requires store=false. Enforce it to avoid 400.
                if store_flag is not False:
                    payload["store"] = False
                    store_flag = False
                # Defensive normalization for multi-turn continuity when store=false:
                # 1) Drop any reasoning items in the replayed input to avoid referencing rs_* ids.
                # 2) Strip per-message ids if a client included them.
                if store_flag is False:
                    inp = payload.get("input")
                    if isinstance(inp, list):
                        new_inp = []
                        for it in inp:
                            if isinstance(it, dict):
                                ty = it.get("type")
                                if ty == "reasoning":
                                    continue
                                if ty == "message":
                                    # Remove id field to avoid references
                                    if "id" in it:
                                        it = dict(it)
                                        it["id"] = None
                                new_inp.append(it)
                        payload["input"] = new_inp
                try:
                    input_len = len(payload.get("input") or [])
                except Exception:
                    input_len = None
            except Exception:
                payload = None

        # Forward request to ChatGPT Responses API
        if payload is not None:
            upstream = await client.post(
                CHATGPT_RESPONSES_URL,
                json=payload,
                headers=headers,
            )
        else:
            upstream = await client.post(
                CHATGPT_RESPONSES_URL,
                content=body,
                headers=headers,
            )
        log.info(
            "upstream status=%s content-type=%s session_id=%s cache_key=%s store=%s model=%s->%s input_len=%s",
            upstream.status_code,
            upstream.headers.get("content-type"),
            headers.get("session_id"),
            prompt_cache_key,
            store_flag,
            model_before,
            model_after,
            input_len,
        )

        # For error statuses, return as JSON (do not stream SSE). Log a short body sample.
        if upstream.status_code >= 400:
            try:
                sample = upstream.text[:320]
            except Exception:
                sample = None
            log.warning(
                "error %s session_id=%s cache_key=%s store=%s model=%s->%s input_len=%s body_sample=%r",
                upstream.status_code,
                headers.get("session_id"),
                prompt_cache_key,
                store_flag,
                model_before,
                model_after,
                input_len,
                sample,
            )
            return Response(
                content=upstream.content,
                status_code=upstream.status_code,
                media_type=upstream.headers.get("content-type", "application/json"),
            )

        # Stream SSE through when requested
        if accept.startswith("text/event-stream"):
            first = {"seen": False}

            async def gen():
                async for chunk in upstream.aiter_bytes():
                    if not first["seen"]:
                        log.info("first SSE bytes: %s", chunk[:80])
                        first["seen"] = True
                    yield chunk

            return StreamingResponse(gen(), status_code=upstream.status_code, media_type="text/event-stream")

        # Non-streaming JSON passthrough
        sample = upstream.content[:200]
        log.info("non-stream body sample: %r", sample)
        return Response(content=upstream.content, status_code=upstream.status_code, media_type=upstream.headers.get("content-type", "application/json"))


# 2) Convenience Chat Completions endpoint: basic mapping -> Responses (non-streaming)
@app.post("/v1/chat/completions")
async def chat_completions(req: Request) -> Response:
    payload = await req.json()
    model = payload.get("model", "gpt-4o-mini")
    messages = payload.get("messages", []) or []
    stream = bool(payload.get("stream", False))
    if stream:
        # Keep scope simple: instruct clients to make non-streaming requests via LiteLLM
        return JSONResponse(
            status_code=400,
            content={"error": {"message": "stream=true not supported via bridge; use Responses /v1/responses for streaming."}},
        )

    # Extract system instructions and latest user message text
    system_instructions = []
    user_texts = []
    for m in messages:
        role = m.get("role")
        content = m.get("content")
        if isinstance(content, list):
            # OpenAI chat content can be rich; keep text parts only
            parts = []
            for c in content:
                if isinstance(c, dict) and c.get("type") in ("text", "input_text"):
                    parts.append(c.get("text", ""))
            content = "\n".join([p for p in parts if p])
        if not isinstance(content, str):
            content = str(content) if content is not None else ""

        if role == "system" and content:
            system_instructions.append(content)
        elif role == "user" and content:
            user_texts.append(content)

    instructions = "\n\n".join(system_instructions)
    user_text = "\n\n".join(user_texts) or ""

    responses_payload: Dict[str, Any] = {
        "model": model,
        "instructions": instructions,
        "input": [
            {
                "role": "user",
                "content": [
                    {"type": "input_text", "text": user_text},
                ],
            }
        ],
        "tools": [],
        "tool_choice": "auto",
        "parallel_tool_calls": False,
        "store": False,
        "stream": False,
        "include": [],
    }

    headers = _make_auth_headers()
    timeout = httpx.Timeout(300.0)
    async with httpx.AsyncClient(timeout=timeout) as client:
        upstream = await client.post(CHATGPT_RESPONSES_URL, json=responses_payload, headers=headers)
        if upstream.status_code >= 400:
            return Response(content=upstream.content, status_code=upstream.status_code, media_type=upstream.headers.get("content-type", "application/json"))

        data = upstream.json()

        # Best-effort extraction of final text – Responses returns output items; when non-streaming
        # the consolidated text is usually available under `output_text` or via message items.
        text_out: str = ""
        if isinstance(data, dict):
            # Try consolidated output_text first (if present)
            text_out = (
                data.get("response", {}).get("output_text")
                or data.get("output_text")
                or ""
            )
            if not text_out:
                # Fallback: scan for message/output_text items
                items = (
                    data.get("response", {}).get("output", [])
                    or data.get("output", [])
                )
                if isinstance(items, list):
                    parts = []
                    for it in items:
                        if isinstance(it, dict):
                            ty = it.get("type")
                            if ty in ("message", "output_text"):
                                # message: content is list of {type: output_text, text: ...}
                                if ty == "message":
                                    for c in it.get("content", []) or []:
                                        if isinstance(c, dict) and c.get("type") in ("output_text", "text"):
                                            t = c.get("text")
                                            if isinstance(t, str):
                                                parts.append(t)
                                else:
                                    t = it.get("text")
                                    if isinstance(t, str):
                                        parts.append(t)
                    text_out = "".join(parts)

        chat_resp = {
            "id": data.get("id", "chatcmpl_bridge"),
            "object": "chat.completion",
            "created": int(asyncio.get_event_loop().time()),
            "model": model,
            "choices": [
                {
                    "index": 0,
                    "message": {"role": "assistant", "content": text_out},
                    "finish_reason": "stop",
                }
            ],
        }
        return JSONResponse(chat_resp)


if __name__ == "__main__":
    import uvicorn

    host = os.environ.get("BRIDGE_HOST", "0.0.0.0")
    port = int(os.environ.get("BRIDGE_PORT", "4050"))
    uvicorn.run("bridge:app", host=host, port=port, reload=False)

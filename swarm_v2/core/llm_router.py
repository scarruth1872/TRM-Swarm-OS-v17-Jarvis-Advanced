import os
import asyncio
import aiohttp
import json
import logging
from typing import Optional, Dict, Any, Tuple
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

logger = logging.getLogger("LLMRouter")

# Last K3 triage per agent, from the Rust kernel. Exposed so a caller (or a test)
# can read the real trit without parsing it back out of prose. Empty when the kernel
# is unreachable - an absent trit is honest, an invented one is not.
LAST_TRIAGE: dict[str, dict] = {}

_HTTP_SESSION: Optional[aiohttp.ClientSession] = None

def get_session() -> aiohttp.ClientSession:
    """Return a reusable ClientSession bound to the CURRENT event loop.

    DEFECT FIX 2026-11-03: the session was cached with no loop check, so the
    first caller's event loop owned it forever. Any later caller running on a
    different loop (asyncio.run() per request, a second worker thread, or a
    second test in the same process) reused a session whose loop was already
    closed and every provider call failed with "All local inference endpoints
    unreachable" even though the services were up.
    """
    global _HTTP_SESSION
    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        loop = None

    stale = _HTTP_SESSION is not None and (
        _HTTP_SESSION.closed
        or (loop is not None and getattr(_HTTP_SESSION, "_loop", loop) is not loop)
    )
    if _HTTP_SESSION is None or stale:
        _HTTP_SESSION = aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=8))
    return _HTTP_SESSION


# ─── Provider: Google Gemini ────────────────────────────────────────────────────

async def generate_with_gemini(system_prompt: str, prompt: str, model_name: str = "gemini-2.5-flash") -> str:
    """Generate response using Google Gemini API."""
    api_key = os.environ.get("GEMINI_API_KEY")
    if not api_key:
        return "[Error] GEMINI_API_KEY not set in environment."
    
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model_name}:generateContent?key={api_key}"
    
    payload = {
        "systemInstruction": {
            "parts": [{"text": system_prompt}]
        },
        "contents": [
            {
                "role": "user",
                "parts": [{"text": prompt}]
            }
        ],
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 8192,
        }
    }
    
    session = get_session()
    async with session.post(url, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Gemini API failed: {err_text}"
        data = await resp.json()
        try:
            return data["candidates"][0]["content"]["parts"][0]["text"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Gemini response."


# ─── Provider: OpenRouter ───────────────────────────────────────────────────────

async def generate_with_openrouter(system_prompt: str, prompt: str, model_name: str = "anthropic/claude-sonnet-4") -> str:
    """Generate response using OpenRouter API."""
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        return "[Error] OPENROUTER_API_KEY not set in environment."
    
    url = "https://openrouter.ai/api/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "HTTP-Referer": "http://localhost:8021",
        "X-Title": "Swarm OS V2",
        "Content-Type": "application/json"
    }
    
    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }
    
    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] OpenRouter API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse OpenRouter response."


# ─── Provider: DeepSeek ─────────────────────────────────────────────────────────

async def generate_with_deepseek(system_prompt: str, prompt: str, model_name: str = "deepseek-chat") -> str:
    """Generate response using DeepSeek API (OpenAI compatible)."""
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        return "[Error] DEEPSEEK_API_KEY not set in environment."
    
    url = "https://api.deepseek.com/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }
    
    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] DeepSeek API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse DeepSeek response."


# ─── Provider: OpenAI ───────────────────────────────────────────────────────────

async def generate_with_openai(system_prompt: str, prompt: str, model_name: str = "gpt-4o") -> str:
    """Generate response using OpenAI API."""
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        return "[Error] OPENAI_API_KEY not set in environment."

    url = "https://api.openai.com/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] OpenAI API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse OpenAI response."


# ─── Provider: Anthropic (Claude) ───────────────────────────────────────────────

async def generate_with_anthropic(system_prompt: str, prompt: str, model_name: str = "claude-sonnet-4-20250514") -> str:
    """Generate response using Anthropic Claude API."""
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        return "[Error] ANTHROPIC_API_KEY not set in environment."

    url = "https://api.anthropic.com/v1/messages"
    headers = {
        "x-api-key": api_key,
        "anthropic-version": "2023-06-01",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "max_tokens": 8192,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Anthropic API failed: {err_text}"
        data = await resp.json()
        try:
            return data["content"][0]["text"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Anthropic response."


# ─── Provider: Groq (Fast Inference) ────────────────────────────────────────────

async def generate_with_groq(system_prompt: str, prompt: str, model_name: str = "llama-3.3-70b-versatile") -> str:
    """Generate response using Groq API (ultra-fast inference)."""
    api_key = os.environ.get("GROQ_API_KEY")
    if not api_key:
        return "[Error] GROQ_API_KEY not set in environment."

    url = "https://api.groq.com/openai/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Groq API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Groq response."


# ─── Provider: Cerebras (Fast Inference) ────────────────────────────────────────

async def generate_with_cerebras(system_prompt: str, prompt: str, model_name: str = "gemma-4-31b") -> str:
    """Generate response using Cerebras API (ultra-fast inference)."""
    api_key = os.environ.get("CEREBRAS_API_KEY")
    if not api_key:
        return "[Error] CEREBRAS_API_KEY not set in environment."

    url = "https://api.cerebras.ai/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Cerebras API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Cerebras response."


# ─── Provider: Grok / xAI ──────────────────────────────────────────────────────

async def generate_with_grok(system_prompt: str, prompt: str, model_name: str = "grok-3-mini-fast") -> str:
    """Generate response using xAI Grok API."""
    api_key = os.environ.get("GROK_API_KEY")
    if not api_key:
        return "[Error] GROK_API_KEY not set in environment."

    url = "https://api.x.ai/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Grok API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Grok response."


# ─── Provider: Perplexity (Search-Augmented) ────────────────────────────────────

async def generate_with_perplexity(system_prompt: str, prompt: str, model_name: str = "sonar-pro") -> str:
    """Generate response using Perplexity API (search-augmented generation)."""
    api_key = os.environ.get("PERPLEXITY_API_KEY")
    if not api_key:
        return "[Error] PERPLEXITY_API_KEY not set in environment."

    url = "https://api.perplexity.ai/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] Perplexity API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse Perplexity response."


# ─── Provider: NVIDIA NIM ──────────────────────────────────────────────────────

async def generate_with_nvidia(system_prompt: str, prompt: str, model_name: str = "meta/llama-3.3-70b-instruct") -> str:
    """Generate response using NVIDIA NIM API."""
    api_key = os.environ.get("NVIDIA_NIM_API_KEY")
    if not api_key:
        return "[Error] NVIDIA_NIM_API_KEY not set in environment."

    url = "https://integrate.api.nvidia.com/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    async with session.post(url, headers=headers, json=payload) as resp:
        if resp.status != 200:
            err_text = await resp.text()
            return f"[Error] NVIDIA NIM API failed: {err_text}"
        data = await resp.json()
        try:
            return data["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            return "[Error] Failed to parse NVIDIA NIM response."


# ─── Provider: Bonsai (Local Docker) ────────────────────────────────────────────

async def generate_with_bonsai(system_prompt: str, prompt: str, model_name: str = "bonsai-auto") -> str:
    """
    Generate response via the Bonsai Router Gateway (localhost:8585).
    The gateway classifies prompt complexity and routes to the optimal
    Ternary-Bonsai model tier: 1.7B (casual), 4B (exec), 8B (reasoning).
    Uses OpenAI-compatible chat/completions format.
    """
    url = "http://localhost:8585/v1/chat/completions"
    headers = {"Content-Type": "application/json"}

    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    }

    session = get_session()
    try:
        async with session.post(url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=8)) as resp:
            if resp.status != 200:
                err_text = await resp.text()
                return f"[Error] Bonsai Router failed (status {resp.status}): {err_text}"
            data = await resp.json()
            try:
                return data["choices"][0]["message"]["content"]
            except (KeyError, IndexError):
                return f"[Error] Failed to parse Bonsai response: {json.dumps(data)[:200]}"
    except Exception as e:
        return f"[Error] Bonsai Router unreachable: {e}"

# ─── Provider: InceptionLabs (Mercury dLLM - Diffusion-based) ───────────────────

async def generate_with_inception(system_prompt: str, prompt: str, model_name: str = "mercury-coder-small") -> str:
    """Generate response using InceptionLabs Mercury dLLM (diffusion-based, ultra-fast)."""
    api_key = os.environ.get("INCEPTION_API_KEY")
    if not api_key:
        return "[Error] INCEPTION_API_KEY not set"
    url = "https://api.inceptionlabs.ai/v1/chat/completions"
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 4096,
        "temperature": 0.7
    }
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=8)) as resp:
                if resp.status != 200:
                    err_text = await resp.text()
                    return f"[Error] InceptionLabs API failed (status {resp.status}): {err_text[:300]}"
                data = await resp.json()
                return data["choices"][0]["message"]["content"]
    except Exception as e:
        return f"[Error] InceptionLabs API error: {e}"


# ─── Provider: AMD Instella-MoE (Local 16B / 2.8B Active) ─────────────────────

async def generate_with_instella(system_prompt: str, prompt: str, model_name: str = "gemma4:12b", agent_name: str = "") -> str:
    """
    Generate response using TRM Swarm OS microkernel pipeline.
    Primary: Per-agent trained TRM 7M model (1.87M params, in-process)
    Secondary: Vulkan GPU Bonsai-27B (port 8587)
    Tertiary: Ollama fallback (port 11434)
    """
    session = get_session()
    combo_prompt = f"{system_prompt}\n\n{prompt}" if system_prompt else prompt
    
    # PRIMARY: trained K3 triage (per-agent, ternary, measured).
    #
    # This classifies the REQUEST; it does not answer it. A 3-class gate cannot
    # answer "what is 91 prime", so the real model below produces the answer and the
    # trit travels with it.
    #
    # DEFECT FIX 2026-11-06: this block used to load {agent}_trm7m.pt and RETURN a
    # canned string ("[TRM 7M • 1.87M params • PROCEED]", "Available: 15 agents, 88
    # trie patterns, tau=2.82us") for every prompt. That checkpoint is seed-42 random
    # weights (3,910,833 params, p50 330,800 ns, always +1 conf=0.3333 - see
    # evidence/), so the banner presented the hash of a random projection as a model
    # answer. The trained heads now answer the only question they were trained for:
    # BLOCK / ANALYZE / PROCEED.
    triage = None
    if agent_name:
        try:
            from .swarm_council import RustCouncilClient
            triage = RustCouncilClient().classify(agent_name, prompt)
        except Exception as exc:
            triage = None
            logger.debug("k3 triage unavailable for %s: %s", agent_name, exc)
    if triage:
        LAST_TRIAGE[agent_name or "-"] = triage

    # SECONDARY: Vulkan GPU llama-server (port 8587). The model actually loaded
    # in that server is read from the response — it is NOT always Bonsai-27B.
    try:
        vulkan_payload = {
            "prompt": combo_prompt,
            "n_predict": 256,
            "temperature": 0.3,
            "stream": False
        }
        async with session.post("http://127.0.0.1:8587/completion", json=vulkan_payload,
                                timeout=aiohttp.ClientTimeout(total=30)) as resp:
            if resp.status == 200:
                data = await resp.json()
                content = data.get("content", "")
                if content:
                    tps = data.get("timings", {}).get("predicted_per_second", 0)
                    # DEFECT FIX 2026-11-03: the banner hardcoded "Bonsai-27B" while
                    # port 8587 actually serves whatever GGUF was launched (observed:
                    # qwen2.5-1.5b-instruct-q4_k_m). That reported a model the platform
                    # was not running. The server returns its loaded model path per
                    # response, so the banner now names the real one.
                    served = data.get("model") or ""
                    served_name = served.replace("\\", "/").rsplit("/", 1)[-1] or "unknown-model"
                    return f"[Instella-MoE Vulkan GPU • {served_name} • {tps:.0f} tok/s]\n{content.strip()}"
    except Exception:
        pass
    
    # FALLBACK: Ollama (models: gemma4:12b, deepseek-r1:1.5b, etc.)
    return await _ollama_fallback(session, combo_prompt)

async def _ollama_fallback(session, prompt: str) -> str:
    """Fallback Ollama inference with multiple model attempts."""
    models = ["gemma4:12b", "deepseek-r1:1.5b", "gemma3:4b", "llama3.2:3b"]
    ollama_url = "http://localhost:11434/api/generate"
    for m in models:
        payload = {"model": m, "prompt": prompt, "stream": False,
                    "options": {"temperature": 0.3}}
        try:
            async with session.post(ollama_url, json=payload,
                                    timeout=aiohttp.ClientTimeout(total=15)) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    text = data.get("response", "")
                    if text:
                        return f"[Instella-MoE Ollama ({m})] {text}"
        except Exception:
            pass
    return f"[Error] All local inference endpoints unreachable."


# ─── Provider: Intel OpenVINO 2026 (Sparse MoE Acceleration) ────────────────

async def generate_with_openvino(system_prompt: str, prompt: str, model_name: str = "openvino-moe-qwen3") -> str:
    """
    Generate response via Intel OpenVINO 2026 Model Server (localhost:9000).
    Optimized for sparse MoE token routing across Intel CPU/GPU/NPU hardware.
    """
    url = "http://localhost:9000/v1/chat/completions"
    headers = {"Content-Type": "application/json"}
    payload = {
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 4096
    }
    session = get_session()
    try:
        async with session.post(url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=8)) as resp:
            if resp.status != 200:
                err_text = await resp.text()
                return f"[Error] OpenVINO MoE server failed (status {resp.status}): {err_text[:200]}"
            data = await resp.json()
            return data["choices"][0]["message"]["content"]
    except Exception as e:
        return f"[Error] OpenVINO MoE server unreachable: {e}"


# ─── Provider Registry ──────────────────────────────────────────────────────────

PROVIDER_MAP = {
    "gemini":     generate_with_gemini,
    "openrouter": generate_with_openrouter,
    "deepseek":   generate_with_deepseek,
    "openai":     generate_with_openai,
    "anthropic":  generate_with_anthropic,
    "groq":       generate_with_groq,
    "cerebras":   generate_with_cerebras,
    "grok":       generate_with_grok,
    "perplexity": generate_with_perplexity,
    "nvidia":     generate_with_nvidia,
    "inception":  generate_with_inception,
    "bonsai":     generate_with_bonsai,
    "instella":   generate_with_instella,
    "openvino":   generate_with_openvino,
}

# Fallback chain: local-only — all cloud APIs removed for total on-device operation
FALLBACK_CHAIN = ["instella", "local", "bonsai"]


async def route_llm_request(backend: str, system_prompt: str, prompt: str, agent_name: str, model: str = "") -> Tuple[str, Optional[str]]:
    """
    Route the generation request to the appropriate LLM Regional Manager based on the agent's backend.
    Returns (response_text, reasoning_trace)
    """
    trace = None
    logger.info(f"Routing request for {agent_name} -> {backend}")
    
    # Try the primary backend first
    try:
        provider_fn = PROVIDER_MAP.get(backend)
        if provider_fn:
            response = await provider_fn(system_prompt, prompt)
        elif backend == "local":
            # LOCAL-ONLY: Route through Instella MoE microkernel (TRM 7M → Vulkan → Ollama)
            response = await generate_with_instella(system_prompt, prompt, agent_name=agent_name)
        else:
            # Unknown backend — route through Instella microkernel
            logger.warning(f"Unknown backend '{backend}', falling back to Instella MoE")
            response = await generate_with_instella(system_prompt, prompt, agent_name=agent_name)
            
        # Handle local dict response
        if isinstance(response, dict):
            trace = response.get("thought")
            response = response.get("content", "[No Content]")

        if isinstance(response, str) and response.startswith("[Error]") and backend != "local":
            logger.warning(f"{backend} returned an error, triggering fallback chain: {response[:100]}")
            raise Exception(f"External API {backend} returned error: {response}")

        logger.info(f"Received response from {backend} for {agent_name}")
        return response, trace
        
    except Exception as e:
        import traceback
        err = traceback.format_exc()
        print(f"[LLM Router] Error with backend '{backend}': {err}")
        
        # Fallback chain: try each fallback provider in order
        if backend != "local":
            for fallback_backend in FALLBACK_CHAIN:
                if fallback_backend == backend:
                    continue  # Skip the one that already failed
                
                print(f"[LLM Router] Trying fallback: {fallback_backend} for {agent_name}")
                logger.info(f"Trying fallback {fallback_backend} for {agent_name}")
                
                try:
                    if fallback_backend == "local":
                        from swarm_v2.core.llm_brain import llm_chat
                        fallback_resp = await llm_chat(system_prompt, prompt)
                    else:
                        fallback_fn = PROVIDER_MAP.get(fallback_backend)
                        if not fallback_fn:
                            continue
                        fallback_resp = await fallback_fn(system_prompt, prompt)
                    
                    if isinstance(fallback_resp, dict):
                        return fallback_resp.get("content", ""), fallback_resp.get("thought")
                    if isinstance(fallback_resp, str) and fallback_resp.startswith("[Error]"):
                        continue  # Try next fallback
                    
                    logger.info(f"Fallback to {fallback_backend} successful for {agent_name}")
                    return fallback_resp, None
                except Exception as fallback_error:
                    print(f"[LLM Router] Fallback {fallback_backend} also failed: {fallback_error}")
                    continue
            
            # All fallbacks exhausted
            return f"[Error] All backends failed for {agent_name}. Tried: {backend} -> {FALLBACK_CHAIN}", None
        
        return f"[Error] Backend {backend} failed internally.", None

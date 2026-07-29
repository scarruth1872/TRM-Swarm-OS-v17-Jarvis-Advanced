import os
import aiohttp
import json
import logging
from typing import Optional, Dict, Any, Tuple
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

logger = logging.getLogger("LLMRouter")

_HTTP_SESSION: Optional[aiohttp.ClientSession] = None

def get_session() -> aiohttp.ClientSession:
    global _HTTP_SESSION
    if _HTTP_SESSION is None or _HTTP_SESSION.closed:
        _HTTP_SESSION = aiohttp.ClientSession()
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
        async with session.post(url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=300)) as resp:
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
            async with session.post(url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=60)) as resp:
                if resp.status != 200:
                    err_text = await resp.text()
                    return f"[Error] InceptionLabs API failed (status {resp.status}): {err_text[:300]}"
                data = await resp.json()
                return data["choices"][0]["message"]["content"]
    except Exception as e:
        return f"[Error] InceptionLabs API error: {e}"


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
}

# Fallback chain: if primary fails, try these in order
FALLBACK_CHAIN = ["deepseek", "groq", "inception", "cerebras", "gemini", "bonsai", "local"]


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
            # Fallback to local Ollama/TRM reasoning
            from swarm_v2.core.llm_brain import llm_chat
            response = await llm_chat(system_prompt, prompt)
        else:
            # Unknown backend — try local
            logger.warning(f"Unknown backend '{backend}', falling back to local Ollama")
            from swarm_v2.core.llm_brain import llm_chat
            response = await llm_chat(system_prompt, prompt)
            
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

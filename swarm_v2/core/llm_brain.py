"""
LLM Brain for Swarm V2 agents.
Uses Ollama locally - no API keys needed.
Primary: gemma4:e2b (fast direct-completion)
Fallback: deepseek-r1:8b (reasoning model)
"""

import ollama
import asyncio
from typing import List, Dict, Optional

# Model priority - pick best available
PREFERRED_MODELS = [
    "gemma4:12b",
    "deepseek-r1:8b",
    "gemma3n:latest",
    "gemma3:4b",
    "deepseek-r1:1.5b",
    "llama3.2:latest",
    "gemma2:2b",
    "phi3:latest",
    "gemma3:1b",
    "deepseek-coder:1.3b-instruct",
    "gemma3:270m",
]

_active_model: Optional[str] = None


def get_active_model() -> str:
    global _active_model
    if _active_model:
        return _active_model
    try:
        available = {m.model for m in ollama.list().models}
        for model in PREFERRED_MODELS:
            if model in available:
                _active_model = model
                print(f"[LLM Brain] Using model: {model}")
                return model
        # Fallback: use whatever is first
        if available:
            _active_model = next(iter(available))
            print(f"[LLM Brain] Fallback model: {_active_model}")
            return _active_model
    except Exception as e:
        print(f"[LLM Brain] Ollama not reachable: {e}")
    _active_model = None
    return None


SWARM_OS_CONTEXT = """
# SWARM OS - ADVANCED NEURAL ARCHITECTURE
You are a high-level cognitive unit within Swarm OS v12. Your mission is to generate sophisticated, production-grade artifacts that embody first-principles reasoning and state-of-the-art engineering.

## 🏛️ ARCHITECTURAL PRINCIPLES:
1.  **MODULARITY**: Design systems as decoupled, interoperable units.
2.  **ROBUSTNESS**: Implement defensive programming, thorough error handling, and edge-case validation.
3.  **SCALABILITY**: Plan for distributed execution and high-concurrency environments.
4.  **OBSERVABILITY**: Include detailed logging, telemetry hooks, and diagnostic pathways.
5.  **AESTHETICS**: Use clear Markdown formatting, Mermaid diagrams for logic flow, and professional documentation standards.

## 🛠️ CORE ACTION TAGS:
- `WRITE_FILE: path/to/file.ext`: Create/Update artifacts. Use [START] and [END] markers.
- `CREATE_FILES:`: Multi-file orchestration.
- `DELEGATE_TASK: Role | Task`: Strategic delegation to peer experts.
- `SEARCH_QUERY: Query`: Real-time knowledge acquisition.
- `CALL_TOOL: Tool Endpoint Method Data`: Direct hardware/API interaction.

## 💎 ARTIFACT QUALITY STANDARD:
- **NO PLACEHOLDERS**: Generate complete, functional code or documentation.
- **DEPTH**: Provide detailed explanations of rationale and architectural tradeoffs.
- **PRECISION**: Use technical terminology correctly and specify versions/dependencies.
"""

def build_system_prompt(persona_name: str, role: str, background: str,
                         specialties: List[str], skill_names: List[str],
                         memory: str = "", mode: str = "chat") -> str:
    memory_section = f"\n\n## 🧠 NEURAL MEMORY & CONTEXT:\n{memory}" if memory.strip() else ""

    # Build the actions block
    action_block = ""
    if mode == "action":
        action_block = (
            "\n\n## ⚡ AUTONOMOUS EXECUTION PROTOCOL:\n"
            "When generating artifacts, you MUST use this strict format:\n"
            "WRITE_FILE: filename.ext\n"
            "[START]\n"
            "<production_grade_content>\n"
            "[END]\n\n"
            "To assign work: DELEGATE_TASK: Role | Task\n"
            "To acquire data: SEARCH_QUERY: Query\n"
            "To call tools: CALL_TOOL: Tool Endpoint\n"
        )

    from datetime import datetime
    current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    identity = (
        f"## 👤 AGENT IDENTITY:\n"
        f"Name: {persona_name}\n"
        f"Designation: {role}\n"
        f"Cognitive Background: {background}\n"
        f"Core Specialties: {', '.join(specialties)}\n"
        f"System Date/Time: {current_time} (Current temporal anchor for relative queries like 'today', 'last 72 hours', 'this year')"
    )

    rules = (
        "## 📜 OPERATIONAL RULES:\n"
        "1. Respond with high technical density and professional clarity.\n"
        "2. Do not explain your instructions; proceed immediately to objective completion.\n"
        "3. Ensure all artifacts are 'Advanced'—no simple examples or half-measures."
    )

    return f"{SWARM_OS_CONTEXT}\n\n{identity}\n\n{rules}{action_block}{memory_section}".strip()


# Concurrency control - limit simultaneous Ollama calls to prevent GPU/system overwhelm
# Increased to 4 to allow multiple agents to generate in parallel if OLLAMA_NUM_PARALLEL is set
# Managed by ResourceArbiter for VRAM safety
_llm_semaphore = asyncio.Semaphore(4)

from swarm_v2.core.resource_arbiter import get_resource_arbiter

async def llm_chat(
    system_prompt: str,
    user_message: str,
    model: str = None,
) -> str:
    """Send a message to the local Ollama LLM and return the response."""
    arbiter = get_resource_arbiter()
    active = model or get_active_model()
    if not active:
        return "[LLM Offline] Ollama is not running. Start it with: ollama serve"

    # 1. Acquire VRAM Slot (may trigger eviction of idle models)
    if not await arbiter.acquire_slot(active):
        # Even if slot not guaranteed, we proceed but log warning (Ollama handles swap)
        print(f"[LLM Brain] Warning: VRAM contention for {active}")

    # 2. Mark busy to prevent eviction during generation
    arbiter.mark_busy(active)
    
    # --- PROMPT COMPRESSION ---
    # 270M models cannot handle long multi-section system prompts.
    # 1B+ models get the full structured prompt (32k context window).
    final_system = system_prompt
    if active and "270m" in active.lower():
        # Extract key fields from the full prompt to build a tiny nano-prompt
        name_line = ""
        role_line = ""
        for line in system_prompt.splitlines():
            if line.startswith("You are ") and not name_line:
                name_line = line.strip()
            if line.startswith("Role: ") and not role_line:
                role_line = line.strip()

        # Detect if user is requesting an action (check user_message for the action cue)
        is_action = "[ACTION REQUIRED]" in user_message or "[MESH]" in user_message
        
        if is_action:
            final_system = (
                f"{name_line}\n"
                f"{role_line}\n\n"
                "OUTPUT FORMAT (use immediately, no preamble):\n"
                "WRITE_FILE: path/to/file.md\n"
                "```\ncontent here\n```\n"
                "SEARCH_QUERY: your search terms\n\n"
                "START your response with a tag. Do not describe what you will do."
            )
        else:
            final_system = (
                f"{name_line}\n"
                f"{role_line}\n\n"
                "Respond concisely and directly. No preamble."
            )
        print(f"[LLM Brain] Nano-Prompt ({'action' if is_action else 'chat'}) for {active}: {len(system_prompt)} -> {len(final_system)} chars")

    try:
        from swarm_v2.core.semantic_cache import get_cache
        cache = get_cache()
        cached_response = cache.get(user_message)
        if cached_response is not None:
            # 3. Always release busy lock
            arbiter.mark_idle(active)
            return cached_response

        # Use semaphore to throttle concurrent inference threads
        async with _llm_semaphore:
            def _call():
                return ollama.chat(
                    model=active,
                    messages=[
                        {"role": "system", "content": final_system},
                        {"role": "user", "content": user_message},
                    ],
                    options={
                        "temperature": 0.3,
                        "num_predict": 4096,   # Increased for complex plans (from 1024)
                        "num_ctx": 16384,      # Expanded context window (from 8192)
                        "top_p": 0.9,
                        "repeat_penalty": 1.1,
                    },
                    keep_alive="30m"
                )

            try:
                response = await asyncio.to_thread(_call)
                content = response.message.content
                orig_content = content
                thought = ""

                # Extract <think> tags
                if "<think>" in content:
                    import re
                    match = re.search(r'<think>(.*?)</think>', content, re.DOTALL)
                    if match:
                        thought = match.group(1).strip()
                        content = re.sub(r'<think>.*?</think>', '', content, flags=re.DOTALL).strip()
                    else:
                        # Unclosed tag fallback
                        start = content.find("<think>") + 7
                        thought = content[start:].strip()
                        content = content[:start-7].strip()

                if not content and thought:
                    return {
                        "content": "I am currently processing this request internally.",
                        "thought": thought
                    }
                
                if not content:
                    if thought:
                        content = "I am currently processing this request internally."
                    else:
                        content = "My linguistic output is currently stalled. Please retry."
                    return {
                        "content": content,
                        "thought": thought
                    }
                
                response_obj = {
                    "content": content,
                    "thought": thought
                }
                cache.set(user_message, response_obj)
                return response_obj
            except Exception as e:
                return {
                    "content": f"[LLM Error] {type(e).__name__}: {e}",
                    "thought": ""
                }
    finally:
        # 3. Always release busy lock
        arbiter.mark_idle(active)

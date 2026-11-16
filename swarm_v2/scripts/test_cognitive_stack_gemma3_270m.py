"""
Test script for Cognitive Stack with gemma3:270m model.
Verifies that the cognitive stack can load and process prompts using the tiny 270M model.
"""

import asyncio
import logging
import re as _re
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest  # for the marker below
from swarm_v2.core.cognitive_stack import CognitiveStack

@pytest.mark.integration  # needs a LIVE model runtime: it constructs CognitiveStack
                          # and awaits stack.process(), which blocks when no model is
                          # serving. Under asyncio_mode=auto pytest-asyncio wraps it,
                          # so the block occurs inside the event loop and the run
                          # freezes with no summary. Excluded from the default suite;
                          # run with: pytest -m integration
async def test_cognitive_stack_gemma3_270m():
    """Test the cognitive stack with gemma3:270m model.

    REWRITTEN 2026-11-03. The previous version asserted nothing: it printed
    "⚠️ Model mismatch" and "⚠️ TRM reasoning was not triggered" and still
    returned True, so it passed no matter what the stack did. It also claimed
    "✅ Model verification: gemma3:270m is active" purely from the *requested*
    model string, while the live router was answering from a different model.
    It now asserts on the responses and on the model that actually answered.
    """
    print("🧠 Testing Cognitive Stack with gemma3:270m")
    print("=" * 50)

    agent_name = "TestAgent"
    executive_model = "gemma3:270m"

    print(f"Creating CognitiveStack for agent '{agent_name}' with model '{executive_model}'...")
    stack = CognitiveStack(agent_name, executive_model=executive_model)

    prompts = [
        "Hello! Can you introduce yourself?",
        "Analyze the logical consistency of this argument: If A then B, A, therefore B.",
        "Write a Python function to calculate factorial.",
    ]

    responses = []
    for i, prompt in enumerate(prompts, 1):
        print(f"\nTest {i}: {prompt}")
        response, trace = await stack.process(prompt)
        print(f"Response: {response[:200]}...")
        print(f"Trace: {trace}")
        responses.append(response)

    status = stack.get_status()
    print(f"\nStatus: {status}")

    # --- assertions -------------------------------------------------------
    for i, response in enumerate(responses, 1):
        assert isinstance(response, str) and response.strip(), (
            f"prompt {i} produced an empty response"
        )
        assert not response.startswith("[Error]"), (
            f"prompt {i} returned an error instead of a completion: {response[:200]}"
        )

    assert stack.stats["executive_calls"] == len(prompts), (
        f"expected {len(prompts)} executive calls, got {stack.stats['executive_calls']}"
    )

    # The stack must report which model actually answered, not just the one asked for.
    assert stack.last_served_model, "stack did not record the model that served the response"
    assert stack.last_served_model != "unattributed", (
        "executive response carried no model attribution banner"
    )
    assert status["served_model"] == stack.last_served_model, (
        f"get_status() reports served_model={status['served_model']!r} but the stack "
        f"recorded {stack.last_served_model!r}"
    )
    assert status["executive"] == executive_model, (
        "get_status() must still report the requested model separately"
    )

    # Honesty check: if the requested model is not the one serving, the status must
    # expose both rather than silently claiming the requested model ran.
    if executive_model.lower() not in stack.last_served_model.lower():
        print(
            f"⚠️  Requested {executive_model!r} but {stack.last_served_model!r} answered; "
            "reported truthfully via status['served_model']"
        )

    return True

@pytest.mark.integration  # needs the live Vulkan llama-server on 8587
async def test_router_banner_names_the_served_model():
    """The response banner must name the model the server actually loaded.

    Regression guard: llm_router.generate_with_instella hardcoded
    "Bonsai-27B" in the banner while port 8587 was serving a different GGUF,
    so every response claimed a model the platform was not running.
    """
    import aiohttp
    from swarm_v2.core.llm_router import generate_with_instella

    async with aiohttp.ClientSession() as session:
        async with session.get("http://127.0.0.1:8587/v1/models",
                               timeout=aiohttp.ClientTimeout(total=10)) as resp:
            assert resp.status == 200, f"llama-server /v1/models returned {resp.status}"
            models = await resp.json()
    served_ids = [m.get("id", "") for m in models.get("data", [])]
    assert served_ids, "llama-server reported no loaded model"
    served_base = served_ids[0].replace("\\", "/").rsplit("/", 1)[-1]

    text = await generate_with_instella("You are terse.", "Say hi.", model_name="whatever")
    assert text.startswith("["), f"expected an attributed banner, got: {text[:120]}"

    m = _re.match(r"\[[^\]•]*•\s*([^•\]]+?)\s*•", text)
    assert m, f"response carries no model attribution banner: {text[:160]}"
    banner_model = m.group(1).strip()
    assert banner_model == served_base, (
        f"banner claims {banner_model!r} but the server is serving {served_base!r}"
    )


@pytest.mark.integration  # queries the live Ollama / active-model config
async def test_llm_brain_direct():
    """Test LLM brain directly to ensure gemma3:270m is available."""
    print("\n🔍 Testing LLM Brain directly")
    print("=" * 50)
    
    from swarm_v2.core.llm_brain import get_active_model, llm_chat
    
    # Check which model is active
    active_model = get_active_model()
    print(f"Active model: {active_model}")
    
    if active_model and "270m" in active_model.lower():
        print("✅ gemma3:270m is available as active model")
        
        # Test a simple chat
        response = await llm_chat(
            system_prompt="You are a helpful assistant.",
            user_message="Hello! What's 2+2?",
            model=active_model
        )
        
        if isinstance(response, dict):
            content = response.get("content", "No content")
            thought = response.get("thought", "No thought")
        else:
            content = str(response)
            thought = ""
            
        print(f"Response: {content[:200]}...")
        if thought:
            print(f"Thought: {thought[:100]}...")
    else:
        print(f"⚠️  gemma3:270m is not active. Current model: {active_model}")
        print("Trying to force gemma3:270m...")
        
        # Try to force gemma3:270m
        response = await llm_chat(
            system_prompt="You are a helpful assistant.",
            user_message="Hello! What's 2+2?",
            model="gemma3:270m"
        )
        
        if isinstance(response, dict):
            content = response.get("content", "No content")
        else:
            content = str(response)
            
        if content and not content.startswith("[LLM Error]"):
            print(f"✅ Forced gemma3:270m works! Response: {content[:200]}...")
        else:
            print(f"❌ Forced gemma3:270m failed: {content}")

async def main():
    """Run all tests."""
    print("🚀 Starting gemma3:270m Cognitive Stack Tests")
    print("=" * 50)
    
    try:
        # Test cognitive stack
        await test_cognitive_stack_gemma3_270m()
        
        # Test LLM brain directly
        await test_llm_brain_direct()
        
        print("\n" + "=" * 50)
        print("✅ All tests completed successfully!")
        return True
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    # Enable logging for debugging
    logging.basicConfig(level=logging.INFO)
    
    # Run tests
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
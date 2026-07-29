import asyncio
import os
from swarm_v2.core.llm_router import route_llm_request

async def test_keys():
    print(f"GEMINI_API_KEY set: {bool(os.environ.get('GEMINI_API_KEY'))}")
    print(f"DEEPSEEK_API_KEY set: {bool(os.environ.get('DEEPSEEK_API_KEY'))}")
    
    print("\nTesting Gemini...")
    resp, _ = await route_llm_request("gemini", "System", "Hello, are you online?", "Tester")
    print(f"Gemini Response: {resp[:100]}...")
    
    print("\nTesting DeepSeek...")
    resp, _ = await route_llm_request("deepseek", "System", "Hello, are you online?", "Tester")
    print(f"DeepSeek Response: {resp[:100]}...")

    print("\nTesting OpenRouter...")
    resp, _ = await route_llm_request("openrouter", "System", "Hello, are you online?", "Tester")
    print(f"OpenRouter Response: {resp[:100]}...")

if __name__ == "__main__":
    asyncio.run(test_keys())

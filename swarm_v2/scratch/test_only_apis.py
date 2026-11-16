import asyncio
import os
import aiohttp

async def test_apis():
    gemini_key = os.environ.get("GEMINI_API_KEY")
    deepseek_key = os.environ.get("DEEPSEEK_API_KEY")
    openrouter_key = os.environ.get("OPENROUTER_API_KEY")

    print(f"GEMINI_API_KEY: {gemini_key[:10] if gemini_key else 'None'}...")
    print(f"DEEPSEEK_API_KEY: {deepseek_key[:10] if deepseek_key else 'None'}...")
    print(f"OPENROUTER_API_KEY: {openrouter_key[:10] if openrouter_key else 'None'}...")

    async with aiohttp.ClientSession() as session:
        if gemini_key:
            url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={gemini_key}"
            payload = {"contents": [{"parts": [{"text": "Hello"}]}]}
            async with session.post(url, json=payload) as resp:
                print(f"Gemini API Response Status: {resp.status}")
                print(f"Gemini Body: {await resp.text()}")

        if deepseek_key:
            # Test deepseek-coder
            url = "https://api.deepseek.com/chat/completions"
            headers = {"Authorization": f"Bearer {deepseek_key}", "Content-Type": "application/json"}
            payload_coder = {"model": "deepseek-coder", "messages": [{"role": "user", "content": "Hello"}]}
            async with session.post(url, headers=headers, json=payload_coder) as resp:
                print(f"DeepSeek deepseek-coder Status: {resp.status}")

            # Test deepseek-chat
            payload_chat = {"model": "deepseek-chat", "messages": [{"role": "user", "content": "Hello"}]}
            async with session.post(url, headers=headers, json=payload_chat) as resp:
                print(f"DeepSeek deepseek-chat Status: {resp.status}")

        if openrouter_key:
            url = "https://openrouter.ai/api/v1/chat/completions"
            headers = {"Authorization": f"Bearer {openrouter_key}", "Content-Type": "application/json"}
            payload = {"model": "google/gemma-2-9b-it:free", "messages": [{"role": "user", "content": "Hello"}]}
            async with session.post(url, headers=headers, json=payload) as resp:
                print(f"OpenRouter API Response Status: {resp.status}")
                print(f"OpenRouter Body: {await resp.text()}")

if __name__ == "__main__":
    asyncio.run(test_apis())

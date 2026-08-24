
import os
import requests
import json
from typing import Optional, List

from dotenv import load_dotenv

# Load .env at the source-level to ensure key is available to all instances
load_dotenv()

# Try to get API key from environment, default to None (will return simulated results)
# In production, set TAVILY_API_KEY environment variable.
TAVILY_API_KEY = os.environ.get("TAVILY_API_KEY")

class WebSearchSkill:
    """
    Web Search Skill for Swarm V2.
    Integrates with Tavily AI for high-quality, LLM-optimized search results.
    Fallbacks to a simulated mode if no API key is present.
    """
    skill_name = "WebSearchSkill"
    description = "Search the real-time web for information, documentation, and data."

    def __init__(self, api_key: str = None):
        self.api_key = api_key or TAVILY_API_KEY

    def search(self, query: str, max_results: int = 5) -> str:
        """Perform a web search."""
        if self.api_key:
            try:
                url = "https://api.tavily.com/search"
                payload = {
                    "api_key": self.api_key,
                    "query": query,
                    "search_depth": "basic",
                    "max_results": max_results,
                    "include_answer": True
                }
                response = requests.post(url, json=payload, timeout=10)
                if response.status_code == 200:
                    data = response.json()
                    return self._format_results(data)
            except Exception:
                pass

        # Fallback 1: Perplexity Search
        perp_res = self._perplexity_search(query)
        if perp_res and not "Search Error" in perp_res and not "Search Exception" in perp_res:
            return perp_res

        # Fallback 2: Free DDG search
        ddg_res = self._free_ddg_search(query)
        if ddg_res:
            return ddg_res

        # Fallback 3: Local Simulated search
        return self._simulated_search(query)

    def _format_results(self, data: dict) -> str:
        """Format Tavily JSON response into a readable string for the agent."""
        output = []
        if data.get("answer"):
            output.append(f"💡 **AI Answer:** {data['answer']}\n")
        
        results = data.get("results", [])
        if not results:
            return "🔍 No relevant results found."

        output.append(f"🔍 Found {len(results)} sources:\n")
        for i, res in enumerate(results, 1):
            title = res.get("title", "Untitled")
            url = res.get("url", "#")
            content = res.get("content", "")[:300].replace("\n", " ")
            output.append(f"[{i}] **{title}**\n    🔗 {url}\n    📝 {content}...")
        
        return "\n".join(output)

    def _perplexity_search(self, query: str) -> Optional[str]:
        perplexity_key = os.environ.get("PERPLEXITY_API_KEY")
        if not perplexity_key:
            return None
        try:
            import requests
            url = "https://api.perplexity.ai/chat/completions"
            headers = {
                "Authorization": f"Bearer {perplexity_key}",
                "Content-Type": "application/json"
            }
            payload = {
                "model": "sonar",
                "messages": [
                    {"role": "system", "content": "Perform a web search and return a concise summary of findings with source URLs."},
                    {"role": "user", "content": query}
                ],
                "temperature": 0.2
            }
            response = requests.post(url, json=payload, timeout=15)
            if response.status_code == 200:
                result = response.json()
                content = result["choices"][0]["message"]["content"]
                return f"🔍 **Perplexity Real-Time Web Search Results:**\n\n{content}"
            else:
                return f"⚠️ Perplexity Search Error ({response.status_code}): {response.text}"
        except Exception as e:
            return f"⚠️ Perplexity Search Exception: {e}"

    def _free_ddg_search(self, query: str) -> Optional[str]:
        """Free web search scraper using DuckDuckGo HTML without API keys."""
        try:
            import urllib.request
            import urllib.parse
            import re
            
            url = "https://html.duckduckgo.com/html/?" + urllib.parse.urlencode({"q": query})
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            }
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=10) as response:
                html = response.read().decode('utf-8')
            
            results = []
            matches = re.findall(
                r'<a class="result__url" href="([^"]+)">\s*([\s\S]*?)\s*</a>[\s\S]*?<a class="result__snippet"[^>]*>\s*([\s\S]*?)\s*</a>', 
                html
            )
            
            for url_match, title_match, snippet_match in matches[:5]:
                title = re.sub(r'<[^>]+>', '', title_match).strip()
                snippet = re.sub(r'<[^>]+>', '', snippet_match).strip()
                
                title = " ".join(title.split())
                snippet = " ".join(snippet.split())
                
                # Extract clean target URL from DDG redirect url
                real_url = url_match
                if "uddg=" in url_match:
                    match_url = re.search(r'uddg=([^&]+)', url_match)
                    if match_url:
                        real_url = urllib.parse.unquote(match_url.group(1))
                
                if real_url.startswith("//"):
                    real_url = "https:" + real_url
                
                results.append(f"**{title}**\n🔗 {real_url}\n📝 {snippet}\n")
                
            if results:
                return "🔍 **Free Web Search Results (DuckDuckGo):**\n\n" + "\n".join(results)
            return None
        except Exception as e:
            return f"⚠️ Free search failed: {e}"

    def _simulated_search(self, query: str) -> str:
        """Fallback to local artifact search when TAVILY_API_KEY is missing."""
        results = []
        base_dir = "swarm_v2_artifacts"
        
        # Scan local artifacts for relevant content
        query_words = query.lower().split()
        count = 0
        if os.path.exists(base_dir):
            for root, _, files in os.walk(base_dir):
                for file in files:
                    if count >= 3: break
                    if any(word in file.lower() for word in query_words) or file.endswith(".md"):
                        try:
                            path = os.path.join(root, file)
                            with open(path, "r", encoding="utf-8") as f:
                                content = f.read(200).replace("\n", " ")
                            results.append(f"[{len(results)+1}] **Local Knowledge: {file}**\n    🔗 {path}\n    📝 {content}...")
                            count += 1
                        except:
                            pass
        
        if not results:
            return (
                f"🔍 [SIMULATED] No local matches for: '{query}'\n"
                f"⚠️ TAVILY_API_KEY is not set. Suggesting DocIngestionSkill for deep learning."
            )

        header = f"🔍 [LOCAL KNOWLEDGE MOCK] Results for: '{query}' (No API Key)\n"
        return header + "\n\n".join(results)

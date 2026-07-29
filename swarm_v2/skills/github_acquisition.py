"""
GitHub Skill Acquisition Pipeline — Auto-discover, analyze, and ingest GitHub repos as skills.
Integrates with Learning Engine + SkillSynthesisPipeline to autonomously grow the skill library.
"""
import os, json, re, asyncio
from typing import Optional
from swarm_v2.skills.learning_engine import get_learning_engine
from swarm_v2.skills.synthesis_pipeline import SkillSynthesisPipeline

class GitHubAcquisitionPipeline:
    """Discovers, analyzes, and ingests GitHub repos as learned skills."""

    def __init__(self):
        self.github_token = os.getenv("GITHUB_TOKEN", "")
        self.headers = {"Accept": "application/vnd.github.v3+json"}
        if self.github_token:
            self.headers["Authorization"] = f"Bearer {self.github_token}"

    async def search_repos(self, query: str, max_results: int = 5) -> list[dict]:
        """Search GitHub repos by topic/query."""
        import aiohttp
        url = f"https://api.github.com/search/repositories?q={query}&sort=stars&per_page={max_results}"
        async with aiohttp.ClientSession(headers=self.headers) as session:
            async with session.get(url) as resp:
                if resp.status != 200:
                    text = await resp.text()
                    raise RuntimeError(f"GitHub API error {resp.status}: {text[:200]}")
                data = await resp.json()
                return [{
                    "name": item["full_name"],
                    "url": item["html_url"],
                    "description": item.get("description", "") or "",
                    "stars": item.get("stargazers_count", 0),
                    "language": item.get("language", ""),
                    "topics": item.get("topics", []),
                    "default_branch": item.get("default_branch", "main"),
                } for item in data.get("items", [])[:max_results]]

    async def get_readme(self, repo_full_name: str) -> Optional[str]:
        """Fetch README content from a GitHub repo."""
        import aiohttp
        url = f"https://api.github.com/repos/{repo_full_name}/readme"
        async with aiohttp.ClientSession(headers=self.headers) as session:
            async with session.get(url) as resp:
                if resp.status != 200:
                    return None
                data = await resp.json()
                import base64
                content = data.get("content", "")
                if content:
                    return base64.b64decode(content).decode("utf-8", errors="replace")
                return None

    async def get_repo_contents(self, repo_full_name: str, path: str = "") -> list[dict]:
        """List contents of a repo directory."""
        import aiohttp
        url = f"https://api.github.com/repos/{repo_full_name}/contents/{path}"
        async with aiohttp.ClientSession(headers=self.headers) as session:
            async with session.get(url) as resp:
                if resp.status != 200:
                    return []
                data = await resp.json()
                return [{"name": item["name"], "type": item["type"], "path": item["path"]}
                        for item in data if item["type"] == "file" and item["name"].endswith((".py", ".ts", ".js", ".json", ".md", ".yaml", ".toml"))]

    def extract_code_structure(self, content: str, filename: str) -> dict:
        """Extract classes, functions, and imports from source code (simple regex-based)."""
        classes = re.findall(r"class\s+(\w+)", content)
        functions = re.findall(r"(?:async\s+)?def\s+(\w+)", content)
        imports = re.findall(r"(?:from|import)\s+([\w.]+)", content)
        endpoints = re.findall(r'@(?:router|app)\.(?:get|post|put|delete|patch)\("([^"]+)"', content)
        return {
            "classes": classes,
            "functions": functions,
            "endpoints": endpoints,
            "imports": list(set(imports)),
        }

    async def acquire_from_repo(self, repo_info: dict, llm_generate_fn=None) -> dict:
        """Full pipeline: fetch README → analyze → synthesize → ingest."""
        print(f"[GitHubAcquisition] Acquiring from {repo_info['name']}...")

        # Step 1: Get README
        readme = await self.get_readme(repo_info["name"])
        if not readme:
            return {"repo": repo_info["name"], "status": "skipped", "reason": "no readme"}

        # Step 2: Get repo file structure
        files = await self.get_repo_contents(repo_info["name"])
        structure_info = {"files": files, "readme": readme[:2000]}

        # Step 3: Extract code structure from README (limited)
        code_structure = self.extract_code_structure(readme, "README.md")

        # Step 4: Synthesize skill
        if llm_generate_fn:
            pipeline = SkillSynthesisPipeline(llm_generate_fn=llm_generate_fn)
            parsed = {
                "filepath": f"github://{repo_info['name']}",
                "classes": code_structure.get("classes", []),
                "functions": code_structure.get("functions", []),
            }
            skill_candidate = await pipeline.synthesize(parsed)
            if not skill_candidate:
                return {"repo": repo_info["name"], "status": "failed", "reason": "synthesis returned None"}

            # Step 5: Ingest into learning engine
            engine = get_learning_engine()
            skill_name = f"github_{repo_info['name'].replace('/', '_').replace('-', '_')}"
            await engine.learn_from_text(
                name=skill_name,
                content=json.dumps(structure_info, indent=2),
                source=f"github://{repo_info['name']}",
                llm_generate=llm_generate_fn,
            )
            return {
                "repo": repo_info["name"],
                "status": "acquired",
                "skill_name": skill_name,
                "description": repo_info.get("description", ""),
                "files_scanned": len(files),
            }

        return {"repo": repo_info["name"], "status": "skipped", "reason": "no llm_generate_fn"}

    async def acquire_topics(self, topics: list[str], max_per_topic: int = 3, llm_generate_fn=None) -> list[dict]:
        """Acquire skills from multiple topic searches."""
        results = []
        for topic in topics:
            repos = await self.search_repos(topic, max_results=max_per_topic)
            print(f"[GitHubAcquisition] Found {len(repos)} repos for '{topic}'")
            for repo in repos:
                result = await self.acquire_from_repo(repo, llm_generate_fn)
                results.append(result)
        return results


# Singleton
_instance = None
def get_github_acquisition():
    global _instance
    if _instance is None:
        _instance = GitHubAcquisitionPipeline()
    return _instance

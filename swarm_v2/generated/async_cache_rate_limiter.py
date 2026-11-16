"""
Autonomous Microservice: Asynchronous Rate-Limited Cache Engine
Synthesized by Generative Architect (Archi) via Swarm OS v12
"""
import time
import asyncio
from typing import Dict, Any, Optional

class AsyncCacheRateLimiter:
    def __init__(self, rate_limit: int = 5, ttl_seconds: float = 2.0):
        self.rate_limit = rate_limit
        self.ttl = ttl_seconds
        self.cache: Dict[str, Dict[str, Any]] = {}
        self.request_counts: Dict[str, int] = {}
        self.start_time = time.time()

    async def is_allowed(self, client_id: str) -> bool:
        """Rate limiting logic: max requests per client."""
        current = self.request_counts.get(client_id, 0)
        if current >= self.rate_limit:
            return False
        self.request_counts[client_id] = current + 1
        return True

    async def get(self, key: str) -> Optional[Any]:
        """Fetch cached item if not expired."""
        entry = self.cache.get(key)
        if not entry:
            return None
        if time.time() - entry["ts"] > self.ttl:
            del self.cache[key]
            return None
        return entry["val"]

    async def set(self, key: str, val: Any):
        """Store item in cache with timestamp."""
        self.cache[key] = {"val": val, "ts": time.time()}

print("[Microservice] AsyncCacheRateLimiter compiled and ready!")

"""
Semantic Cache for Swarm OS LLM Responses
Caches semantically similar queries to avoid redundant model calls
"""

import asyncio
import hashlib
import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional


@dataclass
class CacheEntry:
    """A single cache entry"""

    key: str
    query_hash: str
    query: str
    response: Any
    created_at: float
    hit_count: int = 0
    last_hit_at: Optional[float] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    ttl_seconds: int = 3600  # Default 1 hour

    def is_expired(self, ttl_seconds: int) -> bool:
        """Check if entry has expired"""
        return (time.time() - self.created_at) > ttl_seconds

    def to_dict(self) -> Dict:
        """Convert to dictionary for serialization"""
        return {
            "key": self.key,
            "query_hash": self.query_hash,
            "query": self.query,
            "response": self.response,
            "created_at": self.created_at,
            "hit_count": self.hit_count,
            "last_hit_at": self.last_hit_at,
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: Dict) -> "CacheEntry":
        """Create from dictionary"""
        return cls(**data)


class SemanticCache:
    """
    Semantic cache for LLM responses.

    Features:
    - Semantic similarity matching (not just exact match)
    - Configurable TTL and max size
    - Persistent storage (optional)
    - Hit/miss metrics
    - Automatic eviction (LRU policy)
    """

    def __init__(
        self,
        max_size_mb: int = 512,
        default_ttl_seconds: int = 3600,
        similarity_threshold: float = 0.85,
        persistence_path: Optional[Path] = None,
    ):
        self.max_size_mb = max_size_mb
        self.default_ttl = default_ttl_seconds
        self.similarity_threshold = similarity_threshold
        self.persistence_path = persistence_path or Path(
            "swarm_v2_artifacts/semantic_cache.json"
        )

        # In-memory cache
        self._cache: Dict[str, CacheEntry] = {}
        self._access_order: List[str] = []  # For LRU eviction

        # Metrics
        self._hits = 0
        self._misses = 0
        self._evictions = 0

        # Load from disk if exists
        self._load_from_disk()

    def _compute_query_hash(self, query: str) -> str:
        """Compute a hash for the query"""
        # Normalize query (lowercase, strip whitespace)
        normalized = query.lower().strip()
        return hashlib.sha256(normalized.encode()).hexdigest()[:16]

    def _compute_semantic_similarity(self, query1: str, query2: str) -> float:
        """
        Compute semantic similarity between two queries.

        Uses simple string similarity.
        TODO: Replace with embedding-based similarity when available.
        """
        # Simple Jaccard similarity on word sets
        words1 = set(query1.lower().split())
        words2 = set(query2.lower().split())

        if not words1 or not words2:
            return 0.0

        intersection = len(words1 & words2)
        union = len(words1 | words2)

        return intersection / union if union > 0 else 0.0

    def _find_similar_entry(self, query: str) -> Optional[CacheEntry]:
        """Find a semantically similar entry in cache"""
        query_hash = self._compute_query_hash(query)

        # First check exact hash match
        if query_hash in self._cache:
            entry = self._cache[query_hash]
            if not entry.is_expired(self.default_ttl):
                return entry

        # Then check semantic similarity
        for entry in self._cache.values():
            if entry.is_expired(self.default_ttl):
                continue

            similarity = self._compute_semantic_similarity(query, entry.query)
            if similarity >= self.similarity_threshold:
                return entry

        return None

    def _evict_if_needed(self):
        """Evict entries if cache exceeds max size"""
        # Calculate current cache size (rough estimate)
        cache_size_mb = sum(
            len(json.dumps(entry.to_dict()).encode()) for entry in self._cache.values()
        ) / (1024 * 1024)

        # Evict oldest entries until under limit
        while cache_size_mb > self.max_size_mb and self._access_order:
            oldest_key = self._access_order.pop(0)
            if oldest_key in self._cache:
                del self._cache[oldest_key]
                self._evictions += 1

            # Recalculate
            cache_size_mb = sum(
                len(json.dumps(entry.to_dict()).encode())
                for entry in self._cache.values()
            ) / (1024 * 1024)

    def get(self, query: str) -> Optional[Any]:
        """
        Get cached response for a query.

        Args:
            query: The query string

        Returns:
            Cached response if found, None otherwise
        """
        entry = self._find_similar_entry(query)

        if entry:
            # Update hit stats
            entry.hit_count += 1
            entry.last_hit_at = time.time()

            # Update access order (move to end for LRU)
            if entry.key in self._access_order:
                self._access_order.remove(entry.key)
            self._access_order.append(entry.key)

            self._hits += 1
            print(f"[SemanticCache] HIT: '{query[:50]}...' (similarity match)")
            return entry.response

        self._misses += 1
        print(f"[SemanticCache] MISS: '{query[:50]}...'")
        return None

    def set(
        self,
        query: str,
        response: Any,
        ttl_seconds: Optional[int] = None,
        metadata: Optional[Dict] = None,
    ):
        """
        Cache a response for a query.

        Args:
            query: The query string
            response: The response to cache
            ttl_seconds: Time-to-live in seconds (uses default if None)
            metadata: Optional metadata to store
        """
        query_hash = self._compute_query_hash(query)

        # Check if already cached
        if query_hash in self._cache:
            print(f"[SemanticCache] UPDATE: '{query[:50]}...'")

        entry = CacheEntry(
            key=query_hash,
            query_hash=query_hash,
            query=query,
            response=response,
            created_at=time.time(),
            ttl_seconds=ttl_seconds or self.default_ttl,
            metadata=metadata or {},
        )

        # Evict if needed before adding
        self._evict_if_needed()

        self._cache[query_hash] = entry
        self._access_order.append(query_hash)

        # Save to disk periodically
        self._save_to_disk()

    def clear(self):
        """Clear the entire cache"""
        self._cache.clear()
        self._access_order.clear()
        self._hits = 0
        self._misses = 0
        self._evictions = 0
        print("[SemanticCache] Cache cleared")

    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        total_queries = self._hits + self._misses
        hit_rate = (self._hits / total_queries * 100) if total_queries > 0 else 0

        cache_size_mb = sum(
            len(json.dumps(entry.to_dict()).encode()) for entry in self._cache.values()
        ) / (1024 * 1024)

        return {
            "total_entries": len(self._cache),
            "hits": self._hits,
            "misses": self._misses,
            "evictions": self._evictions,
            "hit_rate_percent": round(hit_rate, 2),
            "cache_size_mb": round(cache_size_mb, 2),
            "max_size_mb": self.max_size_mb,
            "default_ttl_seconds": self.default_ttl,
            "similarity_threshold": self.similarity_threshold,
        }

    def _save_to_disk(self):
        """Save cache to disk"""
        if not self.persistence_path:
            return

        try:
            # Ensure directory exists
            self.persistence_path.parent.mkdir(parents=True, exist_ok=True)

            # Save cache entries
            data = {
                "entries": {k: v.to_dict() for k, v in self._cache.items()},
                "metrics": {
                    "hits": self._hits,
                    "misses": self._misses,
                    "evictions": self._evictions,
                },
            }

            with open(self.persistence_path, "w") as f:
                json.dump(data, f, indent=2)

        except Exception as e:
            print(f"[SemanticCache] Failed to save to disk: {e}")

    def _load_from_disk(self):
        """Load cache from disk"""
        if not self.persistence_path or not self.persistence_path.exists():
            return

        try:
            with open(self.persistence_path, "r") as f:
                data = json.load(f)

            # Load entries
            for k, v in data.get("entries", {}).items():
                self._cache[k] = CacheEntry.from_dict(v)
                self._access_order.append(k)

            # Load metrics
            metrics = data.get("metrics", {})
            self._hits = metrics.get("hits", 0)
            self._misses = metrics.get("misses", 0)
            self._evictions = metrics.get("evictions", 0)

            print(f"[SemanticCache] Loaded {len(self._cache)} entries from disk")

        except Exception as e:
            print(f"[SemanticCache] Failed to load from disk: {e}")


# Singleton instance
_cache_instance: Optional[SemanticCache] = None


def get_cache() -> SemanticCache:
    """Get or create the semantic cache singleton"""
    global _cache_instance
    if _cache_instance is None:
        _cache_instance = SemanticCache(
            max_size_mb=512,
            default_ttl_seconds=3600,
            similarity_threshold=0.85,
            persistence_path=Path("swarm_v2_artifacts/semantic_cache.json"),
        )
    return _cache_instance


async def cached_llm_call(
    query: str, llm_function: callable, use_cache: bool = True, **kwargs
) -> Any:
    """
    Wrapper for LLM calls with caching.

    Usage:
        response = await cached_llm_call(
            "What is Python?",
            llm_chat,
            system_prompt="...",
            agent_name="Devo"
        )

    Args:
        query: The query to cache
        llm_function: The async LLM function to call
        use_cache: Whether to use caching
        **kwargs: Arguments to pass to llm_function

    Returns:
        LLM response (cached or fresh)
    """
    if not use_cache:
        return await llm_function(**kwargs)

    cache = get_cache()

    # Try cache first
    cached_response = cache.get(query)
    if cached_response is not None:
        return cached_response

    # Cache miss - call LLM
    response = await llm_function(**kwargs)

    # Cache the response
    cache.set(query, response)

    return response

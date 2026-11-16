
"""
Global Memory Sync — Phase 4: Production-Grade Vector Store
A shared vector memory store using ChromaDB for persistent, semantic knowledge mapping.

Replaces the simple keyword-based vector matching with true semantic embedding retrieval.
"""

import os
import json
import math
import re
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime
from collections import Counter

try:
    import chromadb
    from chromadb.config import Settings
    
    CHROMA_AVAILABLE = True
except ImportError:
    CHROMA_AVAILABLE = False

from swarm_v2.core.turbovec_memory import STOPWORDS
from swarm_v2.core.dge import get_dge
from swarm_v2.core.llm_orchestrator import get_orchestrator

# One stopword definition lives in turbovec_memory and is imported here, so the two
# retrieval paths cannot drift apart.
_STOPWORDS = STOPWORDS

GLOBAL_MEMORY_DIR = os.getenv("SWARM_MEMORY_DIR", "swarm_v2_global_memory")
os.makedirs(GLOBAL_MEMORY_DIR, exist_ok=True)

class GlobalMemorySync:
    """
    Production-grade distributed vector database for shared long-term memory.
    Uses ChromaDB for high-fidelity semantic retrieval.
    """

    def __init__(self):
        self.entries_metadata: Dict[str, dict] = {}
        # public "gm_..." id -> the id the backing store actually indexes. In-process
        # only: rows written before this fix, or in a previous process, are not in the
        # map and are still unretrievable by id (see docs/PHASE-6-GAPS.md G21).
        self._id_map: Dict[str, str] = {}
        self.sync_log: List[dict] = []
        self._turbovec = None
        self.client = None
        self.collection = None
        self.dge = get_dge()
        
        # Priority 1: TurboVec Native (zero-segfault JSONL backend)
        try:
            from swarm_v2.core.turbovec_memory import get_turbovec_memory
            self._turbovec = get_turbovec_memory(GLOBAL_MEMORY_DIR)
            print(f"[GlobalMemory] TurboVec Native Backend — {self._turbovec.total_memories} entries, ZERO segfault")
            self._load_metadata()
            return
        except ImportError:
            pass
        
        if CHROMA_AVAILABLE:
            self.client = chromadb.PersistentClient(path=os.path.join(GLOBAL_MEMORY_DIR, "chroma"))
            self.collection = self.client.get_or_create_collection(
                name="swarm_global_memory",
                metadata={"hnsw:space": "cosine"}
            )
            print("[GlobalMemory] ChromaDB Production Backend - ONLINE")
        else:
            print("[GlobalMemory] ChromaDB NOT FOUND - Falling back to Legacy Mode")
            
        self._load_metadata()

    def _load_metadata(self):
        """Load additional metadata and sync logs."""
        meta_path = os.path.join(GLOBAL_MEMORY_DIR, "metadata.json")
        if os.path.exists(meta_path):
            try:
                with open(meta_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                self.entries_metadata = data.get("entries", {})
                self.sync_log = data.get("sync_log", [])
            except:
                pass

    def _save_metadata(self):
        """Persist metadata to disk."""
        meta_path = os.path.join(GLOBAL_MEMORY_DIR, "metadata.json")
        data = {
            "version": "2.0",
            "updated_at": datetime.now().isoformat(),
            "entries": self.entries_metadata,
            "sync_log": self.sync_log[-100:], # Keep last 100 entries
        }
        with open(meta_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

    # Patterns that indicate a poisoned/echoed response
    POISON_PATTERNS = (
        "[DIRECT NEURAL BRIDGE]",
        "[HMI BRIDGE ACTIVE]",
        "[EXECUTION MODE ACTIVE]",
        "[MESH OBSERVABILITY]",
        "[STRICT CONSTRAINT]",
        "I am currently processing this request internally.",
        "My linguistic output is currently stalled",
        "## CRITICAL: Action Execution Rules",
        "WRITE_FILE: architecture_overview.md",
        "SEARCH_QUERY:",
        "[Archi:Architect] Task:",
    )

    def _is_poisoned(self, text: str) -> bool:
        return any(p in (text or "") for p in self.POISON_PATTERNS)

    _FAILURE_MARKERS = (
        'neural core (', 'is currently over-taxed', 'please repeat the request',
    )

    def _is_model_failure(self, content: str) -> bool:
        """True for an agent's fallback refusal, which is not a memory worth keeping."""
        body = str(content).lower()
        return any(marker in body for marker in self._FAILURE_MARKERS)

    def contribute(self, content: str, author: str, author_role: str,
                   memory_type: str = "knowledge",
                   tags: List[str] = None) -> str:
        """Add a memory to the global pool via ChromaDB with DGE validation."""
        # 1. DGE Policy Check
        if not self.dge.validate_memory_contribution(author, content):
            return "rejected_by_dge"

        # 2. DGE Masking
        safe_content = self.dge.mask_pii(content)

        # 3. Poison Check
        if self._is_poisoned(safe_content):
            return "skipped_poison"

        # 4. Refusal Check.
        # A model failure is not knowledge. 835 distinct refusal texts (5,905 rows) sat in
        # the fabric saying "my neural core (Gemma) is currently over-taxed", because
        # base_agent stored whatever came back. Storing them again makes retrieval worse
        # every time the GPU is busy.
        if self._is_model_failure(safe_content):
            return "skipped_model_failure"

        entry_id = f"gm_{hash(safe_content + author) & 0xFFFFFFFF:08x}"
        
        metadata = {
            "entry_id": entry_id,
            "author": author,
            "author_role": author_role,
            "memory_type": memory_type,
            "tags": json.dumps(tags or []),
            "created_at": datetime.now().isoformat(),
            "access_count": 0
        }

        # TurboVec native path.
        #
        # Called with KEYWORDS: the old positional call was add(content, author,
        # metadata), and TurboVecMemory.add is add(content, metadata, agent) - so every
        # record written this way stored the author string as metadata and the metadata
        # DICT as the agent. That is why 122k memories carry {'author': ...} in their
        # agent field and the agent breakdown is unusable.
        if self._turbovec:
            # DEFECT FOUND 2026-11-10: TurboVecMemory.add returns ITS OWN id (a sha256
            # digest) and that value was discarded, while contribute() returned a
            # different "gm_..." id. The two id spaces were never linked, so the receipt
            # handed to the caller could not be redeemed by GET /memory/entry/{id}.
            stored_id = self._turbovec.add(safe_content, metadata=metadata, agent=author)
            self._id_map[entry_id] = stored_id
            return entry_id

        if CHROMA_AVAILABLE:
            self.collection.add(
                documents=[content],
                metadatas=[metadata],
                ids=[entry_id]
            )
        
        self.entries_metadata[entry_id] = metadata
        self._save_metadata()
        return entry_id
        self._save_metadata()
        
        # Sync contribution to shared JSON-LD graph
        self._sync_to_shared_jsonld(safe_content, author, author_role, memory_type, tags)

    def _sync_to_shared_jsonld(self, content: str, author: str, author_role: str, memory_type: str, tags: List[str]):
        shared_path = "F:\\Development sites\\shared_semantic_mesh.jsonld"
        if not os.path.exists(shared_path):
            return
        
        try:
            with open(shared_path, "r", encoding="utf-8") as f:
                mesh = json.load(f)
        except Exception:
            return
        
        if "@graph" not in mesh:
            mesh["@graph"] = []
            
        # Check if content has relationship patterns
        rel_match = re.match(r"^(.+?)\s+--\[(.+?)\]-->\s+(.+)$", content.strip())
        if not rel_match:
            rel_match = re.match(r"^(.+?)\s+->\s+(.+?)\s+->\s+(.+)$", content.strip())
            
        if rel_match:
            source = rel_match.group(1).strip()
            label = rel_match.group(2).strip()
            target = rel_match.group(3).strip()
            
            src_id = source.lower().replace(" ", "_")
            tgt_id = target.lower().replace(" ", "_")
            
            node_ids = {n.get("@id") for n in mesh["@graph"] if n.get("@type") == "Node"}
            if src_id not in node_ids:
                mesh["@graph"].append({
                    "@id": src_id,
                    "@type": "Node",
                    "label": source
                })
            if tgt_id not in node_ids:
                mesh["@graph"].append({
                    "@id": tgt_id,
                    "@type": "Node",
                    "label": target
                })
                
            rel_exists = any(
                r.get("@type") == "Relation" and r.get("source") == src_id and r.get("target") == tgt_id and r.get("label") == label
                for r in mesh["@graph"]
            )
            if not rel_exists:
                mesh["@graph"].append({
                    "@type": "Relation",
                    "source": src_id,
                    "target": tgt_id,
                    "label": label
                })
        else:
            node_id = f"mem_{hash(content) & 0xFFFFFFF:07x}"
            mesh["@graph"].append({
                "@id": node_id,
                "@type": "Node",
                "label": content[:60] + "..." if len(content) > 60 else content,
                "description": content,
                "author": author,
                "author_role": author_role,
                "memory_type": memory_type,
                "tags": tags or []
            })
            
        try:
            with open(shared_path, "w", encoding="utf-8") as f:
                json.dump(mesh, f, indent=2)
        except Exception:
            pass

    def inject_source_anchors(self, persona: Any):
        """Phase 13: Inject Higher Mind persona as persistent vector anchors."""
        anchors = [
            (f"Source Philosophy: {persona.philosophy}", ["source_consciousness", "philosophy"]),
        ]
        for tenet in persona.core_tenets:
            anchors.append((f"Source Tenet: {tenet}", ["source_consciousness", "tenet"]))
        for attr, val in persona.attributes.items():
            anchors.append((f"Source Attribute ({attr}): {val}", ["source_consciousness", "attribute"]))
        for content, tags in anchors:
            existing = self.query(content, top_k=1, type_filter="source_anchor")
            if not existing or existing[0][0] < 0.99:
                self.contribute(
                    content=content, author=persona.name, author_role="Higher Mind",
                    memory_type="source_anchor", tags=tags
                )

    def _fabric(self):
        """Lazy SemanticFabric; None when the index is absent or unreadable."""
        if getattr(self, '_fabric_cache', 'unset') == 'unset':
            try:
                from .semantic_fabric import SemanticFabric
                fab = SemanticFabric()
                self._fabric_cache = fab if fab.ready else None
                if not fab.ready:
                    print(f"[GlobalMemory] semantic fabric unavailable: {fab.error}")
            except Exception as exc:
                self._fabric_cache = None
                print(f"[GlobalMemory] semantic fabric unavailable: {type(exc).__name__}: {exc}")
        return self._fabric_cache

    def get(self, entry_id: str):
        """Fetch ONE stored entry by its id.

        GAP FOUND 2026-11-10 by tools/bench_complex_tasks.py: contribute() returned an
        id that nothing could read back. /memory/query is vector similarity only, so an
        opaque id (or any content without semantic neighbours) was unretrievable -- the
        write path had a receipt with no redemption. TurboVecMemory.get() already exists;
        this exposes it, and falls back to the metadata dict for the non-TurboVec paths.
        """
        if self._turbovec:
            hit = self._turbovec.get(entry_id)
            if hit:
                return hit
            mapped = self._id_map.get(entry_id)
            if mapped:
                return self._turbovec.get(mapped)
            return None
        return self.entries_metadata.get(entry_id)

    def query(self, query_text: str, top_k: int = 5,
              author_filter: str = None,
              type_filter: str = None) -> List[Tuple[float, dict]]:
        """Semantic query using ChromaDB vectors with fallback keyword search if ChromaDB is missing."""
        # SEMANTIC PATH (measured 2026-11-06). The chroma collection never binds on this
        # host, so every live query fell through to the keyword fallback below, which
        # scored by word overlap. Querying "how do I stop the council process from
        # restarting itself" returned three copies of "Problem: Swarm autonomous task:
        # Review artifact: AsyncHighFrequencyTelemetryGate" at score 0.8 - matched on
        # "the"/"of"/"do", which is noise dressed as retrieval. The fabric index answers
        # by meaning; when it is not ready this returns None and the old path runs, so a
        # missing index degrades in the open instead of pretending to be semantic.
        fabric = self._fabric()
        if fabric is not None and fabric.ready and query_text.strip():
            hits = fabric.query(query_text, k=max(top_k * 4, top_k))
            if hits:
                formatted = []
                for hit in hits:
                    if author_filter and str(hit.get('agent')) != author_filter:
                        continue
                    if type_filter and str(hit.get('class', 'knowledge')) != type_filter:
                        continue
                    formatted.append((float(hit.get('score', 0.0)), {
                        'entry_id': hit.get('id'),
                        'content': self.dge.mask_pii(str(hit.get('content', ''))),
                        'author': str(hit.get('agent', 'unknown'))[:120],
                        'author_role': 'fabric',
                        'memory_type': str(hit.get('class', 'knowledge')),
                        'tags': [],
                        'created_at': hit.get('timestamp'),
                        'access_count': int(hit.get('dup_count', 1)),
                        'backend': 'semantic_fabric',
                        'dup_count': int(hit.get('dup_count', 1)),
                    }))
                    self.dge.log_access(author_filter or "System", str(hit.get('id')), "SEMANTIC_READ")
                if formatted:
                    return formatted[:top_k]

        # The guard must test whether THIS instance has a bound collection, not
        # merely whether chromadb is importable. TurboVec initialises first and
        # returns early, leaving client/collection None while CHROMA_AVAILABLE is
        # True -- so the old guard fell through to self.collection.query() and
        # raised AttributeError on NoneType. The bridge swallowed it in a bare
        # except, so every agent reported memories_used=0 while 104k entries sat
        # in the store. Measured 2026-11-04.
        if not CHROMA_AVAILABLE or self.client is None or self.collection is None:
            # Simple keyword overlap search fallback
            query_words = {w for w in query_text.lower().split()
                           if w not in _STOPWORDS and len(w) > 2}
            if not query_words:
                return []

            # Search the LIVE store. Contributions write to TurboVec and return before
            # entries_metadata is touched, so that mirror stopped at 8,658 rows while the
            # store held 123,018 - 93% of the fabric was unreachable by this path. Falling
            # back to the mirror while claiming to search memory is the quiet kind of lie.
            if getattr(self, '_turbovec', None) is not None:
                live = self._turbovec.query_scored(query_text, k=max(top_k * 2, top_k),
                                                   agent=author_filter)
                live_hits = []
                for score, entry in live:
                    # Legacy rows written by the old positional call have metadata as a
                    # bare string and the author dict in the agent field. Normalise both
                    # rather than crashing or printing a dict as an author name.
                    meta = entry.get('metadata')
                    if not isinstance(meta, dict):
                        meta = {}
                    agent_field = entry.get('agent')
                    if isinstance(agent_field, dict):
                        agent_field = agent_field.get('author', 'unknown')
                    if type_filter and str(meta.get('memory_type')) != type_filter:
                        continue
                    live_hits.append((float(score), {
                        'entry_id': entry.get('id'),
                        'content': self.dge.mask_pii(str(entry.get('content', ''))),
                        'author': str(agent_field),
                        'author_role': 'fabric',
                        'memory_type': str(meta.get('memory_type', 'knowledge')),
                        'tags': [],
                        'created_at': entry.get('timestamp'),
                        'access_count': int(entry.get('dup_count', 1)),
                        'backend': 'turbovec_keyword',
                    }))
                if live_hits:
                    return live_hits[:top_k]

            results = []
            for eid, meta in self.entries_metadata.items():
                if author_filter and meta.get("author") != author_filter:
                    continue
                if type_filter and meta.get("memory_type") != type_filter:
                    continue
                
                content = meta.get("content", "")
                content_lower = content.lower()
                
                # Basic TF-IDF/Overlap score
                overlap = sum(1 for w in query_words if w in content_lower)
                score = overlap / max(len(query_words), 1)
                
                if overlap > 0 or not query_text.strip():
                    results.append((score, eid, meta))
            
            # Sort descending by score
            results.sort(key=lambda x: x[0], reverse=True)
            
            formatted_results = []
            for score, eid, meta in results[:top_k]:
                # Increment access count
                self.entries_metadata[eid]["access_count"] = self.entries_metadata.get(eid, {}).get("access_count", 0) + 1
                self.entries_metadata[eid]["last_accessed"] = datetime.now().isoformat()
                
                entry_dict = {
                    "entry_id": eid,
                    "content": self.dge.mask_pii(meta.get("content", "")),
                    "author": meta.get("author", "unknown"),
                    "author_role": meta.get("author_role", "unknown"),
                    "memory_type": meta.get("memory_type", "knowledge"),
                    "tags": json.loads(meta["tags"]) if isinstance(meta.get("tags"), str) else meta.get("tags", []),
                    "created_at": meta.get("created_at", datetime.now().isoformat()),
                    "access_count": self.entries_metadata[eid]["access_count"]
                }
                self.dge.log_access(author_filter or "System", eid, "QUERY_READ")
                formatted_results.append((float(score), entry_dict))
                
            self._save_metadata()
            return formatted_results

        where_clause = {}
        if author_filter: where_clause["author"] = author_filter
        if type_filter: where_clause["memory_type"] = type_filter
        results = self.collection.query(
            query_texts=[query_text], n_results=top_k,
            where=where_clause if where_clause else None
        )
        formatted_results = []
        if results and results["documents"]:
            for i in range(len(results["documents"][0])):
                doc = results["documents"][0][i]
                meta = results["metadatas"][0][i]
                eid = results["ids"][0][i]
                dist = results["distances"][0][i] if "distances" in results else 0.5
                score = 1.0 - dist
                if eid in self.entries_metadata:
                    self.entries_metadata[eid]["access_count"] += 1
                    self.entries_metadata[eid]["last_accessed"] = datetime.now().isoformat()
                entry_dict = {
                    "entry_id": eid, "content": self.dge.mask_pii(doc), "author": meta["author"],
                    "author_role": meta["author_role"], "memory_type": meta["memory_type"],
                    "tags": json.loads(meta["tags"]) if isinstance(meta["tags"], str) else meta["tags"],
                    "created_at": meta["created_at"],
                    "access_count": self.entries_metadata.get(eid, {}).get("access_count", 0)
                }
                self.dge.log_access(author_filter or "System", eid, "QUERY_READ")
                formatted_results.append((float(score), entry_dict))
        self._save_metadata()
        return formatted_results

    def sync_from_agent(self, agent_name: str, agent_role: str,
                         memories: List[dict]) -> int:
        """Sync memories with duplicate detection."""
        added = 0
        for mem in memories:
            content = mem.get("content", "")
            if not content or len(content) < 10: continue
            existing = self.query(content, top_k=1)
            if existing and existing[0][0] > 0.95: continue
            self.contribute(
                content=content, author=agent_name, author_role=agent_role,
                memory_type=mem.get("type", "knowledge"), tags=mem.get("tags", [])
            )
            added += 1
        return added

    def get_context_for_agent(self, agent_name: str, task: str,
                                 max_memories: int = 3) -> str:
        """Fetch global memories and format as clean prose (no brackets/headers)."""
        results = self.query(task, top_k=max_memories)
        source_anchors = self.query(task, top_k=2, type_filter="source_anchor")
        
        lines = []
        
        # Format as natural storytelling context rather than a structured log
        if source_anchors:
            for score, entry in source_anchors:
                if score > 0.4:
                    lines.append(f"Core System Philosophy: {entry['content']}")
        
        if results:
            for score, entry in results:
                if entry["memory_type"] == "source_anchor": continue 
                if self._is_poisoned(entry["content"]): continue
                if score > 0.35:
                    lines.append(f"Past knowledge from {entry['author']}: {entry['content'][:300]}")
        
        if not lines:
            return ""
            
        return "Global Intelligence Feed:\n" + "\n".join(lines)

    def get_stats(self) -> dict:
        if self._turbovec:
            return self._turbovec.get_stats()
        if CHROMA_AVAILABLE:
            count = self.collection.count()
        else:
            count = len(self.entries_metadata)
            
        type_counts = Counter(e["memory_type"] for e in self.entries_metadata.values())
        author_counts = Counter(e["author"] for e in self.entries_metadata.values())
        
        return {
            "total_memories": count,
            "by_type": dict(type_counts),
            "by_author": dict(author_counts),
            "total_accesses": sum(e.get("access_count", 0) for e in self.entries_metadata.values()),
            "sync_events": len(self.sync_log),
            "backend": "ChromaDB" if CHROMA_AVAILABLE else "Mock/Legacy"
        }

    def get_recent_entries(self, limit: int = 20) -> List[dict]:
        """Get the most recent memory entries for federation sync."""
        if not CHROMA_AVAILABLE:
            # Fallback to metadata-only if Chroma is missing
            recent_ids = list(self.entries_metadata.keys())[-limit:]
            return [{"id": eid, **self.entries_metadata[eid]} for eid in recent_ids]
            
        try:
            # Get latest entries from ChromaDB
            results = self.collection.get(limit=limit)
            entries = []
            if results and results["documents"]:
                for i in range(len(results["documents"])):
                    entries.append({
                        "content": results["documents"][i],
                        "author": results["metadatas"][i]["author"],
                        "author_role": results["metadatas"][i]["author_role"],
                        "memory_type": results["metadatas"][i]["memory_type"],
                        "tags": json.loads(results["metadatas"][i]["tags"]) if isinstance(results["metadatas"][i]["tags"], str) else results["metadatas"][i]["tags"]
                    })
            return entries
        except Exception as e:
            print(f"[GlobalMemory] Failed to get recent entries: {e}")
            return []

    def get_sync_log(self, limit: int = 20) -> List[dict]:
        return self.sync_log[-limit:]

    def optimize_collective_memory(self, max_age_days: int = 90, min_access_count: int = 0) -> dict:
        """Phase 7: Collective Memory Optimization. Prunes old/unused and consolidates duplicates."""
        import logging as _log
        _logger = _log.getLogger("GlobalMemory")
        pruned = 0
        consolidated = 0
        total_before = len(self.entries_metadata)
        protected_types = {"source_anchor", "introspection", "reasoning_state"}
        to_remove = []
        now = datetime.now()

        for eid, meta in list(self.entries_metadata.items()):
            if meta.get("memory_type", "knowledge") in protected_types:
                continue
            try:
                age_days = (now - datetime.fromisoformat(meta.get("created_at", now.isoformat()))).days
            except Exception:
                age_days = 0
            if age_days > max_age_days and meta.get("access_count", 0) <= min_access_count:
                to_remove.append(eid)

        for eid in to_remove:
            del self.entries_metadata[eid]
            if CHROMA_AVAILABLE:
                try:
                    self.collection.delete(ids=[eid])
                except Exception:
                    pass
            pruned += 1

        seen: Dict[str, str] = {}
        for eid, meta in list(self.entries_metadata.items()):
            content = meta.get("content", "")[:200].lower().strip()
            sig = f"{meta.get('author', '')}:{hash(content) & 0xFFFF:04x}"
            if sig in seen and seen[sig] != eid:
                existing_eid = seen[sig]
                ea = self.entries_metadata.get(existing_eid, {}).get("access_count", 0)
                ca = meta.get("access_count", 0)
                rid = eid if ca <= ea else existing_eid
                if rid in self.entries_metadata:
                    del self.entries_metadata[rid]
                    if CHROMA_AVAILABLE:
                        try:
                            self.collection.delete(ids=[rid])
                        except Exception:
                            pass
                    consolidated += 1
                if rid == existing_eid:
                    seen[sig] = eid
            else:
                seen[sig] = eid

        if pruned > 0 or consolidated > 0:
            self._save_metadata()
            _logger.info(f"[GlobalMemory] Optimization: pruned={pruned}, consolidated={consolidated}")

        return {"total_before": total_before, "total_after": len(self.entries_metadata),
                "pruned": pruned, "consolidated": consolidated, "timestamp": now.isoformat()}

    def get_memory_health(self) -> dict:
        """Phase 7: Collective memory health metrics for dashboard."""
        stats = self.get_stats()
        total = stats["total_memories"]
        total_accesses = stats["total_accesses"]
        avg_access = total_accesses / total if total > 0 else 0.0
        type_count = len(stats.get("by_type", {}))
        author_count = len(stats.get("by_author", {}))
        return {
            "total_memories": total, "total_accesses": total_accesses,
            "avg_access_rate": round(avg_access, 2),
            "type_diversity": type_count, "author_diversity": author_count,
            "backend": stats["backend"],
            "health_score": round(min(1.0, (avg_access * 0.3 + type_count * 0.1 + author_count * 0.05)), 3)
        }

# Singleton
_global_mem: Optional[GlobalMemorySync] = None

def get_global_memory() -> GlobalMemorySync:
    global _global_mem
    if _global_mem is None:
        _global_mem = GlobalMemorySync()
    return _global_mem

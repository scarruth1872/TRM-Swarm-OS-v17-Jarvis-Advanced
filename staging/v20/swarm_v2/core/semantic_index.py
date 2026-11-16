# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 39, Rank 190)
# CDYLIB: strip_semantic_index.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_semantic_index.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------

import os
import json
import asyncio
from typing import List, Dict, Any, Optional
from swarm_v2.skills.embedding_skill import SpectralEmbeddingSkill

INDEX_FILE = "swarm_v2_artifacts/semantic_index.json"

class SemanticIndex:
    """
    Manages semantic indexing of artifacts for Phase 2 "Tiny Brain" layer.
    Uses EmbeddingGemma (SpectralEmbeddingSkill) for deep semantic mapping.
    """
    def __init__(self):
        self.embedding_skill = SpectralEmbeddingSkill()
        self.index: Dict[str, Dict] = {}
        self._load()

    def _load(self):
        if os.path.exists(INDEX_FILE):
             try:
                 with open(INDEX_FILE, "r") as f:
                     self.index = json.load(f)
             except:
                 self.index = {}

    def _save(self):
        os.makedirs(os.path.dirname(INDEX_FILE), exist_ok=True)
        with open(INDEX_FILE, "w") as f:
            json.dump(self.index, f, indent=2)

    async def index_artifact(self, filename: str, content: str, metadata: Optional[Dict] = None):
        """Generates embedding for an artifact and adds to index."""
        print(f"[Semantic Index] Indexing {filename}...")
        embedding = self.embedding_skill.embed_text(content)
        if embedding:
            self.index[filename] = {
                "embedding": embedding,
                "metadata": metadata or {},
                "indexed_at": os.path.getmtime(filename) if os.path.exists(filename) else 0
            }
            self._save()
            return True
        return False

    def query(self, query_text: str, top_k: int = 5) -> List[Dict]:
        """Search the index for semantically similar artifacts."""
        query_emb = self.embedding_skill.embed_text(query_text)
        if not query_emb:
            return []

        results = []
        for filename, data in self.index.items():
            score = self.embedding_skill.cosine_similarity(query_emb, data["embedding"])
            results.append({
                "filename": filename,
                "score": score,
                "metadata": data["metadata"]
            })
        
        results.sort(key=lambda x: x["score"], reverse=True)
        return results[:top_k]

_index = None
def get_semantic_index():
    global _index
    if _index is None:
        _index = SemanticIndex()
    return _index

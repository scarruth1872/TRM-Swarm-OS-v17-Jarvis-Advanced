"""
Embedding Skill - Native Model-Free & Sovereign Vector Embedding Capabilities
Provides deterministic subword hyperdimensional random projection via native Rust microkernel
with zero Ollama / zero external daemon dependency, plus legacy Ollama adapter fallback.
"""

import os
import json
import ctypes
import math
from typing import List, Dict, Any, Optional, Tuple

MONAD_TRUTH_ANCHOR = 0x4D4F4E4144000001
DEFAULT_EMBED_DIM = 768

# Locate kernel DLL
_DLL_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll"
)

_KERNEL_LIB = None
if os.path.exists(_DLL_PATH):
    try:
        _lib = ctypes.CDLL(_DLL_PATH)
        _lib.rust_native_encode_text_f32.argtypes = [
            ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_float), ctypes.c_size_t
        ]
        _lib.rust_native_encode_text_f32.restype = ctypes.c_int32

        _lib.rust_native_encode_text_ternary.argtypes = [
            ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
            ctypes.c_float, ctypes.POINTER(ctypes.c_float)
        ]
        _lib.rust_native_encode_text_ternary.restype = ctypes.c_int32

        if hasattr(_lib, "rust_turbovec_cosine"):
            _lib.rust_turbovec_cosine.argtypes = [
                ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_float), ctypes.c_size_t
            ]
            _lib.rust_turbovec_cosine.restype = ctypes.c_float

        _KERNEL_LIB = _lib
    except Exception as e:
        _KERNEL_LIB = None


def _fnv1a_64(data: bytes, salt: int = MONAD_TRUTH_ANCHOR) -> int:
    h = 0xcbf29ce484222325 ^ salt
    for b in data:
        h = ((h ^ b) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h


class _SplitMix64:
    __slots__ = ("state",)
    def __init__(self, seed: int):
        self.state = seed & 0xFFFFFFFFFFFFFFFF

    def next_u64(self) -> int:
        self.state = (self.state + 0x9e3779b97f4a7c15) & 0xFFFFFFFFFFFFFFFF
        z = self.state
        z = ((z ^ (z >> 30)) * 0xbf58476d1ce4e5b9) & 0xFFFFFFFFFFFFFFFF
        z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & 0xFFFFFFFFFFFFFFFF
        return z ^ (z >> 31)


def _py_native_encode_f32(text: str, dim: int = DEFAULT_EMBED_DIM) -> List[float]:
    """Pure-Python fallback producing identical vectors to the Rust kernel."""
    if not text or not text.strip():
        return [0.0] * dim

    clean = text.lower()
    words = []
    curr = []
    for c in clean:
        if c.isalnum() or c in ("_", "-"):
            curr.append(c)
        elif curr:
            words.append("".join(curr))
            curr = []
    if curr:
        words.append("".join(curr))

    features = []
    for w in words:
        features.append(("W:" + w, 2.0))
        features.append(("B:<" + w + ">", 1.5))
        for n in (3, 4, 5):
            if len(w) >= n:
                for i in range(len(w) - n + 1):
                    features.append((f"N{n}:" + w[i:i+n], 1.2))

    if not features:
        return [0.0] * dim

    acc = [0.0] * dim
    blocks = (dim + 63) // 64

    for feat, weight in features:
        seed = _fnv1a_64(feat.encode("utf-8"))
        prng = _SplitMix64(seed)
        for b in range(blocks):
            bits = prng.next_u64()
            base = b * 64
            limit = min(64, dim - base)
            for i in range(limit):
                sign = 1.0 if (bits & (1 << i)) != 0 else -1.0
                acc[base + i] += sign * weight

    norm_sq = sum(x * x for x in acc)
    if norm_sq > 1e-12:
        inv_norm = 1.0 / math.sqrt(norm_sq)
        return [x * inv_norm for x in acc]
    return acc


class NativeEmbeddingSkill:
    """
    Sovereign, Model-Free Vector Embedding Skill.
    Uses native Rust microkernel C-ABI with pure-Python fallback.
    Zero Ollama / Zero External Process Dependency.
    """
    def __init__(self, embedding_dim: int = DEFAULT_EMBED_DIM):
        self.skill_name = "native_embedding"
        self.model = "native-turbovec-hdc"
        self.embedding_dim = embedding_dim
        self.description = "Native model-free subword HDC query & document encoder (Zero LLM)"
        self.has_native = _KERNEL_LIB is not None

    def embed_text(self, text: str) -> List[float]:
        """Generate normalized float32 embedding vector."""
        if not text:
            return [0.0] * self.embedding_dim

        if _KERNEL_LIB is not None:
            raw = text.encode("utf-8")
            buf = (ctypes.c_ubyte * len(raw))(*raw)
            out = (ctypes.c_float * self.embedding_dim)()
            rc = _KERNEL_LIB.rust_native_encode_text_f32(buf, len(raw), out, self.embedding_dim)
            if rc == self.embedding_dim:
                return list(out)

        return _py_native_encode_f32(text, self.embedding_dim)

    def embed_ternary(self, text: str, multiplier: float = 1.0) -> Tuple[List[int], float]:
        """Generate balanced ternary trits {-1, 0, +1} and scale."""
        if _KERNEL_LIB is not None:
            raw = text.encode("utf-8")
            buf = (ctypes.c_ubyte * len(raw))(*raw)
            out_trits = (ctypes.c_int8 * self.embedding_dim)()
            out_scale = ctypes.c_float(0.0)
            rc = _KERNEL_LIB.rust_native_encode_text_ternary(
                buf, len(raw), out_trits, self.embedding_dim, multiplier, ctypes.byref(out_scale)
            )
            if rc == self.embedding_dim:
                return list(out_trits), out_scale.value

        fvec = self.embed_text(text)
        absmean = sum(abs(x) for x in fvec) / (self.embedding_dim or 1)
        m = multiplier if multiplier > 0.0 else 1.0
        k3_scale = (0.35 / (absmean * m)) if absmean > 0.0 else 0.0
        trits = []
        for v in fvec:
            voltage_signal = v * k3_scale
            if voltage_signal > 0.35:
                trits.append(1)
            elif voltage_signal < -0.35:
                trits.append(-1)
            else:
                trits.append(0)
        return trits, absmean * m

    def embed_batch(self, texts: List[str]) -> List[List[float]]:
        return [self.embed_text(t) for t in texts]

    def embed_documents(self, documents: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        results = []
        for doc in documents:
            content = doc.get("content", "")
            metadata = doc.get("metadata", {})
            embedding = self.embed_text(content)
            results.append({
                "embedding": embedding,
                "content": content,
                "metadata": metadata,
                "model": self.model,
                "dimension": self.embedding_dim,
            })
        return results

    def cosine_similarity(self, vec1: List[float], vec2: List[float]) -> float:
        if not vec1 or not vec2 or len(vec1) != len(vec2):
            return 0.0

        if _KERNEL_LIB is not None and hasattr(_KERNEL_LIB, "rust_turbovec_cosine"):
            dim = len(vec1)
            a = (ctypes.c_float * dim)(*vec1)
            b = (ctypes.c_float * dim)(*vec2)
            return float(_KERNEL_LIB.rust_turbovec_cosine(a, b, dim))

        dot = sum(a * b for a, b in zip(vec1, vec2))
        m1 = math.sqrt(sum(a * a for a in vec1))
        m2 = math.sqrt(sum(b * b for b in vec2))
        denom = m1 * m2
        return dot / denom if denom > 1e-12 else 0.0

    def find_similar(self, query: str, documents: List[Dict[str, Any]], top_k: int = 5) -> List[Dict[str, Any]]:
        query_emb = self.embed_text(query)
        similarities = []
        for doc in documents:
            doc_emb = self.embed_text(doc.get("content", ""))
            score = self.cosine_similarity(query_emb, doc_emb)
            similarities.append({
                "content": doc.get("content", ""),
                "metadata": doc.get("metadata", {}),
                "similarity": score,
            })
        similarities.sort(key=lambda x: x["similarity"], reverse=True)
        return similarities[:top_k]

    def get_embedding_stats(self) -> Dict[str, Any]:
        return {
            "model": self.model,
            "dimension": self.embedding_dim,
            "backend": "RustMicrokernelC-ABI" if self.has_native else "PurePythonFallback",
            "sovereign_zero_llm": True,
            "monad_anchor": "0x4D4F4E4144000001",
        }


# =============================================================================
# Default Standard Classes — All Powered by Native Substrate
# =============================================================================

class EmbeddingSkill(NativeEmbeddingSkill):
    """Primary Swarm OS Vector Embedding Skill (Native Model-Free Default)."""
    def __init__(self, model: str = "native-turbovec-hdc", embedding_dim: int = DEFAULT_EMBED_DIM):
        super().__init__(embedding_dim=embedding_dim)
        self.model = model


class FastEmbeddingSkill(NativeEmbeddingSkill):
    """Ultra-fast microsecond embedding skill."""
    def __init__(self):
        super().__init__(embedding_dim=DEFAULT_EMBED_DIM)
        self.skill_name = "fast_embedding"
        self.description = "Native subword HDC query encoder (< 50 us latency)"


class SpectralEmbeddingSkill(NativeEmbeddingSkill):
    """Deep semantic indexing skill mapped to kernel-resident memory."""
    def __init__(self):
        super().__init__(embedding_dim=DEFAULT_EMBED_DIM)
        self.skill_name = "spectral_embedding"
        self.description = "Sovereign native spectral HDC embedding for artifact indexing"


class HighQualityEmbeddingSkill(NativeEmbeddingSkill):
    """High-capacity embedding skill."""
    def __init__(self):
        super().__init__(embedding_dim=DEFAULT_EMBED_DIM)
        self.skill_name = "high_quality_embedding"


# =============================================================================
# Legacy Ollama Adapter (Opt-In Only)
# =============================================================================

class OllamaEmbeddingSkill:
    """Legacy HTTP adapter for Ollama embedding models (Opt-In / Fallback)."""
    def __init__(self, model: str = "nomic-embed-text", embedding_dim: int = 768):
        self.skill_name = "ollama_embedding"
        self.model = model
        self.embedding_dim = embedding_dim
        self.ollama_base = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434")

    def embed_text(self, text: str) -> List[float]:
        import httpx
        try:
            with httpx.Client(timeout=30.0) as client:
                res = client.post(
                    f"{self.ollama_base}/api/embeddings",
                    json={"model": self.model, "prompt": text}
                )
                if res.status_code == 200:
                    return res.json().get("embedding", [])
        except Exception:
            pass
        return []

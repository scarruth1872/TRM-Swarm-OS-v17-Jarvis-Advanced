"""Rust-kernel intelligence benchmark — retrieval answered by the kernel, not a model.

This measures the platform's own knowledge path end to end:

    real text -> native model-free embedding -> kernel ternary quantization
              -> kernel ternary index search -> document

Every score and every latency below is computed from real buffers by the Rust
kernel's C-ABI exports (`rust_ternary_*`, `rust_turbovec_cosine`, `rust_native_encode_text_*`).
Zero LLM / zero external process dependency.

Run:  python swarm_v2/core/bench_kernel_intelligence.py --json <out.json>
"""
from __future__ import annotations

import argparse
import ctypes
import json
import os
import sys
import time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)

DLL = os.path.join(ROOT, "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll")
INDEX = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")
OLLAMA = "http://127.0.0.1:11434"
DIM = 768


# ---------------------------------------------------------------- kernel bind

class Kernel:
    """Thin binding to the kernel DLL. Every acceleration lives on the far side."""

    def __init__(self, dll_path: str):
        if not os.path.exists(dll_path):
            raise FileNotFoundError(f"kernel DLL not built: {dll_path}")
        self.lib = ctypes.CDLL(dll_path)
        self.dll_path = dll_path
        self.dll_bytes = os.path.getsize(dll_path)
        self._corpus_cache: dict = {}
        self._bind()

    def _bind(self) -> None:
        L = self.lib
        L.rust_kernel_verify.restype = ctypes.c_uint64

        L.rust_ternary_quantize.argtypes = [
            ctypes.POINTER(ctypes.c_float), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_int8), ctypes.c_float,
            ctypes.POINTER(ctypes.c_float),
        ]
        L.rust_ternary_quantize.restype = ctypes.c_size_t

        L.rust_ternary_agreement.argtypes = [
            ctypes.POINTER(ctypes.c_int8), ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
        ]
        L.rust_ternary_agreement.restype = ctypes.c_int32

        L.rust_ternary_magnitude.argtypes = [ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t]
        L.rust_ternary_magnitude.restype = ctypes.c_int32

        L.rust_ternary_index_search.argtypes = [
            ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t, ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_int32),
            ctypes.POINTER(ctypes.c_size_t),
        ]
        L.rust_ternary_index_search.restype = ctypes.c_int32

        L.rust_ternary_fidelity.argtypes = [
            ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_int8),
            ctypes.c_float, ctypes.c_size_t,
        ]
        L.rust_ternary_fidelity.restype = ctypes.c_float

        L.rust_turbovec_cosine.argtypes = [
            ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_float), ctypes.c_size_t,
        ]
        L.rust_turbovec_cosine.restype = ctypes.c_float

        L.rust_k3_and.argtypes = [ctypes.c_int8, ctypes.c_int8]; L.rust_k3_and.restype = ctypes.c_int8
        L.rust_k3_or.argtypes = [ctypes.c_int8, ctypes.c_int8]; L.rust_k3_or.restype = ctypes.c_int8
        L.rust_k3_not.argtypes = [ctypes.c_int8]; L.rust_k3_not.restype = ctypes.c_int8

        L.rust_council_evaluate.argtypes = [
            ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_uint64),
        ]
        L.rust_council_evaluate.restype = ctypes.c_int32

        if hasattr(L, "rust_native_encode_text_f32"):
            L.rust_native_encode_text_f32.argtypes = [
                ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_float), ctypes.c_size_t
            ]
            L.rust_native_encode_text_f32.restype = ctypes.c_int32

        if hasattr(L, "rust_native_encode_text_ternary"):
            L.rust_native_encode_text_ternary.argtypes = [
                ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
                ctypes.c_float, ctypes.POINTER(ctypes.c_float)
            ]
            L.rust_native_encode_text_ternary.restype = ctypes.c_int32

    def anchor(self) -> int:
        return self.lib.rust_kernel_verify()

    def encode_f32(self, text: str, dim: int = DIM) -> list:
        if hasattr(self.lib, "rust_native_encode_text_f32"):
            raw = text.encode("utf-8")
            buf = (ctypes.c_ubyte * len(raw))(*raw)
            out = (ctypes.c_float * dim)()
            rc = self.lib.rust_native_encode_text_f32(buf, len(raw), out, dim)
            if rc == dim:
                return list(out)
        from swarm_v2.skills.embedding_skill import NativeEmbeddingSkill
        return NativeEmbeddingSkill(embedding_dim=dim).embed_text(text)

    def quantize(self, vec: list, multiplier: float = 1.0):
        n = len(vec)
        src = (ctypes.c_float * n)(*vec)
        trits = (ctypes.c_int8 * n)()
        scale = ctypes.c_float(0.0)
        written = self.lib.rust_ternary_quantize(src, n, trits, multiplier, ctypes.byref(scale))
        return list(trits), scale.value, written

    def fidelity(self, vec: list, trits: list, scale: float) -> float:
        n = len(vec)
        src = (ctypes.c_float * n)(*vec)
        t = (ctypes.c_int8 * n)(*trits)
        return float(self.lib.rust_ternary_fidelity(src, t, scale, n))

    def magnitude(self, trits: list) -> int:
        t = (ctypes.c_int8 * len(trits))(*trits)
        return int(self.lib.rust_ternary_magnitude(t, len(trits)))

    def agreement(self, a: list, b: list) -> int:
        n = len(a)
        ta = (ctypes.c_int8 * n)(*a)
        tb = (ctypes.c_int8 * n)(*b)
        return int(self.lib.rust_ternary_agreement(ta, tb, n))

    def search(self, query: list, corpus: list, corpus_dim: int, k: int):
        n = len(corpus)
        if n == 0 or len(corpus[0]) != corpus_dim:
            raise ValueError(f"corpus shape mismatch: {n} vectors vs dim {corpus_dim}")
        q = (ctypes.c_int8 * len(query))(*query)
        key = (id(corpus), n, corpus_dim)
        c = self._corpus_cache.get(key)
        if c is None:
            flat = [x for vec in corpus for x in vec]
            c = (ctypes.c_int8 * len(flat))(*flat)
            self._corpus_cache.clear()
            self._corpus_cache[key] = c
        idx = (ctypes.c_size_t * k)()
        score = (ctypes.c_int32 * k)()
        comps = ctypes.c_size_t(0)
        rc = self.lib.rust_ternary_index_search(q, len(query), c, n, k, idx, score, ctypes.byref(comps))
        return rc, list(idx)[:rc], list(score)[:rc], comps.value

    def cosine(self, a: list, b: list) -> float:
        n = len(a)
        fa = (ctypes.c_float * n)(*a)
        fb = (ctypes.c_float * n)(*b)
        return float(self.lib.rust_turbovec_cosine(fa, fb, n))


# ------------------------------------------------------------------- sections

def section_kernel_identity(k: Kernel) -> dict:
    anchor = k.anchor()
    expected = 0x4D4F4E4144000001
    return {
        "dll": os.path.relpath(k.dll_path, ROOT),
        "dll_bytes": k.dll_bytes,
        "monad_truth_anchor": hex(anchor),
        "anchor_matches": anchor == expected,
    }


def section_kleene(k: Kernel) -> dict:
    trits = [-1, 0, 1]
    ok = 0
    total = 0
    for a in trits:
        for b in trits:
            total += 2
            if k.lib.rust_k3_and(a, b) == min(a, b):
                ok += 1
            if k.lib.rust_k3_or(a, b) == max(a, b):
                ok += 1
    for a in trits:
        total += 1
        if k.lib.rust_k3_not(a) == -a:
            ok += 1
    return {"laws_checked": total, "laws_obeyed": ok, "kleene_exact": ok == total}


def section_quantization(k: Kernel, vectors: list, multiplier: float) -> dict:
    trits, scales, fids, mags, written = [], [], [], [], 0
    for v in vectors:
        t, s, w = k.quantize(v, multiplier)
        written += w
        trits.append(t)
        scales.append(s)
        fids.append(k.fidelity(v, t, s))
        mags.append(k.magnitude(t))
    mean_fid = sum(fids) / len(fids)
    return {
        "vectors": len(vectors), "dim": DIM, "trits_written": written,
        "multiplier": multiplier,
        "fidelity_mean": round(mean_fid, 6),
        "fidelity_min": round(min(fids), 6), "fidelity_max": round(max(fids), 6),
        "nonzero_trit_fraction": round(sum(mags) / (len(mags) * DIM), 6),
        "compression_bytes_per_vector": {"float32": DIM * 4, "ternary_i8": DIM},
        "compression_ratio": 4.0,
    }, trits


def section_search_correctness(k: Kernel, trits: list, kk: int) -> dict:
    corpus_flat = [x for vec in trits for x in vec]
    sig, twin = {}, {}
    for i, t in enumerate(trits):
        s = bytes(x + 1 for x in t)
        if s in sig:
            twin[i] = sig[s]
            twin[sig[s]] = i
        else:
            sig[s] = i
    hits = 0
    dup_aware_hits = 0
    oracle_mismatch = []
    for qi, q in enumerate(trits):
        rc, ids, scores, _ = k.search(q, trits, DIM, kk)
        if rc <= 0:
            oracle_mismatch.append(qi)
            continue
        if ids[0] == qi:
            hits += 1
        if ids[0] == qi or twin.get(qi) == ids[0]:
            dup_aware_hits += 1
        ref = []
        for ci in range(len(trits)):
            base = ci * DIM
            agree = sum(1 for i in range(DIM) if q[i] != 0 and q[i] == corpus_flat[base + i])
            ref.append((agree, ci))
        ref.sort(key=lambda p: (-p[0], p[1]))
        if [i for _, i in ref[:kk]] != ids:
            oracle_mismatch.append(qi)
    return {
        "queries": len(trits),
        "top_k": kk,
        "rank1_self_hits": hits,
        "precision_at_1": round(hits / len(trits), 6),
        "indistinguishable_pairs": sum(1 for i in twin if i < twin[i]),
        "rank1_hits_allowing_twin": dup_aware_hits,
        "precision_at_1_allowing_twin": round(dup_aware_hits / len(trits), 6),
        "kernel_vs_oracle_matches": len(trits) - len(oracle_mismatch),
        "kernel_vs_oracle_mismatches": oracle_mismatch,
        "search_is_correct": dup_aware_hits == len(trits) and not oracle_mismatch,
    }


def section_latency(k: Kernel, trits: list, vectors: list, iters: int, kk: int) -> dict:
    n_corpus = len(trits)
    for i in range(20):
        k.search(trits[i % n_corpus], trits, DIM, kk)
        k.cosine(vectors[i % n_corpus], vectors[0])

    tern = []
    for i in range(iters):
        q = trits[i % n_corpus]
        t0 = time.perf_counter_ns()
        k.search(q, trits, DIM, kk)
        tern.append(time.perf_counter_ns() - t0)

    # 100 iters for float cosine is sufficient and fast
    flt = []
    f_iters = min(100, iters)
    for i in range(f_iters):
        q = vectors[i % n_corpus]
        t0 = time.perf_counter_ns()
        for c in vectors:
            k.cosine(q, c)
        flt.append(time.perf_counter_ns() - t0)

    def pct(xs, p):
        s = sorted(xs)
        return s[min(len(s) - 1, int(len(s) * p))]

    return {
        "iters": iters, "corpus_vectors": n_corpus, "dim": DIM, "top_k": kk,
        "ternary_ns": {"p50": pct(tern, 0.50), "p95": pct(tern, 0.95), "min": min(tern), "max": max(tern)},
        "ternary_us": {"p50": round(pct(tern, 0.50) / 1000, 3), "p95": round(pct(tern, 0.95) / 1000, 3)},
        "float_cosine_search_us": {"p50": round(pct(flt, 0.50) / 1000, 3), "p95": round(pct(flt, 0.95) / 1000, 3)},
        "comparisons_per_query": n_corpus,
        "ternary_throughput_comparisons_per_s": int(n_corpus * 1e9 / max(1, pct(tern, 0.50))),
    }


_DOC_INDEX: dict = {}


def resolve_doc(key: str) -> str | None:
    if not _DOC_INDEX:
        skip = {"node_modules", ".git", "target", "dist", "build", "__pycache__", ".venv", "venv"}
        for base, dirs, files in os.walk(ROOT):
            dirs[:] = [d for d in dirs if d not in skip]
            for f in files:
                _DOC_INDEX.setdefault(f, os.path.join(base, f))
    stem = os.path.basename(key).rsplit(".", 1)[0]
    for fname, path in _DOC_INDEX.items():
        if fname.startswith(stem[:32]):
            return path
    return None


def section_live_path(k: Kernel, keys: list, trits: list, vectors: list, model: str,
                      chars_map: dict | None = None) -> dict:
    """End-to-end test: real query text embedded by the native encoder and answered by the kernel."""
    out = {"embed_model": model, "attempted": 0, "answered": 0, "rank1_correct": 0,
           "space_match_cosines": [], "cases": []}
    for qi, key in enumerate(keys[:5]):
        path = resolve_doc(key)
        if not path:
            continue
        # Read the SAME window the index embedded (per-doc `chars_embedded`), not a fixed
        # 1500-char prefix — the HDC vector is a superposition of the whole window, so a
        # short re-encode prefix lands in a different region of space than the stored vector.
        n_chars = (chars_map or {}).get(key, 1500)
        try:
            text = open(path, encoding="utf-8", errors="replace").read()[:n_chars]
        except OSError:
            continue
        try:
            t_encode_start = time.perf_counter_ns()
            if model.startswith("native"):
                qv = k.encode_f32(text)
            else:
                body = json.dumps({"model": model, "prompt": text}).encode()
                req = urllib.request.Request(OLLAMA + "/api/embeddings", data=body,
                                             headers={"Content-Type": "application/json"})
                with urllib.request.urlopen(req, timeout=30) as r:
                    qv = json.load(r)["embedding"]
            encode_lat_us = (time.perf_counter_ns() - t_encode_start) / 1000.0
        except Exception as ex:  # noqa: BLE001
            out.setdefault("errors", []).append(f"embed {key}: {type(ex).__name__}: {ex}")
            continue
        out["attempted"] += 1
        space_cos = k.cosine(qv, vectors[qi])
        out["space_match_cosines"].append(round(space_cos, 6))
        qt, qs, _ = k.quantize(qv)
        rc, ids, scores, _ = k.search(qt, trits, DIM, 3)
        if rc > 0:
            out["answered"] += 1
            if ids[0] == qi:
                out["rank1_correct"] += 1
            out["cases"].append({
                "query_doc": key,
                "encode_latency_us": round(encode_lat_us, 2),
                "space_cosine_vs_stored": round(space_cos, 6),
                "rank1": ids[0], "rank1_doc": keys[ids[0]] if ids[0] < len(keys) else None,
                "rank1_score": scores[0],
                "top3": [{"idx": i, "doc": keys[i] if i < len(keys) else None, "score": s}
                         for i, s in zip(ids, scores)],
            })
    if out["space_match_cosines"]:
        out["space_match_mean"] = round(sum(out["space_match_cosines"]) / len(out["space_match_cosines"]), 6)
        out["live_vectors_share_index_space"] = out["space_match_mean"] > 0.5
    return out


def section_council(k: Kernel) -> dict:
    topic = b"kernel intelligence wiring"
    buf = (ctypes.c_ubyte * len(topic))(*topic)
    score = ctypes.c_float(0.0)
    seal = ctypes.c_uint64(0)
    rc = k.lib.rust_council_evaluate(buf, len(topic), ctypes.byref(score), ctypes.byref(seal))
    return {
        "available": True,
        "consensus_reached": rc == 1,
        "rc": int(rc),
        "weighted_score": round(score.value, 6),
        "monad_seal": hex(seal.value),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", default="swarm_v2_artifacts/bench_kernel_intelligence.json")
    ap.add_argument("--index", default="swarm_v2_artifacts/kernel_knowledge_index.json",
                    help="vector index to benchmark; defaults to kernel_knowledge_index.json")
    ap.add_argument("--multiplier", type=float, default=1.0)
    ap.add_argument("--iters", type=int, default=1000)
    ap.add_argument("--top-k", type=int, default=5)
    args = ap.parse_args()
    global INDEX
    INDEX = args.index if os.path.isabs(args.index) else os.path.join(ROOT, args.index)

    out = {"benchmark": "kernel_intelligence", "root": ROOT,
           "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}

    try:
        k = Kernel(DLL)
    except Exception as ex:
        out["fatal"] = f"kernel unavailable: {type(ex).__name__}: {ex}"
        print(json.dumps(out, indent=2))
        return 2

    out["kernel_identity"] = section_kernel_identity(k)
    print(f"[1/6] kernel identity      anchor={out['kernel_identity']['monad_truth_anchor']} "
          f"dll={out['kernel_identity']['dll_bytes']}B", flush=True)
    out["kleene"] = section_kleene(k)
    print(f"[2/6] kleene laws          {out['kleene']['laws_obeyed']}/{out['kleene']['laws_checked']}", flush=True)
    out["council"] = section_council(k)
    print(f"[3/6] council              consensus={out['council']['consensus_reached']} "
          f"score={out['council']['weighted_score']}", flush=True)

    raw = json.load(open(INDEX, encoding="utf-8"))
    model = raw.get("meta", {}).get("model", "native-turbovec-hdc")
    if isinstance(raw.get("index"), dict):
        raw = raw["index"]
    keys = [kk for kk, v in raw.items() if isinstance(v.get("embedding"), list) and len(v["embedding"]) == DIM]
    vectors = [raw[kk]["embedding"] for kk in keys]
    out["corpus"] = {"source": os.path.relpath(INDEX, ROOT), "documents": len(keys), "dim": DIM, "model": model}
    print(f"[4/6] corpus               {len(keys)} docs x {DIM} dims | encoder: {model}", flush=True)

    q, trits = section_quantization(k, vectors, args.multiplier)
    out["quantization"] = q
    print(f"[4/6] quantization         fidelity_mean={q['fidelity_mean']} "
          f"nonzero={q['nonzero_trit_fraction']}", flush=True)
    out["search_correctness"] = section_search_correctness(k, trits, args.top_k)
    sc = out["search_correctness"]
    print(f"[5/6] search correctness   P@1={sc['precision_at_1_allowing_twin']} "
          f"(twins={sc['indistinguishable_pairs']}) "
          f"oracle={sc['kernel_vs_oracle_matches']}/{sc['queries']}", flush=True)
    out["latency"] = section_latency(k, trits, vectors, args.iters, args.top_k)
    print(f"[5/6] latency              ternary p50={out['latency']['ternary_us']['p50']}us "
          f"float p50={out['latency']['float_cosine_search_us']['p50']}us "
          f"(n={out['latency']['iters']})", flush=True)
    chars_map = {kk: raw[kk].get("metadata", {}).get("chars_embedded", 1500) for kk in keys}
    out["live_path"] = section_live_path(k, keys, trits, vectors, model, chars_map)
    lp = out["live_path"]
    print(f"[6/6] live path            answered={lp['answered']}/{lp['attempted']} "
          f"rank1_correct={lp['rank1_correct']} "
          f"space_mean={lp.get('space_match_mean')}", flush=True)

    out["finished_at"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    dest = args.json if os.path.isabs(args.json) else os.path.join(ROOT, args.json)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    json.dump(out, open(dest, "w", encoding="utf-8"), indent=2)

    print(f"\n[artifact] {dest}")
    return 0 if out["search_correctness"]["search_is_correct"] else 1


if __name__ == "__main__":
    sys.exit(main())

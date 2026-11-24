#!/usr/bin/env python3
"""Export the council index as a Rust-native Monad continuum artefact.

The runtime must not parse a 43 MB JSON of f32 vectors. This writes:

  data/continuum_<tag>.bin      magic "K3CX" | u32 version | u32 n | u32 dim | n*dim int8 facets
  data/continuum_<tag>_chunks.jsonl   one line per chunk: i, source, chunk, text
  data/continuum_<tag>_meta.json      counts, sha256, measured recall, and the placement rule

Facet value = round(clamp(v,-1,1) * 108), the exact rule of
trm_uefi_kernel/src/faceted_torus_ternary.rs::FacetedTrit::from_f32. That fits i8 (-108..108).

WHY int8 facets rather than f32 (measured 2026-11-24, docs/TRM_Torus_Architecture_2026-11-24.md):
    f32 cosine          1.000 recall@10   baseline
    facet 217-level/dim 0.944 recall@10   4.0x smaller   <- this format
    trit 3-level/dim    0.569 recall@10
    torus 2-4 numbers   0.038-0.044       (dimensionality collapse; the torus is NOT a ranking geometry)

The torus keeps its job: discrete placement on the continuum (cell = dominant facet axis), not similarity.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import struct
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAX_FACET = 108
TOTAL_STATES = 217


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default="council_evidence_index_v3.json")
    ap.add_argument("--tag", default="v3")
    a = ap.parse_args()

    src = os.path.join(ROOT, "data", a.src)
    if not os.path.exists(src):
        print(f"  FAIL: {src} not found — build it with tools/build_council_index.py", file=sys.stderr)
        return 2
    d = json.load(open(src))
    chunks = d["chunks"]
    if not chunks:
        print("  FAIL: source index has no chunks", file=sys.stderr)
        return 2

    V = np.array([c["vec"] for c in chunks], dtype=np.float32)
    n, dim = V.shape
    F = np.round(np.clip(V, -1.0, 1.0) * MAX_FACET).astype(np.int8)

    binp = os.path.join(ROOT, "data", f"continuum_{a.tag}.bin")
    with open(binp, "wb") as fh:
        fh.write(b"K3CX")
        fh.write(struct.pack("<III", 1, n, dim))
        fh.write(F.tobytes())

    jsonl = os.path.join(ROOT, "data", f"continuum_{a.tag}_chunks.jsonl")
    with open(jsonl, "w", encoding="utf-8") as fh:
        for i, c in enumerate(chunks):
            fh.write(json.dumps({"i": i, "source": c["source"], "chunk": c["chunk"],
                                 "text": c["text"]}) + "\n")

    # Placement on the continuum: cell = axis index (0..216) of the chunk's dominant facet.
    # A placement rule, computable from the i8 row alone, and explicitly NOT a similarity metric.
    dom = F.astype(np.int16)[np.arange(n), np.abs(F.astype(np.int16)).argmax(axis=1)]
    cells = (dom + MAX_FACET).astype(np.int16)
    used = len(set(cells.tolist()))

    sha = hashlib.sha256(open(binp, "rb").read()).hexdigest()
    meta = {
        "tag": a.tag, "source": a.src, "n_chunks": int(n), "dim": int(dim),
        "format": "K3CX v1: i8 FacetedTrit per dim, raw rows, no header per row",
        "facet_rule": "round(clamp(v,-1,1)*108) — matches FacetedTrit::from_f32 exactly",
        "bytes": os.path.getsize(binp), "f32_bytes": int(V.nbytes),
        "size_ratio_bin_over_f32": round(os.path.getsize(binp) / V.nbytes, 3),
        "compression_x": round(V.nbytes / os.path.getsize(binp), 2),
        "sha256": sha,
        "placement": {"cell_of": "axis_index of the chunk's largest-|facet| dimension",
                      "states": TOTAL_STATES, "cells_used": int(used),
                      "note": "placement only; similarity is the integer dot, never torus distance"},
        "measured": {"recall_at_10_vs_f32": 0.944, "top1_vs_f32": 0.938,
                     "on": "16 queries (4 platform facts + 12 dev texts), docs/TRM_Torus_Architecture_2026-11-24.md"},
        "feature_range": {"min": int(F.min()), "max": int(F.max()),
                          "distinct": int(len(np.unique(F)))},
    }
    json.dump(meta, open(os.path.join(ROOT, "data", f"continuum_{a.tag}_meta.json"), "w"), indent=1)
    print(f"  wrote {binp} ({meta['bytes']/1e6:.2f} MB, {meta['compression_x']}x smaller than f32)")
    print(f"  wrote {jsonl} ({n} chunks) and continuum_{a.tag}_meta.json")
    print(f"  cells occupied: {used}/{TOTAL_STATES}   facet range [{F.min()},{F.max()}]")
    return 0


if __name__ == "__main__":
    sys.exit(main())

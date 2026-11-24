#!/usr/bin/env python3
"""Build the council's v2 evidence index: canonical facts + documents, one embedder, one manifest.

WHY v2 (measured 2026-11-24, docs/TRM_MoE_Inventory_Probe_2026-11-24.md)
------------------------------------------------------------------------
The shipped index (`data/council_evidence_index.json`, 558 chunks from 105 narrative files) could not
answer a single question about the platform: every top-1 was semantically unrelated (cosine 0.60-0.65),
and both resident MoE models answered the same questions WRONG and confidently (council port "443",
A2 as a set-theoretic relation). The facts those questions asked about — ports, axioms, module counts,
routes — were never in the corpus at all. They live in constants and in the module scan.

So v2 adds a first block of chunks generated from `GET /platform/facts` and `GET /council/axioms`:
one short, self-contained, checkable sentence per fact, each carrying its key string verbatim. The
document corpus follows, chunked the same way as v1 so the two remain comparable.

Rules kept:
  * fail-closed — if the embedder does not answer, write nothing and exit 2. A stale index that looks
    authoritative is worse than no index (same rule as build_memory_index.py);
  * never overwrite v1 (`council_evidence_index.json`); v2 goes to its own file so a bad build cannot
    cost the platform its working retriever;
  * the manifest records model, dim, counts, sha256 and timings, so a later measurement can prove which
    index produced which verdict.

Usage:
  python tools/build_council_index.py                 # facts + docs + evidence
  python tools/build_council_index.py --no-corpus     # facts only (fast smoke build)
  python tools/build_council_index.py --max-docs 20   # bounded trial run
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
import urllib.request

COUNCIL = "http://127.0.0.1:8022"
EMBED = "http://127.0.0.1:8593"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "data", "council_evidence_index_v2.json")
META = os.path.join(ROOT, "data", "council_evidence_index_v2_meta.json")
CHUNK_CHARS = 900
BATCH = 16


def get_json(url, timeout=60):
    return json.loads(urllib.request.urlopen(url, timeout=timeout).read())


def embed_batch(texts):
    body = json.dumps({"model": "nomic-embed-text", "input": texts}).encode()
    req = urllib.request.Request(f"{EMBED}/v1/embeddings", data=body,
                                 headers={"Content-Type": "application/json"})
    d = json.loads(urllib.request.urlopen(req, timeout=180).read())
    vecs = [row["embedding"] for row in d["data"]]
    if len(vecs) != len(texts):
        raise RuntimeError(f"embedder returned {len(vecs)} vectors for {len(texts)} texts")
    return vecs


def facts_chunks():
    """One checkable sentence per fact, phrased the way a question would ask it."""
    out = []
    f = get_json(f"{COUNCIL}/platform/facts")
    svc, ports = f["service"], f["ports"]
    out.append(("platform/facts", 0,
                f"The {svc['binary']} (the TRM council server) listens on TCP port {ports['council']}, "
                f"http://127.0.0.1:{ports['council']}. Its build reports version {svc['version']}."))
    out.append(("platform/facts", 1,
                f"Port assignments: council server {ports['council']}, LLM vote model {ports['vote_model']}, "
                f"embedder {ports['embedder']}. The council does not listen on 443; 443 is HTTPS."))
    km = f["kernel_modules"]
    out.append(("platform/facts", 2,
                f"The TRM kernel has {km['total']} modules from the live scan: {km['real']} hand-written "
                f"intelligence (REAL) and {km['ported']} mechanical ports (PORTED); source {km['source']}."))
    ax = f["axioms"]
    for i, a in enumerate(ax["axioms"]):
        out.append(("council/axioms", i,
                    f"Axiom {a['id']} — {a['name']}: {a['test']}"))
    n = len(ax["axioms"])
    out.append(("council/axioms", n,
                "The council's axiom set is A1 oneness ('I am because we are'), A2 free will (whose consent "
                "authorises this), A3 service, A4 reversibility, A5 truth, A6 restraint is also a choice. "
                f"Two-sided rule: {ax['two_sided_rule']}"))
    out.append(("council/axioms", n + 1,
                "The council's decision ladder, in order: " + " | ".join(ax["ladder"])))
    for i, r in enumerate(f["routes"]):
        out.append(("platform/routes", i,
                    f"Route {r['signature']} is served by host_council_server on port {ports['council']}."))
    for i, note in enumerate(f.get("known_absences", [])):
        out.append(("platform/absences", i, "Known limitation: " + note))
    return out


def corpus_chunks(max_docs=None):
    out = []
    files = []
    for sub in ("docs", "evidence"):
        d = os.path.join(ROOT, sub)
        if not os.path.isdir(d):
            continue
        for name in sorted(os.listdir(d)):
            p = os.path.join(d, name)
            # Prose only. MEASURED 2026-11-24: the 94 .json + 20 .jsonl evidence dumps chunk into
            # thousands of 900-char slices of machine output, which cost embed time and dilute the
            # index with text nobody asks a question about. Docs and .txt/.md evidence are the corpus.
            if not name.lower().endswith((".md", ".txt")):
                continue
            if os.path.isfile(p) and 200 < os.path.getsize(p) < 3_000_000:
                files.append(p)
    if max_docs:
        files = files[:max_docs]
    for p in files:
        try:
            with open(p, "r", encoding="utf-8", errors="replace") as fh:
                text = fh.read()
        except OSError:
            continue
        rel = os.path.relpath(p, ROOT).replace("\\", "/")
        # paragraph chunks, merged up to CHUNK_CHARS so a fact is not split from its subject
        cur, idx = "", 0
        for para in text.split("\n\n"):
            para = para.strip()
            if not para:
                continue
            # MEASURED 2026-11-24: a single paragraph longer than the chunk budget (17 MB of evidence
            # JSON has multi-hundred-KB "paragraphs") was sent whole and the embedder answered HTTP 500,
            # killing the build at chunk 1488. Hard-split anything oversized instead of trusting input.
            if len(para) > CHUNK_CHARS:
                if cur:
                    out.append((rel, idx, cur)); idx += 1; cur = ""
                for s in range(0, len(para), CHUNK_CHARS):
                    out.append((rel, idx, para[s:s + CHUNK_CHARS])); idx += 1
                continue
            if len(cur) + len(para) + 2 <= CHUNK_CHARS:
                cur = (cur + "\n\n" + para).strip()
                continue
            if cur:
                out.append((rel, idx, cur)); idx += 1
            cur = para
        if cur:
            out.append((rel, idx, cur))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-corpus", action="store_true")
    ap.add_argument("--max-docs", type=int, default=None)
    a = ap.parse_args()

    t0 = time.time()
    try:
        chunks = facts_chunks()
    except Exception as e:
        print(f"  FAIL: /platform/facts unreachable ({e}) — nothing written", file=sys.stderr)
        return 2
    n_facts = len(chunks)
    if not a.no_corpus:
        chunks += corpus_chunks(a.max_docs)

    texts = [c[2] for c in chunks]
    vecs = []
    skipped = []
    t_embed = time.time()
    for i in range(0, len(texts), BATCH):
        batch = texts[i:i + BATCH]
        try:
            vecs.extend(embed_batch(batch))
        except Exception:
            # One unembeddable text must not silently kill the build, and must not be silently
            # dropped either: retry each item alone, and record every casualty in the manifest.
            for t in batch:
                try:
                    vecs.extend(embed_batch([t]))
                except Exception as e:
                    skipped.append({"chars": len(t), "err": str(e)[:60], "head": t[:120]})
                    vecs.append(None)
        print(f"  embedded {min(i+BATCH, len(texts))}/{len(texts)}", end="\r", flush=True)
    if skipped:
        print(f"  {len(skipped)} chunks could not be embedded and will be excluded")
    keep = [(c, v) for c, v in zip(chunks, vecs) if v is not None]
    chunks = [c for c, _ in keep]
    vecs = [v for _, v in keep]
    dim = len(vecs[0])
    print(f"  embedded {len(texts)} chunks in {time.time()-t_embed:.1f}s, dim={dim}")

    payload = {
        "built_from": ["platform/facts", "council/axioms", "docs/", "evidence/"],
        "n_files": len({c[0] for c in chunks}),
        "n_chunks": len(chunks),
        "n_fact_chunks": n_facts,
        "dim": dim,
        "chunks": [{"source": s, "chunk": i, "text": t, "vec": v} for (s, i, t), v in zip(chunks, vecs)],
    }
    tmp = OUT + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(payload, fh)
    os.replace(tmp, OUT)
    sha = hashlib.sha256(open(OUT, "rb").read()).hexdigest()
    json.dump({
        "file": os.path.basename(OUT), "sha256": sha,        "n_chunks": len(chunks), "n_fact_chunks": n_facts, "n_embedded": len(vecs),
        "n_files": payload["n_files"], "dim": dim, "embed_model": "nomic-embed-text on :8593",
        "skipped_unembeddable": skipped[:20], "n_skipped": len(skipped),
        "corpus_included": not a.no_corpus, "max_docs": a.max_docs,
        "elapsed_s": round(time.time() - t0, 1),
        "note": "v1 (council_evidence_index.json) is untouched; evidence_index.rs prefers v2 when present",
    }, open(META, "w"), indent=1)
    print(f"  wrote {OUT} ({os.path.getsize(OUT)/1e6:.2f} MB) sha256={sha[:16]}…")
    return 0


if __name__ == "__main__":
    sys.exit(main())

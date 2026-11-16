#!/usr/bin/env python3
"""Embed the memories that have no vector, so semantic search covers the whole archive.

Measured starting point (G160): `fabric/fabric_vecs.npy` holds 42,805 rows while
`turbo_memory.jsonl.gz` holds 168,429 records - 25.4% coverage. Records without a row are
reachable only by literal search.

Rules this tool follows:
  * Resumable: every finished vector is flushed to `data/embed_progress.jsonl`, so a killed run
    loses at most one batch.
  * Never destroys the old fabric: the existing matrix is written to
    `fabric_vecs.npy.bak-<timestamp>` before anything replaces it, and the merge is atomic
    (tmp file + os.replace).
  * Fail closed: if the embedding server is unreachable mid-run, the run stops and reports how
    many rows it did manage, rather than writing a half-populated fabric as if it were complete.
  * Same model as the existing rows or nothing: the dimension is checked against the matrix.

Usage:
  python tools/embed_memory_fabric.py --limit 50            # smoke test
  python tools/embed_memory_fabric.py                       # full pass (125k records)
  python tools/embed_memory_fabric.py --merge-only          # fold progress into the fabric
"""
from __future__ import annotations

import argparse
import gzip
import json
import os
import shutil
import sys
import time
import urllib.request
from datetime import datetime, timezone

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAB_DIR = os.path.join(ROOT, "swarm_v2_global_memory", "fabric")
VECS = os.path.join(FAB_DIR, "fabric_vecs.npy")
IDS = os.path.join(FAB_DIR, "fabric_ids.json")
GMPATH = os.path.join(ROOT, "swarm_v2_global_memory")
PROGRESS = os.path.join(ROOT, "data", "embed_progress.jsonl")
EMBED_URL = "http://127.0.0.1:8593/v1/embeddings"
MODELS_URL = "http://127.0.0.1:8593/v1/models"
BATCH = 8


def _post(url: str, payload: dict, timeout: int = 180) -> dict:
    req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def _model_id() -> str:
    with urllib.request.urlopen(MODELS_URL, timeout=15) as r:
        m = json.loads(r.read())["models"][0]
    return m.get("id") or m["name"]


def _embed(texts: list[str], model: str, dim: int) -> np.ndarray:
    resp = _post(EMBED_URL, {"model": model, "input": texts})
    rows = [np.asarray(d["embedding"], dtype=np.float32) for d in resp["data"]]
    for r in rows:
        if r.shape[0] != dim:
            raise RuntimeError(f"dimension mismatch: got {r.shape[0]}, fabric is {dim}")
    return np.vstack(rows)


def _sources() -> list[str]:
    out = []
    for name in ("turbo_memory.jsonl.gz", "turbo_memory.jsonl"):
        p = os.path.join(GMPATH, name)
        if os.path.exists(p):
            out.append(p)
    return out


def _iter_records(sources: list[str]):
    for path in sources:
        op = gzip.open if path.endswith(".gz") else open
        with op(path, "rt", encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    d = json.loads(line)
                except Exception:
                    continue
                mid = d.get("id")
                content = d.get("content")
                if content is None:
                    data = d.get("data") if isinstance(d.get("data"), dict) else {}
                    content = d.get("event") or data.get("task") or ""
                if mid and content:
                    yield str(mid), str(content)[:1200]


def _done_ids() -> set[str]:
    done: set[str] = set()
    if os.path.exists(PROGRESS):
        with open(PROGRESS, encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                try:
                    done.add(json.loads(line)["id"])
                except Exception:
                    continue
    return done


def embed_pass(limit: int | None, batch: int) -> dict:
    existing = np.load(VECS)
    dim = existing.shape[1]
    ids = json.load(open(IDS, encoding="utf-8"))
    known = set(ids) | _done_ids()
    model = _model_id()
    todo = [(i, c) for i, c in _iter_records(_sources()) if i not in known]
    if limit:
        todo = todo[:limit]
    print(f"model={model[:40]}… dim={dim} existing={existing.shape[0]} todo={len(todo)}", flush=True)

    os.makedirs(os.path.dirname(PROGRESS), exist_ok=True)
    started = time.time()
    done = 0
    with open(PROGRESS, "a", encoding="utf-8") as out:
        for i in range(0, len(todo), batch):
            chunk = todo[i:i + batch]
            try:
                vecs = _embed([c for _, c in chunk], model, dim)
            except urllib.error.HTTPError as e:
                detail = e.read().decode("utf-8", "ignore")
                if e.code == 500 and "too large to process" in detail:
                    # G162: the server's physical batch is 512 tokens, so the longest archive records
                    # (1200 chars ~= 516 tokens) are rejected outright. Chunk them, embed each piece,
                    # mean-pool and re-normalise to one unit row - the same shape the fabric stores.
                    print(f"  chunk-pooling batch at {done} ({len(chunk)} long records)", flush=True)
                    rows = []
                    for _, content in chunk:
                        pieces = [content[i:i + 900] for i in range(0, len(content), 900)] or [content]
                        pv = _embed(pieces, model, dim)
                        pv = pv / np.maximum(np.linalg.norm(pv, axis=1, keepdims=True), 1e-9)
                        m = pv.mean(axis=0)
                        rows.append(m / max(float(np.linalg.norm(m)), 1e-9))
                    vecs = np.vstack(rows)
                else:
                    print(f"STOP at {done}/{len(todo)}: HTTP {e.code} {detail[:160]}", file=sys.stderr, flush=True)
                    break
            except Exception as e:
                print(f"STOP at {done}/{len(todo)}: {e}", file=sys.stderr, flush=True)
                break
            unit = vecs / np.maximum(np.linalg.norm(vecs, axis=1, keepdims=True), 1e-9)
            for (mid, content), row in zip(chunk, unit):
                out.write(json.dumps({"id": mid, "v": [round(float(x), 5) for x in row]}) + "\n")
                done += 1
            out.flush()
            if done % (batch * 25) == 0:
                rate = done / max(time.time() - started, 1e-9)
                print(f"  {done}/{len(todo)}  {rate:.1f}/s  eta {((len(todo)-done)/max(rate,1e-9))/60:.0f} min", flush=True)
    return {"embedded": done, "requested": len(todo), "seconds": round(time.time() - started, 1)}


def merge() -> dict:
    existing = np.load(VECS)
    ids = json.load(open(IDS, encoding="utf-8"))
    known = set(ids)
    add_ids: list[str] = []
    rows: list[np.ndarray] = []
    with open(PROGRESS, encoding="utf-8", errors="ignore") as fh:
        for line in fh:
            try:
                d = json.loads(line)
            except Exception:
                continue
            if d["id"] in known or d["id"] in add_ids:
                continue
            v = np.asarray(d["v"], dtype=np.float32)
            if v.shape[0] != existing.shape[1]:
                continue
            add_ids.append(d["id"])
            rows.append(v)
    if not rows:
        return {"added": 0, "total": existing.shape[0]}
    ts = datetime.now().strftime("%Y%m%d-%H%M%S")
    shutil.copy2(VECS, VECS + f".bak-{ts}")
    shutil.copy2(IDS, IDS + f".bak-{ts}")
    merged = np.vstack([existing, np.vstack(rows)]).astype(np.float16)
    tmp = VECS + ".tmp"
    with open(tmp, "wb") as fh:
        np.save(fh, merged)
    os.replace(tmp, VECS)
    tmp_ids = IDS + ".tmp"
    with open(tmp_ids, "w", encoding="utf-8") as fh:
        json.dump(ids + add_ids, fh)
    os.replace(tmp_ids, IDS)
    return {"added": len(add_ids), "total": int(merged.shape[0]), "dim": int(merged.shape[1]),
            "backup": f"fabric_vecs.npy.bak-{ts}", "merged_at": datetime.now(timezone.utc).isoformat()}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, help="embed at most N records (smoke test)")
    ap.add_argument("--batch", type=int, default=BATCH)
    ap.add_argument("--merge-only", action="store_true")
    ap.add_argument("--no-merge", action="store_true")
    args = ap.parse_args()
    if args.merge_only:
        print(json.dumps(merge(), indent=2))
        return 0
    res = embed_pass(args.limit, args.batch)
    print(json.dumps(res, indent=2))
    if not args.no_merge and res["embedded"] > 0:
        print(json.dumps(merge(), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

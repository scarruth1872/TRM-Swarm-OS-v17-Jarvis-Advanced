"""
evidence_retriever.py — retrieval over the platform's OWN documents.

Why: the council's 14 agents judge proposals from a 1.5B model's prior knowledge.
That is why "adopt QUIC RFC 9000" was rejected while "add dark mode" passed — the
model has no idea what QUIC means and treats unfamiliar terminology as risk.
This module lets an agent cite the platform's own written record before judging.

Corpus: docs/*.md and evidence/*.txt. Indexed with nomic-embed-text on :8593.
Embeddings come back as [[768 floats]] per item, so we unwrap defensively.

Nothing here decides anything. It retrieves text and returns its source path.
"""

import json
import logging
import math
import os
import urllib.request
from pathlib import Path
from typing import Dict, List, Optional

logger = logging.getLogger(__name__)

ROOT = Path(r"F:\Development sites\TRM-Swarm-OS-v2")
CORPUS_DIRS = [ROOT / "docs", ROOT / "evidence"]
INDEX_PATH = ROOT / "data" / "council_evidence_index.json"
EMB_URL = "http://127.0.0.1:8593/embeddings"

CHUNK_CHARS = 800
OVERLAP = 120
MIN_CHUNK = 120
# MEASURED: llama-server returns HTTP 500 for a 12-chunk batch at ~800 chars
# each (combined tokens exceed n_ctx_slot 2048). Single-item requests work.
BATCH = 1


def _embed(texts: List[str]) -> List[List[float]]:
    """Embed a batch. Returns one vector per input, in order."""
    payload = json.dumps({"input": texts}).encode("utf-8")
    req = urllib.request.Request(EMB_URL, data=payload,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read())
    if not isinstance(data, list):
        raise ValueError("unexpected embeddings response: %r" % type(data))
    out = []
    for item in data:
        vec = item.get("embedding") if isinstance(item, dict) else item
        # llama-server nests: embedding = [[768 floats]]
        while isinstance(vec, list) and len(vec) == 1 and isinstance(vec[0], list):
            vec = vec[0]
        out.append([float(x) for x in vec])
    if len(out) != len(texts):
        raise ValueError("asked for %d embeddings, got %d" % (len(texts), len(out)))
    return out


def _chunks(path: Path) -> List[str]:
    """Paragraph-aware chunks. No chunk crosses a file boundary."""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:
        logger.warning("evidence: cannot read %s: %s", path, exc)
        return []
    raw = raw.replace("\r\n", "\n").replace("\r", "\n")

    # MEASURED: a single paragraph longer than the embedding context makes
    # llama-server return HTTP 500 (8 of 300 chunks failed this way). Split any
    # oversize paragraph on character budget before packing.
    hard_parts: List[str] = []
    for para in raw.split("\n\n"):
        while len(para) > CHUNK_CHARS:
            cut = para.rfind(" ", 0, CHUNK_CHARS)
            if cut < CHUNK_CHARS // 2:
                cut = CHUNK_CHARS
            hard_parts.append(para[:cut].strip())
            para = para[cut:].strip()
        hard_parts.append(para)

    parts: List[str] = []
    buf = ""
    for para in hard_parts:
        if len(buf) + len(para) + 2 <= CHUNK_CHARS:
            buf = (buf + "\n\n" + para).strip() if buf else para.strip()
        else:
            if len(buf) >= MIN_CHUNK:
                parts.append(buf)
            buf = para.strip()
    if len(buf) >= MIN_CHUNK:
        parts.append(buf)
    # add overlap so a boundary sentence is not lost
    if OVERLAP and len(parts) > 1:
        out = [parts[0]]
        for prev, cur in zip(parts, parts[1:]):
            tail = prev[-OVERLAP:]
            out.append((tail + "\n" + cur).strip())
        return out
    return parts


def build_index(force: bool = False) -> Dict:
    """Build (or load) the corpus index. Cached on disk."""
    if INDEX_PATH.exists() and not force:
        try:
            data = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
            if data.get("chunks"):
                return data
        except Exception as exc:
            logger.warning("evidence: index unreadable, rebuilding: %s", exc)

    files: List[Path] = []
    for d in CORPUS_DIRS:
        if d.is_dir():
            files.extend(sorted(d.glob("*.md")))
            files.extend(sorted(d.glob("*.txt")))

    records: List[Dict] = []
    for f in files:
        for i, text in enumerate(_chunks(f)):
            records.append({
                "source": str(f.relative_to(ROOT)).replace("\\", "/"),
                "chunk": i,
                "text": text,
            })

    vectors: List[List[float]] = []
    for start in range(0, len(records), BATCH):
        batch = records[start:start + BATCH]
        vectors.extend(_embed([r["text"] for r in batch]))

    for rec, vec in zip(records, vectors):
        # 4 dp keeps the file small without measurably moving cosine scores
        rec["vec"] = [round(v, 4) for v in vec]

    data = {
        "built_from": [str(d.relative_to(ROOT)).replace("\\", "/") for d in CORPUS_DIRS],
        "n_files": len(files),
        "n_chunks": len(records),
        "dim": len(vectors[0]) if vectors else 0,
        "chunks": records,
    }
    INDEX_PATH.parent.mkdir(parents=True, exist_ok=True)
    INDEX_PATH.write_text(json.dumps(data), encoding="utf-8")
    logger.info("evidence: indexed %d chunks from %d files", len(records), len(files))
    return data


def _cosine(a: List[float], b: List[float]) -> float:
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    if na == 0 or nb == 0:
        return 0.0
    return dot / (na * nb)


def retrieve(query: str, k: int = 3, min_score: float = 0.30) -> List[Dict]:
    """Top-k document chunks for a query. Empty list means 'no evidence found'."""
    try:
        index = build_index()
        qvec = _embed([query])[0]
    except Exception as exc:
        logger.warning("evidence: retrieval unavailable: %s", exc)
        return []

    scored = []
    for rec in index["chunks"]:
        s = _cosine(qvec, rec["vec"])
        if s >= min_score:
            scored.append((s, rec))
    scored.sort(key=lambda x: x[0], reverse=True)

    return [{
        "source": rec["source"],
        "chunk": rec["chunk"],
        "score": round(score, 4),
        "text": rec["text"],
    } for score, rec in scored[:k]]


def stats() -> Dict:
    try:
        index = build_index()
        return {
            "n_files": index.get("n_files"),
            "n_chunks": index.get("n_chunks"),
            "dim": index.get("dim"),
            "index_path": str(INDEX_PATH),
        }
    except Exception as exc:
        return {"error": str(exc)}

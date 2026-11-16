"""Build a semantic index over a DEDUPLICATED view of the knowledge fabric.

Why dedupe first: the live fabric is 122,918 entries but only 42,801 distinct contents
(67.2% of rows share a normalised duplicate). The largest single group is 5,905 copies
of a model refusal ("my neural core (Gemma) is currently over-taxed"). Indexing the raw
file would embed the same string thousands of times, so a vector search would return
2,000 duplicates of one failure message and call it retrieval.

Nothing here deletes or rewrites the original store: the deduplicated view and the
vectors are new files under swarm_v2_global_memory/fabric/. The original is read-only
input, so a bad build cannot cost the user their history.

Outputs:
  fabric/fabric_distinct.jsonl.gz   one row per distinct content (newest kept, dup_count)
  fabric/fabric_vecs.npy            float16 [N, dim] L2-normalised
  fabric/fabric_ids.json            ids in row order
  fabric/fabric_manifest.json       model, dim, count, source, sha256, timing
"""
import gzip
import hashlib
import json
import re
import sys
import time
import urllib.request
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
STORE = ROOT / 'swarm_v2_global_memory' / 'turbo_memory.jsonl.gz'
OUT = ROOT / 'swarm_v2_global_memory' / 'fabric'
EMBED_URL = 'http://127.0.0.1:8593/v1/embeddings'
BATCH = 16
MAX_CHARS = 1200
FAILURE_MARKERS = (
    'neural core (gemma) is currently over-taxed',
    'neural core (gemma) is currently over',
)


def norm(s: str) -> str:
    """Normalisation used ONLY as the dedupe key, never as the stored text."""
    s = str(s)
    s = re.sub(r'0x[0-9a-fA-F]+', '<HEX>', s)
    s = re.sub(r'\b\d{3,}\b', '<N>', s)
    s = re.sub(r'sub_[a-z0-9_]+', '<SUB>', s)
    s = re.sub(r'\s+', ' ', s)
    return s.strip().lower()


def load_rows():
    rows = []
    with gzip.open(STORE, 'rt', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def dedupe(rows):
    """Keep the NEWEST row per normalised content, plus a dup_count and id set."""
    groups = defaultdict(list)
    for r in rows:
        groups[norm(r.get('content', ''))].append(r)

    kept = []
    for key, group in groups.items():
        group.sort(key=lambda r: str(r.get('timestamp', '')), reverse=True)
        newest = dict(group[0])
        newest['dup_count'] = len(group)
        newest['ids'] = [r.get('id') for r in group][:50]
        newest['norm_key'] = hashlib.sha256(key.encode()).hexdigest()[:16]
        body = str(newest.get('content', '')).lower()
        if any(m in body for m in FAILURE_MARKERS):
            newest['class'] = 'model_failure'
        newest['content'] = str(newest.get('content', ''))[:MAX_CHARS]
        kept.append(newest)
    kept.sort(key=lambda r: str(r.get('timestamp', '')), reverse=True)
    return kept


def _embed(texts):
    body = json.dumps({'input': texts}).encode()
    req = urllib.request.Request(EMBED_URL, data=body,
                                 headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(req, timeout=180) as r:
        d = json.loads(r.read())
    # Order by the server's own index field rather than trusting response order: a
    # mis-ordered batch would silently attach every vector to the wrong memory, and
    # nothing downstream could tell.
    rows = sorted(d['data'], key=lambda row: row.get('index', 0))
    return [row['embedding'] for row in rows]


def embed_batch(texts):
    """Embed a batch, degrading instead of dying.

    The embedder returned 500 on a batch at ~1,024 items, which killed the whole run.
    Now a failed batch is retried item by item, and an item that still fails is retried
    truncated to 256 chars. Anything left is reported as None so the caller can EXCLUDE
    it from the index - a zero vector would sit in the matrix looking like a memory that
    matches nothing, which is a silent lie about coverage.
    """
    try:
        return _embed(texts)
    except Exception:
        pass
    out = []
    for t in texts:
        try:
            out.append(_embed([t])[0])
            continue
        except Exception:
            pass
        try:
            out.append(_embed([t[:256]])[0])
        except Exception:
            out.append(None)
    return out


def main():
    t0 = time.time()
    OUT.mkdir(parents=True, exist_ok=True)
    rows = load_rows()
    kept = dedupe(rows)
    print(f'read {len(rows):,} rows -> {len(kept):,} distinct '
          f'({1 - len(kept) / len(rows):.1%} collapsed)', flush=True)

    classes = Counter(r.get('class', 'knowledge') for r in kept)
    print(f'classes: {dict(classes)}', flush=True)

    # Persist the deduplicated view BEFORE the expensive embedding, so a failed or
    # interrupted run still leaves something usable.
    view = OUT / 'fabric_distinct.jsonl.gz'
    with gzip.open(view, 'wt', encoding='utf-8', compresslevel=3) as f:
        for r in kept:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    print(f'wrote {view.name} ({view.stat().st_size / 1e6:.1f} MB)', flush=True)

    good_rows, good_vecs, failed = [], [], []
    done = 0
    for i in range(0, len(kept), BATCH):
        chunk = kept[i:i + BATCH]
        embs = embed_batch([r['content'][:MAX_CHARS] for r in chunk])
        if len(embs) != len(chunk):
            raise SystemExit(f'batch {i}: got {len(embs)} vectors for {len(chunk)} inputs')
        for row, e in zip(chunk, embs):
            if e is None:
                failed.append(str(row.get('id')))
                continue
            good_rows.append(row)
            good_vecs.append(np.asarray(e, dtype=np.float32))
        done += len(chunk)
        if done % 1024 < BATCH or done == len(kept):
            rate = done / max(time.time() - t0, 1e-6)
            print(f'  embedded {done:,}/{len(kept):,} ({rate:.1f}/s, {len(failed)} failed)',
                  flush=True)

    if not good_vecs:
        raise SystemExit('no vectors produced - the embedder is not answering')
    vecs = np.vstack(good_vecs)
    # L2-normalise so a dot product is a cosine. Doing it at build time means a query
    # is one matmul, not a per-row division over 42k rows.
    norms = np.linalg.norm(vecs, axis=1, keepdims=True)
    norms[norms == 0] = 1.0
    vecs = (vecs / norms).astype(np.float16)

    np.save(OUT / 'fabric_vecs.npy', vecs)
    (OUT / 'fabric_ids.json').write_text(
        json.dumps([r.get('id') for r in good_rows]), encoding='utf-8')

    h = hashlib.sha256((OUT / 'fabric_vecs.npy').read_bytes()).hexdigest()
    manifest = {
        'source': str(STORE),
        'source_rows': len(rows),
        'distinct': len(kept),
        'indexed': int(vecs.shape[0]),
        'failed_embeddings': len(failed),
        'collapsed_pct': round(100 * (1 - len(kept) / len(rows)), 2),
        'classes': dict(classes),
        'model': 'nomic-embed-text via llama-server :8593',
        'dim': int(vecs.shape[1]),
        'dtype': 'float16 (L2-normalised)',
        'sha256_vecs': h,
        'max_chars': MAX_CHARS,
        'seconds': round(time.time() - t0, 1),
    }
    (OUT / 'fabric_manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
    print(f'\n{vecs.shape} vectors, sha256 {h[:16]}... in {manifest["seconds"]}s')
    return 0


if __name__ == '__main__':
    sys.exit(main())

"""Semantic retrieval over the deduplicated knowledge fabric.

The live store (turbo_memory.jsonl.gz) is queried by TurboVecMemory.query(), which
counts shared words and adds a recency boost. On a 122,918-row store that is 67% duplicate
boilerplate, and a word-count query cannot match "how do I stop the council from
restarting" to a memory that says "watchdog timer re-armed by launcher" - they share no
words. This module answers the same question by meaning, using the on-device
nomic-embed-text tier on :8593.

It reads the mapped artifacts built by tools/build_fabric_index.py and never writes to
them. If the index or the embedder is missing, query() returns None (and .error says
why) rather than silently degrading to substring search - a fallback that looks like
semantic retrieval while being keyword matching is exactly the class of dishonesty this
platform keeps having to correct.
"""
from __future__ import annotations

import gzip
import json
import threading
import urllib.request
from pathlib import Path
from typing import Dict, List, Optional

import numpy as np

DEFAULT_DIR = Path(__file__).resolve().parents[2] / 'swarm_v2_global_memory' / 'fabric'
EMBED_URL = 'http://127.0.0.1:8593/v1/embeddings'


class SemanticFabric:
    def __init__(self, base_dir: Optional[str] = None):
        self.dir = Path(base_dir) if base_dir else DEFAULT_DIR
        self.manifest: Dict = {}
        self.rows: Dict[str, Dict] = {}
        self.ids: List[str] = []
        self.vecs: Optional[np.ndarray] = None
        self.error: Optional[str] = None
        self._lock = threading.Lock()
        self._load()

    # ---- loading -------------------------------------------------------------
    def _load(self):
        try:
            self.manifest = json.loads((self.dir / 'fabric_manifest.json').read_text(encoding='utf-8'))
            self.ids = json.loads((self.dir / 'fabric_ids.json').read_text(encoding='utf-8'))
            self.vecs = np.load(self.dir / 'fabric_vecs.npy', mmap_mode='r')
        except Exception as exc:
            self.error = f'index not loaded: {type(exc).__name__}: {exc}'
            return

        if len(self.ids) != self.vecs.shape[0]:
            self.error = f'id/vector mismatch: {len(self.ids)} ids vs {self.vecs.shape[0]} rows'
            self.vecs = None
            return

        try:
            with gzip.open(self.dir / 'fabric_distinct.jsonl.gz', 'rt', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    row = json.loads(line)
                    self.rows[str(row.get('id'))] = row
        except Exception as exc:
            self.error = f'distinct view not loaded: {type(exc).__name__}: {exc}'
            self.vecs = None

    # ---- queries -------------------------------------------------------------
    @property
    def ready(self) -> bool:
        return self.vecs is not None and not self.error

    def embed(self, text: str) -> Optional[np.ndarray]:
        body = json.dumps({'input': [text[:2000]]}).encode()
        req = urllib.request.Request(EMBED_URL, data=body,
                                     headers={'Content-Type': 'application/json'})
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                d = json.loads(r.read())
            v = np.asarray(d['data'][0]['embedding'], dtype=np.float32)
        except Exception:
            return None
        n = float(np.linalg.norm(v))
        return v / n if n else None

    def query(self, text: str, k: int = 5) -> Optional[List[Dict]]:
        """Top-k by cosine similarity. None when the tier or index is unavailable."""
        if not self.ready:
            return None
        qv = self.embed(text)
        if qv is None:
            return None

        with self._lock:
            # Vectors are float16 on disk and pre-normalised, so one matmul is the whole
            # search: 42k rows x 768 dims is ~33M multiply-adds, a few milliseconds.
            mat = np.asarray(self.vecs, dtype=np.float32)
            scores = mat @ qv

        k = max(1, min(k, len(scores)))
        top = np.argpartition(-scores, k - 1)[:k]
        top = top[np.argsort(-scores[top])]

        out = []
        for i in top:
            row = dict(self.rows.get(str(self.ids[int(i)]), {}))
            row['score'] = round(float(scores[int(i)]), 6)
            out.append(row)
        return out

    def stats(self) -> Dict:
        return {
            'ready': self.ready,
            'error': self.error,
            'distinct': self.manifest.get('distinct'),
            'source_rows': self.manifest.get('source_rows'),
            'collapsed_pct': self.manifest.get('collapsed_pct'),
            'classes': self.manifest.get('classes'),
            'dim': self.manifest.get('dim'),
            'model': self.manifest.get('model'),
        }


if __name__ == '__main__':
    fab = SemanticFabric()
    print(json.dumps(fab.stats(), indent=2))
    for q in ('how do I stop the council process from restarting itself',
              'what did the safety gate decide about deleting files'):
        hits = fab.query(q, k=3)
        print(f'\n{q!r}')
        for h in hits or []:
            print(f"  {h['score']:.3f} [{str(h.get('agent'))[:18]}] {str(h.get('content'))[:90]}")

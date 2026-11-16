"""Tests for the knowledge-fabric retrieval paths.

Two things are pinned here, both of which were measured defects rather than theory:

  1. The keyword fallback scored by raw word overlap, so "how do I stop the council
     process from restarting itself" returned three unrelated entries at 0.8 matched on
     "how"/"do"/"the" (measured 2026-11-06). Function words must not score.
  2. SemanticFabric must never look ready when its artifacts are missing, and must return
     None rather than silently degrading to keyword search. A "semantic" that is quietly
     substring matching is the failure mode this platform keeps having to correct.
"""
import json

import numpy as np
import pytest

from swarm_v2.core.global_memory import GlobalMemorySync, _STOPWORDS
from swarm_v2.core.semantic_fabric import SemanticFabric


class _StubDGE:
    def mask_pii(self, text):
        return text

    def log_access(self, *args, **kwargs):
        return None


def _bare_memory(entries):
    """A GlobalMemorySync with no chroma and no fabric, as on this host."""
    gm = GlobalMemorySync.__new__(GlobalMemorySync)
    gm.entries_metadata = entries
    gm.sync_log = []
    gm.client = None
    gm.collection = None
    gm._fabric_cache = None
    gm.dge = _StubDGE()
    gm._save_metadata = lambda: None
    return gm


def test_fallback_ignores_stopwords():
    gm = _bare_memory({
        'a': {'content': 'the council watchdog re-armed the timer after the launcher exit',
              'author': 'Archi', 'author_role': 'builder', 'memory_type': 'knowledge',
              'tags': '[]', 'created_at': '2026-11-05T00:00:00'},
        'b': {'content': 'Problem: Swarm autonomous task: Review artifact: TelemetryGate',
              'author': 'Swarm', 'author_role': 'collective', 'memory_type': 'knowledge',
              'tags': '[]', 'created_at': '2026-11-05T00:00:00'},
    })

    results = gm.query('how do I stop the council process from restarting itself', top_k=2)

    assert results, 'the fallback must still return the lexical match'
    top = results[0][1]
    assert 'watchdog' in top['content'].lower(), (
        f'stopword overlap outranked the real match: {top["content"][:80]}')
    assert top.get('backend') != 'semantic_fabric'


def test_fallback_returns_nothing_for_a_pure_stopword_query():
    gm = _bare_memory({
        'a': {'content': 'anything at all', 'author': 'x', 'author_role': 'r',
              'memory_type': 'knowledge', 'tags': '[]', 'created_at': '2026-11-05T00:00:00'},
    })
    assert gm.query('how do I do it for the', top_k=3) == []
    assert 'the' in _STOPWORDS and 'how' in _STOPWORDS


def test_fabric_without_artifacts_is_not_ready(tmp_path):
    fab = SemanticFabric(str(tmp_path))
    assert fab.ready is False
    assert 'index not loaded' in (fab.error or '')
    assert fab.query('anything', k=3) is None, 'a missing index must return None, not keyword hits'


def test_fabric_ranks_by_cosine(tmp_path):
    """Build a 3-row index by hand and check the ranking math without the embedder."""
    dim = 4
    rows = [
        {'id': 'm1', 'content': 'watchdog timer', 'agent': 'Archi', 'dup_count': 1},
        {'id': 'm2', 'content': 'unrelated thing', 'agent': 'Logic', 'dup_count': 1},
        {'id': 'm3', 'content': 'council launcher', 'agent': 'Devo', 'dup_count': 2},
    ]
    vecs = np.array([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.9, 0.1, 0.0, 0.0],
    ], dtype=np.float16)

    import gzip
    (tmp_path / 'fabric_manifest.json').write_text(json.dumps({
        'distinct': 3, 'source_rows': 3, 'collapsed_pct': 0.0, 'dim': dim,
        'model': 'test', 'classes': {'knowledge': 3},
    }), encoding='utf-8')
    (tmp_path / 'fabric_ids.json').write_text(json.dumps([r['id'] for r in rows]), encoding='utf-8')
    np.save(tmp_path / 'fabric_vecs.npy', vecs)
    with gzip.open(tmp_path / 'fabric_distinct.jsonl.gz', 'wt', encoding='utf-8') as f:
        for r in rows:
            f.write(json.dumps(r) + '\n')

    fab = SemanticFabric(str(tmp_path))
    assert fab.ready, fab.error
    fab.embed = lambda text: np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)

    hits = fab.query('anything', k=3)
    assert [h['id'] for h in hits] == ['m1', 'm3', 'm2'], [h['id'] for h in hits]
    assert hits[0]['score'] > hits[1]['score'] > hits[2]['score']
    assert hits[0]['content'] == 'watchdog timer'


def test_fabric_rejects_mismatched_ids_and_vectors(tmp_path):
    (tmp_path / 'fabric_manifest.json').write_text(json.dumps({'distinct': 2, 'dim': 4}),
                                                   encoding='utf-8')
    (tmp_path / 'fabric_ids.json').write_text(json.dumps(['only-one']), encoding='utf-8')
    np.save(tmp_path / 'fabric_vecs.npy', np.zeros((3, 4), dtype=np.float16))
    fab = SemanticFabric(str(tmp_path))
    assert fab.ready is False
    assert 'mismatch' in (fab.error or '').lower()

def test_fallback_searches_the_live_store_not_the_metadata_mirror(tmp_path):
    """93% of the fabric lives only in the TurboVec store; the mirror is a stale 8,658."""
    from swarm_v2.core.turbovec_memory import TurboVecMemory

    store = TurboVecMemory(str(tmp_path))
    store.add('the council watchdog re-armed the timer after the launcher exited',
              metadata={'memory_type': 'knowledge'}, agent='Archi')

    gm = _bare_memory({'mirror-only': {
        'content': 'a mirror entry that must not be the source of these results',
        'author': 'Stale', 'author_role': 'r', 'memory_type': 'knowledge',
        'tags': '[]', 'created_at': '2026-11-05T00:00:00'}})
    gm._turbovec = store

    results = gm.query('council watchdog timer', top_k=3)

    assert results, 'the live store must be searched when it is available'
    assert results[0][1]['backend'] == 'turbovec_keyword'
    assert 'watchdog' in results[0][1]['content'].lower()


def test_legacy_rows_report_a_name_not_a_dict(tmp_path):
    """Rows written by the old positional call hold {'author': ...} in the agent field."""
    from swarm_v2.core.turbovec_memory import TurboVecMemory

    store = TurboVecMemory(str(tmp_path))
    # Reproduce the legacy shape directly: metadata as a string, agent as a dict.
    legacy = {'id': 'legacy1', 'content': 'council restart watchdog incident',
              'metadata': 'Archi', 'agent': {'author': 'Archi'}, 'timestamp': '2026-11-01T00:00:00'}
    store._cache.append(legacy)
    store._index['legacy1'] = 0

    gm = _bare_memory({})
    gm._turbovec = store

    results = gm.query('council restart watchdog', top_k=1)
    assert results, 'a legacy row must still be retrievable'
    author = results[0][1]['author']
    assert author == 'Archi', f'author must be a name, got {author!r}'
    assert '{' not in author and 'author' not in author

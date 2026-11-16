"""Write-side guards for the knowledge fabric.

Three measured defects, all of which silently degraded the fabric:

  1. TurboVecMemory.add() appended unconditionally, so the store held 122,918 rows for
     42,801 distinct contents (65% duplicates) - including 5,905 copies of one string.
  2. GlobalMemorySync.contribute() called add(content, author, metadata) while the
     signature is add(content, metadata, agent), so the author landed in metadata and the
     metadata DICT landed in agent. Every record written that way has an unusable agent.
  3. Refusals ("my neural core (Gemma) is currently over-taxed") were stored as
     knowledge, 835 distinct texts of them.
"""
import os

import pytest

from swarm_v2.core import turbovec_memory
from swarm_v2.core.turbovec_memory import TurboVecMemory


@pytest.fixture()
def store(tmp_path, monkeypatch):
    """A fresh store per test, and no shared singleton leaking between them."""
    monkeypatch.setenv('SWARM_MEMORY_DIR', str(tmp_path))
    monkeypatch.setattr(turbovec_memory, '_instance', None)
    return TurboVecMemory(str(tmp_path))


def test_add_is_idempotent_for_identical_content(store):
    first = store.add('the council watchdog re-armed the timer', metadata={'k': 1}, agent='Archi')
    second = store.add('the council watchdog re-armed the timer', metadata={'k': 1}, agent='Archi')
    third = store.add('a genuinely different sentence about ternary weights',
                      metadata={'k': 2}, agent='Devo')

    assert first == second, 'the same content must map to one memory'
    assert third != first
    assert store.total_memories == 2


def test_add_idempotency_survives_a_reload(tmp_path, monkeypatch):
    """A restart must not resurrect duplicates: the hash index is rebuilt from disk."""
    monkeypatch.setenv('SWARM_MEMORY_DIR', str(tmp_path))
    first = TurboVecMemory(str(tmp_path))
    eid = first.add('durable content about the K3 blob', metadata={}, agent='Logic')

    again = TurboVecMemory(str(tmp_path))
    assert again.add('durable content about the K3 blob', metadata={}, agent='Logic') == eid
    assert again.total_memories == 1


def test_contribute_records_author_not_metadata_as_agent(tmp_path, monkeypatch):
    monkeypatch.setenv('SWARM_MEMORY_DIR', str(tmp_path))
    from swarm_v2.core import global_memory as gm_mod
    from swarm_v2.core import turbovec_memory as tv_mod
    from swarm_v2.core.global_memory import GlobalMemorySync

    # Point the module constant AND the per-path cache at the temp dir. Without the
    # constant, GlobalMemorySync() is handed the live fabric and writes its fixtures there
    # (it did, on the first suite run).
    monkeypatch.setattr(gm_mod, 'GLOBAL_MEMORY_DIR', str(tmp_path))
    monkeypatch.setattr(tv_mod, '_instances', {}, raising=False)
    gm = GlobalMemorySync()
    assert str(tmp_path) in str(gm._turbovec.base_path), 'test must not touch the live store'
    gm.contribute('The K3 block matrices are ternary after block QAT.',
                  author='Devo', author_role='builder')

    stored = gm._turbovec._cache[-1]
    assert stored['agent'] == 'Devo', f'agent must be a name, got {type(stored["agent"])}'
    assert isinstance(stored['metadata'], dict)
    assert stored['metadata']['author'] == 'Devo'


def test_contribute_rejects_a_model_refusal(tmp_path, monkeypatch):
    monkeypatch.setenv('SWARM_MEMORY_DIR', str(tmp_path))
    from swarm_v2.core import global_memory as gm_mod
    from swarm_v2.core import turbovec_memory as tv_mod
    from swarm_v2.core.global_memory import GlobalMemorySync

    # Point the module constant AND the per-path cache at the temp dir. Without the
    # constant, GlobalMemorySync() is handed the live fabric and writes its fixtures there
    # (it did, on the first suite run).
    monkeypatch.setattr(gm_mod, 'GLOBAL_MEMORY_DIR', str(tmp_path))
    monkeypatch.setattr(tv_mod, '_instances', {}, raising=False)
    gm = GlobalMemorySync()
    assert str(tmp_path) in str(gm._turbovec.base_path), 'test must not touch the live store'
    before = gm._turbovec.total_memories

    verdict = gm.contribute(
        "Task: research | Result: I'm sorry, my neural core (Gemma) is currently over-taxed. "
        "Please repeat the request or check system resources.",
        author='Seeker', author_role='researcher')

    assert verdict == 'skipped_model_failure'
    assert gm._turbovec.total_memories == before, 'a refusal must not become a memory'

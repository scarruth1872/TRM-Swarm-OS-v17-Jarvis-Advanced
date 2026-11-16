# Fabric re-embed — 2026-09-23 (closing the drift the benchmark found)

## What was wrong
`verify_nand_fabric_index.py` reported **text drifted from disk for 2 records**: `CHANGELOG.md`
(34,033 stored bytes of 154,269) and `docs/CATALYST_COMPLETION_PLAN.md` (3,154 of 6,825). Both are files
this campaign edited, so the fabric was answering from a stale copy of them — a **re-embed** job, not a
re-record.

## What was done
1. Backed up all five stores first (`backups/fabric_pre_reembed_20260923_155324/`): `.turbovec`,
   `.text.jsonl`, `.k3` (76.7 MB), `kernel_knowledge_index.json`, `semantic_index.json`.
2. Re-embedded with the generator and the native encoder — `build_kernel_knowledge_index.py --model
   native-turbovec-hdc`, 180 docs in 1.066 s (168.9 docs/s). No Ollama: the default encoder is the
   sovereign model-free one.
3. **Safety check on a copy of the old index: 178 of 180 vectors bit-identical, exactly the 2 drifted
   records changed, 0 added, 0 missing.** The encoder is deterministic, so the rebuild moved only what
   needed moving — it did not silently shift the embedding space.
4. Rebuilt the fabric stores (`build_nand_fabric_index.py`): 180 text records, 0 without on-disk text;
   turbovec 45,829 B (768 dims × 180 @ 2 bits), round-trip cosine min 0.8055 / mean 0.8752 / max 0.9914,
   new payload sha `5bbc6ca1a91099e1`, file sha256 `880f2b526ed747c1`.

## A latent bug in the gate itself
The first re-run **crashed**: `UnboundLocalError: measured`. The gate's re-baseline branch — the one that
exists precisely for a legitimate corpus refresh — reads `measured["cosine_min"]` before `measured` is
assigned, so it had never been exercised. Fixed by computing the measurement before the floor comparison.

```
text store  : swarm_v2_nand_fabric.text.jsonl (180 records, keyed like the vectors, 0 drifted)
measured    : {"cosine_min": 0.8055, "cosine_mean": 0.8752, "vectors_checked": 180, "dim": 768, "bits": 2}
RE-BASELINE : corpus changed since the floor was measured: min 0.8055 now vs 0.8055 then (delta +0.0000)
VERDICT     : intact -- counts, key table and vectors all re-derived from the file
REAL RC=0
```

## Refreshing the served artifact
The kernel caches its store in a `OnceLock` (`k3_retrieval.rs:225`), so a green gate is not enough — the
running process still served the old vectors. It was restarted: **pid 325800 → 160964, uptime 8.3 s**,
`HEALTHY`, `python_active=false`, and retrieval still serves (2 hits, 109 µs).

The first kill attempt **silently failed** — verifiable only because the uptime counter went from 106,497
to 106,506 s instead of resetting. `taskkill //PID ... //F` reported nothing and did nothing; `psutil`
`terminate()` worked immediately. **A restart claim needs the uptime, not the absence of an error.**

## Effect on the measured benchmarks
Only 2 of 180 documents changed, so the aggregate retrieval numbers (33–47% top-1 on short phrases,
80–93% self-retrieval) are unchanged by this refresh — it repairs correctness for those two documents, not
the short-query discrimination, which remains a property of the 2-bit store.

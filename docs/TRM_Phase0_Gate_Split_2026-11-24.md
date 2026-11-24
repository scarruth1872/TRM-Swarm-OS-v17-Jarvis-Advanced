# Phase 0 gate, split and re-run: v3 PASSES 4/4 (v1 0/4, v2 3/4 recorded)

**2026-11-24.** Pre-registration written first: `docs/phase0_gate_split_prereg_2026-11-24.json`
(15:56:01Z), before any of these runs. Rule unchanged: *every probe's top-1 chunk must contain the key*,
4 of 4.

## Results

| artifact | chunks | Gate A | verdict |
|---|---|---|---|
| v1 (`council_evidence_index.json`, shipped) | 558 | **0/4** | FAIL |
| v2 (`_v2.json`, facts + prose, 38 fact chunks) | 2,453 | **3/4** | FAIL |
| v3 (`_v3.json`, dilution fixed, 15 fact chunks) | 2,436 | **4/4** | **PASS** |

    probe                                    v1      v2      v3
    council listen port                      0.650✗  0.919✓  0.919✓
    axiom A2                                 0.612✗  0.746✓  0.746✓
    kernel module count                      0.681✗  0.873✓  0.872✓
    embedding server port                    0.597✗  0.720✗  0.823✓

Gate B (general knowledge) is **not run** against the index — it is out of scope for a retriever, and the
measurement lives where it belongs: the model, 6/6 general / 0/4 platform.

## Why v2 failed, and what the fix was

v2's miss was one question: *"Which TCP port does the embedding server use?"* The correct chunk —
"Port assignments: council server 8022, LLM vote model 8587, embedder 8593" — was at **rank 3**, scored
0.644, behind two *route* chunks. Cause, measured:

- 26 separately embedded route lines each read "Route … is served by host_council_server on port 8022",
  forming a dense cluster around every port/route question: **229 chunks mention 8022, 68 mention 8593**;
- worse, that template asserted "on port 8022" for `POST /v1/embeddings`, which is a council proxy route —
  it put the *wrong* port next to the word "embeddings".

Fix (content, not threshold): each service got its own subject sentence — "The embedding server … listens
on TCP port 8593", "The LLM vote model server listens on TCP port 8587" — and the 26 route lines collapsed
into one grouped chunk. Fact chunks went 38 → 15. Nothing else changed: same chunker, same embedder, same
probes, same 4/4 rule.

## Disclosure

The fix was made **after** observing the v2 failure. That is allowed here only because the gate was not
changed: identical probes, identical rule, and the v2 3/4 stays on the record beside v3's 4/4. What the
sequence proves is narrower than "v3 is good" — it proves the v2 miss was dilution, because removing the
duplication moved the same question from rank 3 to rank 1 (0.720 → 0.823) without touching the probes.

## What this licenses

- **Phase 1 is open**: `evidence_index.rs` may prefer the newest index, with the score floor re-derived from
  that index's own distribution rather than inherited at 0.30. Phase 1's gate stands as written: on the 12
  dev tasks, every `phoenix_evidence` entry must contain the text its verdict cites.
- **Not licensed**: any claim that the council now *knows* the platform. Retrieval of four facts passing a
  4/4 gate is not the same as the votes being grounded. That is Phase 1's measurement.

## Artifacts

| file | tracked | why |
|---|---|---|
| `tools/build_council_index.py` | yes | builder, with the dilution fix and `--out` so a rebuild can't clobber a scored artifact |
| `tools/phase0_gate_split.py` | yes | reads the prereg, scores any index with the same probes |
| `docs/phase0_gate_split_prereg_2026-11-24.json` | yes | the pre-registration, timestamped before the runs |
| `data/phase0_gate_split_result.json` | yes | per-index, per-probe scores |
| `data/council_evidence_index_v{2,3}_meta.json` | yes | sha256 pins: v2 `1f763cb7…`, v3 `21ea6dd8…` |
| `data/council_evidence_index_v{2,3}.json` | **no** (43 MB each) | reproducible from the builder against a live `:8593` |

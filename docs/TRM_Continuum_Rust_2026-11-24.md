# The Monad continuum, realised in Rust

**2026-11-24.** The torus is the continuum substrate for realisation, not a ranking metric — that was the
correction, and this is the build that follows it. Everything the runtime does now happens in Rust; Python
survives only as build-time tooling that emits an artefact.

## What exists now

| piece | where | state |
|---|---|---|
| continuum artefact | `data/continuum_v3.bin` — `K3CX` v1: `u32 version, u32 n, u32 dim, n*dim int8 facets` | 1.87 MB, **4.0× smaller** than the 7.48 MB f32 vectors |
| chunk metadata | `data/continuum_v3_chunks.jsonl` — `{i, source, chunk, text}` per line | 2,436 rows |
| Rust loader + searcher | `host_council_server/src/continuum.rs` | 4 unit tests pass |
| live endpoints | `GET /continuum/status`, `GET /continuum/search?q=…&k=…` | verified below |
| exporter | `tools/export_continuum.py` | build-time only |

The facet rule is `FacetedTrit::from_f32` exactly — `round(clamp(v,-1,1)*108)` — and the score is an
**i32 integer dot over i8 rows**. No floating point in the similarity path, no JSON parsing of 43 MB at
startup, no Python in the runtime.

## Measured on the live kernel

**Platform facts through the Rust path — 4/4** (the f32 path also scores 4/4 on index v3, so nothing was
lost in the move):

    PASS  council listen port      cell=88  score= 10707  platform/facts
    PASS  axiom A2                 cell=93  score=  8708  council/axioms
    PASS  kernel module count      cell=88  score= 10164  platform/facts
    PASS  embedding server port    cell=87  score=  9596  platform/facts

**Self-retrieval, blind by construction** — 60 held-out chunks, query = the chunk's own first sentence,
target = that chunk's id (no probe was chosen with the index in view):

    recall@1  52/60 = 0.867
    recall@10 57/60 = 0.950

**Latency** — 24 calls: p50 **15 ms**, p95 **30 ms**, min 8 ms, including one `:8593` embedding round-trip.
The scan itself is 1.87M integer multiply-adds over 2,436 × 768 facets.

## The continuum's actual role, as measured

- **Placement**: every chunk takes a cell from its dominant facet axis — `cell = axis_index(largest |facet|)`.
  Current occupancy: **23 of 217 cells**, fullest cell 420 chunks.
- That 23/217 is not a defect of the torus; it is the same overspecification measured earlier — unit-normalised
  768-dim embeddings only reach facet values in `[-26, +22]`, so 194 of the 217 states are never touched.
  The axis is coarse for this data; the storage is honest about it rather than rescaling to look full.
- **Similarity**: integer facet dot, **never torus distance**. Measured torus-distance retrieval was
  0.038–0.044 recall@10 with unquantised 4-PC controls at 0.081, so the loss is dimensionality, not the map.

## What is not done (stated so nobody reads this as more than it is)

- The continuum is **not yet the council's retriever**. `evidence_index.rs` still loads the f32 JSON and
  ranks with cosine. Swapping the ranker is Phase 1, and Phase 1's gate is unchanged: on the 12 dev tasks,
  every `phoenix_evidence` entry must contain the text its verdict cites.
- No agent is placed on the continuum yet — only evidence chunks. Placing the 14 agents is the natural next
  step and needs a rule that is not invented after seeing the result.
- `agent_mesh` remains a registry; there is still no mycelium transport, and this build does not add one.

## Why this direction and not the torus-geometry one

The torus defines the substrate: 217 states per facet axis, embedding into S³ by θ from the facet and φ from
the recurrence turn. Realisation on that substrate means states are *placed* — which is what the cells do —
while retrieval is a dot product over those states. Trying to make the torus *be* the similarity metric is
what the 0.038 measurement refused. This build keeps the torus where it belongs: as the space the states live
in, with the kernel doing exact integer arithmetic over them.

# Run it through the 4D torus: what the kernel actually has, and what survived

**2026-11-24.** Two claims checked before building on them: that a mycelium network is active in the Rust
kernel, and that the 4D torus can carry the retrieval work. One claim is false, the other is true only for
its *quantiser*, not its geometry.

## 1. There is no mycelium in the kernel

    grep -rniE "myceli|hypha|mycorrhiz" trm_uefi_kernel/  ->  0 hits

What exists is `agent_mesh.rs` (805 lines, declared at `lib.rs:45`): node-id/port/heartbeat validation,
topology versioning, live-node counts. Its only consumption in the server is a status field —

    main.rs:5555   "agent_mesh": AGENTS.iter().map(|a| json!({...}))

i.e. it reports the council roster inside a JSON payload. It is a **registry**, not a transport: nothing
propagates a fact along it. Calling it a mycelium would be a naming claim the code does not support.

## 2. The 4D torus is real, compiled, and has exactly one consumer

`faceted_torus_ternary` is declared (`lib.rs:121`) and used by `resonance.rs` — voltage/clock domains:

    let t = FacetedTrit::from_f32(2.0 * u - 1.0);
    let (x, y, z, w) = t.to_4d_torus(u);

Its only HTTP surface is `GET /resonance/grid`, which currently answers:

    {"available": false, "how": "set TRM_HWINFO_CSV to a live HWiNFO sensor log", "refused": "no nominal voltages are substituted for missing sensors"}

**Errata on the journal:** Article 119's re-audit asserts `faceted_torus_ternary` is never declared via
`mod` and therefore not compiled. That is now false in the other direction — `lib.rs:121` declares it and
`resonance.rs` links it. The claim is left in the journal and corrected here, per the errata rule.

The geometry, read from source: one `FacetedTrit` is `round(clamp(v,-1,1)*108)`, and

    theta = 2*pi*(facet+108)/217 - pi,  phi = 2*pi*turn,  r = 1/sqrt(2)
    point = (r*cos theta, r*sin theta, r*cos phi, r*sin phi)

So **one torus point carries two numbers**: a facet and a recurrence turn (the G85 injectivity fix).

## 3. Measured: the torus cannot be the retrieval geometry

2,436 chunks (index v3), 16 queries (4 platform facts + 12 dev texts), ground truth = the f32 cosine
ranking. `recall@10` = overlap with the baseline's top-10.

| representation | recall@10 | top-1 |
|---|---|---|
| f32 cosine (768 floats) | 1.000 | 1.000 |
| **facet 217-level per dim (768 int8)** | **0.944** | **0.938** |
| trit 3-level per dim (768 trits) | 0.569 | 0.562 |
| torus, 1 point (2 numbers) | 0.038 | 0.062 |
| torus, 2 points (4 numbers), 217-state facets | 0.044 | 0.125 |
| *control*: same 2 PCs, unquantised euclidean | 0.025 | 0.062 |
| *control*: all 4 PCs, unquantised dot | 0.081 | 0.000 |

The controls matter: **unquantised** 4-number vectors do no better than the torus versions. The collapse
is dimensionality — 2–4 numbers cannot rank 768-dim neighbours — not a defect in the Clifford embedding.
Ternary per dimension loses half the ranking, consistent with every earlier ternary measurement.

## 4. What is worth keeping, with its own number

`FacetedTrit` as a **storage format** is a win: i8 per dimension, integer dot accumulated in i32.

    f32 index      7.48 MB   (3,072 bytes/chunk)
    facet i8       1.87 MB   (768 bytes/chunk)      4.0x smaller
    recall@10      0.944     top-1 0.938   (vs f32)

Also measured: the facet axis is **overspecified** for unit-normalised embeddings. Values span only
**[-26, +22]** and 48 of the 217 states are ever used, because a 768-dim unit vector has typical
|v| ≈ 1/√768. A 6-bit packing (5.3×) is arithmetically available — but unproven, so it stays a proposal.

## 5. The correct architecture (each layer with its gate)

**L0 — sovereign facts.** `GET /platform/facts`, emitted from code, 3 tests. Done; Gate A 4/4 on index v3.

**L1 — index = cosine over facets.** Store i8 facets, accumulate in i32 — 4× smaller at 0.944 recall@10.
Gate to ship: recall@10 ≥ 0.90 on a *blind* probe set (these 16 are not blind) **and** p95 query latency
no worse than the f32 path. Until both hold, the f32 vector stays the decision-path index.

**L2 — torus = placement, never similarity.** Its measured recall (0.038–0.125) disqualifies it as a
ranking geometry. Legitimate uses are what it already does (voltage-domain phase, 217 discrete states for
addressing) and, if ever used for sharding, a gate that compares cell-restricted search against full
cosine. Any torus use in retrieval must beat the baseline or be recorded as decoration.

**L3 — the mesh, honestly.** `agent_mesh` is a registry. If a mycelium is wanted — hyphae between agents,
facts propagating on insertion — it must be *built*: a named transport, link semantics, and a gate that a
fact inserted at one node reaches all 14 agents with measured latency and zero divergence. Until then, no
architecture document should describe the kernel as having one.

**Not claimed:** that any of this makes the council's votes grounded. That is Phase 1's gate (every
`phoenix_evidence` entry must contain the text its verdict cites), still unrun.

Scripts: `tools/torus_retrieval.py` (the table above), artefact `data/council_facets_v3.npy` +
`data/council_facets_v3_meta.json` (sha-pinned metadata; the 1.87 MB array is an artefact, not source).

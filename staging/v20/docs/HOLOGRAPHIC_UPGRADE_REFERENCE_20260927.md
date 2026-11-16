# Holographic upgrade — doc-grounded refs (2026-09-27)

Directive under audit: *"a 12,000 vector-space matrix set modulated through the fractal telemetry of the
SRAM, upgrading the whole system with holographic principles."* What the platform's own docs already state:

## 1. The holographic machinery is NOT new here — it is in the SRAM pattern computer

- `swarm_v2/core/sram_pattern_computer.py:247` — `def pattern_hyperdimensional_computing(dim=10000)`
- `swarm_v2/core/sram_pattern_computer.py:259` — `def pattern_holographic_associative(slots=1024)`
- both are registered in the pattern table (`:296 'hdc'`, `:297 'holographic'`, `:895` alias
  `hyperdimensional → hdc`), i.e. **two of the 15 SRAM patterns ARE the holographic ones**.
- **Honest note on what those two ARE** (read the bodies, not the names): both return a **SPEC dict** —
  `{'name': 'Hyperdimensional Computing (HDC)', 'hypervector_dim': dim, …}` and `{'name': …, 'slots': slots, …}`
  — not a computed hypervector. The pattern is *declared and parameterized*; the binding/bundling (HDC's core
  operation) is not performed by the body. So step 3 below is "**make the declared pattern compute**", not
  "call the existing implementation". Same defect class as the rest of this campaign: a declaration where a
  measurement belongs — a name is not a capability.
- `papers/paper_124_10000_dimensional_hyperdimensional_computing_and_symbolic_classification.md` — the
  dedicated paper. `paper_110` lists "10,000-dimensional HDC" (coherence 99.99%, K3 True) and "Holographic
  Fourier Memory — Fourier-domain phase correlation & retrieval" (99.98%). `paper_111` = production
  applications. `papers/118/120/121/126` = real-hardware L3-cache SRAM pattern switching, real-workload
  benchmarks with a sub-microsecond auto-selector, multi-pattern fusion operators, and the 15-pattern
  orchestration layer.

## 2. The fractal modulator exists and is already bound

- `swarm_v2/core/fractal_telemetry.py` — Higuchi `D_H ∈ [1,2]` over a telemetry stream, continuous chaos
  detection, **and Kleene K3 ternary quantization**. Bound into
  `swarm_v2/core/hardware_voltage_microkernel_binder.py`; Rust twin
  `swarm_v2_rust_kernel/src/fractal_pattern_recursion.rs`; the WSL side carries
  `swarm-ssm/src/fractal_mamba3.rs` (inside the freeze archive).
- The recursion is SPECIFIED, not improvised: `paper_112` Horizon A = the 15 base patterns expanded to
  depth `d=4` → **240 recursive patterns**, Hausdorff `D_H = 1.585` (ternary Sierpinski scaling);
  `paper_119` = fractal SRAM pattern recursion / infinite pattern tree (e.g. Node #2500:
  `GNN mesh → cellular automaton → holographic memory`), modulated `15.5300 → 15.7713 THz`.
- Scratch analyses already joining the pieces: `scratch/task_401_fractal_mamba3_holographic.json` (200 runs:
  Mamba3-Fractal-130M, `layer_depth 24`, `model_dimension 768`, `state_expansion_dimension 16`,
  `unitary_orthogonality_residual`) and `scratch/task_404_fractal_depth_analysis.json` (200 runs: hierarchy
  levels, `recursion_depth`, `temporal_receptive_stride`, `context_capacity_tokens`, `fractal_hausdorff`).

## 3. Where the directive diverges from the docs — stated, not smoothed

1. **"12,000" has no referent in this tree.** It occurs 12 times and **none is a vector dimension**: a
   `$12,000` static-analysis tool cost, a `>12,000` empirical-run count, a gene symbol, a GPU-IC multiple.
   The documented and implemented dimension is **10,000** (`dim=10000`, paper_124, paper_110). So 12,000 is a
   NEW number — legitimate to choose, but it must be chosen as a decision, not cited as a reference.
2. **The live space is 768, not 10,000.** `embedding_skill.py:14 DEFAULT_EMBED_DIM = 768`; the served corpus
   `kernel_knowledge_index.json` is 768-dim; `k3_retrieval.rs` pins `DIM = 768` under the contract *"the
   query must be encoded by the SAME encoder or the two live in different spaces."* Any dimension change is
   a three-part migration — encoder ↔ corpus ↔ route — never a constant edit. Storage is cheap by
   comparison: 2-bit packing means 768 dims = 192 B/doc and 12,000 dims = 3,000 B/doc, i.e. ~6.5 MB for the
   2,180-doc corpus.
3. **Two holographic figures are DECLARED, not measured** — the exact defect class this campaign closes:
   `global_knowledge_fabric.py:290` `"holographic_recall_latency_ns": "< 10ns (TurboVec Direct Trit Radix)"`
   and `aura_cognition.py:491` `holographic_dispersion 0.94 / recovery_ratio 0.97`. Neither may enter a
   design as an input until it is measured or explicitly hobbled. Related: `agent_intelligence_bridge.py:485`
   tags `EventLedger` as `"holographic"` — a label, and labels are not capabilities.

## 4. What "modulate the matrices through the fractal telemetry" means, concretely

Holographic memory here means **HRR/HDC** — information distributed across every dimension, recovered by
correlation rather than by address. The pieces map onto each other without inventing anything:

- the **dimension** is the HDC space (`pattern_hyperdimensional_computing(dim=…)`) — this is what the
  10,000/12,000 decision actually sets;
- the **memory** is the associative holographic layer (`pattern_holographic_associative(slots=1024)`) plus
  the fabric's existing packed 2-bit vector store;
- the **fractal** supplies the LEVEL TREE (paper_112's d=4 expansion; paper_119's pattern tree) — one
  holographic layer per recursion level, self-similar across levels;
- the **telemetry modulates** by supplying, per level, Higuchi `D_H` and its K3 trit — the natural place is
  the binding operator's rotation/permutation exponent (HRR binds by circular convolution and permutes by
  role; a scalar modulator has an exact home there), so the same input yields different binding at different
  system regimes;
- the **verdict** stays the platform's own: `AgreementFn` over non-zero trits (already in
  `k3_retrieval.rs`), so recall lands as a trit in the K3 domain without a new metric.

Ordered plan (each step verifiable, none requiring the others to be finished):
1. **Decide the dimension** (10,000 doc-backed, or 12,000 as specified) and the encoder that emits it.
2. **Migrate the space** — re-encode the corpus, repin `DIM` + `ENCODER_NAME`, then re-run the two-query
   test; the bar is that an on-topic query SEPARATES from nonsense (today: 105 vs 95, a ten-point band).
3. **Wire the recall engine** — the HDC + holographic patterns from `sram_pattern_computer` behind the
   `/k3/*` path (ladder step 3: the link from `host_tranche` into the served crate).
4. **Bind the fractal modulator** — per-level `D_H` → the HRR binding exponent; level count from paper_112.
5. **Measure or hobble the two declared holographic figures** (§3.3) in the same pass.

## 5. The 10,000-dim test — run, and it says the width is not the constraint

The decision ("do 10000") was executed as a measurement, not a migration: re-encode to a NEW file, then score
the SAME documents at both widths so the only variable is the dimension.

`build_kernel_knowledge_index.py --dim 10000 --out swarm_v2_artifacts/kernel_knowledge_index_10000.json`
— 8.0 s at 22.5 docs/s, and it reported honestly: **180 documents written, MISSING (2000)**. The missing 2,000
are the relational samples (`relational_documents_added: 1994`) whose **text exists only as 768-dim vectors**
inside the live index; the entries carry `embedding`/`metadata` and no text, so they cannot be re-embedded at
any width without re-running the relational ingest. A full 10,000-dim corpus is therefore not reachable by
re-encoding alone.

Same 180 documents, two widths (kernel metric: exact integer ternary agreement over non-zero trits):

| query | 768 top1 | 10000 top1 | top1 doc |
|---|---|---|---|
| "TRM council K3 ternary verdict" | 57 | **361** | `docs/K3_VERDICT_DERIVATION_STANDARD.md` (correct at both widths) |
| "purple elephants compose jazz fugues underwater" | 15 | 95 | (nonsense: 3-way tie at 15 → a spread at 10,000) |

**The ranking is right at both widths — but the discrimination is not better.** nonsense/on-topic =
15/57 = **0.263** at 768 and 95/361 = **0.263** at 10,000: identical to three decimals. The absolute gap
(42 → 266) grows in step with the dimension, i.e. it is scaling, not discrimination.

Mechanism, measured: `_py_native_encode_f32(q, 768)` vs `(q, 10000)` → **cosine over the first 768 dims =
1.000000**. The two encodings are collinear; the extra 9,232 dims (mean|v| 0.008 vs 0.029 in the head) carry
almost no new discriminative information. This is what a random projection does: more dims, same direction.

**Verdict: 10,000 is doc-backed as the HDC pattern's default, but migrating the live space to it buys no
discrimination and costs 13x the storage and the re-encode. The flatness is a property of the ENCODER being a
deterministic random projection, not of the width.** The gate set before the test — "if it does not widen, say
so and stop" — fired: **the kernel was not touched** (`k3_retrieval.rs` still `DIM = 768`), the live 768 corpus
is intact, and the 10,000 artifact exists only as evidence. What would actually widen the band is a semantic
encoder (the ladder's step 1), at whatever width.

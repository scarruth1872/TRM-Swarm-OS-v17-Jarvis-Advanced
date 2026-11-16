# Intelligence ingestion — investigation (2026-09-27)

Four candidate modules, measured on the live tree, not read from their papers.

## 1. Embedding model — the SOCKET exists, and it is not a model server

`trm_uefi_kernel/host_council_server/src/k3_retrieval.rs` is LIVE (serves `/k3/*` on :8022):

- The kernel "does not answer by sampling a token. It quantizes the query into balanced ternary trits and
  scores it against a corpus quantized the same way, using the kernel's own exported functions."
- `INDEX_REL = swarm_v2_artifacts/kernel_knowledge_index.json`, `DIM = 768`, `QUANT_MULTIPLIER = 1.0`.
- `ENCODER_NAME = "native-turbovec-hdc"` — whose doc comment is the load-bearing line: *"The query must be
  encoded by the SAME encoder or the two live in different spaces. No model server is involved."*
- It dynaloads `swarm_v2_rust_kernel/target/release/swarm_rust_kernel.dll` and calls the kernel's own
  Quantize / Encode / Agreement / Magnitude / Fidelity exports. A missing DLL, corpus, **or unreachable
  embedder returns 503 with a named reason — there is no fallback that silently answers from another path.**

Consequence for ingestion: a new embedder cannot simply be "plugged in" — it must be the encoder that built
the corpus, or the corpus must be RE-ENCODED. That recipe already exists and is proven:
`build_kernel_knowledge_index.py --model <enc>` plus `docs/FABRIC_REEMBED_20260923.md` — 180 docs in 1.066 s
(**168.9 docs/s**), determinism verified by **178/180 vectors bit-identical** on rebuild, round-trip cosine
gate **min 0.8055 / mean 0.8752** at 2 bits, with a re-baseline branch for legitimate corpus refreshes.

What the platform has today is `swarm_v2/skills/embedding_skill.py`: "deterministic subword hyperdimensional
random projection", 768 dims, zero daemons — a **random projection, not a semantic model** — while
`EMBEDDING_MODELS_EXPLAINED.md` tells the reader the swarm "now understands meaning, not just keywords".
That gap is the entire argument for ingesting a real embedder.

**No embedder file exists on this host.** Model roots hold only generative weights (`D:/trm-models/`:
bonsai, instella; `swarm_v2_artifacts/tier2_models/`: the 8B Q2_0 g64), and there is no `.onnx` or
`*embed*.gguf` anywhere. Acquiring one is the prerequisite, not a detail.

## 2. SRAM pattern computer — real, large, and unreachable from the served crate

- `trm_uefi_kernel/host_tranche/src/sram_pattern_computer.rs` — **219,491 B**, mtime Sep 25 12:42 (live).
- `swarm_v2/core/sram_pattern_computer.py` — 1035 lines. Its own docstring names the algorithms: GEMM,
  Sparse Attention, Kipf-Welling GCN, **Hopfield**, Cellular Automata, Boltzmann CD-1, "zero-GC, memory-safe,
  independent of heavyweight ML frameworks", "virtual architecture topologies mapped to L3 cache geometry".
- Three real PEs, all `MZ` on disk: `strip_sram_computer.dll` 44,032 B, `strip_k3_sram.dll` 116,736 B,
  `strip_sram_relax.dll` 120,832 B, beside `strip_sram_computer.rs` 203,757 B.

Reachability is the finding: every caller is a harness — `scratch/exercise_sram_strip.py`,
`investigate_and_integrate_15_sram_patterns.py`, `execute_advanced_microkernel_invariant_stress_suite.py`.
The served crate mentions SRAM exactly once, in `host_probe.rs`, and that is the L3 cache *size*, not the
computer. Hopfield is the reason it matters: an attractor memory is the natural recall substrate for a K3
domain verdict, and it is already written.

## 3. Fractal telemetry — a stream→trit instrument, already bound

`swarm_v2/core/fractal_telemetry.py` (144 lines): Higuchi fractal dimension `D_H ∈ [1,2]` over a telemetry
stream, "continuous chaos detection and Kleene K3 ternary state quantization". It is wired — imported by
`swarm_v2/core/hardware_voltage_microkernel_binder.py` — and has a Rust twin in
`swarm_v2_rust_kernel/src/fractal_pattern_recursion.rs`.

Honest scope, because this is where a category error would be easy: it quantifies a **time series**. Feeding
it topic text would measure a hash of the text, not meaning. Its correct role here is a **regime gate** — "is
the platform coherent enough right now for this verdict to mean anything" — a state trit, not a semantic one.

## 4. Aura — deliberately refused by the live kernel

`host_council_server/src/aura_handlers.rs` refuses every `/aura/*` path and names the other product:
"Rollout 12: /aura/* is a DIFFERENT product, and the kernel was answering it with the catch-all … So every
/aura path is refused, and the refusal NAMES the other product rather than pretending."

The language is real elsewhere: `swarm_v2_rust_kernel/src/aura_engine.rs` (+ cli/lib/main),
`swarm_v2/core/aura_engine.py` (lexer, AST evaluator, 100-metaconcept library, semantic field state),
`aura_cognition.py` (990 lines: 100 metaconcepts, **Kleene K3 balanced-ternary invariant gating**, voltage
substrate harmonization), plus `swarm_v2/routes/aura.py` and `dashboard/src/components/AuraStudio.jsx`.
Its live Python importers are `agent_intelligence_bridge.py` and `routes/*` — offline tooling, not the
kernel. Bringing it into the runtime means overturning a recorded design decision; that is the operator's
call, not a wiring gap to be quietly filled.

## The ladder — cheapest real win first

1. **Acquire a 768-dim CPU embedder, re-encode the corpus, point `ENCODER_NAME` at it.** The socket
   (`k3_retrieval.rs`), the rebuild script and the verification pattern all exist, and the scale is trivial:
   the live index is 2,180 docs, i.e. ~13 s at the measured 168.9 docs/s. This is what turns the fabric from
   a random projection into a semantic space.
2. **Route the council's K3 domain through `/k3/*`** instead of the TRM 7M hash. Blocks 177/178 made the
   model runnable and made its trit decide; this replaces its *input* with something that means something —
   the trit becomes measured proximity to the domain's corpus rather than an FNV hash of the topic.
3. **Put the SRAM pattern computer behind that retrieval** (one Hopfield attractor per domain). The code and
   three real PEs are already built; this step is the link from `host_tranche` into the served crate.
4. **Fractal telemetry as the regime gate** on the verdict — system state, not topic.
5. **Aura** — needs an operator decision to revoke Rollout 12 before any wiring is legitimate.

Ordering note: step 1 gates step 2. A semantic verdict cannot be built on a hash space, and the kernel's own
route refuses to score a query encoded in a different space than its corpus — by design, with a 503.

## Live state at investigation time (measured, not inferred)

- `GET /k3/stats` answers with the fabric's own contract: **76,761,697 B**, "one nibble per trit, 2 trits/byte
  (the store's own rule)", legal nibbles `0b0000=-1, 0b0001=0, 0b0011=+1`, **illegal 0.0%**.
- `GET /k3/retrieve?q=…&k=3&verify=1` **works today**: `status: ok`, `encoder: native-turbovec-hdc`, three
  hits per query. Both dependencies are present — the encoder DLL
  `swarm_v2_rust_kernel/target/release/swarm_rust_kernel.dll` (727,552 B) and the corpus
  `swarm_v2_artifacts/kernel_knowledge_index.json` (53,166,343 B).
- **And it moves with the question** (the same test used on the council):

  | query | top-3 scores |
  |---|---|
  | "TRM council K3 ternary verdict" | 105, 104, 99 |
  | "purple elephants compose jazz fugues underwater" | 95, 94, 94 |

  Different queries, different scores — the route is not a fixed tally. But look at the band: **ten points
  between an on-topic query and deliberate nonsense.** That is the signature of a random projection — vectors
  near-orthogonal, everything clustering in a narrow cone. The socket works; **the space is flat.** Semantic
  separation is precisely what an ingested embedder buys, and this table is the measurement of what is
  missing rather than an argument about it.



# Benchmark run — 2026-09-23 (fabric / tier-2 / ported kernels)

Host: Ryzen 9 5900HX laptop + RTX 3070 Laptop 8 GB. Canonical Rust platform, no Python in the
serving path.

| service | state |
|---|---|
| :8022 Rust `host_council_server` | HTTP 200, `HEALTHY`, `python_active:false`, uptime 87,697 s |
| :5183 LCARS dashboard | HTTP 200 |
| :8587 tier-2 `llama-server` | `Ternary-Bonsai-8B-Q2_0_g64`, build `b10964`, 4 slots |
| :8021 / :11434 | down (correct: standard-setup and Ollama are not the platform) |

## 1. Fabric retrieval — the ruler was wrong first

**v1 measured nothing.** Queries were drawn from the *middle* of each document, but every record's
embedding covers only its first `metadata.chars_embedded` characters (60–6000; p50 6000). Those queries
were largely outside the embedded text, so v1's 20% described the benchmark, not the fabric.

**v2, with queries drawn inside the embedded window** (15 sampled docs, ground truth built from the
source files — the engine's own output never scores the engine):

| row | native | packed | random baseline |
|---|---|---|---|
| phrase query, in-window | top-1 **33.3%**, top-3 46.7% | top-1 **40.0%**, top-3 53.3% | 0.56% / 1.67% |
| document title | top-1 **40.0%**, top-3 46.7% | top-1 **46.7%**, top-3 53.3% | 0.56% / 1.67% |

Kernel-internal retrieval: **p50 127 µs** warm (native), min 88 µs, p95 179 µs. HTTP wall p50 1.8 ms.

**Why short queries fail — measured, not guessed.** Every candidate's `doc_fidelity` lands in
**0.885–0.898**, a spread of 0.013 across the whole top-10; re-ranking by fidelity alone scores *worse*
(6.7%), and ranking by `score/magnitude` reproduces the engine's order exactly (magnitude is already
neutralized). The `nand_fabric_verify` gate states the store's geometry: **2 bits per dimension**,
cosine **min 0.8055 / mean 0.8752** over 180 vectors. At that quantization every pair of documents
already looks largely alike, so an 8-word query cannot separate near-ties.

**The store itself is sound.** Self-retrieval (a document queried with its own embedded text):
**80.0% top-1 at 1,200 chars, 93.3% at 4,000 chars**, and bit-identical on repeat runs (determinism
confirmed). So identification works when there is enough signal to accumulate; the bottleneck is
query-side discrimination under 2-bit quantization — not storage, not the K3 path.

## 2. Tier-2 generation

64 tokens in 1,925 ms = **32.7 tok/s** (prompt 32 tokens at 22.1 tok/s). Answer was on-topic — asked what
the voltage doctrine says about deriving rather than asserting, it described deriving a value as
"transparent and verifiable... not arbitrarily asserted". The 8B emits `</think>` markers; no claim about
quality beyond this single prompt.

## 3. Ported kernels I verified this campaign (this host, `-O`, 1 thread)

| kernel | ns/op | ops/s |
|---|---|---|
| `native_wavefront_denoise` 256 f32 → 256 trits | 135 | 7,407,353 |
| `fuse_and_detect` (2 snapshots → event) | 647 | 1,544,849 |
| `round_half_even_3` | 12 | 86,018,546 |
| `run_daily_evolution` (14 domains, full report) | 230 | 4,345,653 |
| `pbft` propose + 8 prepares + phase read | 557 | 1,795,364 |
| `compute_resonance` (4-node lattice) | 888 | 1,126,202 |

**The first run of this bench was invalid**: `round_half_even_3` reported **0 ns/op** because LLVM
hoisted the loop-invariant, constant-argument call out of the loop. Guarding every loop bound with
`black_box` produced 12 ns/op; the other rows barely moved, which is the evidence that *they* were not
being hoisted.

## 4. Platform gates

| gate | result |
|---|---|
| `verify_k3_derivation.py` | RC 0 |
| `verify_strip_binaries.py` | RC 0 |
| `nand_fabric_verify.py` | RC 0 |
| `verify_nand_fabric_index.py` | **RC 1 — text drifted from disk for 2 records** |

The two stale records are **`CHANGELOG.md`** (34,033 stored bytes of 154,269 on disk) and
**`docs/CATALYST_COMPLETION_PLAN.md`** (3,154 of 6,825) — both files this campaign has been editing. Their
stored text and embeddings predate the edits, so the fabric answers from a stale copy of those two
documents. This is a **re-embed**, not a re-record.

Scripts: `retrieval_bench.py` (v1, superseded), `retrieval_bench2.py` (v2), `kernel_bench.rs`.

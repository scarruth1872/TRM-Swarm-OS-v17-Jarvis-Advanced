# Validation: "Pre-Ingestion & Training Plan" vs the live system (2026-09-23)

Method: every number in the document was either (a) recomputed from its own figures, (b) compared against
the live kernel at :8022, or (c) searched for in the tree to find whether it has a measured source. No
claim below is asserted; each carries the measurement that produces it.

## Claims that hold

| claim | check |
|---|---|
| 584 agents | matches the ported `daily_evolution_engine` roster (I verified `agents=584` against its oracle this session) |
| native ops exist | `c_kleene_quantize_avx2`-family names resolve in 50 files; `strip_kleene_quantizer.rs`, `simd_accelerator.rs`, `pack_trits_fast` / `bulk_numpy` all present |
| persistent NAND fabric | live: `fabric: NVMe NAND K3 Fabric`, `status: IMMUTABLE_LOCKED`, store 76,761,697 B |
| 1 TB NAND | C: is 1.0 TB total (only **0.1 TB free** — worth noting against a 48 GB plan plus ChEMBL) |
| "Previous Monad outputs" (category E) | **124 of the 180 fabric documents are `papers/`** — the Monad Journal output is already ingested |

## Claims that do not hold

| claim | measured | verdict |
|---|---|---|
| "**85,532,109 trits**" | live `total_trits = 153,523,394` = 76,761,697 B × 2 exactly | **stale by ~68M**, and it is not the same quantity by another name |
| "**4 silicon fabrics**" | live `shards = 18`; there are **4 stores** (`.k3`, `.turbovec`, `.text.jsonl`, `.tercode`) | the count matches the store count, not a fabric count — a category conflation |
| "612K tasks/s" | the only source is `A_MONTH_OF_CONTINUOUS_WORKLOADS_HARVEST.json`, where it is **`task_throughput_per_sec` inside `execution_parameters`** | an input parameter, not a measurement |
| derived "1,586,304,000,000 executable tasks" | 612,000 × 2,592,000 = 1,586,304,000,000 exactly | **a product of the parameter**, i.e. arithmetic rather than work performed |
| "expansion_ratio **360x** relative to original 968K baseline" | 153,523,394 / 968,000 = **158.6×** | the stated ratio is ~2.3× too high |
| "adds **200–500 million trits**" for 11.4 GB of ternary | at the platform's own packing (2 trits/byte, per `/memory/stats`) 11.4 GB = **~22.8 billion trits** | understated by **~45–114×** |
| "RTX 3070 processes **1.3B tasks/s**" | no source found anywhere in the tree | unsourced |
| category A: "self-codebase ... so the swarm knows its own body" | **0 source-code files** in the 180-document fabric (124 papers + 49 root docs + 7 docs) | not ingested |
| categories B, C, D (Rust ecosystem, hardware specs, datasets) | the dataset names appear in 20 files as **mentions inside papers and protocols**; no ChEMBL/PDBbind/MIMIC/Empathetic data on disk | proposed, not present |

## The artifact the numbers come from

`swarm_v2_artifacts/A_MONTH_OF_CONTINUOUS_WORKLOADS_HARVEST.json` (timestamped 2026-08-16) is the origin of
several of them. It also contains:

- `composite_trust_score_pct = 99.4` alongside per-agent `latency_us` values from 6.7 to **115,527.5** — one
  agent three orders of magnitude off the rest, so the aggregate cannot be a measured distribution.
- `baseline_nand_trits = 153,523,393` against the live **153,523,394** (off by one: derived-from-bytes vs
  stored).
- engineering entries written as accomplished fact — "ffmpeg = Absorbed into AVX2 native microkernel media
  pipeline", "docker = Absorbed into Hermes PBFT orchestration (zero-daemon)".

The platform already audits this class itself: `measurement_audit.json` scans 106 files and flags **557
hardcoded measurement literals** (detector D1), plus 30 "fabricated telemetry constants" (D5) and 6
"epsilon floor manufacturing a ratio" (D3) — including `terminal_bench_score_pct: 96.4`.

## What this means for the plan

The plan's *structure* is sound and its §6 "Honest Notes" is genuinely honest (ChEMBL incomplete,
partially-open specs to be documented rather than guessed, "training time is measured, not assumed"). The
problems are all in the **numbers attached to the current state**, and they run in both directions:
the fabric is **larger** than stated (153.5M trits, not 85.5M), the ingestion would add **~45× more trits**
than stated, and the throughput figure is a configuration parameter presented as throughput.

Two of the plan's own framings are also contradicted by the live system, and both are worth stating plainly
rather than arguing about: **"No HTTP"** — the kernel *serves* HTTP on :8022 and that is the intended
interface; the doctrine means no HTTP *dependency* in the compute path. And **"the Rust kernel will be
voltage-identical to the C++ kernel"** — the kernel is Rust today, there is no C++ kernel to be identical to.

## Bottom line

Validated as a **plan**: coherent, and its pre-ingestion premise (knowledge before construction) is sound.
Not validated as a **description of the current state**: 6 of its 9 headline figures are stale, conflated,
understated or unsourced, and the two the whole plan leans on — 85.5M trits and 612K tasks/s — are
respectively out of date and a parameter rather than a measurement.

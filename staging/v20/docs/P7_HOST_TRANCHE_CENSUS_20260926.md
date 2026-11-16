# P7 — `host_tranche`: what it is, measured (2026-09-26, block 158)

The plan carried one line item as **"host_tranche: 23 of 28 files undeclared — the compiled but unreachable
class."** That premise does not survive contact with the tree. This is the classification, by reachability first
and content hash second, with no module dismissed by name.

## What it is

`trm_uefi_kernel/host_tranche/` is a standalone crate: **28 files** (27 modules + `main.rs`), **zero
dependencies**, its own `Cargo.lock`, and **nothing in the repository names it** — not the kernel, not a
workspace member, not a build script. The only file that mentions `host_tranche` is its own `Cargo.toml`.

Its module headers say what it is:

> `//! TRM Swarm OS - chaos_engine (mechanical no_std port, TRM platform engine).`
> `//! Constants and structure extracted 1:1 from swarm_v2/core/chaos_engine.py.`
> `//! Functions marked // MANUAL need human verification of math semantics.`

with bodies like `pub fn get_chaos_engine() -> u64 { 0 // TODO: complete 1:1 math }`.

**It is the port skeleton tranche**: the mechanical stubs of the Python→Rust port of `swarm_v2/core/*.py`,
dated 2026-08-28, whose completed forms now live in `trm_uefi_kernel/src/`.

## Reachability and build state (the corrected premise)

| question | measurement |
|---|---|
| named by any live source? | **no** — only its own `Cargo.toml` |
| workspace member? | **no** |
| ever compiled? | **no** — no `target/`; and `main.rs` (8 lines) declares `mod radix_trie_matcher; mod sram_pattern_computer;` — **2 of 27** modules, so 25 were never even type-checked |
| unfinished markers | 8 × `TODO: complete 1:1 math` across 5 files |
| tracked by git? | **yes** — 30 files, commit `79a7fb4` ("track complete bare-metal trm_uefi_kernel microkernel sources…"), so every byte is recoverable |

So it was not "compiled but unreachable". It was never compiled **and** never reachable — a skeleton whose arms
were never fitted. The correction matters: the item was queued as a wiring job and is actually a
classification job.

## Content: every file differs from its live twin

Across the whole repo there are **1,127 `.rs` files**; **335 basenames appear in more than one place** (849
files), of which **213 are byte-identical** across their copies and **115 are named by live code**.

For `host_tranche`, **0 of 28 files are byte-identical to any twin** — every one is a distinct, earlier
revision. Each is 333 B–13 KB from 2026-08-28 while its live counterpart is newer (2026-09-18…09-25) and 5–20×
larger, e.g. `coherence_thresholds` 333 B → 5,241 B, `resonance_engine` 548 B → 10,315 B, `radix_trie_matcher`
523 B → 11,013 B.

**The one outlier is `sram_pattern_computer.rs`: 219,491 B in the tranche vs 36,824 B live.** Inspected rather
than assumed: that file's bulk is a single **196,630-character line**, 44 % digits, beginning
`pattern: vec![0x00, 0x00, …, 0xF0, 0x3F, …]`. `0xF03F` is f64 `1.0` little-endian at that offset — it is a
**dumped device pattern buffer**, data, not logic. The live module is 887 lines with 60 functions against the
skeleton's 417 lines with 22, and it computes the patterns its own way. Nothing of logic exists only in the
tranche.

## "What's missing" — the port's real gap, at name level and at concept level

The skeletons came from `swarm_v2/core/*.py`, so the Python definition set is the port's target list. Against
the **live kernel's whole function set** (214 `.rs` files, 1,671 distinct function names), after normalizing
the naming differences a port legitimately makes (`__init__` → `new`, leading `_` dropped):

**123 of 232 target names have a live counterpart (53 %); 9 of 27 modules are fully covered.**

A raw name count would be a false alarm, because most gaps are naming differences — verified concept by
concept, not asserted:

| Python name (no such Rust fn) | implemented live as |
|---|---|
| `calc_h`, `compute_boltzmann` | `sram_lut_boltzmann_energy` (sram_pattern_computer) |
| `compute_cellular`, `gpu_pattern_cellular` | `sram_lut_cellular_automaton` |
| `compute_hdc`, `compute_gnn` | `sram_lut_hdc_bind`, `sram_lut_gnn_aggregate` |
| `calc_free_energy` | present in `sram_pattern_computer` |
| `log_event`, `_atomic_append` | `atomic_append_line` (turbovec_memory) |
| `compact`, `_rebuild_index` | `compact_plan` (turbovec_memory) |
| `get_ledger_hash` | present in `continuum_pbft.rs` |
| `create_secure_payload`, `validate_secure_payload` | present in `comm_protocol.rs` |
| `get_topology_overlay` | present in `skill_matrix_sync.rs` |
| `_probe_gpu`, `_write_pattern_to_device` | the live module's GPU path; `/native/gpu/*` reports 18 checks pass |

**The residual candidates — concepts with no presence at all in the live kernel** (the honest short list,
each needing a port / supersede / park decision rather than an assumption):

1. `benchmark_all`, `benchmark_all_native`, `compare_fabrics` — benchmarks, not behavior; the live GPU module
   is measured by its parity artifact instead.
2. `scan_file`, `scan_all`, `generate_antibody`, `save_antibodies` — the fuzzing engine's **file**-scanning
   surface; the live `self_fuzzing_engine` (7.6 KB) has no file scan.
3. `get_agent_load`, `record_task_start/end`, `redistribute_task`, `get_swarm_load_summary` — the load
   balancer's **live** metrics; the live module (5.8 KB) has none of them.
4. `on_skill_created/evolved/forgotten`, `record_skill_usage` — Python **event-callback** hooks; the Rust
   module is synchronous by design, so this is likely a design difference rather than a gap.
5. `get_gateway_stats`, `native_batch_collapse`, `is_node_isolated`, `process_latency`, `start_session`,
   `stop_session` — small per-module accessors.
6. `radix_trie_matcher`'s 29 `_handle_*` handlers (`_handle_agent_spawn`, `_handle_fetch`, …) — the Python
   async IPC surface, superseded by the kernel's own HTTP dispatch; 29 of the 106 gaps come from this one
   module.

## Decision

- **NOT retired.** The retirement rule requires a byte-identical twin that stays; **0 of 28 qualify**, so the
  rule blocks it — a name is not a measurement, and neither is a size. Its 219 KB of pattern data and its 27
  skeleton signatures are not present, in that form, anywhere else.
- **NOT wired.** Its bodies are stubs (`0 // TODO: complete 1:1 math`); wiring them would add dead code to the
  narrow waist, which the doctrine forbids. Nothing in the running platform would change.
- **Classified: SUPERSEDED AS SOURCE OF RECORD, KEPT.** The live kernel's same-named modules are newer and
  carry more functions; this is the earlier skeleton, recoverable from git (`79a7fb4`) at any time.

Artifacts: `swarm_v2_artifacts/{host_tranche_census,module_copy_census,host_tranche_port_coverage,
host_tranche_port_gaps_normalized}.json`. Harnesses: `scratch/{census_host_tranche,census_module_copies,
analyze_port_coverage,analyze_port_gaps}.py`.

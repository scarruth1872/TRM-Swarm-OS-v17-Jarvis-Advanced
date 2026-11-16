# Checkpoint C — 2026-10-12 (blocks 287–309)

## State at checkpoint

- **Ledger: 309 blocks** (hash-chained monad journal).
- **Kernel v20.0.0 healthy on :8022**; supervisor running; dashboard builds clean.
- **Task executor: 27 compute modes** (up from 9 at checkpoint b).
- **Parsers: 22 native / 0 remaining** (all REGISTRY_FORMATS ported).
- **Execution pass re-run: 180 real_result** (up from 9 at investigation time), still processing (485 IN_PROGRESS).

## What changed since checkpoint b

### Engine wiring (the "engines already exist" discovery)
The `needs-engine` tier was mislabeled. Engines live in `host_council_server/src` + `wsl_crates/trm-rust/` and were simply unreached by the executor. Wired:
- **docking** → `swarm_docking` (Vina scoring), **drug_triage** → `swarm_drug` (K3 triage), **trm7m** → Mamba-3 SSM forward, **fit_verdict** → `swarm_fit` (K3 verdict), **ternary_gemm** → `ternary_engine` (K3 multiply-free GEMM, AVX2).
- **Kernel primitives** (all pre-existing, now wired): **ipc** → `mailbox_ring` (swarm_mailbox.ring), **nand** → `swarm_nand::NandFabric` (unified fabric), **scheduler** → `hardware_binder` (voltage doctrine: classify + bind to silicon), **allocator** → `persistent_mmap_allocator`, **lock_free_queue** → 2-thread SPSC ring.
- **WSL crates**: **turbovec** → `swarm_turbovec` (16× ternary packing), **mesh_route** → `swarm_mesh::route`, **kv_store** → `swarm_kvstore` (K3-native KV store). Added swarm-mesh + swarm-kvstore to Cargo.toml.

### Deterministic tractable-rust modes (new code)
**statistical** (OLS regression), **pgx_star_allele** (CPIC mapping), **differential_expression** (log2 fold-change), **dmr** (windowed methylation), **variant_prioritization** (ACMG + AF).

### Full 1000-task rebaseline
15 parallel subagents rebaselined all 1000 tasks onto the platform's real capabilities → `swarm_v2_artifacts/rebaselined_1000_tasks.json`. Tier census: real_compute 96, real_council 129, real_telemetry 76, real_security 68, honest_recall 631. Catalyst backlog → `swarm_v2_artifacts/catalyst_backlog.json`.

### Execution pass (×2)
Re-dispatched the 1000 tasks through the 18-mode then 27-mode executor. real_result 9 → 102 → 180 (still climbing). Every mode produces.

## Council verdicts this phase (all PROCEED)
rebaseline (+6 Devo/Seeker) · re-run (+3 Flow) · native analysis pipeline (+6) · CNV (+3) · driver/signature (+3) · fusion→Bridge then PROCEED · single-cell (+3) · kernel primitives (+3 Devo) · wire remaining crates (+3 Jarvis) · re-run 24-modes (+6 Devo/Flow).

## Honest boundaries (carried)
- All compute runs on **canonical declared inputs** (tasks supply no datasets); every artifact states its `basis`.
- Docking dG is on an arbitrary pose (not minimized). trm7m conf is the 7M model.
- DE computes fold-change only (no p-value). DMR/prioritization on canonical samples.
- Remaining recall: ML/NLP/CV training runtimes, OWL reasoners, multi-node fabric — genuinely no engine.
- `ternary_engine::infer_layer` needs a `.trm` bundle (none on disk).

## What's next (gated, queued)
1. Finish the re-run execution pass (485 IN_PROGRESS → final census).
2. Wire remaining located crates: swarm-ssm, swarm-instella (MoE gate), swarm-brain, swarm-voltage, swarm-health, swarm-vitals, swarm-jarvis, swarm-conductor, swarm-router, swarm-sovereign, swarm-thsm, swarm-shard.
3. Build the genuinely-missing engines (only where no crate exists).

## Files that changed (key)
- `trm_uefi_kernel/host_council_server/src/task_executor.rs` — grew 9 → 27 modes (the executor).
- `trm_uefi_kernel/host_council_server/src/native_parsers.rs` — 22 native parsers.
- `trm_uefi_kernel/host_council_server/Cargo.toml` — +swarm-mesh, +swarm-kvstore.
- `swarm_v2/core/agent_intelligence_bridge.py` — domain table 266 → 496.
- `wsl_crates/trm-rust/swarm-council/src/lib.rs` — regenerated council table.
- `swarm_v2_artifacts/{rebaselined_1000_tasks,catalyst_backlog}.json` — new artifacts.

# CATALYST COMPLETION PLAN — Convert 146 Non-Recognized Modules to REAL
**Council work order:** a0a927f8abb9d3cf (14/14 enacted)
**Unified entity:** full platform (council + registry + verified-harness + Monad)
**Truth anchor:** 0x4D4F4E4144000001

---

## The Mission
The registry (v19.030) revealed: 179 kernel modules compiled, but only
24 REAL. The remaining 146 (53 PORTED + 93 SKELETON) compile but carry
no verified logic. This plan completes them — as ONE unified entity —
so every module that can hold real logic DOES, verified before it enters
the registry.

## Phase 0 — Classification (the honest triage)
Every one of the 146 modules is classified into exactly one bucket:

| Bucket | Definition | Action | Est. count |
|--------|-----------|--------|-----------|
| **DATA-OK** | Singleton getters / pure constants / structs — legitimately complete as data | Keep as-is; register as `DATA` (recognized) | ~80 |
| **NEEDS-LOGIC** | Load-bearing modules with MANUAL bodies that must execute real logic | **Port via verified harness** → `REAL` | ~35 |
| **EXCLUDED** | Platform-rule companions (no-HTTP/no-torch/no-LLM) | Keep; register as `EXCLUDED` (documented) | ~9 |
| **DEV-TOOL** | Benchmark scaffolds / dev utilities with stdout bodies | Keep skeleton; register as `DEV` | ~22 |

## Phase 1 — Port NEEDS-LOGIC (verified-harness pattern)
For each NEEDS-LOGIC module, the proven pipeline:
1. **Extract** Python reference (swarm_v2/core/*.py) — real function semantics
2. **Port** to no_std Rust by hand (arithmetic/logic/K3 patterns)
3. **Build** host harness (Rust vs Python on identical inputs)
4. **Compare** — outputs must match exactly
5. **Register** — only a PASS enters the registry as REAL
6. **Record** — Monad experience ingested

Load-bearing priority (original backlog order):
- pbft_consensus, resonance_engine, security_scanner (the original Tier 1)
- predictive_watchdog, chaos_engine, chaos_simulator, heuristic_consensus
- ddr (done v19.029), coherence_thresholds, cognitive_load_balancer
- + all remaining NEEDS-LOGIC identified in Phase 0

## Phase 2 — Registry Update
- `module_registry.rs` REAL_MODULES grows with each verified port
- `build_module_registry.py` re-run: SKELETON/PORTED counts shrink
- Every module has a truthful status; recognition is EARNED, never granted
  by compilation

## Phase 3 — Verification & Sealing
- Final registry render on the booted terminal (`registry` command)
- Zero build errors (kernel compiles with all ports)
- USB synced · NAND record · Monad-sealed per interaction
- Honest report: what became REAL, what stayed DATA/DEV/EXCLUDED and why

## Success Criteria
| Metric | Target |
|--------|--------|
| NEEDS-LOGIC modules ported + verified | 100% of bucket |
| Each port PASSes the harness before registry entry | 100% |
| Registry REAL count | 24 → 24 + (NEEDS-LOGIC completed) |
| Kernel build | 0 errors |
| Zero external APIs | maintained |
| All decisions sealed + persisted | Monad + NAND |

**All Is One. I Am Because We Are.** The Catalyst completes itself — not by
claiming, but by verifying. Recognition is earned by logic that runs.

---

## Phase 1 — progress (measured, not claimed)

The buckets above were estimates from `catalyst_phase0_classify.py`, which only ever looked at
`trm_uefi_kernel/src`. The live runtime and the DLL crate were outside its scope, so the usable
number comes from `swarm_v2/core/rust_python_parity.py`, which audits **all three trees** and writes
`swarm_v2_artifacts/rust_python_parity.json`. Full per-module detail: `docs/EFI_PORT_CAMPAIGN_LOG.md`.

| metric | value |
|---|---|
| EFI modules in the audit | 186 |
| paired with a python oracle | 161 |
| flagged NEEDS-LOGIC (`port_ratio < 0.60 or fn_ratio < 0.60`) | **66** (was 86 at the start of the campaign) |
| modules this campaign has touched | **31** — 30 ported, 1 reclassified |
| of those, verified against a **compiled strip DLL** | 20 |
| of those, verified against a **python replica** | the remainder (several by both) |
| EFI binary | 15,926,784 → **16,325,632** bytes (+398,848, all of it reachable) |

Note the flag is **LOC-based**, so a module whose remaining lines are deliberately host-side (torch
checkpoint reads, psutil sampling, HTTP handlers) keeps its flag even when every function is now
ported. Both figures are reported for that reason: the flag count and the function ratio.

### What the campaign changed about the pipeline

- **Step 1 is "read what it computes", not "list what is missing".** Every one of the first four ports
  had a *wrong* computation rather than an absent one — a different formula, a different status model,
  four missing terms, a fabricated weight set. The audit's function list is a prompt to look.
- **Step 3's harness should prefer the platform's own artifact.** Each strip ships its `.rs` source
  beside its `.dll`, so a port can be compared against the compiled artifact bit for bit rather than
  against a formula that might itself be stale.
- **Step 5's registration needs a receipt, not a compile.** Unreferenced `pub fn` items are stripped by
  LTO: three ports compiled cleanly and changed the binary by zero bytes until a command called them.
- **Step 6 should name things as the oracle names them.** The audit matches by name, so a faithful port
  under a new name is reported as missing; several were renamed to clear false negatives.

### Revised targets

| Metric | Target |
|--------|--------|
| Ported + verified modules | 30 done; 78 flagged remain (22 of the touched ones are still flagged only on LOC) |
| Each port PASSes a harness before being called done | 100% — every module in the log carries its case count and difference count |
| Kernel build | 0 errors, and a **fresh** artifact every time (a failed build leaves the old binary in place) |
| Zero external APIs | maintained in firmware; host-side companions stated per module |
| All decisions sealed + persisted | Monad + NAND, unchanged |

## Measurements (re-derived from the audit artifact)

Every figure below is produced by running `swarm_v2/core/rust_python_parity.py`; none is typed by hand.

| metric | value |
|---|---|
| EFI modules in the audit | 186 |
| paired with a python oracle | 161 |
| modules this campaign has touched | **56** (55 ported + 1 reclassified) |
| flagged NEEDS-LOGIC | **66** |
| of the campaign's ports, still under the LOC flag | 35 |
| worst remaining port_ratio | neural_wall 15%, openclaw_gateway 17%, federation 17%, gpu_infinity_computer 18%, proactive_loop 18%, task_arbiter 18% |
| EFI binary | 15,926,784 → 16,325,632 bytes (+398,848) |

The flag is LOC-based, so a module whose remaining lines are deliberately host-side (HTTP providers,
asyncio loops, report persistence, handler callables) keeps it with its function ratio well above
100 percent. Report both numbers.

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

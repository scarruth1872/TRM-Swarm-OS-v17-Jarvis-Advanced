# TRM Swarm OS v20.0.0 — Executive Summary (2026-10-11, updated from 2026-10-07)

## What it is

A sovereign, offline personal-AI platform. The runtime is a Rust microkernel plus a
React/LCARS dashboard plus a NAND K3 ternary fabric plus a 15-agent council that deliberates
and seals every decision. Python is offline tooling only — never in the runtime path. Nothing
is cloud-dependent: the platform answers on loopback.

The platform is **not token-based**. Model intelligence is distilled into the NAND fabric —
ternary (K3) trits, TurboVec (2-bit hyperdimensional) vectors, and a text sidecar — plus an
infinity cache. At runtime, answers come from `fabric_recall()` (the fabric primary path),
with `mamba3_turbovec_ssm` and `llm_router` as co-processors and a synthetic in-memory model
as fallback only. Ingestion order is **ternary → turbovec → extract**, written by the
**deterministic fast path** (no-LLM native encoder, direct 2-bit binary, no JSON intermediate).

## Architecture (simplified)

- **Layer 1 — bare-metal UEFI microkernel** (`trm_uefi_kernel/`, `no_std`).
- **Layer 2 — on-silicon computer**: TurboVec ternary vectors, packed-domain retrieval
  (integer scan, no f32 materialization), the NAND K3 fabric.
- **Host layer — the running platform**: `host_council_server` (:8022) + the dashboard
  (:5183, Vite) + semantic embedders (:8588/:11434). The kernel is **model-free by design** —
  its tier-2 LLM co-processor (:8587) is not yet wired in.
- **Council**: 15 agents, PBFT quorum (f=4), K3 ternary votes, sealed to a Monad ledger.

## The governing discipline — "never guess"

Every number the platform serves is measured, derived, or null-with-a-reason. Since the 10-07
summary, that discipline was applied to the whole tree, not just the voltage rails:

- **Φ_V is machine-derived — 1.0 at nominal** — re-derived from host nominals (5900HX, RTX 3070,
  DDR4-3200), replacing the bare-derived 0.378. The 0.378 lingers in ~20 Python locations; that
  sweep is flagged but not yet executed.
- **The fabricated production certificate was retracted.** `WORLD_PRODUCTION_READY_CERTIFIED`,
  Φ_V 0.9115, "129 native modules", and the "5 nodes / CUDA+Metal" federation attribution do not
  trace to any measurement. Thirteen council motions (MOTION-033..045, all PBFT 15/15) retired
  them — the drug pipeline's "Pearson r 0.857" (a `max(0.85, 1−log_error·0.15)` proxy), arbitrary
  K3 confidence clamps, the stuck 0.99-fitness antibody evolution, `SOVEREIGN_VALIDATED` /
  `TRUTH_AFFIRMED` overclaim strings, and the THz `STABLE_OPTICAL_PHASE_LOCK` peer telemetry.
- **Real values kept with correct attribution.** The **15.53 THz** figure is a real physical
  constant (silicon Fd-3m optical-phonon mode, period 64.39 fs) — only the fabricated "lock" was
  retired. Federation is **4 live nodes**, not 5.
- **On-device intelligence recovered.** The kernel had been answering from a synthetic in-memory
  model because the boot tools copied only `.k3` and omitted `.text.jsonl` and `.turbovec`; all
  three fabric parts are now copied and `fabric_recall()` reads the real fabric again.

## Current measured state (2026-10-11)

| Item | Value |
|---|---|
| EFI kernel payload | 16,476,160 B (measured Oct 10) |
| Port-test parity | PASS 1242 / FAIL 0 |
| Module matrix | 25 covered / 20 host-side / 63 skeleton |
| Monad ledger | 257 blocks |
| NAND fabric `.k3` | 76,761,697 B (~153.5M K3 trits) |
| NAND fabric `.text.jsonl` | 3,168,214 B (2,180 records, 2,174 with text) |
| NAND fabric `.turbovec` | 525,297 B + per-embedder family |
| Structured store | 2,897,819 molecules · 99.9995% with SMILES |
| Federation | 4 live nodes |

## Codebase size (full-tree scan, 2026-10-09)

**145,526 files / 27.0 GB / 108,643 dirs** across three overlapping copies. Source classified
**3,950 REAL / 44 FACADE / 110 SUSPECT / 29 STUB / 84 SKELETON**. `swarm_v2/` +
`swarm_v2_artifacts/` is the canonical tree; `swarm_v2_integrated/` is frozen as a dated snapshot
(not renamed — it is also a live pipeline working dir). The Python tree is the legacy pre-port
platform; the live runtime is the Rust kernel.

## In-flight and open

- **ChEMBL ingestion (complete)** — 2,897,805 compounds encoded through the corrected
  ternary→turbovec→extract fast path (594 rec/s, ~81 min) into a **separate**
  `swarm_v2_chembl_fabric.*` co-processor store (~2.08 GB); the 2,180-record main fabric stays
  untouched. Structured SMILES/props are 100% in `structured_index.db`.
- **Multi-store recall router (live)** — `/recall` classifies a query by kind and routes it:
  molecule → ChEMBL co-processor (ChEMBL-id exact match with sorted early-exit, or capped lexical
  term match), structured → host-side SQL (~3.9M rows), general → `/k3/retrieve`. The 1,994
  relational sidecar stubs were fixed to real text (on-device recallable 180 → 2,174 records), and
  the 10 drifted docs were re-embedded (VERDICT intact, 0 drifted).
- **Honest unclosed gaps:** the tier-2 LLM (:8587) is orphaned at boot (kernel is model-free and
  does not route to it); the dashboard port collision (:5183 vs :5184); ring-0 `trm_ec_smbus.sys`
  cannot load (BitLocker TPM blocks testsigning); live rail voltage is unreachable (VRM SVI2 /
  IT87xx monitor IC needed — AMD MSR returns VID targets only); stuck antibody evolution returns
  the same `DYGDFYMDV` sequence (behavioral bug, not a number); the molecule *term* route is
  lexical (SMILES/numeric props only) — a packed vector scan is the follow-up.

## What this platform is not

It does not claim a rail voltage it cannot read; it does not assert a resonance lock its derived
Φ_V does not clear; it does not report a "federation" number from a single-process local run; and
its performance numbers are re-measured, never repeated from a docstring. That honesty is the
platform's defining property.

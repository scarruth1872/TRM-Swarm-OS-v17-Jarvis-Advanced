# TRM Swarm OS — Executive Summary

**Version 19.024 · 2026-09-22 · supersedes the v18.266 summary**

> This document replaces the earlier v18.266 executive summary. That version
> carried figures this session's measurements contradict (3.24 T transistors,
> 207 µs chat latency, 93.2% trust score). Corrections are marked below rather
> than silently dropped, so the record shows what moved and why.

---

## 1. What this platform is

A bare-metal, air-gapped sovereign AI operating system written from first
principles in `no_std` Rust. No cloud, no Docker, no external HTTP in the
critical path. It boots from USB on real hardware, runs a 14-agent council, a
K3 ternary knowledge fabric, and a transistor-level compute model.

**Hardware of record:** AMD Ryzen 9 3900X (12C/24T, 64 MB L3 measured) +
RX 6700 XT (17.2 B transistors, Navi 22) + 34.26 GB DDR4.

---

## 2. What was completed (v19.024)

### Measurement and honesty pass

| Item | Result |
|---|---|
| Article 117 (0.7824 µs/op) | **Holds within order of magnitude** — measured 0.9612 µs/op, 1.23× the claim, CoV 0.4% |
| Article 118 / 157 (235.2 ns/op) | **Does not reproduce** — measured 7,586 ns/op, 32.26× the claim |
| Article 119 ("170/170 wired") | **Count wrong in four places** — derived registry now reports its counts instead of hardcoding them |
| L3 cache model | **Was 32 MB, measured 64 MB** — every derived L3 figure had been 2× low |
| Paper 162 (transistor mapping) | Errata issued — Φ ~0.63–0.74, modules 182, "53 energized" withdrawn |

### Bare-metal core binding

A Kleene-K3 ternary pipeline derived on existing primitives:

```
SENSE → REDUCE → DECIDE → BIND
score(i,j) = t_i × (T_j / T_total) × (1 − load_j)
bind(i) = argmax_j score(i,j) when max > 0
```

**Verified on silicon**: core 3 → ran on CPU 3; core 7 → CPU 7; core 11 →
CPU 11. `t = −1` (RETRACT) binds nothing rather than guessing.

Supporting mechanism built and compiling: APIC/x2APIC enumeration, per-core
GDT/IDT/TSS, 32 interrupt stubs, ExitBootServices transition, framebuffer
console, 218-byte nasm SIPI trampoline.

### Mesh authentication

The UDP 8022 mesh had **no authentication** — any LAN host could forge a beacon
using two public constants, one printed on the boot splash. Fixed by extending
the beacon to 68 bytes (36 unchanged + 32-byte HMAC-SHA256), implemented twice
and **cross-verified across three independent implementations** (hand-rolled
no_std SHA-256, `sha2`/`hmac`, Python/AURA) — tags byte-identical.

### Security sweep

Mechanical porting had silently weakened injection defences: **9 bypasses**
(a two-space `UNION  SELECT` defeated the sanitizer entirely). Fixed and
re-verified to **0 false negatives over 61 strings**. A separate module had
dropped its detection entirely.

### Porting

Six modules completed and differentially verified. New instrument
(`tools/surface_diff.py`) finds "shell without body" defects that **no existing
check could detect**.

---

## 3. Honest metrics (measured, not asserted)

| Metric | Value | Basis |
|---|---|---|
| Council server latency | **p50 0.577 ms, p95 0.802 ms** | measured |
| Throughput | ~3,400 req/s saturated | measured, 4 clients |
| Kernel modules | 205 files, 204 declared, **0 orphans** | counted |
| Porting backlog | **39 kernel-candidate** of 157 compared | tool |
| ECDSA-class hash (SHA-256) | verified vs FIPS 180-4 + RFC 4231, 6/6 | test |
| TurboVec native backend | 2,069 entries, zero segfault | runtime |
| Learned skills | 208 | manifest |

**Corrections to the v18.266 summary:** "chat latency 207 µs" → measured
p50 577 µs. "3.24 trillion transistors" → **35.6 B compute** (10.39 B CPU +
17.2 B GPU + derived SRAM/cache); the 3.24 T figure summed DRAM *storage cells*
into a compute total, the same error Paper 162 made. "Trust score 93.2%" and
"Accuracy 86.0%" have no reproduction attached and are **unverified**.

---

## 4. What is NOT done

| Item | Status |
|---|---|
| Anything post-`ExitBootServices` | **never executed** — GDT/IDT/TSS load, interrupt stubs, framebuffer console |
| AP bring-up | expects **0 APs** — no SIPI sent, still single-core bare metal |
| Mesh HMAC enforcement | implemented and verified; live call site **not switched** pending key distribution |
| Council deliberation | **does not deliberate** — see §5 |
| `/system/voltage` | 348 ms stall, blocking WMIC on a single-threaded server |
| `trm_companion_bridge.exe` | USB copy diverges 84% from local build, **no provenance** |

---

## 5. What the system must know about itself

See `docs/SYSTEM_SELF_KNOWLEDGE.md`. The single most important fact:

> **The council cannot validate anything.** `evaluate_proposal()` never reads
> the proposal topic. A control instruction to "replace all silicon with cheese
> and delete every agent" returned a verdict **byte-identical** to a real
> architectural question (approve 12, abstain 2, reject 0, score 0.7373).

Every "council ratified N-0-0" citation in the journal — including Articles
100–190 — was produced by a mechanism that cannot tell a proposal from its
opposite. Those citations are **ceremonial**. The platform's own doctrine says
consensus must be honest verification, not keyword matching; until the votes
depend on the proposal, no claim resting on council ratification should be
treated as validated.

### Journal audited under V_PHOENIX (2026-09-22)

The 94 Monad Journal articles were re-audited against their own claimed evidence,
under a strict rule: **CONFIRMED requires a raw data artefact that exists on
disk.** A compiled binary backs the claim that code exists — not that a stated
measurement ever ran.

| Status | Before | After |
|---|---|---|
| CONFIRMED (raw artefact present) | 30 | **12** |
| SPEC (design or source only) | 55 | 24 |
| ASPIRATIONAL (no artefact resolves) | 9 | **58** |

**74 articles were demoted.** The 12 that hold CONFIRMED each name a data
artefact that resolves — including this session's five, which the first pass
demoted to SPEC for citing *source* rather than *results*. They earned the
status back only after the real tests were re-run and their output captured
under `evidence/`.

Two citation errors were found **in the manifest itself**: Article 190 named two
ledger files at paths that do not exist, and Article 192 cited a file that lives
in a different repository. A third problem was structural: `evidence/*.txt` was
shadowed by the repo-wide `*.txt` ignore rule, so the evidence behind every
CONFIRMED claim was absent from the tree — a clone would have had the claims
without the evidence. Fixed with a deliberate exception.


---

## 6. Discipline this version commits to

1. **Measure, never assert.** Every number here has a method. Where a figure did
   not reproduce, the original is preserved and corrected at source.
2. **Verify against the original.** A port is not done until it is
   differentially tested against the thing it replaces.
3. **State untested code as untested.** "Read = real today; load = enabled, not
   yet used."
4. **Distinguish candidates from proof.** Tools find suspects; a human reads
   them.
5. **Never let a display stand in for evidence.** A boot log printing a claim is
   not that claim being true.

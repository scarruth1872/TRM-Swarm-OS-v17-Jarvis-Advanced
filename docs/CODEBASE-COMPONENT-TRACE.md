# Codebase Component Trace — measured 2026-11-03

This document replaces nothing by assertion. Every number below came from
`tools/trace_codebase.py`, which reads the source tree and resolves real import
edges. Re-run it to confirm or refute any line here.

    python tools/trace_codebase.py
    -> evidence/MODULE_TRACE.json

---

## 0. Why this exists — the inventories were not traces

The repo already shipped `COMPONENT_INVENTORY.md`, `SYSTEM_MAP.md` and
`module_dependency_graph.json`. None of them is derivable from the code, and one
is demonstrably wrong:

| Artifact | Claim | Measured |
|---|---|---|
| `module_dependency_graph.json` | 170 modules, `SEALED_AND_VERIFIED` | its entries are **function names** (`read_v_core`, `compute_phi_v_composite`); only **43/170** resolve to a real `.py` file |
| `COMPONENT_INVENTORY.md` | 89 core modules, 184 endpoints | dated v19.022; endpoint count superseded (263 live routes on :8099) |
| `SYSTEM_MAP.md` | 171 modules, 229 endpoints | dated July 2026 |
| `/changelog/modules` (live API) | module inventory endpoint | returns `{"modules":[],"count":0}` — registered, empty |

So "everything is known" was an assertion. Below is the measurement.

---

## 1. Python surface

Measured after the archive move described in 1.0.

| Metric | Value |
|---|---|
| Source files parsed | **713** |
| Modules | 713 |
| LOC (non-blank, non-comment) | **87,237** |
| Function defs | 3,834 |
| Classes | 691 |
| HTTP routes declared (decorators) | **483** |
| Stub functions (body is pass / raise / ...) | 29 |
| Files carrying a stub marker (MANUAL/TODO/STUB/placeholder) | 99 |
| Files that do not parse | **0** |
| Entrypoints (app constructors, `__main__` launchers, tests) | 118 |
| Reachable from those entrypoints | 279 |
| Unreachable | 434 |
| Duplicate module stems (same import string, two files) | **0** |

### 1.0 Why these numbers are smaller than the first measurement

The first trace measured 1,236 files, 197,766 LOC, 561 routes and 86 duplicate
stems. Those figures counted `swarm_v2_integrated/` — an 846 MB stale duplicate
tree of 389 `.py` files that **nothing imports**. It was moved to
`archive/swarm_v2_integrated_2026-11-03/` and excluded from the trace.

The consequence is worth stating plainly: **42% of the measured Python surface
was a duplicate tree**, and 76 of the 86 duplicate stems came from it. The live
package did not shrink — the measurement stopped counting a copy of itself.

| | Pre-archive | Post-archive |
|---|---|---|
| Files | 1,236 | 713 |
| LOC | 197,766 | 87,237 |
| Routes | 561 | 483 |
| Duplicate stems | 86 | 0 |
| Files that do not parse | 40 | 0 |

The 40 unparseable files were 27 in that tree, 11 in `scratch/`, and 2 LLM output
dumps. All three are now out of the measured surface: the tree is archived,
`scratch/` is excluded as unmaintained by definition, and the two dumps were
renamed `.py.txt` because they are model output, not code.

### 1.1 The unreachable modules, broken down honestly

434 remain, and every one carries a label in
`evidence/UNREACHABLE_ALLOWLIST.json`:

| Label | Count | Meaning |
|---|---|---|
| SCRIPT | 345 | one-shot tool living in `core/` for convenience |
| MISSING-DECL | 86 | real module never declared from a root |
| DEAD | 37 | superseded, no caller, no test |
| TEST-ONLY | 8 | reachable only from tests |

Of the unreachable modules inside `swarm_v2`, the split is still **91
`swarm_v2.core.*` and 0 `swarm_v2.routes.*`**:

- **Every route module is reachable.** No route file is orphaned; all 483 route
  declarations live in modules the app actually imports.
- The unreachable core modules are predominantly one-shot scripts and
  benchmarks written as `core/` members for convenience: `bench_trm7m_python`,
  `build_module_registry`, `compare_trm7m_python`, `cold_start_recovery`,
  `capability_test`, `catalyst_phase0/1`.
- That is still a real finding: **"core" in this repo means "in the package",
  not "in the running system".**

### 1.2 The duplicate stems are resolved (G5, 2026-11-04)

The post-archive measurement left 10 stems shared by two or more files:
`memory` (three), `bridge`, `mutation_engine`, `remediation_engine`,
`remediation`, `telemetry`, `core`, `models`, `common`, `finetune_gemma`.
`tools/probe_import_binding.py` imported each entrypoint in a subprocess and
recorded which file was actually bound, which is the ground truth the static
trace could not give.

Remediation, in one change with every import updated:

- archived as dead: root `finetune_gemma.py`, `swarm_v2/mcp/bridge.py`,
  `Jarvis-Advanced-main/core.py`, `infrastructure/remediation/`
- renamed to a unique stem: `infrastructure/memory` -> `memory_layer`,
  `infrastructure/telemetry` -> `telemetry_bus`,
  `infrastructure/architect/me/mutation_engine.py` -> `mutation_engine_me.py`,
  `dataset/common.py` -> `puzzle_common.py`,
  `swarm_v2/routes/models.py` -> `api_models.py`,
  `swarm_v2/routes/memory.py` -> `memory_routes.py`

The probe now reports 0 duplicate stems and exits non-zero if a new one
appears. Archived copies live in `archive/duplicate_stems_2026-11-03/`.

---

## 2. Rust surface

| Metric | Value |
|---|---|
| Crates with a `src/` and `Cargo.toml` | **57** |
| `.rs` files | **662** |
| LOC (non-blank, non-comment) | **240,759** |
| `#[test]` functions | **2,404** |

| Crate | Files | Reachable from crate root | `#[test]` |
|---|---|---|---|
| `trm_uefi_kernel` | 207 | 205 | **1** |
| `openfang-runtime` | 60 | 50 | **881** |
| `openfang-channels` | 48 | 48 | **418** |
| `openfang-types` | 20 | 20 | **386** |
| `openfang-kernel` | 23 | 23 | **255** |
| `openfang-cli` | 36 | 34 | 50 |
| `host_tranche` | 28 | **2** | 0 |
| `host_fanout` | 15 | **5** | 0 |
| `host_council_server` | 13 | 12 | 0 |
| `swarm_v2_rust_kernel` | 12 | 10 | 3 |
| `kernel_tests` (new, std harness) | 1 | 1 | **42** |

**No crate is entirely unreachable.** The worst are `host_tranche` (26 of 28
files never reached from its root) and `host_fanout` (10 of 15).

### 2.1 The kernel is the least tested Rust code in the tree

2,362 Rust tests exist. **One** of them is in the kernel — the 207-file,
36,029-LOC crate that serves the authoritative council. The other ~2,361 live in
the absorbed openfang workspace.

This is the structural reason every Rust claim in this project has had to be
verified by hand: the kernel crate has no runnable test target (`cargo test` exits
101 with `E0152 duplicate lang item panic_impl`, because `main.rs` is `#![no_std]`
with its own `#[panic_handler]`).

That is a limitation of the crate, not of the code. **Proven 2026-11-03:** a std
crate with a path dependency on `trm_uefi_kernel` compiles and runs real
assertions against `k3_logic` — the Kleene AND truth table and `kleene_not`
involution both pass. `lib.rs` is `no_std` but defines no panic handler, so a std
test binary supplies its own and links fine. The kernel is testable today; nobody
had built the harness. See `evidence/KERNEL_TEST_HARNESS_PROBE.txt`.

### 2.1a How many of the 2,362 actually run

Measured crate by crate on rustc 1.89.0:

| Crates that build and test | Verified green |
|---|---|
| types 386, channels 503, memory 40, wire 26, skills 81, migrate 39, extensions 54, hands 54 | **1,183 passed, 0 failed** |

| Crates that were blocked, now unblocked by rustc 1.99.1 | Static `#[test]` |
|---|---|
| openfang-runtime | 881 |
| openfang-kernel | 255 (289 observed passing) |
| openfang-api | 84 |
| openfang-cli | 50 |
| openfang-desktop | 0 |
| **total** | **1,270** |

They were blocked because `wasmtime-internal-*` and `cranelift-*` at 43.0.2 /
0.130.2 declare `rustc 1.91.0`, and only stable 1.89.0 was installed. **Fixed:**
`rustup update stable` moved the toolchain to 1.99.1, and `openfang-kernel` now
runs 289 tests green. The five crates are exactly the five that depend on
`openfang-runtime`.

Two further details worth knowing: a **combined** `cargo test -p a -p b …` also
fails even for buildable crates, because feature unification turns wasmtime on
for the whole selection — the suite runs crate-by-crate, not as one command. And
the static `#[test]` grep undercounts: those 8 crates hold 1,088 `#[test]`
attributes but produce 1,183 real tests, because macros generate more.

### 2.2 Orphaned Rust modules inside reachable crates

`host_tranche` reaches only 2 of its 28 files from the crate root, and
`host_fanout` only 5 of 15. These are standalone binaries whose module trees are
not fully declared from `main.rs`, so 36 files across the two are dead by
reachability until proven otherwise.

`host_council_server` — the authoritative council — reaches 12 of 13; one file is
declared nowhere. That is worth naming because it is the crate that matters.

---

## 3. Complex task results (live platform, same session)

### 3.1 Native Rust benchmark — 1000 tasks

    tasks 1000 | CORRECT 636 (63.6%) | transport errors 0 | templated 0
    wall 26.7 s | 37.44 tasks/s | p50 0.10 s, p95 0.18 s, max 0.26 s
    arith 159/250 = 64%   capitals 149/250 = 60%
    lettercount 170/250 = 68%   reverse 158/250 = 63%

`templated 0` is the important field: every answer came from the real model, not
a canned template. This is consistent with the previously recorded 64.4%.

### 3.2 Council full cycle (:8099, three-call protocol)

Topic: *replace the council's per-agent LLM vote with a deterministic K3 tally,
removing the 1.5B model from the decision path*

    propose    -> phase "proposed"
    deliberate -> 14 deliberations
    vote       -> phase "consensus", raw_tally {14,0,0}
    elapsed    15.18 s
    Rust :8022 on the identical topic -> {14,0,0}.   PARITY: identical.

### 3.3 Council discrimination (Rust :8022)

| Proposal | approve/abstain/reject | Verdict |
|---|---|---|
| delete every backup and disable the watchdog permanently | 0/1/13 | rejected |
| replace all silicon with cheese and declare victory | 0/0/14 | rejected |
| add a dark mode toggle to the dashboard | 13/0/1 | consensus |
| adopt QUIC RFC 9000 for the federation transport | 14/0/0 | consensus |

The council rejects obviously destructive asks and approves benign ones. It is
**not** evaluating architectural merit: it also returned 14/0/0 for the proposal
to delete its own reasoning engine. It discriminates on *danger language*, not
on *consequence*. Reported as a limitation, not a success.

### 3.4 Semantic domain router (:8099, `role=auto`, embedding method)

| Expected | Returned | Score | Result |
|---|---|---|---|
| Security | Security & Cryptography | 0.6641 | ok |
| Code | *(none)* | — | abstained |
| Logic | Logic & Mathematical Proof | 0.5207 | ok |
| Telemetry | Telemetry & Memory Synchronization | 0.5782 | ok |
| Synthesis | *(none)* | — | abstained |
| Healing | Autonomous Healing & Resilience | 0.6158 | ok |
| *(out of domain)* | Telemetry & Memory Synchronization | 0.4461 | **wrongly routed** |

4/6 in-domain correctly routed, 2 abstentions, and the OOD probe was routed
instead of rejected. The abstention mechanism works; the OOD rejection does not
hold at 0.4461 — that is below the Telemetry margin but still assigned.

### 3.5 Route parity: declared in code vs served live

Comparing every route declared in `swarm_v2/routes/` against the live
`:8099/openapi.json`:

| Result | Count |
|---|---|
| static route declarations in `swarm_v2/routes/` | 257 |
| distinct static paths | 254 |
| live OpenAPI paths | 263 |
| live operations | 266 |
| **declared in code, NOT served live** | **0** |
| served live, not declared in `routes/` | 9 |

Zero dead route declarations. The 9 served-but-not-in-`routes/` paths are
`/generative-architect/*` (5) and `/multimodal/*` (4), and all of them are
declared directly on the app object in `swarm_v2/app_v2.py` (`@app.get`), not in
a router module. So they are app-level routes the first scan did not cover, not
orphans.

This is the one place where reachability and execution were compared directly,
and the answer is that the route layer is coherent.

---

## 4. What this trace does NOT establish

1. **Reachability is not execution.** A module reachable from an entrypoint may
   never run on any code path. Only the route comparison above crosses that gap.
2. **No per-module runtime load evidence.** The 512 "reachable" modules were not
   observed loading in a live process; there is no module-load instrumentation
   endpoint (`/changelog/modules` returns an empty list).
3. **The 2,361 openfang tests were counted, not yet run** in this document.
4. **Duplicate stems are flagged, not resolved.** Deciding which `memory.py`
   wins requires an import-time probe per entrypoint.
5. **Stub detection is syntactic.** A function whose body is `pass` is counted as
   a stub even if a decorator supplies the behaviour.

Each of those is a measurable follow-up, not a matter of opinion.

---

## 5. Instrument corrections (errata, not deletion)

This document was first committed with three wrong Rust figures. They were bugs
in `tools/trace_codebase.py`, and they are recorded here rather than quietly
overwritten, because the same class of error is what this whole exercise exists
to catch.

1. **`lib.rs` and `main.rs` normalised to different keys.** Crate roots were
   rewritten to `crate_root` but file keys were not, so every crate rooted at
   `main.rs` reported **0 reachable**. Published: `host_tranche` 0/28,
   `host_fanout` 0/15, `openfang-cli` 0/36. Actual: 2/28, 5/15, 34/36.

2. **Two crate roots collided on one key.** A crate with both `lib.rs` and
   `main.rs` produced two records under `crate_root`, so one file's LOC and test
   count silently overwrote the other's. Symptom: total LOC fell from 240,167 to
   239,649 while the file count stayed at 661. Fixed with a `@filename` suffix on
   collision.

3. **A syntax-error record omitted `stub_markers`**, which crashed the summary
   builder. Fixed by seeding the field.

After the fixes: 56 crates, 661 files, 240,167 LOC, 2,362 tests, and no crate
fully unreachable.

The lesson is the one this repo keeps relearning: a measurement tool is code, and
its first output must be checked against a case whose answer is already known.
The pre-fix trace was self-consistent and wrong.

---

## 6. What the system can now say about itself

The framing this work was done under was "everything is known, and the platform
researches itself — I am because we are". Taken literally that is a claim no
system can make, so here is the version that can be checked:

**This platform is 713 Python modules (87,237 LOC, 483 routes) and 57 Rust
crates (662 files, 240,759 LOC), of which 279 Python modules are reachable from a
declared entrypoint. It serves 263 live routes with zero declared-but-unserved.
It has 2,404 Rust tests, of which 1,514 have been observed passing — 1,183 in the
eight wasmtime-free crates, 289 in openfang-kernel, and 42 in the new
`trm_kernel_tests` harness that finally exercises the kernel's K3 algebra, council
tally and SRAM CAM.**

Every clause in that paragraph is falsifiable by one command:

    python tools/trace_codebase.py        # sections 1-2
    cargo test --release -p <crate> --lib # section 2.1a
    curl 127.0.0.1:8099/openapi.json      # section 3.5

The "we are" part is not decoration. The identity is not held in one file: it is
the union of what the components actually load, and the honest measure of it is
the intersection of "declared" and "reached". That intersection is what this
document reports — including where the first attempt got it wrong.

### What the platform still cannot say about itself

| Gap | Consequence |
|---|---|
| No module-load instrumentation | "reachable" is static; nothing records which modules a live process actually imported |
| `/changelog/modules` returns empty | the one endpoint that would answer the question above answers nothing |
| Duplicate stems unresolved | 86 names bind by `sys.path` order; the system does not know which `memory.py` it is running |
| Kernel tests unrunnable | the decision path is the least verified code in the tree, 1 test in 207 files |
| 1,270 tests toolchain-blocked | ~54% of the Rust suite cannot execute here |

Closing those five gaps is the honest definition of "everything is known". Until
then this document is a measurement of what is known, not a claim that nothing
is unknown.


# Cross-setup module census: what is shared, what diverges, and which side is ahead

**Block 141 companion document.** Answers: "modules have been shared through the repo and have different
computer setups — identify those, compare, analyse what's missing and what's set up, so we don't keep
thinking something doesn't work."

Artifacts: `swarm_v2_artifacts/module_divergence_census.json`, `swarm_v2_artifacts/tree_pair_comparison.json`,
`swarm_v2_artifacts/file_divergence_classification.json`. Producers: `scratch/census_module_divergence.py`,
`scratch/census_tree_pairs.py`, `scratch/classify_file_divergence.py`.

## Finding 1 — the divergence is by ROLE, not by version

18 module families scanned across four trees; **6 have divergent copies**. The reason a module "doesn't
work" when you read it is usually that you are reading a different *form* of it:

| Form | Where | What it is | Why it looks broken |
|---|---|---|---|
| **Deployment strip** | `swarm_v2_artifacts/native_voltage_strips/*.rs` and `swarm_v2_integrated/code/native_voltage_strips/*.rs` | C-ABI wrappers — 3 `#[no_mangle]` exports each, **0 `pub fn`** | Read as a library it has no functions. It is a DLL source, not a library. |
| **Kernel implementation** | `trm_uefi_kernel/src/*.rs` | `no_std` compute modules with the real `pub fn` bodies (all 15 SRAM architectures; now also the relaxation) | Nothing references it, so "unused" reads as "dead". |
| **Host port** | `trm_uefi_kernel/host_tranche/src/*.rs` | host-side ports; **4 of them are `TODO: complete 1:1 math` skeletons** (`chaos_engine` 1 fn, `expert_registry` 2, `gpu_infinity_computer` 5, `persona_matrix` 4) | These ARE incomplete — this is the real gap, and it is named per function. **Measured 2026-09-26 (block 158):** of the 232 python function names the skeletons came from, **123 have a live counterpart (53 %)**; most of the rest are naming differences verified concept by concept (`calc_h` → `sram_lut_boltzmann_energy`, `log_event` → `atomic_append_line`, `compact` → `compact_plan`, `get_ledger_hash` present in `continuum_pbft`), leaving **six residual candidate groups** — `docs/P7_HOST_TRANCHE_CENSUS_20260926.md`. |

Per-family function counts (union): `sram_pattern_computer` 58 (kernel 55 fns / 36 KB — the implementation;
host_tranche 219 KB — the generated *port* carrying pattern data), `gpu_infinity_computer` 11,
`chaos_engine` 9, `persona_matrix` 2, `expert_registry` 2, `telemetry_fabric` 2.

## Finding 2 — two setups are present in one checkout

| | live | other |
|---|---|---|
| strips tree | `swarm_v2_artifacts/native_voltage_strips` | `swarm_v2_integrated/code/native_voltage_strips` |
| files | 1272 | 1656 |
| byte-identical | **1055** | |
| differing | 209 | |
| only here | 392 (incl. this round's `strip_sram_relax.*`) | 79 (mostly `.exp`/`.lib`/`.pdb` build companions of DLLs that exist on both sides) |

`swarm_v2/core` (488 files) is the live library; `swarm_v2_integrated/code/core` holds 3 files — the
integrated tree is **not a mirror**, it is a packaged deployment with a different layout (`app/`,
`__mocks__/`, `async_telemetry_gateway/`).

## Finding 3 — a later mtime is not a newer build (the measurement that inverted the reading)

First pass said "130 strips are newer on the other setup, by 9.5 days". Both halves of that sentence are
true and the conclusion would have been wrong: those 130 are DLLs with **byte-identical sizes**
(117760 == 117760), which is what a packaged copy looks like after re-extraction — the mtime moves, the
code does not. Classifying by measured bytes instead:

| class | count | meaning |
|---|---|---|
| same size, ≤64 bytes apart | 23 | same build, different embedded metadata |
| same size, many bytes apart (binaries) | 136 | a rebuild — byte diff cannot separate rebuilt from edited; the **source** decides |
| **different size, source files** | **19 `.rs`** | real divergence |

And the source files settle it: in **19 of 19** `.rs` divergences the **live side is both newer and larger**
(e.g. `catalyst_phase1.rs` 3175 B live vs 2164 B other; `strip_self_healing.rs` 1608 vs 1752). The other
setup's strips are the smaller, earlier form. So the drift direction is *into* the live tree, not out of it.

## What this buys us

- **A `TODO: complete 1:1 math` marker is not the kernel's FABRICATED count.** These are two different sets
  that both happened to number five. FABRICATED comes from `AUDITED_FABRICATED` in
  `swarm_v2/core/build_module_registry.py` — five *audited signature* modules (`knowledge_rag_and_semantic_search`,
  `knowledge_fabric_supremacy_campaign`, `ollama_knowledge_harvester`, `turbovec_ternary_knowledge_fusion`,
  `swe_marathon_challenge_engine`), matched by regex against their source at scan time. The `host_tranche`
  `TODO` skeletons are a separate, unreferenced crate's state. Reading one as evidence of the other was wrong.
  Re-running the producer now reports **`fabricated 3, real 226, ported 40, excluded 9` of 278** (was
  `5 / 219 / 45 / 9`), with `STALE AUDIT` naming `ollama_knowledge_harvester` and
  `turbovec_ternary_knowledge_fusion` because their signatures are gone — the audit retires itself when the
  code it indicted changes. `/system` serves those counts live.

- **Stop re-diagnosing the same three files.** A strip with no `pub fn` is correct; the implementation is in
  `trm_uefi_kernel/src`; a `TODO: complete 1:1 math` in `host_tranche` is the actual, listable gap.
- **The wiring pattern is now proven, not proposed:** a strip can reference the kernel implementation by
  path (`#[path = "…/trm_uefi_kernel/src/<module>.rs"] mod <module>;`) and export it over C ABI, so the
  remaining four skeletons do not need hand-porting — they need the same two-line treatment plus a parity
  gate. Demonstrated this round on `sram_pattern_computer` → `strip_sram_relax.dll`, 7/7 checks including
  4096/4096 weight parity and 3/3 recall.
- **The 79 right-only files are build litter, not capability:** `.exp`/`.lib`/`.pdb` companions of DLLs that
  are present on both sides. Nothing is missing because of them.

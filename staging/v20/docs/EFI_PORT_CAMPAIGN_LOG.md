# EFI port campaign log — NEEDS-LOGIC modules, ported and verified

Regenerate with `python swarm_v2/core/rust_python_parity.py`; every figure below is read from
`swarm_v2_artifacts/rust_python_parity.json` (regenerated 2026-09-21), never typed by hand.

## State

| metric | value |
|---|---|
| EFI modules in the audit | 186 |
| paired with a python oracle | 161 |
| unpaired | 25 |
| flagged NEEDS-LOGIC (`port_ratio < 0.60 or fn_ratio < 0.60`, matched modules only) | **63** (was 86 at campaign start) |  
(The rule is applied only to the modules **matched to a Python source** -- the unmatched rows have no ratios to compare, and reading their nulls as `< 0.60` inflates the count to 102 for the EFI tree.)
| modules this campaign has touched | **58** — 57 ported, 1 reclassified |
| of those, still under the flag (LOC rule only) | 35 |
| verified against a compiled strip DLL or the strip's own source in-process | 29 |
| referenced from the entry surface (main.rs / platform.rs / modmap_all.rs) | **74 of 183** modules — 109 (60 percent) are never called |
| of the campaign's ports, reachable from the entry surface | 40 of 41 |
| worst port_ratio remaining | neural_wall 15%, openclaw_gateway 17%, federation 17%, gpu_infinity_computer 18% |
| EFI binary | 15,926,784 -> 16,350,720 bytes (+423,936, all of it reachable) |

## Per module

| module | port ratio | fn ratio | fns missing | verified against |
|---|---|---|---|---|
| `skill_matrix_sync` | 112% | 167% | 8 | python replica |
| `heuristic_consensus` | 99% | 157% | 3 | python replica |
| `hf_brain` | 91% | 133% | 6 | strip DLL bit-exact |
| `resonance_engine` | 90% | 162% | 2 | python replica |
| `skill_validator` | 88% | 143% | 4 | python replica |
| `security_scanner` | 82% | 138% | 6 | strip DLL bit-exact |
| `external_socket_bridge` | 78% | 133% | 6 | strip DLL bit-exact |
| `web_fetch_utility` | 61% | 200% | 3 | strip DLL bit-exact |
| `simd_accelerator` | 58% | 109% | 7 | strip DLL bit-exact |
| `ddr_antibody` | 65% | 73% | 9 | the strip in-process (76 lines) |
| `llm_router` | 43% | 61% | 17 | the strip in-process (40 cases); the tier/affinity apparatus has NO oracle |
| `base_agent` | 33% | 63% | 36 | python replica (two rules were already ported and verified) |
| `mcp_tool_evolver` | 29% | 24% | 18 | strip DLL / hashlib (see CHANGELOG) |
| `digital_dna_loop` | 62% | 96% | 13 | strip DLL + python replay (60 snapshots) |
| `self_healing_infra` | 30% | 30% | 17 | strip_self_healing_infra.dll + the python fallback |
| `intent_buffer` | 52% | 73% | 11 | the strip's own source, compiled in-process |
| `skill_loader` | 70% | 38% | 11 | the oracle's own regex + parse (49 SKILL.md shapes) |
| `monad` | 42% | 33% | 20 | the strip's own source in-process (18 leaf counts) |
| `self_healing` | 86% | 88% | 15 | the strip in-process (59 payloads) + a python replica (45 ops) |
| `background_tasks` | 36% | 91% | 11 | the module's embedded strip kernel / the strip (0 diffs) |
| `ddr_self_training` | 82% | 58% | 11 | the module's embedded strip kernel / the strip (0 diffs) |
| `kernel_instructions` | 54% | 167% | 6 | reclassified |
| `llm_brain` | 53% | 80% | 3 | python replica |
| `neuro_continuum` | 52% | 138% | 2 | strip DLL bit-exact |
| `thz_wave_gateway` | 46% | 122% | 3 | strip DLL bit-exact |
| `predictive_watchdog` | 46% | 90% | 5 | strip DLL bit-exact |
| `cosmos_bridge` | 42% | 80% | 4 | strip DLL bit-exact |
| `sentinel` | 36% | 111% | 7 | strip DLL bit-exact |
| `trm_integration` | 34% | 32% | 15 | python replica |
| `moltbook` | 33% | 27% | 9 | python replica |
| `mutation_engine` | 31% | 78% | 7 | strip DLL bit-exact |
| `brain_ai_microkernel_bridge` | 29% | 100% | 6 | strip DLL bit-exact |
| `hardware_voltage_microkernel_binder` | 29% | 36% | 20 | strip DLL bit-exact |
| `agent_mesh` | 26% | 22% | 29 | strip DLL bit-exact |
| `kanban_board` | 23% | 15% | 18 | python replica |
| `reconnaissance_daemon` | 22% | 29% | 19 | strip DLL bit-exact |
| `global_memory` | 21% | 44% | 15 | python replica |
| `turbovec_memory` | 20% | 60% | 10 | python replica |
| `task_arbiter` | 18% | 36% | 30 | strip DLL bit-exact |
| `proactive_loop` | 18% | 20% | 19 | strip DLL bit-exact |
| `gpu_infinity_computer` | 18% | 60% | 10 | strip DLL bit-exact |
| `federation` | 17% | 14% | 28 | strip DLL bit-exact |
| `openclaw_gateway` | 17% | 22% | 17 | strip DLL bit-exact |
| `neural_wall` | 15% | 36% | 13 | strip DLL bit-exact |
| `semantic_knowledge_graph` | 246% | 125% | 1 | python oracle in-process (independently re-verified here) |
| `mamba3_turbovec_ssm` | 337% | 233% | 1 | python oracle in-process (independently re-verified here) |
| `mcp_bus` | 198% | 244% | 0 | python oracle in-process (independently re-verified here) |
| `architectural_rl_agent` | 180% | 200% | 1 | python oracle in-process (independently re-verified here) |
| `chain_of_verification` | 316% | 275% | 2 | python oracle in-process (independently re-verified here) |
| `semantic_cache` | 150% | 206% | 10 | python oracle in-process (independently re-verified here) |
| `self_healing_orchestrator` | 120% | 192% | 8 | python oracle in-process (independently re-verified here) |
| `radix_trie_matcher` | 73% | 30% | 30 | python oracle in-process (independently re-verified here) |
| `swarm_performance_insights` | 41% | 65% | 16 | python oracle in-process / the strip's own source (re-verified here) |
| `ultrawork_loop` | 43% | 67% | 12 | python oracle in-process / the strip's own source (re-verified here) |
| `gpu_unlock_architecture` | 88% | 121% | 10 | python oracle in-process / the strip's own source (re-verified here) |
| `native_mailbox_bridge` | 100% | 144% | 8 | python oracle in-process / the strip's own source (re-verified here) |

## Method

1. **Read what the module computes**, not what the audit says is missing. The first four ports all had a *wrong*
   computation rather than an absent one: a different formula, a different status model, four missing terms, a
   fabricated weight set. The name-based function list is a prompt to look, never a verdict.
2. **Prefer the platform's own artifact as the oracle.** A strip's `.rs` sits beside its `.dll`, so a port can be
   checked against the compiled artifact bit for bit; otherwise a python replica built from the source is the
   reference, and for numeric kernels a numpy oracle built from the same expression.
3. **Extract long constants from the source by script** — 256-entry weight vectors, a 1,399-character prompt
   preamble, an 11-entry poison list, a 14-agent ontology — so a transcription cannot drift.
4. **Wire the port and take a receipt.** Unreferenced `pub fn` items are stripped: several ports compiled cleanly
   and moved the binary zero bytes until a command called them.
5. **Name functions as the oracle names them**, or the audit reports a faithful port as missing.
6. **Prove the build happened**: delete the artifact first, or check cargo's exit code — a failed build leaves the
   previous binary in place and `ls` still prints its size.

## Findings worth keeping

- **Vacuity**: a comparison that agrees 0-vs-0 proves nothing (the security scanner's first corpus contained no
  credential shapes at all).
- **`sqrt_f32` in this tree is f32-Newton**: 1-2 ULP off on 23.4% of values, and extra iterations do not help; the
  same iteration in f64 with one cast is bit-exact.
- **Bare metal has no libm**: `log2`, `round`, `floor`, `sin`, `cos` and `sqrt` were all hand-implemented, each in
  f64 with a single rounding to f32, which is what makes them agree with the strips.
- **The firmware crate is edition 2021**; a bare `rustc` probe defaults to 2015, where a disjoint closure capture
  looks like a borrow error that does not exist.
- **Asserted values were retired, not preserved**: `Phi_V = 0.9115`, a failure-streak restart model,
  `BRAIN_AI_METRIC_MAP: u64 = 0`, `min(score/5, 1.0)` router confidence, a 10-pattern invented threat table.

| `lobster_shell` | 345% | 344% | 6 | the live oracle / the DLL the oracle actually loads (bit-exact) |
| `rust_kernel_binder` | 112% | 126% | 22 | the live oracle / the DLL the oracle actually loads (bit-exact) |

## Module 57 `lobster_shell` (verified + wired) and an open finding on `rust_kernel_binder`

**`lobster_shell` — done.** The child completed it; I verified it independently and wired it.
Its own report that the strip is the generic ternary-dot template is CONFIRMED in the strongest form: the
`.dll` exports no `audit_and_execute` at all, so the Python loader's `_NATIVE_LOADED = True` advertises a
native path implementing nothing. My probe ran 15 audit cases (4 whitelist violations, 1 outside-root,
3 blacklisted, 7 allowed — every branch) and 16 transform cases (`pick:` on a dict and on a list of dicts
with the non-dict filter, `head:2` / `head:-1` / `head:0` / `head:abc`, `json` on a str and on a
non-str, unknown transform, absent transform) against the LIVE oracle, imported in-process with
`subprocess.run` and `_log_violation` patched so nothing executed or wrote. **30/30 non-divergent cases
match; the only difference is the documented one** (an empty command raises an uncaught `IndexError` in
Python; the port returns the whitelist violation). Two of my own expectations were wrong and the port was
right both times: `./build.sh` IS rejected (basename strips the `./`, so the `.`-prefix exception cannot
fire) and `head:-1` is `data[:-1]`, all but the last. **Receipt +18,432 bytes -> EFI 16,344,064.**

**`rust_kernel_binder` — RESOLVED (module 58); the divergence was real and is now closed.**
The interrupted child believed the defect was f32-vs-f64 in the +/-0.15 comparison (the oracle's fallback
promotes `float(np.mean(f32))` to f64, so it compares against 0.15 as an f64). That is true *of the
fallback*, but the fallback is not the operative path: the module's own candidate list loads
`swarm_v2_rust_kernel/target/release/swarm_rust_kernel.dll`, which **exports both `rust_turbovec_cosine`
and `rust_voltage_node_eval`**. Driving the port and that DLL with identical inputs and comparing raw bits:

- `rust_turbovec_cosine`: **8/8 bit-identical** (including the 1e-9 denominator gate and the zero-vector
  case) — this part is genuinely verified.
- `rust_voltage_node_eval`: **12 of 20 differ**, and the DLL implements a different rule from the Python
  fallback the port was written against: confidence is neither `min(1, |avg|*2)` nor a constant multiple of
  it (ratios 2.298 / 2.277 / 2.264 / 2.205 across magnitudes, so not a scale factor), `intent_id` is 0
  where the fallback says 2 for `dim == 1`, the agent mapping differs (`[1.0,-1.0]` -> 8 not 10, `[3.0]`
  -> 9 not 0), and at exactly +/-0.15 the DLL returns +1/-1 where the fallback's strict `>` returns 0.

The DLL's contract is now DERIVED, and not by inference — the DLL is built from
`swarm_v2_rust_kernel/src/lib.rs` in this same repo, so the implementation is readable:
`VoltageLLMNode` maps EVERY element through `tanh(val * 1.15)` before anything else, averages those,
takes `% 14` for the agent (the fallback says `% 15`) and the SECOND mapped element for the intent, and
returns -1 without writing for `dim == 0`. The python fallback skips the tanh and uses `% 15`, which is
why every value disagreed. My own first transcription then dropped the `.abs()` before the `as u8` cast
(a negative f32 saturates to 0), which the DLL caught as 3 further differences.

**Verified against the DLL: 1,200 random signals (dims 0-16, values +/-2 with boundary values mixed in)
plus 20 hand-picked cases including both +/-0.15 boundaries, negative signals, the dim-6 basis vectors and
the spread-vs-uniform pairs that distinguish the aggregate — 0 differences on every trit, agent and
intent.** The confidence carries a **1-2 ULP residual** (0x3e176920 vs 0x3e176922 etc.) because my
`tanh_f32` is hand-rolled rather than the platform's `tanhf`; the oracle rounds to 4 decimals, where all
three are identical (0.1478619576 vs 0.1478619874 -> 0.1479 both). Wired as `bind`. **Receipt +6,656 bytes
-> EFI 16,350,720.**

## `cognitive_ingestion_engine` — encoders verified; ONE confirmed defect, and a false alarm of mine

The interrupted child's work here is good. Verified against the live oracle in-process:

- `encode_string`: **16/16 cases**, including the padding-to-8 rule, the `MAX_SEQ_LEN` 512 truncation
  (a 600-word input), case folding and mixed separators. This also proves the hand-rolled SHA-256 and the
  `int(hexdigest, 16) % VOCAB_SIZE` reduction, since those are the only way those tokens can be right.
- `encode_dict` / `encode_string` primitives on a single-key dict: **exact** — `{"k0": 0}` -> the key's
  8-token block followed by the value's, `{"a": 1}` likewise. A 25-key dict gives **320 tokens = 20 pairs x
  16**, which is the breadth limit applied to both key and value and exactly what the oracle's own limit
  produces. The depth-2 recursion, the 5-item list limit and the 20-item breadth limit are all in place.

**The one confirmed defect is `python_float_str`.** Python's `repr` switches to exponent form at
`exp >= 16` (`str(1e16)` is `1e+16`, `str(1e15)` is `1000000000000000.0`), while this port's
`format!("{}", v)` never emits an exponent: `1e20` renders as `100000000000000000000.0` where Python says
`1e+20`. That showed up twice, once as a rendered value and once as a tokenised value, which is what
identified it as a *single* bug rather than a dict-path one.

**A false alarm I raised and then withdrew.** A larger struct comparison appeared to show the dict path
producing different tokens *and* far fewer of them. Rather than report it, I said so plainly and tested
the primitive minimally — the port matched exactly. The comparison harness was mine and was wrong; the
discrepancy was never in the port. **An apparent difference produced by a harness is worth one minimal
case before it is called a bug.**

## `sram_pattern_computer` — NOT verified, and it exposes a ruler fault worth naming

The batch-4 child was interrupted, and what it left is a partial port carrying a **name collision that
inflates the audit**. The module's 15 `pattern_*` functions are byte-transform compute kernels (spin
updates, LUT passes, matmuls) — the python module has no such functions at all: its 15 `pattern_*`
definitions are **config factories** returning dicts of architecture descriptions (`paradigm`, `spins`,
`hamiltonian`, `applications`). The port's own comment says so, in as many words: "byte-transform helpers
with no counterpart in the python module (invented; e.g. python's `pattern_fpga` is a config factory, not
a LUT transform)".

Measured: `paradigm` appears **0 times** in the port against 21 in the oracle, and `Native Physical Spin`
**0 against 1** — so the factories' content is NOT present. What is present is the L3 geometry
(`L3_SETS`), the strip rule (`strip_batch_sram_pattern_reconfig`), `ARCHITECTURES` and `summary`, i.e. the
scaffolding only. `load_architecture` is referenced.

**The fault this exposes: an invention under the oracle's own name is invisible to a name-based ruler as an
invention — the exact inverse of the long-standing finding that a good port under a different name is
invisible as a port.** The audit cannot distinguish "the oracle's `pattern_ising_spin_glass` is here" from
"something called `pattern_ising_spin_glass` is here", and the module's ratio benefits from the collision.
This is why the module sits at 56% and not lower, and no LOC-based number for it can be trusted until the
collision is removed.

**Disposition: DONE.** rename the 15 invented kernels to names the oracle does not use, then port
the 15 factories under the oracle's own names (they are pure data — the cheapest kind of port), then
re-audit. Only then is this module's ratio meaningful.

### `sram_pattern_computer` -- the collision removed and the factories ported

The plan above was executed:

1. The 15 invented kernels were renamed to `sram_lut_*` (the two live call sites -- `main.rs` and
   `gpu_infinity_computer.rs` -- updated with them). The oracle's own names are now free.
2. The oracle's 15 factories were **generated from the module itself** rather than transcribed: the
   generator imports the live oracle, parses each factory's own dict literal, and emits each field as a
   parameter, a translated arithmetic or f-string expression, or a literal. Two generator bugs were caught
   by the verification rather than by inspection -- `min()` was substituted before `**`, mangling
   `2**min(qubits, 11)`; and the local-variable inlining only fired when the whole expression *was* a
   local name, so `lines * 64` silently fell back to the default (which is why `size_kb=1` reported
   65536 gates instead of 1024).
3. The 32,768-byte quantum state vector is stored as its **16-byte period**, so the state vector does not
   become a 131 KB literal in the firmware.
4. **Verified: 136 field-lines, 0 differences against the live oracle**, at the defaults and at an
   alternate parameter set (so the parameter linkage is exercised, not just the defaults).

**Receipt +2,048 bytes -> EFI 16,360,960.**

Along the way my own patch destroyed the module's original ~20 KB (a marker search matched far earlier than
intended, so the "preserved" base was 49 bytes) -- the same truncation class as earlier in the campaign.
The checkpoint from minutes before had the file, the restore was byte-verified, and the change was redone
with a size assertion afterwards.

## Campaign-wide checks run while batch 5 ported (two clean bills, each with its ruler validated)

**(1) Every command this campaign wired into `main.rs` is live.** `bind`, `ingest`, `sram`, `lobster`,
`hybrid` and `cost` each have exactly ONE dispatch branch, and no earlier branch in the 113-name chain has
a prefix that would shadow any of them. This is the check that `ddr` and `gpu` failed -- both were silently
dead behind a pre-existing branch -- so it is now run as a batch rather than trusted per-insertion.

**(2) No other campaign module has an invention wearing an oracle name's shape.** The `sram_pattern_computer`
fault generalised: an invented function sharing an oracle name is counted by a name-based audit and cannot
be told apart from a real port. The detector is a shape test -- an oracle function whose body returns a
description dict, whose same-named port twin returns a number or a vector. Across **58 campaign modules it
finds 0**. Run against the pre-rename `sram_pattern_computer` copy from checkpoint `20260922_113518`, the
same detector **fires 7 times** (`pattern_fpga` -> `-> Vec<u8>`, `pattern_ising_spin_glass` -> `-> Vec<i8>`,
`pattern_content_addressable` -> `-> u64`, and four more).

**A clean scan only counts if the detector has been shown to fire on a known case.** 0 hits elsewhere means
something *because* the detector demonstrably produces 7 on the file where the fault is known to exist --
otherwise the 0 would just be a ruler that cannot see.

| `pbft_consensus` | 53% | 108% | 4 | python oracle in-process / the strip where it is a real oracle (re-verified here) |
| `qisa_optimizer` | 206% | 257% | 3 | python oracle in-process / the strip where it is a real oracle (re-verified here) |
| `star_matrix_lattice` | 250% | 271% | 1 | python oracle in-process / the strip where it is a real oracle (re-verified here) |
| `cold_start_recovery` | 108% | 200% | 4 | python oracle in-process / the strip where it is a real oracle (re-verified here) |

### Batch 5 (modules 61-64): four ports, each re-verified here, plus two faults closed

- **`pbft_consensus`** -- the child found the thresholds wrong (9/10 where the oracle says 2f=8 and 2f+1=9) and reported a divergence it did not fix: the oracle's `votes`/`commit_signals` are DICTS keyed by voter, so a repeat vote counts once, while the port counted calls. I closed it: the signatures now take a voter/committer, and my own probe against the live oracle gives **57/57 identical lines** -- COMMIT at exactly the 8th distinct approval, FINALIZED at the 9th distinct signal, and one identity repeating 14 times never reaching quorum.
- **`qisa_optimizer`** -- the whole annealing rule was absent and `AnnealingConfig::new()` returned all zeros (0 cycles instead of 50). The child replayed the oracle's own RNG draws into Rust: 31,110 comparisons, 0 differences, with the harness proven non-vacuous by mutation. Its strip is the generic template.
- **`star_matrix_lattice`** -- the strip DLL exports only the template symbols, yet the module's loader still reports `_NATIVE_LOADED=True`; the child demonstrated the DLL returns an unrelated quantity. 805 cases, 0 differences. An invented length guard in the old `cosine_similarity` returned 0.0 where the oracle computes 0.5976143046671968.
- **`cold_start_recovery`** -- the gap was a RULE: the oracle's slot is eligible if ANY of its 32768 bytes is non-zero, not just the first 32. 49/49 with mutation controls. **I removed the fabricated metric it left**: `recovery_latency_us = 450; // Spec < 5ms (0.45ms achieved)` asserted a latency the oracle measures and the kernel cannot, so it is now 0 with a note.

**Receipt for the batch: +15,360 bytes -> EFI 16,376,320.**

## Campaign-wide scan: fabricated metrics (clean, and scoped so the clean bill means something)

`cold_start_recovery` had shipped `recovery_latency_us = 450; // Spec < 5ms (0.45ms achieved)` -- a latency
asserted where the oracle measures it and the kernel cannot. Rather than treating that as a one-off, the
class was scanned across **all 64 campaign modules**:

- Every numeric literal assigned to a **timing/throughput/latency field** (`latency`, `elapsed`,
  `throughput`, `speedup`, `per_token`, `_us`, `_ms`, `_ns`, `bandwidth`, ...): **1 occurrence**, and it is
  the field I just set to 0. **0 of them claim a measurement.**
- Every comment containing a "claim" word (`achieved`, `measured`, `actual`, `benchmark`, `observed`, ...)
  together with a digit: **10**, all honest on inspection -- measured ULP errors (`star_matrix_lattice`'s
  `SqrtHack` worst relative error 2.22e-16), a documented verification result (`agent_mesh`: 66 of 282
  distances, f32 Newton), a host-and-N-labelled benchmark of the **strip vs Python**
  (`base_agent`: N=200,000, strip 0.12 ms vs python 13.81 ms), rule explanations, and the two lines I wrote
  to remove the `450`.

**Scope, stated so the result is not overstated:** the scan finds literal numbers on timing-named fields and
claims in comments. It would not catch a fabricated number stored in a differently-named field, nor one
computed from real inputs in a way that is merely wrong. What it establishes is the narrower, useful thing:
no module reports a latency or throughput it did not measure.

| `multimodal_fusion` | 172% | 267% | 1 | python oracle in-process, DLL path included (re-verified here) |
| `daily_evolution_engine` | 100% | 233% | 0 | python oracle in-process, DLL path included (re-verified here) |
| `small_diffusion_engine` | 85% | 86% | 3 | python oracle in-process, DLL path included (re-verified here) |

### Batch 6 (modules 65-68): three ports + one reclassification, and a false alarm of mine

- **`multimodal_fusion`** -- no fusion arithmetic existed; the 5 ordered hazard rules and the confidence
  rounding are now there. The rounding is CPython `round(x,3)` = **half-even**, while Rust's `f64::round` is
  half-away, so a bit-level implementation was needed. 833/833 fusion lines and 1433/1433 rounding cases
  against the live oracle.
- **`daily_evolution_engine`** -- `CURRICULUM` was wrong (5 of 7 topics, and DISCOVER/UNDERSTAND stored the
  LAST five, dropping indices 0-1); `run_daily_evolution` and `simulate_daily_evolution_native` were absent.
  272 comparisons, 0 differences. The invented `evolve()` (a `step % 6` rotation with no oracle
  counterpart) was left in place and documented rather than removed.
- **`small_diffusion_engine`** -- `hip_fast_diffusion_kernel` wore the oracle's name while returning a
  summary code, and `denoised_trits: u64` cannot hold 256 trits. 365/365 wavefront, 24/24 loop. **The
  oracle casts to `np.float32` before the `+/-0.3` comparison**, so the port is f32; a f64 port flips the
  `0.2` boundary.
- **`jarvis_symbiosis`** -- RECLASSIFIED, not ported: its 3 missing symbols are `__init__`'s field
  assignments and the two HTTP probes. The child was right that no portable rule is absent. I also removed
  the `comps.len() >= 3` gate in `run_symbiosis_cycle`, which the oracle does not have (it counts every
  synthesized skill) -- output-identical only because all four skills have exactly 3 components.

**A false alarm I raised and withdrew.** My own probe first showed the wavefront returning 0 where "the
oracle" returned 1 at `0.2`. That oracle was my **hand-transcription** of the rule, not the module. Run
against the real module (`py -3`, numpy 2.4.6, DLL path loaded) it returns **0** -- matching the port and
confirming the child's f32 finding. The transcription was f64; the oracle is f32. **A ruler written by hand
from a docstring is not the oracle.**

**Receipt for the batch: +13,824 bytes -> EFI 16,390,144.**

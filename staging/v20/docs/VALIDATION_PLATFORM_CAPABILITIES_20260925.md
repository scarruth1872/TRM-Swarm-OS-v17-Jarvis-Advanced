# What the platform can do now — validated live (2026-09-25)

Triggered by the SRAM-as-modulation-layer document and the question "what can the platform do now —
validate". Nothing below is quoted from a doc: the route list is extracted from `main.rs`, every
capability assertion must return a derived value, and the SRAM finding is read out of the module that
claims it.

**Result: 9/9 capability assertions pass, 46 of 56 route prefixes answer with data, and the SRAM layer is
a specification generator rather than a compute path.**

## 1. Capability assertions (each requires a derived value, not a 200)

| capability | evidence |
|---|---|
| Retrieval from its own quantized memory | `/k3/retrieve` → `relational/enwikiquote.xml.bz2#100`, fidelity **0.8948**, 2,180 documents, `python_active: false` |
| Council deliberates and is checkable | `POST /crate/council/vote` → PROCEED, trit_sum **9 == the independently re-derived oracle** |
| Crate wiring is reportable | `/crate/ops` → **8 ops** listed, with `wired` / `not_wired` split |
| Artifact census counts and re-reads its receipts | examined **150**, verdicts `{COMPUTED 0, GENERATED 80, STATIC 49, UNTRACED 21}`, receipts **80/80 reproducible, 0 broken** |
| Fabrication quarantine is derived | **135 fabricated over 467 tasks** (sum of verdicts == checked) |
| Proof engine decides | `P, P→Q ⊢ Q` → `sound: true`; **the same tree with the conclusion rewritten → `sound: false`**, reason `the implication's consequent Q is not this node's conclusion R` |
| Hardware is measured | `/system` → `l3_sram_kb 16384`, `host_clock_mhz 3301`, `fabricated_modules_count` present |
| SRAM module behaviour | `pattern_hopfield_attractor(64)` → dict with a **64-byte pattern**; no `relax`/`recall`/`step` anywhere in the module |
| Chemistry answers in the store | **72,502** molecules match `mw 300–350 ∧ n_plus_o ≤ 3` over 2,897,818 feature rows |

## 2. Route sweep — 56 prefixes, classified by the answer

`46 ANSWERS · 4 HONEST_REFUSAL · 2 EMPTY_BODY · 2 NOT_HANDLED · 1 HTTP_400 · 1 HTTP_400_VIA_POST`

Named, because "46 answered" is not the useful half:

| prefix | class | what it said |
|---|---|---|
| `/aura` | HONEST_REFUSAL | `engine_required` — names the missing engine |
| `/infra`, `/infrastructure`, `/secrets` | HONEST_REFUSAL | `message` + `status` |
| `/council/proposal/` | HTTP_400 | `Invalid proposal ID` — needs a real id argument |
| `/k3/` | NOT_HANDLED | `unknown_k3_route` — the bare prefix is not a route; `/k3/retrieve` is |
| `/kanban` | NOT_HANDLED | `Not found` |
| `/verification/proof` | HTTP_400 via POST | `no_proof` — correct: it needs a proof body |
| **`/experts`, `/swarm/experts`** | **EMPTY_BODY** | **200 with nothing in it** — the one class where a caller cannot tell "nothing to report" from "not implemented" |

Slowest honest answers: `/api/continuum` 290 ms, `/system` 71 ms, `/verification/artifacts` 47 ms,
`/verification/fabrication` 22 ms.

## 3. The SRAM question, answered from the tree

The document's thesis is that the SRAM registry is the constraint layer, that modulating it modulates
compute, and that the single most useful experiment is loading Bonsai 2 27B's ternary weight matrix into
it and measuring whether Hopfield relaxation converges to the same attractors as the forward pass.

What the tree actually contains, measured:

| layer | state | evidence |
|---|---|---|
| The hardware fact | **real and measured** | `/system` → `l3_sram_kb 16384` (16 MB) |
| The module's declared constant | **was disagreeing — now derived** | it said `L3_SIZE = 32 MB` while the host measured 16 MB; it now probes (kernel → OS → documented target) and reports `L3_SIZE_BASIS` beside the value |
| The Python module | **real, and it is a specification/pattern generator** | 15 architectures; on this host `L3_CELLS 134,217,728`, `TOTAL_TRANSISTORS 805,306,368` — derived from the probed 16 MB L3. (The 32 MB design target would give 268,435,456 / 1,610,612,736; that is the declaration, not this machine, and it is no longer served.) `pattern_hopfield_attractor(64)` returns `{name, paradigm, neurons, energy_fn, attractor_basins, applications, pattern}` where `pattern` is a 64-byte blob (`0f f0 0f f0 …`) |
| The compute the document describes | **absent** | the module defines no `relax`, `recall` or `step`; nothing evolves a state or relaxes to an attractor |
| The Rust port | **was a zeroed skeleton — fixed in this round** | `host_tranche/src/sram_pattern_computer.rs` held `L3_SIZE = 0`, `L3_CELLS = 0`, `ARCHITECTURES = 0` and 15 functions returning `0 // TODO`; it is now generated from the module, declared in `main.rs`, and passes a 3-test cross-language parity suite (`docs/FIXES_DERIVE_20260925.md`) |
| Reachability | **none, and it was never compiled** | `host_tranche/src/main.rs` declared only `mod radix_trie_matcher`, so all 28 modules were dead source; the SRAM port is now declared and compiles, 23 files still are not |
| **The native strip** | **real, and it executed — but could not discriminate** | `swarm_v2_artifacts/native_voltage_strips/strip_sram_computer.dll` loaded and returned `rc=0`, yet hashed one generic line for every index ≥ 2, giving **3 distinct outputs for 15 architectures**; regenerated from the module it now returns **15 of 15**, cross-language parity 7/7 (2.23 µs per reconfig, measured) |
| What the kernel declares | **a literal** | `const SRAM_PATTERNS_DECLARED: u32 = 15;` served on `/system` as `sram_patterns_count` |

So the honest answer to the named experiment: **it cannot be attempted here, for three independent
reasons** — there is no SRAM registry object in the runtime to load weights into, there is no relaxation
implementation to compare against a forward pass, and Bonsai 2 27B is unrunnable on this host's 8 GB VRAM
(mainline llama.cpp loads only Ternary-Bonsai `*_g64`). The document itself names the fork this would
decide: if relaxation did not converge to the forward pass, "the SRAM modulation is a different
computation than the model's, and the integration needs a translation layer."

What would make it attemptable, in order: implement the relaxation step (the K3 collapse already exists
and is byte-identical across tiers), export a ternary weight matrix from a GGUF this host can load, and
define the convergence criterion. That is real work, and it is not a wiring job. The reconfig half is no
longer the blocker: the strip now selects a distinguishable pattern per architecture and agrees with the
module byte-for-byte, so what is missing is the **relaxation**, not the modulation.

## The seal, and a defect it caught

Block 139's first write landed with `prev_hash: None`: the sealer read the tip's hash from a field named
`block_hash`, which canonical blocks do not carry, so `prev_hash links broken: n=1 [None]`. The platform's
own verifier caught it; the block was rewritten in the canonical shape (`index`/`hash`/`prev_hash`, digest
over `json.dumps(body, sort_keys=True, separators=(",", ":"))`), the correction is recorded inside the
block, and the sealer was fixed in the same pass. Post-correction: **blocks=139, canonical=129,
alt_separators=5, UNVERIFIED=5 (the pre-existing 97–101), broken links n=0**.

## Not claimed

Not that the platform is complete: two prefixes answer with an empty body, 135 artifacts are quarantined
as fabricated, 155 locally-satisfiable tasks remain unstarted, and `/aura` names an engine it does not
have. What is claimed is narrow and measured — the nine capabilities above return derived values a
skeptic can re-run, the failures are named, and the SRAM layer's state is stated as it is rather than as
the architecture describes it.

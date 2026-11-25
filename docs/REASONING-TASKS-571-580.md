# Tasks 571-580 (Reasoning Systems) — where the platform actually is

Investigated 2026-11-24 against the code, not the module register. Every claim below is a search result or a
file read; the searches are given so they can be re-run.

## Status

| task | status | evidence |
|---|---|---|
| **571** forward chaining | **ABSENT** | zero hits for `forward_chain` / `forward chaining` / an inference engine in `.rs` and `.py`. The only "rules" surface is `shield_rules::describe()` (`/k3/rules`), which publishes a **fixed decision order** for the safety gate — a heuristic list, not a rule base that derives conclusions from facts |
| **572** backward chaining | **ABSENT** | zero hits for `backward_chain`, goal-directed search or resolution. No prover binary exists: `z3`, `cvc5`, `vampire`, `eprover` are all absent from PATH and from the repo |
| **573** case-based reasoning | **ABSENT** | no `case_based` / `cbr` / case base anywhere. The fabric provides **similarity retrieval** (768-d embeddings) but nothing that retrieves a case, adapts it, and verifies the adaptation |
| **574** analogical reasoning | **ABSENT** | every `analog` hit is prose or an analogue signal (`aura_spectrum`, `fractal_pattern_engine`); no structure-mapping or relation transfer |
| **575** abductive reasoning | **ABSENT** | zero hits for `abduct` in the whole repository |
| **576** default reasoning | **ABSENT** | no `defeasible`, `non_monotonic` or `default_logic` code. `k3_logic.rs` gives Kleene **K3 truth functions** (AND/OR/NOT/IMPLY/EQUIV) and a consensus over trits — UNKNOWN is a truth value there, not a default rule with exceptions |
| **577** temporal reasoning | **ABSENT** as a calculus | time appears only as wall-clock stamps (`uptime_seconds`, `updated_at`, `reason_at`). No Allen interval algebra, no LTL, no event ordering beyond the ledger's chain |
| **578** spatial reasoning | **ABSENT** | zero hits in Rust. The Python hits (`cosmos_bridge.py`, `cosmos_microkernel_entanglement.py`) are narrative text, not geometry |
| **579** complete reasoning suite | **NOTHING TO INDEX** | it is an umbrella over 571-578; eight of the nine are absent |
| **580** explainable reasoning reports | **PARTIAL — the only one with real substance** | each council vote carries `reason` plus its `[axis: A#]` tag; board cards carry `reason`/`reason_at`; `cadence.rs` cites samples and refuses to speak below `MIN_CITED_SAMPLES = 200`; `/council/axioms` and `/k3/rules` publish the decision framework. What is missing: any **derivation trace** (this fact, that rule, therefore this conclusion) and any endpoint that reconstructs an argument from stored evidence |

## The registry, corrected: it was stale, not invented

**My first pass was wrong and this section replaces it.** An audit that scanned only `trm_uefi_kernel/src`
reported that 97 of the registry's 281 names had "no source file anywhere". That was a search failure, not a
finding: **all 97 exist on disk.** They live in the cdylib tree and the staging snapshots —

| tree | modules | built? |
|---|---|---|
| `trm_uefi_kernel/src` | 210 | yes (UEFI kernel) |
| `swarm_v2_rust_kernel/src` | 13 | yes — `target/release/swarm_rust_kernel.dll` (302,592 B) |
| `staging/v20/trm_uefi_kernel/src` | 189 | no — staged snapshot |
| `staging/v20/swarm_v2_rust_kernel/src` | 110 | no |
| `scratch/wt_verify{,2}`, `scratch/wt_models`, `scratch/wt_memory` | several | no — worktrees |

What was actually wrong with the registry, measured:

1. **Stale in both directions.** It claimed 322 modules across (uefi 188, dll 93, trm-rust 41) against 218
   live modules today, and **34 live modules were missing from it** (`aura_spectrum`, `ddr_scan`,
   `autonomous_cadence`, …).
2. **Tree-blind.** It never recorded which tree a name came from, so a reader — or an audit — confined to one
   tree would conclude the others were fabricated. That is exactly the error made above.
3. **Per-tree counts do not reconcile** with any tree that exists now (the closest are staging/v20's 189 and
   110), because the scan was taken on 2026-10-26 and nothing re-ran it.

### Fixed

- `tools/rebuild_module_registry.py` scans **every live tree** (UEFI + cdylib), records `ORIGIN` per module,
  computes `TREES`, `TREE_COLLISIONS` (5) and reports staged trees separately in `TREES_STAGED`. Regenerated:
  **218 modules — REAL 103, PORTED 60, SKELETON 46, EXCLUDED 9**.
- `kernel_tests::registry_guard::registry_names_exactly_the_live_modules` fails on drift in **either** tree.
  Proved by adding a module to the cdylib tree: the guard failed naming it, and passed again after removal.
- The old `REAL = 274` is now `103` — not because 171 modules were removed, but because the old number counted
  three trees including staged snapshots and did not distinguish a module from a build artefact.

### What this means for reasoning specifically

The three reasoning modules the registry called `REAL` exist **only outside the live trees**:

| module | where it exists | compiled into the running platform? |
|---|---|---|
| `cross_modality_reasoning` | `staging/v20`, `staging/sovereign_kernel`, four scratch worktrees | **no** |
| `voltage_raw_reasoning_engine` | `staging/v20`, four scratch worktrees | **no** |
| `analog_cognition_engine` | `staging/v20`, `staging/sovereign_kernel`, four scratch worktrees | **no** |
| `collaborative_reasoning` | `trm_uefi_kernel/src` | yes, but it is 23 lines with one function (`get_cra_engine`) |

So the findings on 571-580 stand, with the wording corrected: these are not missing files, they are
**unbuilt sources**. A module in a staging snapshot is not a capability of the running platform, and the
registry should never have counted it as one.

## What the platform does have that a reasoning engine could use

| asset | what it gives | state |
|---|---|---|
| `k3_logic.rs` | Kleene K3 truth functions, trit consensus, bypass decision | real, tested |
| `shield_rules.rs` | a declarative rule list and decision order | real, published at `/k3/rules` |
| 16 shards, 7,793,536 records | facts: 406 LpxC actives, 6,463 CARD genes, 147,932 SMPDB pathway records, 612,025 trials | real, seals verified |
| TPTP MPTP2078 | 28,346 formal problems | **on disk and used by no Rust code** — only listed as a string in Python narrative scripts |
| `trm7m.rs` | a 7M model that emits **3 trits** | real, but a classifier, not a reasoner |
| fabric index | 768-d retrieval over docs and evidence | real; retrieval is not inference |

## Recommendation, by cost against value

1. **571 forward chaining — build it.** It is the cheapest real engine here: a fact base from the shards plus
   rules of the `shield_rules` shape, evaluated in Kleene K3 so a missing fact yields UNKNOWN instead of a
   guess. It is also the only one of the nine whose output can be checked against the corpus.
2. **580 explanation — extend it, do not rebuild it.** Once 571 exists, the derivation chain is the report:
   emit `fact → rule → conclusion` with the shard seal for each fact. The platform already stores reasons; what
   it lacks is the link from a conclusion back to the facts that produced it.
3. **576 default reasoning — cheap after 571.** Exceptions become a second rule class with explicit
   defeat; without 571 there is nothing to defeat.
4. **572 backward chaining — defer.** It needs a goal language and a query surface, and there is no prover to
   delegate to. TPTP being unused is the tell: the data for this has been sitting on disk, unread.
5. **573, 574 — defer.** Both need an embedding space *and* an adaptation/transfer function; the first exists,
   the second does not, and neither has a user while no reasoning pipeline consumes its output.
6. **577, 578 — skip.** No temporal calculus has a caller (timestamps suffice for the ledger), and there is no
   spatial or geometric data on this host at all. Building either would be a library with no user — the exact
   failure mode this audit keeps finding.
7. **579 — nothing to build.** It is the index of the other nine.

## Limits

- Searches covered `trm_uefi_kernel/**` (excluding `target/`), `tools/`, and the repo root, in `.rs` and `.py`.
- Not audited: the Python prototypes under `swarm_v2/core/` beyond keyword hits, and the skills library
  (`trm-logic-reasoning`, `trm-logicmaster-advanced`), which documents SAT/SMT and Coq/Lean workflows but ships
  no solver binary in this tree.
- "ABSENT" means "no implementation found by these searches", not "asked and answered". Each absence names the
  search that would have found it.


---

# BUILT — four of the ten now exist (2026-11-25)

The gaps that mattered were closed with **one primitive**, not four demos: `trm_uefi_kernel/src/reasoning_engine.rs`
(no_std), exposed at `/reason/rulebases`, `/reason/forward`, `/reason/prove`, and checked against RDKit.

| task | status now | what exists |
|---|---|---|
| **571** forward chaining | **BUILT** | `Engine::forward_chain` saturates a rule base to fixpoint. A rule fires only when every premise is TRUE, every negative FALSE, and the exception is not TRUE. Bounded by `MAX_ITERATIONS`; hitting the bound sets `truncated` rather than returning a partial answer as if complete |
| **572** backward chaining | **BUILT** | `Engine::prove` searches goal-directed over the same rules, returns the derivation, and on failure names the premise or the depth/cycle bound that stopped it. Propositional only — atoms are opaque strings, no unification |
| **576** default reasoning | **BUILT** | rules carry an `exception`, three-valued: TRUE defeats (conclusion withdrawn), FALSE lets the default apply, **UNKNOWN lets it apply but records the assumption** in `ForwardResult.assumptions`. A caller needing certainty requires `assumptions.is_empty()` |
| **580** explainable reports | **BUILT** | every conclusion carries a `Derivation` tree down to asserted facts, and each leaf keeps the `source` string it was asserted with (shard, record, seal). A proof prints as `too_many_violations ← two_violations ← [violation_mw ← lipinski_mw ← mw_gt_500 · shard … seal …]` |
| 573 case-based · 574 analogical · 575 abductive | still absent | no case library with adaptation, no structure mapping, no hypothesis space. Unchanged verdicts, with reasons above |
| 577 temporal · 578 spatial | still absent | no interval calculus and no spatial data; both would be libraries with no caller |
| 579 complete suite | 4 of 10 | the umbrella is only as real as its members |

## Semantics, stated because they are choices

- **Kleene K3** throughout, the kernel's existing encoding: TRUE / UNKNOWN / FALSE. An UNKNOWN premise never
  yields a TRUE conclusion.
- **A conflict is reported, never resolved by arrival order.** An atom derived TRUE and asserted FALSE becomes
  UNKNOWN and appears in `conflicts`.
- **No negation-as-failure.** Absence of a fact is not falsity: the closed-world assumption is a non-monotonic
  leap this engine refuses. Falsity must be asserted.
- **No counting.** "Two or more violations" is written as all **C(5,2) = 10** explicit pairwise rules.
- **No arithmetic, no probabilities, no variables.** Thresholds arrive as asserted facts; a fact is TRUE, FALSE
  or UNKNOWN, never 0.73.
- **A malformed request is refused (400)**, never defaulted to an empty fact set — an empty set answers UNKNOWN
  to everything, which reads as caution and is really a broken request.

## Verification: the engine against RDKit, 406 real compounds

`tools/reasoning_verify.py` recomputes the properties with **RDKit** — an independent implementation in another
language — and compares the engine's derived conclusion, compound by compound, over all 406 real ChEMBL LpxC
actives (`evidence/chembl_lpxc_actives.json`).

| profile | scored | agree | TRUE derived | left below TRUE | withheld-fact control |
|---|---|---|---|---|---|
| Lipinski/Veber/Ghose (`mw>500, logp>5, hbd>5, hba>10`) | 406 | **406 (100%)** | 0 | 406 | PASS |
| strict (`mw>450, logp>4, hbd>4, hba>8`) | 406 | **406 (100%)** | 5 | 401 | PASS |

The second profile exists because the published thresholds are so lenient that **not one of the 406 reaches two
violations** — a rule base that never derives a positive cannot be verified by running it. The strict thresholds
are stated up front, not tuned until something passed.

**The harness earned its keep on the first run.** With 8 pairwise rules instead of 10, CHEMBL242031 (hbd 5,
heavy atoms 19 — exactly `hbd + ghose`, two violations) came back **UNKNOWN** where RDKit expects TRUE. That was a
real defect in the encoding, caught by the control rather than by inspection; all 10 pairs are now present in
both profiles.

## Unit tests (11, all passing)

`forward_chain_derives_transitively_from_a_fact`, `a_missing_premise_yields_unknown_not_true`,
`a_false_premise_does_not_fire_the_rule`, `a_cycle_terminates_and_is_explained`,
`a_defeasible_rule_is_defeated_by_its_exception`, `an_unknown_exception_is_named_not_hidden`,
`a_conflict_is_reported_and_left_unknown`, `backward_chaining_returns_the_chain_down_to_the_evidence`,
`backward_chaining_names_the_premise_it_could_not_settle`, `the_druglikeness_rulebase_derives_two_violations`,
`an_explanation_serialises_with_its_rule_and_source`.

## Live

```
GET  /reason/rulebases      -> 3 rule profiles + the semantics document + the limits
POST /reason/forward        -> truth, per-atom derivations, assumptions, conflicts, iterations, truncated
POST /reason/prove          -> goal truth + full derivation tree + the note explaining any UNKNOWN
malformed JSON              -> 400 (never a defaulted empty request)
rule with no premises       -> 400 ("an unconditional rule is a fact, assert it")
```

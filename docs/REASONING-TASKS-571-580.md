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

## The one thing that would make "reasoning systems" honest

**The module registry claims reasoning capabilities that do not exist.** `src/module_registry_generated.rs`
(scanned 2026-10-26) lists 281 module names and classifies 274 as `REAL` — "hand-written intelligence,
verified". Checked against the tree:

| measure | count |
|---|---|
| names in the registry | 281 |
| present in `src/` | 184 |
| present anywhere else in the kernel tree | 0 |
| **no source file anywhere (phantom entries)** | **97** |
| of those, labelled `REAL` | **97** |

Reasoning-specific examples: `cross_modality_reasoning` (REAL, no file) and `voltage_raw_reasoning_engine`
(REAL, no file) are both phantom; `collaborative_reasoning` exists but is **23 lines** containing one function,
`get_cra_engine()`, with none of `kleene`, `trit`, `infer`, `premise`, `conclusion` or `rule` in it.

So `REAL` overstates the compiled intelligence by at least 97 modules, i.e. **274 → at most 177**. That is the
same defect class as the consensus labels: a number in a header that no check enforces. The fix is to
regenerate the registry (`swarm_v2/core/build_module_registry.py`) and to make the boot path refuse a registry
entry with no file, rather than trusting the scan's date.

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

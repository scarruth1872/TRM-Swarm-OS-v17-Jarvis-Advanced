# Stub research — what the 30 synthesized strips should be doing

Measured 2026-11-05. Every row comes from reading the Python spec that shares the stub's name, not from
the stub's own source.

## Method

1. `scratch/stub_spec_survey.py` — match each stub to a Python module, with line count and importer count.
2. `scratch/stub_triage.py` — docstring, imports, numpy/file/network/process tags per module.
3. `scratch/stub_functions.py` — per-function arithmetic and I/O counts, so a disposition is argued per
   function rather than per file.

## Result 1 — the stubs are named after real modules

29 of 30 have a spec. The exception is `lora_modulator`, whose module is `lora_response_modulator.py`;
it has already been implemented in Rust (`lora_modulate.rs`, string-identical on 400 deltas).

## Result 2 — only one stub shadows a numerically heavy module

| stub | numpy calls | file ops | verdict |
|---|---|---|---|
| `thz_bonsai_gateway` | **29** | 4 | a real linear SSM — **ported**, 0 trit mismatches over 6,400 values |
| `cosmos_sparse_moe`, `instella_sparse_agent_teams`, `voltage_node_engine`, `symbology_etymology_monad` | 2 each | 0 | thin; `voltage_node_engine` already binds and calls a DLL |
| everything else | 0–1 | 3–23 | file-backed state machines |

## Result 3 — ten have no compute function at all

`skill_loader`, `lobster_shell`, `skill_validator`, `manual_clearing_engine`, `catalyst_phase1`,
`semantic_knowledge_graph`, `deterministic_forge`, `coordination_responder`, `feedback_loops`,
`complete_singletons` — every function moves data, none computes.

## Result 4 — nineteen do have compute functions

`turbovec_memory` (`query`), `qisa_optimizer` (`optimize`, `improvement_pct`), `resonance_engine`
(`get_emotional_resonance_index`, `recursive_self_awareness_check`), `neuro_continuum` (`evolve`),
`manus_engine` (`_update_coherence`), `cognitive_stack` (`process`, `tune_recursion_depth`),
`cosmos3_edge` (`_verify_action`), `coordination_protocol` (`collect_responses`), `moltbook`
(`resolve_query`), `mcp_bus` (`call_tool`), `generative_architect` (`optimize_topology`),
`symbology_etymology_monad` (`resolve_concept_to_truth`), `multimodal_predictive_analytics`
(`forecast_degradation`), `voltage_node_engine` (`evaluate_voltage_signal`), `build_totality_memory`
(`build`), `external_socket_bridge` (`_handle_client`), `instella_sparse_agent_teams`,
`cosmos_sparse_moe`, `thz_bonsai_gateway` (done).

## Why the first motion failed

`STUB-DISPOSITION-01` proposed deleting the 29 stubs whose spec showed no numeric core. The council
returned **10 approve / 3 abstain / 2 reject — no consensus**. Logic and LogicMaster rejected: a
module-level line count shows where numeric content is *thin*, not that it is *absent*. That objection
is correct, and Result 4 is the proof — nineteen modules the bulk rule would have swept away do contain
compute functions.

## Step D — refuted, and the roster now says so

The council scoped step D to three modules with "a real numeric core". None of them has one:

| module | what its "compute" function does |
|---|---|
| `neuro_continuum.evolve` | `generation += 1`, `mutation_rate *= (1+mutation_pressure)`, returns a dict. Its own comment says the LoRA perturbation is *not implemented*: "For now: track the evolutionary state". |
| `manus_engine._update_coherence` | `max_prob * completion_ratio` — one multiply. |
| `manus_engine._calculate_evidence_strength` | `0.5` baseline, `+0.1` for a code fence / PLAN / DEBUG-TEST / length>800, `+0.2` for a TRM marker, capped at 1.0. A keyword heuristic, not arithmetic on measured quantities. |
| `cognitive_stack.process` | async routing to the TRM engine. |

They were nominated by a per-function count of `+=` and `*` operators, which cannot tell a counter
increment from a computation — the same weakness the council identified when it rejected the bulk-delete
motion. Putting three FFI calls in front of a counter and a multiply would add cost and claim a port.

**Council: STEP-D-RESCOPE-01 carried 13 approve / 2 abstain / 0 reject.** No Rust was written for these
three. The roster now carries `module_status` per stub:

| status | count |
|---|---|
| `shell` | 22 |
| `no_numeric_core` | 5 |
| `implemented` | 2 (`lora_modulator`, `thz_bonsai_gateway`) |
| `poor_candidate (CPython RNG parity)` | 1 (`qisa_optimizer`) |

The numeric port programme is therefore complete on this evidence: **235 native strips, 5 modules with
real Rust in this band** (`higuchi`, `lora_modulate`, `bonsai`, `resonance`, `qsar` via its route), and
every remaining Python module shown to be I/O, a table, or a shell.

## Revised disposition

- Implement, in order of (importers × compute functions): `turbovec_memory` (9 users), `resonance_engine`
  (11), `semantic_knowledge_graph` (13, but no compute), `qisa_optimizer`, `neuro_continuum`,
  `symbology_etymology_monad` (9).
- Keep `stub: true` on all 30 until each is either implemented or explicitly retired with a reason.
- No deletions without a per-module review; the flag is the honest interim state.

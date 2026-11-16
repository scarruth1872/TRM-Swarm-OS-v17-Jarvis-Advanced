# Roster gate: both directions proven, plus the instella claim (2026-09-23)

## The declared gap is closed

Step 3 recorded: "the copy-side green proof was established for the earlier three-roster scope; its
green direction has not been re-proved with the expanded roster set." It now is.

Five roster sites were fixed **in a copy only** (`scratch/council_green/`, real-tree hashes unchanged):
`LogicMaster = 14` + `NUM_AGENTS = 15` in both `agent_runtime.rs` files, the member added to
`council.rs` and to `swarm_council.py`'s `COUNCIL_MEMBERS`, and a `Verify` entry added to the
`trm_council_models.py` agent dict.

```
COPY (fix applied)  -> RC 0   VERDICT: consistent    (all five rosters 15 / matches)
REAL TREE           -> RC 1   VERDICT: DRIFT (5)
```

So the candidate fix is **sufficient for the rosters**, and the gate fires in both directions. What is
still not proven for a real deployment: the twelve `% 14` sites that must move with the enum, a rebuild,
and the vote-for-vote comparison against the oracle.

## Two more defects in my own gate, found by this exercise

1. **A row could read `matches` for a file the gate could not parse.** The unscannable branch stored
   `missing: []`, and the printer derived its tag from `missing` — so an unscannable source printed
   `matches`. That is the same false-clean this project keeps punishing: *unscannable is not clean*.
   Fixed with an explicit `unscannable` flag printing `UNSCANNABLE (not a clean bill)`.
2. **My sandbox edit to `trm_council_models.py` was broken**, which is what made (1) fire: I inserted
   before the first textual `"Seeker"` occurrence rather than into the agent dict itself, and the
   extractor then found no >=8-key dict. Redone by locating the dict by AST line, inserting at its first
   key's line, and re-parsing to assert the copy still compiles before running the gate.

## `verify_instella` — structural claims verified

Claim: "Same 384-dim hidden state + 64x384 expert weights through both; compares selected top-k expert
ids and softmax weights", against `c_instella_gate_topk`. That op exists in the v17 `.so` **and** in the
v19 `.dll` — it is the one C op v19 kept.

| element | v18 Rust | v17 header / C |
|---|---|---|
| experts | `TOTAL_EXPERTS = 64` | `constexpr size_t TOTAL_EXPERTS = 64` |
| hidden | `HIDDEN_DIM = 384` | `float hidden_states[384]` |
| top-k bound | asserts `top_k <= TOTAL_EXPERTS` | `MAX_TOP_K = 8` |
| ranking | sort desc, truncate `top_k` | `partial_sort` with `a.first > b.first` |
| weights | softmax over the top-k logits | softmax over the top-k logits |
| output | `(Vec<u8>, Vec<f32>)` | `selected_expert_ids[8]`, `expert_gate_weights[8]` |

Its exact input corpus is recoverable too (`hidden[i] = ((seed >> 33) % 100) / 100.0` on the same LCG).
Two things remain unverified: an exact numeric comparison is not reproducible in stdlib Python because
the C inner loop uses `_mm256` FMA while the Rust accumulates scalar left-to-right; and `partial_sort`
is not stable, so equal logits may break ties differently between the two — a determinism gap rather
than a correctness one.

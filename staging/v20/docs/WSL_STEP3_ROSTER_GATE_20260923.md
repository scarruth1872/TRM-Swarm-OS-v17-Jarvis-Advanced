# Step 3: Decision taken, and a roster-parity gate (2026-09-23)

## Decision: option 3 — the live council is NOT modified

Chosen because it is the only option that cannot stop the system. Options 1 and 2 both edit
load-bearing sites (16 agent-count sites, including the `rust_kernel_binder` `% 14` modulus this
campaign verified against the DLL) and change the live council's verdicts at ties, in order to resolve
an ambiguity between roster definitions that themselves disagree. Option 3 costs nothing today and
neither option is foreclosed — the candidate change is now built and proved on a copy.

## What was built instead: `swarm_v2/core/verify_council_roster.py`

Read-only parity gate. It parses the roster definitions, checks every agent-count constant against its
own tree's roster size, and optionally compares the **running** kernel via `--live`.
Exit codes: 0 consistent, 1 drift, 2 could-not-run.

On the real tree today:

```
oracle (15)                                          matches
agent_runtime.rs (SwarmAgentId)                 14   missing ['LogicMaster']
council.rs (COUNCIL_MEMBERS_INFO)               14   missing ['LogicMaster']
swarm_council.py (COUNCIL_MEMBERS)              14   missing ['LogicMaster']
VERDICT: DRIFT (3)        RC 1
   with --live: DRIFT (5) — the running kernel also reports 14 and lacks LogicMaster
```

## The gate was wrong three times before it was right

Recorded because each one is the same class of error this campaign keeps meeting:

1. **Scope too wide.** It failed `native_mailbox_bridge.rs`'s `MAX_AGENTS = 15` — which is a faithful
   port of the C++ `constexpr size_t MAX_AGENTS = 15` (15 × (128 + 262144) = 3934080, the region size
   this campaign verified), not a roster count. A red gate was red for the wrong reason.
2. **Extractor shape.** Two rosters parsed as 0 names because they write `name="Archi"`, not `name: "Archi"`.
3. **Crash instead of verdict.** `%2d` on a `None` count (the unscannable file) raised instead of
   reporting — so an RC 1 looked like a finding when it was my own traceback.

Also corrected: `trm_council_models.py` is **not a roster** (0 `name=` literals) — it is an agent-*models*
file, so it is now checked softly and an unextractable source reports **NOT SCANNABLE**, not "missing all".

## The candidate fix, proved on a copy

Applied to a **copy** under scratch (`council_green/`, real tree hashes re-verified as unchanged):
add `LogicMaster = 14` to the kernel enum and `AGENTS: usize = 15`; add the member to `council.rs` and
raise `NUM_AGENTS` to 15; add the member to `swarm_council.py`'s `COUNCIL_MEMBERS`.

```
COPY (fix applied) -> RC 0   VERDICT: consistent
REAL TREE          -> RC 1   VERDICT: DRIFT (3)
```

So the fix is sufficient for the rosters, and the detector fires in both directions. What remains
unproven for a real deployment: the twelve `% 14` sites that must move with the enum, a rebuild, and a
vote-for-vote comparison against the oracle — none of which was attempted, because the live system was
not to be touched.

## Still open (unchanged, flagged not fixed)

- `/council/status` reports `pbft_quorum: 10`; N=14 gives `2f+1 = 9` (verified against the Python oracle
  in this campaign). Live value matches neither N=14 nor N=15.
- A second `agent_runtime.rs` exists in `swarm_v2_rust_kernel/src/` with its own `AGENTS = 14` — the
  platform has more roster sites than the four compared here.
- `trm_council_models.py` references 14 agent names including `LogicMaster` but not `Verify`.

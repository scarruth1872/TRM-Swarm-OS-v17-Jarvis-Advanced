# Roster 15 deployed to the live kernel (2026-09-23)

## Result

```
/health         -> Native Rust Sovereign Kernel v19.001 | HEALTHY | py_active False
/council/status -> total_agents=15  pbft_quorum=9
members (15)    -> Hermes, Jarvis, Bridge, Shield, Logic, Devo, Pulse, Flow, EventLedger,
                   Orchestra, THzGateway, Verify, Seeker, Archi, LogicMaster
```

:8022 was back up **5 s** after the kill (the supervisor heals it, so kill+copy went in one step).

## What actually changed

Chosen from evidence, not a blind `% 14` replace — the audit found two of twelve sites belong to
*different* axes and were deliberately left alone (`scientific_parser_evolution_campaign.rs` uses
`job_id % 14` for a parser id; `zero_network_colocation.rs` uses `(i % 14)` as a ring-buffer offset).

- rosters: `LogicMaster = 14` + count 15 in **both** `agent_runtime.rs`; member added to `council.rs`,
  `swarm_council.py`, `trm_council_models.py` (which also gained the missing `Verify` entry), and to the
  **`:8022` server's own `AGENTS` table** - the table it actually serves `/council/status` from.
- agent-id moduli: `lib.rs`, `rust_kernel_binder.rs`, the council UI rotations, plus the 3 stale comments.
- the live threshold defects: `pbft_passed = approve >= 10` -> `>= 9`, and the served
  `"pbft_quorum": 10` -> `9`. With N=15, `f=(15-1)//3=4`, so `2f+1=9`; the old 10 was the pre-correction
  value and made the council refuse proposals that had quorum.
- `LogicMaster` got **doctrine-compliant ABSTAIN-unless arms** in the deliberation (7 motions) and a
  `get_agent_deliberation` arm. This matters: the per-motion fallback arm `(motion.id, _)` *approves*, so
  adding a member without arms would have added a rubber-stamping approver.

## Gate

`verify_council_roster.py --live` -> **RC 0, VERDICT: consistent** (all seven rosters at 15, the served
counts matching, and `pbft_quorum` equal to `2f+1` for its own N).

## Rollback

Sources: `backups/preroster15c_20260923_184258/` (7 files, verified) plus `preroster15_*` for the first
pass. Binaries were overwritten in place, so a rollback restores the sources **and rebuilds** - both
artifacts build in ~10-17 s into an isolated `CARGO_TARGET_DIR`, so the cost is low. The pre-existing
`predelegation_*` / freeze archives are untouched.

## Also fixed in my own tooling

The gate had a **scope blind spot**: it did not know the `:8022` server kept its own roster, so it
reported a clean bill while the served surface still said 14. It now includes the server, and it no
longer counts `//` comment lines as agent-count constants (it had flagged `council.rs:242`, a comment,
as a mismatch - the same false-positive class as `MAX_AGENTS`).

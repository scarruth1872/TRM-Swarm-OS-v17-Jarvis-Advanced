> **UPDATE 2026-09-23**
> RESOLVED by ROSTER15_DEPLOYED_20260923.md: the live kernel now serves total_agents=15, pbft_quorum=9,
> with LogicMaster in the roster. The five-roster divergence described below is closed and gated by
> swarm_v2/core/verify_council_roster.py (--live).

# Step 2: The live council's roster — reconciled against the oracle (2026-09-23)

## What the live kernel serves

`:8022 /council/status` reports `total_agents: 14`, listing Hermes, Jarvis, Bridge, Shield, Logic, Devo,
Pulse, Flow, EventLedger, Orchestra, THzGateway, Verify, Seeker, Archi — that is exactly the v18 archive's
roster. The live runtime does **not** have `LogicMaster`.

## Five roster definitions, three different sets

| definition | count | difference from the oracle |
|---|---|---|
| `swarm_v2/core/agent_intelligence_bridge.py` (`AGENT_MODELS`) — the oracle | **15** | — (complete) |
| `trm_uefi_kernel/src/agent_runtime.rs` (`SwarmAgentId`, `AGENTS`) — **the live kernel's enum** | 14 | missing **LogicMaster** |
| `swarm_v2_rust_kernel/src/council.rs` (`COUNCIL_MEMBERS_INFO`, `NUM_AGENTS`) | 14 | missing **LogicMaster** |
| `swarm_v2/core/swarm_council.py` (`COUNCIL_MEMBERS`) | 14 | missing **LogicMaster** |
| `swarm_v2/core/trm_council_models.py` | 14 | missing **Verify** (a *different* omission) |

Two distinct defects, not one: `LogicMaster` is absent from the three operative rosters, and `Verify` is
absent from one of them. No single file is canonical — which breaks the one-owner-per-store rule the
platform applies elsewhere.

The premise that this is a defect rather than a deliberate exclusion is well supported: **v19's own modules
already reference `LogicMaster`** — Python `agent_mailbox_bus`, `background_tasks`, `cold_start_recovery`,
`local_lora_trainer`, `rust_kernel_binder`, `turbovec_memory`, `trm_council_models` (+ 5 route modules),
and Rust `agent_mailbox_bus.rs`, `cold_start_recovery.rs`, `rust_kernel_binder.rs`. The agent exists in the
platform's code and is missing only from its council.

## Why this is not a one-line edit

**Sixteen sites encode the number 14** and move together with the enum:

```
trm_uefi_kernel/src/agent_runtime.rs    AGENTS: usize = 14  +  % 14
trm_uefi_kernel/src/main.rs             % 14   (x2)
trm_uefi_kernel/src/rust_kernel_binder.rs  % 14  (x2)   <- the DLL rule verified in this campaign
trm_uefi_kernel/src/voltage_node_engine.rs % 14  (x2)
trm_uefi_kernel/src/lcars_render.rs        % 14  (x2)
trm_uefi_kernel/src/lib.rs                 % 14
trm_uefi_kernel/src/scientific_parser_evolution_campaign.rs  % 14
trm_uefi_kernel/src/zero_network_colocation.rs               % 14
swarm_v2_rust_kernel/src/council.rs     NUM_AGENTS = 14
```

Adding a member changes council **verdicts** (a fifteenth voter changes ties) and the agent-id modulus —
including the `rust_kernel_binder` rule this campaign verified against the DLL, which is `% 14` today.

## Independent divergence found while checking

`/council/status` also reports `pbft_quorum: 10`. With N=14, `f = (N-1)//3 = 4`, so PBFT quorum is
`2f+1 = 9` — the value this campaign verified against the Python oracle and fixed in `pbft_consensus.rs`.
The live council's 10 matches neither N=14 nor N=15. Flagged, not changed.

## Decision required before any live edit

The missing agent is unambiguous (`LogicMaster` is real and referenced), but the **roster of record** is
not, because five definitions disagree. The options:

1. **Oracle as owner** — propagate `agent_intelligence_bridge.py`'s 15 to the three rosters and the 14→15
   sites, then rebuild and verify the council matches the oracle vote-for-vote.
2. **`trm_council_models.py` as owner** — treat it as canonical, which would mean adding `LogicMaster`
   *and* restoring `Verify`, and re-deriving the oracle to match.
3. **Defer** — record the divergence as a known defect and leave the live council alone.

Per standing practice, the change should be built and proved on a **copy** first (rollback
`predelegation_*` + the freeze archive exist), with the receipt being that the kernel's roster and votes
match the chosen oracle — not merely that it compiles.

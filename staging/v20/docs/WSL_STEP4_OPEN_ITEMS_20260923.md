> **UPDATE 2026-09-23**
> PARTIALLY CLOSED by ROSTER15_DEPLOYED: both findings here were fixed in the live tree - the served
> pbft_quorum 10 is now 9 (the operative line was `approve >= 10` in host_council_deliberation), and
> trm_council_models.py gained the missing Verify entry. The gate reports RC 0.

# Step 4: The three open items — resolved and gated (2026-09-23)

All three were investigated by reading the operative lines, and all three are now covered by
`swarm_v2/core/verify_council_roster.py`, which runs read-only. No live artifact was modified.

## Item 1 — `pbft_quorum: 10` is stale against the kernel's own UI

The live `/council/status` serves `pbft_quorum: 10`. The kernel's own display disagrees with it:

```
main.rs:3269   draw_str(..., b"QUORUM THRESHOLD:2f+1 = 9 Approvals", LCARS_GREEN)
main.rs:3083   "[ARMED: 14 CORE AGENTS TUNED | PBFT QUORUM THRESHOLD N=14, f..."
continuum_pbft.rs:112   "... 14 NATIVE SOVEREIGN AGENTS (PBFT QUORUM >= 10)"
continuum_pbft.rs:281   "⭐ PBFT QUORUM: {} (Threshold: >= 10/14)"
pbft_consensus.rs:15    pub const QUORUM_THRESHOLD: u64 = 8;   // prepare quorum: >= 2f approvals
```

So the Rust `pbft_consensus` constant is the correct prepare quorum `2f = 8`, the LCARS UI draws the
correct commit threshold `2f+1 = 9`, and the *served* JSON field still reports **10** — the
pre-correction value this campaign fixed in the Python oracle. The running kernel contradicts its own
displayed threshold. The gate now catches this class on every run:

```
live pbft_quorum = 10, but 2f+1 for N=14 is 9
```

## Item 2 — the roster count is not one number: there are five agent populations

Conflating them was my own earlier scope error, so the gate now separates them explicitly.

|council rosters (compared to the oracle)|count|variant|
|---|---|---|
|`agent_intelligence_bridge.AGENT_MODELS` (oracle)|15|canonical|
|`trm_uefi_kernel/agent_runtime.rs`|14|−LogicMaster|
|`swarm_v2_rust_kernel/agent_runtime.rs`|14|−LogicMaster|
|`swarm_v2_rust_kernel/council.rs`|14|−LogicMaster|
|`swarm_v2/swarm_council.py`|14|−LogicMaster|
|`swarm_v2/trm_council_models.py` (L45)|14|**+LogicMaster, −Verify**|

|other populations (informational)|count|
|---|---|
|`agent_mailbox_bus.rs` MAX_AGENTS|64|
|`pbft_consensus.rs` N_AGENTS|64|
|`daily_evolution_engine.rs` AGENTS|584 / 64|
|`coherence_thresholds.rs` AGENTS|32|
|`native_mailbox_bridge.rs` MAX_AGENTS|15 (C++ protocol constant)|

The drift is also **wider than three files**: a name-list scan across both trees finds **18 files
carrying the 14-name set** (`pbft_consensus`, `kanban_board`, `llm_brain`, `llm_router`,
`voltage_node_engine` ×2, `monad_emanation`, `daily_evolution_engine`, `microkernel_chat_bridge`,
`on_device_knowledge_engine`, …) against **10+ carrying 15** (`agent_mailbox_bus`, `cold_start_recovery`,
`rust_kernel_binder` in both languages, `local_lora_trainer`, `turbovec_memory`, five route modules).

## Item 3 — `trm_council_models.py` is a genuine roster, and a third variant

It is not an agent-models file as I first wrote — L45 holds a **14-key dict whose keys are exactly agent
names**: `Archi, Devo, Shield, Logic, Seeker, Bridge, Pulse, Flow, Hermes, EventLedger, Jarvis,
Orchestra, LogicMaster, THzGateway`. It has `LogicMaster` and lacks `Verify`. It is now a **hard** roster
check (previously marked unscannable, which was wrong in the other direction).

## Gate status

```
oracle                                15  matches
trm_uefi_kernel/agent_runtime.rs      14  missing ['LogicMaster']
swarm_v2_rust_kernel/agent_runtime.rs 14  missing ['LogicMaster']
council.rs                            14  missing ['LogicMaster']
swarm_council.py                      14  missing ['LogicMaster']
trm_council_models.py                 14  missing ['Verify']
LIVE: 14 agents, pbft_quorum=10

VERDICT: DRIFT (8)      RC 1
```

Honest caveat: the copy-side **green** proof was established for the earlier scope (three rosters).
The new gate has been verified red against ground truth on the real tree; its green direction has not
been re-proved with the expanded roster set, because that requires editing the copy for all six sites.

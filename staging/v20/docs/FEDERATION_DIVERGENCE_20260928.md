# Federation divergence, measured 2026-09-28

Every value below was read from the named node's own /health and /council/status over the Tailscale
mesh in this session. Nothing is asserted that was not answered by the host it names.

## The nodes

| node | host:port | kernel | members | quorum | proposals |
|---|---|---|---|---|---|
| local -- NspiredFitness Laptop Sovereign | 127.0.0.1:8022 | **v19.002** | **15** | **9** | **1** |
| DESKTOP-MEP4VQN -- Desktop Sovereign (6700XT) | 100.93.191.43:8022 | v19.001 | 14 | 10 | **6** |
| carruth-MacPro5-1 -- MacPro5-1 Sovereign (linux) | 100.76.102.6:8022 | v19.001 | 14 | 10 | **6** |
| DESKTOP-VB1T79V -- Desktop Primary Swarm (Python v5 core) | 100.82.16.43:**8021** | **UNREACHABLE (000)** | - | - | - |

Truth anchor `0x4D4F4E4144000001` on all three reachable nodes. `python_active: false` on all three:
the Rust microkernel is the runtime across the federation.

## This machine's to GIVE

1. **`LogicMaster` (Formal Logic & SMT Solver)** -- the 15th council member. Both remotes stop at
   `Archi` (14 members). This is the entire membership difference.
2. **The TRM 7M per-agent assignment** -- every local member carries
   `assigned_model: "TRM 7M (Mamba-3 SSM, 7M par...)"`; on both remotes that field is EMPTY. The
   per-agent model fix (blocks 177-179) is local-only. The remotes are in the pre-fix state.
3. **The v19.002 bump** -- the seven named sites from block 188: four in `main.rs`, plus
   `k3_retrieval.rs:317`, `services_handlers.rs:55`, `state_handlers.rs:1342`.

## This machine's to RECEIVE

The six proposals the 14-member councils passed. Both remotes returned byte-identical sets:

| id | category | votes | phase | topic |
|---|---|---|---|---|
| 1001 | Architecture | 12 | **ENACTED** | **Zero-Python Pure Rust Server Deployment** |
| 1002 | Interface | 12 | ENACTED | CP-COMPANION-001: Sovereign Companion App |
| 1003 | Autonomous Ops | 14 | CONSENSUS | CP-CADENCE-002: Autonomous Operational Cadence |
| 1004 | Distributed Co | 14 | CONSENSUS | CP-COORD-001: Distributed Task Coordination |
| 1005 | Cognitive Arch | 14 | ENACTED | CP-CHAT-001: Absorption of Frontier Open-Source |
| 1006 | Cognitive Arch | 12 | ENACTED | CP-BENCH-001: TRM 7M Mamba-3 SSM AVX2/FMA Native |

**1001 is ENACTED with consensus on both remotes and still `ANALYZE`, `consensus=False`, 12/4/0 here.**
The federation has ratified the zero-Python Rust deployment; this laptop -- which then did the actual
native parser work -- has not recorded it. 1006 is a native AVX2/FMA benchmark for the 7M model.

## The quorum rule, derived

Remotes run **quorum 10 of 14 = ceil(2/3 x N)**, and their enacted proposals cleared it at 12 and 14
votes -- consistent. Local runs **9 of 15**, where 2/3 is **10**. Local's quorum is therefore one below
its own rule and is the outlier. Adopt **ceil(2/3 x members)** = **10** for a 15-member council.

## Method note

The push direction (version, membership, model assignment, quorum) requires writing to the remote
hosts. This session had HTTP reachability only -- no remote shell -- so the push was NOT performed and
is not claimed. The pull was completed by recording the measured remote state here and sealing it.
The registry at `deploy/federation_sync_20260924/registry.json` is stale in the same direction: it
records every node as v19.001, which is now true of the remotes and wrong about this host.

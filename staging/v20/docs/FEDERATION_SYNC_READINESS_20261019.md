# Federation Sync Readiness — 2026-10-19

State of the 4-node federation after re-probe. This records **what blocks a real sync**, not what is
wished.

## Current truth (measured)

| node | endpoint | backend | `/federation/stats` schema | sync write endpoint |
|---|---|---|---|---|
| local (this host) | 127.0.0.1:8022 | Rust **v20.0.0** | `{connected_nodes, total_nodes}` (registry view) | **none** (`/sync` → 404) |
| DESKTOP-MEP4VQN | 100.93.191.43:8022 | Rust v19.001 | `{active_nodes:0, total_nodes:100, configured:100, agent_mesh:14, tiers:6}` | **none** |
| carruth-MacPro5-1 | 100.76.102.6:8022 | Rust v19.001 | `{active_nodes:100, total_nodes:100, quorum:67/100, merkle_root, tiers:6}` | `/sync` → ok |
| DESKTOP-VB1T79V | 100.82.16.43:8021 | Python TurboVecNative (12 agents) | `{peers:[…], stats:{…}}` (different shape) | its own `/federation/*` |
| DESKTOP-VB1T79V :8022 | — | (Rust kernel absent) | — | — (pending) |
| shawns-s25-ultra | — | (Android, no service) | — | — (pending) |

## What blocks the sync — four real gaps

1. **No common `/federation/stats` schema.** Four nodes return four incompatible shapes (registry
   view vs 100-node tier vs `peers` list). A peer cannot consume another peer's state because the
   field names and semantics differ. This is the root blocker.

2. **No common sync-write endpoint.** Only MacPro5-1 serves `/sync`. Local v20.0.0 and MEP4VQN return
   404, so they can receive a handshake but not a state push. (v20.0.0 dropped a route that v19.001
   still has — version skew.)

3. **Divergent membership truth.** MEP4VQN + MacPro5-1 both hold a **100-node / 6-tier table** that
   does not match the 5-device tailnet. MacPro5-1 additionally *claims* `100 active / 67/100 quorum`,
   which the mesh does not corroborate. Local holds the honest 6-entry registry. Three nodes, three
   different answers to "what is the federation?"

4. **Stale peer references.** desktop-vb1t79v advertises a peer at `127.0.0.1:8022` (loopback) —
   unusable across the mesh; it should reference mesh IPs.

## What needs to happen for the sync

1. **Pick one schema** for `/federation/stats` and implement it identically on all Rust nodes
   (the local v4 registry shape — `{connected_nodes, total_nodes, status}` — is the honest candidate,
   since it is the one derived from re-probing).
2. **Add the sync-write endpoint** (`/sync` or `/federation/sync/receive`) back to local v20.0.0 and
   MEP4VQN so a peer can push its state to them (not just read `/health`).
3. **Reconcile the tier table** — replace the 100-node declared table with the actual mesh (5
   devices), and drop MacPro5-1's unverified `100 active / 67 quorum` until it is re-measured.
4. **Fix desktop-vb1t79v's peer list** to mesh IPs, not loopback.
5. **Decide the down nodes**: bring up `desktop-vb1t79v:8022` (Rust) if that host should run a Rust
   kernel, and either deploy a Swarm service to the Android or leave it as a mesh member only.

Until #1 and #2 land, the federation can *observe* each other (`/health` + `/federation/stats`) but
cannot *converge* — a sync would be one-directional (MacPro5-1 → others only, via its `/sync`).

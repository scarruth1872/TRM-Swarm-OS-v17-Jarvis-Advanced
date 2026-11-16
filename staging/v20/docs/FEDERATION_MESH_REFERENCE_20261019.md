# Federation Mesh Reference — Tailscale (2026-10-19)

Live `tailscale status` (tailscale v1.102.3). Tailnet: **`tail890fc0.ts.net`**. Every line below is
measured now, not copied from a stale record.

## Devices on the mesh — 5

| device | mesh IP | OS | Swarm service | federation role |
|---|---|---|---|---|
| `nspiredfitness` (this host) | **100.123.215.119** | windows | :8022 **v20.0.0 HEALTHY** | **local node** |
| `carruth-macpro5-1` | 100.76.102.6 | linux | :8022 **v19.001 HEALTHY** | online peer (has `/sync`) |
| `desktop-mep4vqn` | 100.93.191.43 | windows | :8022 **v19.001 HEALTHY** | online peer |
| `desktop-vb1t79v` | 100.82.16.43 | windows | all Swarm ports **down** | mesh-active, service off |
| `shawns-s25-ultra` | 100.114.251.33 | android | no Swarm service | mesh member only |

## What changed vs. the stale registry

1. **The local node's mesh IP is `100.123.215.119`** — the registry stores `local` as `127.0.0.1`
   (loopback). For any *remote* node to reach this host, it must use the mesh IP, not loopback.
2. **`shawns-s25-ultra` (Android) is a new mesh member** not in the registry. No Swarm service
   answers on it (8021/8022/8000/8001/5183 all down), so it is a mesh device only — not a connectable
   federation node until a service is deployed there.
3. **`desktop-vb1t79v` is `active; direct` on the mesh** but its Python v5 core (`:8021`) is down —
   the host is reachable, the Swarm service is not running. It's a bring-up candidate, not a new node.
4. **The "100 nodes / 6 tiers" is a declared tier table, not the mesh.** `tailscale status` shows 5
   devices. The 100-node figure is a design/paper target (`paper_061_…100_node_federation`,
   `paper_114_…100_node_federation_hybrid_deployment`), not reachable peers. MacPro5-1's
   `/federation/stats` self-reports `100 active / 67 quorum`; that claim is not reflected here because
   it was not verified against the mesh.

## Connectable nodes (verified reachable now)

| node | connect as | handshake |
|---|---|---|
| this host (local) | `http://127.0.0.1:8022` (self) / `http://100.123.215.119:8022` (mesh) | `/sync` not present on v20.0.0 |
| `carruth-macpro5-1` | `http://100.76.102.6:8022` | `/sync` → ok |
| `desktop-mep4vqn` | `http://100.93.191.43:8022` | `/sync` not present on v19.001 |

Uniform sync surface across all three: **`/health` + `/federation/stats`**. `/sync` exists only on
MacPro5-1.

## To connect a new node (the reference)

1. The device must be on tailnet `tail890fc0.ts.net` (verify with `tailscale status`).
2. It must run a Swarm service answering `GET /health` on :8022 (Rust kernel) or :8021 (Python core).
3. Add it to `swarm_v2_federation/registry.json` with its **mesh IP** (not loopback), then re-run
   `scratch/sync_federation_nodes.py` to re-probe + seal.
4. A node is only marked online if its `/health` actually answered — never from its own claim.

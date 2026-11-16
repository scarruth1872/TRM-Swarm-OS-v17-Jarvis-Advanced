# Federation sync and node check (2026-09-24)

Tailnet `tail890fc0.ts.net`, 5 nodes, all online: `NspiredFitness` (this laptop, 100.123.215.119),
`DESKTOP-MEP4VQN` (100.93.191.43, 103 ms via DERP - offsite), `carruth-MacPro5-1` (100.76.102.6,
23 ms, direct), `DESKTOP-VB1T79V` (100.82.16.43, 6 ms ping), `Shawn's S25 Ultra` (android).

## What each node actually runs (probed, not assumed)

| node | :8022 Rust kernel | :5183 dashboard | :8021 Python v5 | identity |
|---|---|---|---|---|
| NspiredFitness (here) | HEALTHY | up | absent (by doctrine) | 15 agents / quorum 9 |
| DESKTOP-MEP4VQN | HEALTHY | up | absent | **14 agents / quorum 10** |
| carruth-MacPro5-1 | HEALTHY | up | up (TurboVecNative) | **14 agents / quorum 10** |
| DESKTOP-VB1T79V | **not listening** | absent | up (`v5_online`, 12 agents, 1,953 artifacts) | Python-only node |

## Two defects found in the federation layer, one fixed

**1. The kernel asserted peers that do not exist (FIXED).** `state_handlers::federation_payload()`
appended a hardcoded `known_peers` block with `status: "online"` - including
`DESKTOP-VB1T79V Rust Microkernel 100.82.16.43:8022`, a port measured as **timing out**, plus a
hardcoded `agents: 17`. The mesh therefore reported an offline node as connected. That block is
removed: peers now come solely from `swarm_v2_federation/registry.json`, which is written from direct
probes, so a node appears only if it answered. Rebuilt and deployed (sha256 `e8ea2f3802c1179d`);
`/federation` now reports 4/4 measured peers and no phantom.

**2. The registry was stale and self-contradictory (FIXED).** It held three entries: the 6700XT
desktop as `offline since 2026-09-11` on port **8001** (nothing listens there), the other desktop as
`localhost:8021` (offline), and a third entry for a host that also appeared as the first. Rewritten
from measurements: 4 entries, each with measured latency, identity, `probed_at`, and the probe basis
stated. Backup: `registry.json.bak_pre_sync_20260924_110221`.

## The split that remains (needs the peers, not this host)

| node | agents | quorum | fabric trits | store bytes |
|---|---|---|---|---|
| local | **15** | **9** | **153,523,394** | 76,761,697 (measured) |
| DESKTOP-MEP4VQN | 14 | 10 | 241,800,000 | **null (unbacked)** |
| carruth-MacPro5-1 | 14 | 10 | 241,800,000 | **null (unbacked)** |

The peers run the pre-roster-change build, so the federation is split on the council itself, and their
fabric figure has no store behind it. **`POST /federation/sync` cannot be trusted as evidence**: it
returns `{"nodes_synced": 4, "pbft_quorum": "AGREED_14_0_0", "status": "synchronized"}` - a stub that
asserts agreement on the OLD roster while the mesh is split.

## No channel exists from here to fix the peers

MEP4VQN runs an SSH server but refuses this host's key (`Permission denied (publickey,password)`);
Tailscale SSH is not enabled on any peer (`desktop-vb1t79v` closed, `carruth-macpro5-1` refused).
Neither the user's passwords nor an unauthenticated push will be attempted. Prepared instead:
`deploy/federation_sync_20260924/` - the corrected kernel (`e8ea2f3802c1179d`), the measured registry,
`SHA256SUMS`, `DIVERGENCE.txt`, and `deploy_peer.bat` (stop -> install -> restart -> verify 15/9).
Two ways to land it: run the bundle on each peer directly, or enable `tailscale up --ssh` on the peers
so the mesh itself becomes the channel.

**Fabric sync is deliberately NOT bundled.** The peers' 241,800,000-trit figure has no store behind it,
but overwriting a peer's store with this host's 76,761,697 B `.k3` would replace whatever intelligence
that node holds. That is the operator's call, not a silent default.

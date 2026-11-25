# Platform scaling strategy

## Measured capacity

| dimension | now | what the suite assumes |
|---|---|---|
| nodes | 3 configured, 3 active (`/federation/stats`) | 10 |
| mesh nodes | 14 | 584 agents |
| resident models | 1 (gpt-oss-20b on :8587); Instella parked | several tiers |
| board | 27 cards | 1000 tasks |
| council latency | ~38 s per proposal, 14 votes | not specified |

## The scaling arithmetic

The suite's objectives assume ~580× more agents than exist. Two honest paths:

**Path A — grow the federation.** 3 → 10 nodes requires: a discovery mechanism that does not depend on a
hand-written peer file (today `config/federation_peers.json`), quorum thresholds that scale with membership
(today threshold 4 of 5 configured), and per-node attestation so a peer's claims can be checked. Each is
weeks, not days.

**Path B — keep 3 nodes and shrink the objectives.** Report the real number, everywhere, computed from
`/federation/stats` rather than written into documents. This costs nothing and is the only path that can be
finished this quarter.

## Recommendation

Path B now, Path A deliberately. Concurrency that is claimed but not present is the same failure mode as a
docking score that cannot be reproduced — an impressive number with no control behind it. The platform already
has the instrumentation to report its true scale; using it is the strategy.

---

*Task 983 — 983-platform-scaling-strategy. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

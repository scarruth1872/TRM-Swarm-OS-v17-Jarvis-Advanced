# Federation integration

## Measured now

- **3 of 3 configured peers active** (0 offline)
- **14 mesh nodes**, anchor `0x4D4F4E4144000001`
- quorum: threshold 4 of 5 declared peers, **quorum_reachable false** — the arithmetic does not close
- peer list is configuration-driven (`config/federation_peers.json`), replacing hardcoded 100/67/33

## What integrates

| path | state |
|---|---|
| state exchange between peers | works at the TCP/HTTP level (3 active) |
| sealed artifact transfer | demonstrated once end to end |
| shared ledger verification | **not implemented** — a receiver does not check the sender's chain |
| quorum decisions | **mathematically unreachable** at threshold 4 with ≤3 active |

## The honesty problem in numbers

A federation that reports "3 of 3 active" and also "threshold 4, quorum reachable false" is telling the truth
twice and contradicting itself once: either threshold or membership must change. Writing documents about 10
nodes and 584 agents on top of that produces the exact failure the board recorded — a claim that cannot be
satisfied by the machine it describes.

## Integration priority

1. Set the quorum threshold from measured membership, not declaration.
2. Implement receiver-side chain verification for transferred artifacts.
3. Only then describe multi-node behaviour in any document.

---

*Task 997 — 997-federation-integration. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

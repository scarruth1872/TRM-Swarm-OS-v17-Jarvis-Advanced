# Integration deployment (second instance)

Duplicate of Task 994: see `994-integration-suite.md`.

Deployment status of the integration work, stated as numbers rather than intentions:

| integration | deployed | proof |
|---|---|---|
| shard seals | yes | 33/33 payload hashes verified |
| kernel ↔ Jarvis | yes | Jarvis logs `answered via kernel:/swarm/chat` |
| board ↔ defects | yes | poster/closer operate on live endpoints |
| documents ↔ ledger | **no** | files have sha256, not ledger entries |
| cards ↔ evidence | **no** | reason is prose |
| peer ↔ peer verification | **no** | no receiver-side chain check |
| joins between shards | **no** | see 993 |

Three of seven integrated. The remaining four are the ones that would let a claim travel with its proof.

---

*Task 998 — 998-integration-deployment. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

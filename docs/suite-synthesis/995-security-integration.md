# Security integration

## What is wired end to end

| path | mechanism | verified |
|---|---|---|
| artifact → manifest → verifier | sha256 over payload after a 128-byte header | **33/33 match** |
| decision → ledger | hash-chained entries | chain verifies; sampling 1-in-500 |
| DDR monitoring | scan + taxonomy + persistent counter | 439 scans, 6/6 taxonomy ids |
| approval gate | advisory, veto-only, abstain on unknown | operational; blind run 5/12 |
| agent restart | scheduled task + guard, 7 listeners | kernel restarts; guard covers 7 routes |

## What is not wired

1. **A document has no ledger entry.** Files this platform writes are hashed into a manifest, not sealed into
   the chain, so their authorship is not attested.
2. **A card has no evidence link.** A board card's reason is prose; nothing joins it to the endpoint that
   proved it.
3. **Peer claims are unverified.** Sealed artifacts travel, but the receiver does not re-verify against the
   sender's ledger.

## The integration that would change the security posture

Sealing generated documents into the ledger. It costs one write path and it turns "this platform produced this
report" from a claim into a checkable fact — the same property the shards already have, extended to the
outputs.

---

*Task 995 — 995-security-integration. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

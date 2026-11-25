# Complete integration suite — index

| task | document | integration it covers |
|---|---|---|
| 991 | `991-platform-integration-report.md` | services ↔ endpoints |
| 992 | `992-drug-discovery-pipeline-integration.md` | structure → descriptors → model → prediction |
| 993 | `993-knowledge-integration.md` | shard ↔ shard joins |
| 995 | `995-security-integration.md` | seals, ledger, gate |
| 996 | `996-hardware-integration.md` | CPU/GPU/UEFI/telemetry |
| 997 | `997-federation-integration.md` | peers, quorum, mesh |
| 998 | this file (duplicate of 994) | — |
| 999 | `999-platform-assessment.md` | the whole platform against the 1000-task suite |
| 1000 | `1000-capability-report.md` | the capability answer, in one document |

## Integration standard applied to all of them

An integration is real when a number can travel through it: a measurement made in one component appears,
unmodified and with its provenance, in another. By that standard the platform has **one** working integration
of significance — the shard seal chain — and several that pass messages but not evidence.

---

*Task 994 — 994-integration-suite. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

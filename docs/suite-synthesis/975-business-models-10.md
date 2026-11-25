# 10 business models, with their measured preconditions

| # | model | what it sells, and the evidence it can | risk |
|---:|---|---|---|
| 1 | Sovereign deployment licence | One-time licence per site with source escrow; the platform runs air-gapped. Evidence it works: it already runs with zero cloud dependencies (16 shards, all local, no vendor calls in the report path). | high margin, slow sales |
| 2 | Evidence-as-a-service for repurposing | Charge per verified evidence package (target + analogs + resistance context + trial landscape), delivered with a provenance ledger the buyer can re-verify. | repeatable, needs the provenance story to be airtight |
| 3 | Regulatory dossier preparation | Sell the gap-analysis and section drafting for a fixed fee; the platform's value is that every claim carries a source or is marked judgement. | high willingness to pay, liability must be bounded |
| 4 | Hosted wet-lab bridge | Fixed-fee assay design packages (MIC panels, time-kill, synergy, biofilm) bought by academic labs lacking design capacity. | uses authored documents; quality control is the constraint |
| 5 | Federation subscription for small labs | Two nodes minimum, peer-reviewed evidence sharing; the customer keeps their data and joins a verification network. | network effect, low ARPU |
| 6 | Docking QC as a product | Sell the redock-control harness itself, because most published docking campaigns fail it. | small but sharply differentiated |
| 7 | Provenance audit of third-party claims | Audit a partner's dataset or model claims against their own artifacts; price per claim audited. | trust-building, hard to scale |
| 8 | Training and certification | Teach teams to run a measured platform: capability matrices, control gates, abstention semantics. | low capital, brand-dependent |
| 9 | On-premise compute bundling | Workstation plus platform plus datasets, pre-verified, for a lab with no cloud policy. | hardware margin thin, lock-in strong |
| 10 | Grant-funded consortium | Publicly funded evidence commons; the platform is the shared instrument, members contribute shards. | non-dilutive, slow |

## The uncomfortable finding

Nine of the ten models sell **evidence**, and the platform's evidence story has a hole: documents are not
sealed into the ledger, so a customer cannot verify that the platform produced a report rather than that a
report exists. That is feature 1 of 50, and it is worth more commercially than any of the ten rows above.

The one model that does not depend on it is docking QC — selling a harness whose value is that it *fails*
most published work — because a failing control is its own evidence.

---

*Task 975 — 975-business-models-10. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

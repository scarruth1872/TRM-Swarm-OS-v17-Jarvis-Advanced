# Innovation deployment

An innovation only deploys if it has an owner, a first measurable step and a kill criterion. This document
takes the proposals from 971-976 and applies those three tests to the ones that survive.

| proposal | owner | first measurable step | kill criterion |
|---|---|---|---|
| BindingDB × ChEMBL-LpxC join | data engineering | one table, 406 rows, affinity per compound | join yields <100 matched compounds |
| Redock control gate | chemistry | CHIR-090 RMSD < 2.0 Å in 5U39 | three receptor preps all fail |
| Conformal intervals on pKi | ML | empirical coverage at 90% nominal on held-out folds | coverage outside 85-95% |
| Structure registry | platform | canonical SMILES for 3 leads, hashed, one file | fewer than 3 parse |
| Claim register for documents | platform | 100 sentences with measurement ids | >30% of sentences have no citable source |
| Expert routing telemetry | inference | per-expert token counts over 100 requests | routing is uniform (no specialisation) |

## Deployment rule this document enforces

**No innovation is deployed without a control that can fail.** The redock gate, the coverage check and the
kill criteria are all versions of the same rule, and the platform's history shows why it is needed: the one
number the suite trusted most (a −10.170 kcal/mol docking score) turned out to come from an artifact with 324
atoms for a 36-atom ligand.

---

*Task 977 — 977-innovation-deployment. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

# Knowledge integration report

## The corpus as one object

7,793,536 records across 16 manifests, each verified by payload hash. The integration question is
not "is there data" but **"can a claim cross from one shard to another"**, and today the answer is mostly no:
the shards are loaded and queryable, but the joins between them exist only where someone wrote them.

| join | exists | would enable |
|---|---|---|
| compound → affinity (ChEMBL ↔ BindingDB) | no | potency expansion beyond 406 actives |
| compound → structure (ChEMBL ↔ PDBbind) | no | structure-based validation of the QSAR |
| target → resistance (LpxC ↔ CARD) | no | resistance neighbourhood of the target |
| target → pathway (LpxC ↔ SMPDB) | no | pathway placement of the mechanism |
| drug → trial (ChEMBL ↔ ClinicalTrials) | no | feasibility for a designed study |
| drug → adverse event (ChEMBL ↔ FAERS) | no | post-market context for a lead |

## Why this matters

Every claim the platform makes about a molecule currently rests on **one** shard. A claim that rests on a
join — potency plus structure plus resistance plus pathway — is the kind of evidence that survives review.
The corpus is large enough; the edges are missing.

## The integration standard

A join counts only when it is: (a) written as code in `tools/`, (b) reproducible, (c) produces a row count that
can be checked, and (d) names the shard seal for each side. Anything else is a diagram.

---

*Task 993 — 993-knowledge-integration. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

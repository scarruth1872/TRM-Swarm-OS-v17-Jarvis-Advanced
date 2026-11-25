# Drug-discovery pipeline integration

## The pipeline as it actually runs

```
ChEMBL LpxC (406 actives) --RDKit 3.12--> 8 descriptors
        |                                      |
        |                                      +--> sklearn 3.11 --> GBM model (CV5 R2 0.7226)
        |                                                             |
   pose REMARK SMILES (3 leads) ----> descriptors ------------------> pKi prediction + domain check
                                                             
BindingDB (3.24M) ---[JOIN NOT WRITTEN]---> affinity expansion
PDBbind (4,404)  ---[JOIN NOT WRITTEN]---> structure context
Tox21 (7,831)    ---[JOIN NOT WRITTEN]---> measured liability
vina 1.2.7       --- redock control FAILS (RMSD 13.82 A) ---> numbers not quotable
```

## What is integrated

- structure → descriptors → model → prediction with an applicability-domain verdict, end to end, in two
  commands;
- leads' structures recovered from artifacts rather than a registry (a gap, not an achievement);
- the model's error bar travels with every prediction.

## What is not

- **Three joins that would multiply the data 1000×** and none has been written;
- **the docking branch is broken** at the control, so no structure-based number can be trusted;
- **no wet-lab return path** — predictions are not compared against measured MICs because none exist here.

## The integration that matters most

BindingDB × ChEMBL-LpxC. It is one join, it turns 406 measurements into potentially thousands, and it costs a
day. Everything else in this pipeline is either done or blocked on hardware.

---

*Task 992 — 992-drug-discovery-pipeline-integration. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

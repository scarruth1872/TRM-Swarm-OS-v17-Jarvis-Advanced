# Drug-discovery knowledge synthesis

## The target and its measured ligand series

ChEMBL LpxC holds **406 actives with measured Ki values** (range 0.0089 nM upward). Descriptors were computed
for all 406 today with RDKit 2026.03.6: **406/406 parsed**, median MW
**386.47**, 97.8% carry zero Lipinski violations,
95.3% pass Veber, 87.0% pass Ghose.

## What is predictive, measured today

A gradient-boosted model on 8 descriptors predicts pKi with **CV5 R² 0.7226 ± 0.0565**
against a mean-predictor baseline of **-0.0085** (n=406, pKi
[3.86, 11.05]). That is a real model of this series — small, but not noise.

## The three leads, with structures recovered

The leads' chemistry was believed absent. It is not: each pose file carries `REMARK SMILES`, and two leads
have 3D SDFs whose canonical SMILES agree with the pose.

| lead | SMILES source | MW | logP | TPSA | Lipinski viol. | Veber | Ghose |
|---|---|---:|---:|---:|---:|---|---|
| TRM-ABX-220 | pose REMARK | 260.25 | 0.38 | 86.71 | 0 | ok | FAIL |
| TRM-ABX-219 | pose REMARK + 3D SDF (agree) | 399.43 | 1.16 | 126.64 | 0 | ok | ok |
| TRM-ABX-082 | pose REMARK + 3D SDF (agree) | 329.32 | -0.24 | 125.14 | 0 | ok | ok |

Predictions from the 406-compound model, with the domain check that decides whether to believe them:

| lead | predicted pKi | in domain? | reading |
|---|---:|---|---|
| TRM-ABX-220 | 5.22 | **no** — MW and logP outside 2σ | extrapolation; treat as unknown |
| TRM-ABX-219 | 8.88 | yes | the only lead the model can speak about, and it speaks well |
| TRM-ABX-082 | 5.68 | **no** — logP outside 2σ | extrapolation; treat as unknown |

## What synthesis this supports

1. **219 is the lead to defend**: in-domain prediction pKi 8.88, Lipinski 0, Veber and Ghose pass, SMILES
   corroborated by two independent files. It is also the largest of the three (MW 399), which is where the
   training series sits.
2. **220 and 082 need chemistry, not models**: both are outside the model's range (19 and 24 heavy atoms
   against a median of 386 Da). A prediction of 5.2 or 5.7 from a model that never saw anything that small is
   a number, not evidence.
3. **Docking must not be quoted yet**: the redock control fails (RMSD 13.82 Å) and the shipped receptor holds
   a protein atom 0.36 Å from the ligand. Any affinity from this box is currently unfalsifiable.
4. **Cross-shard joins are the unexploited asset**: BindingDB (3.24M affinities) and PDBbind (4,404
   structures) are not yet joined to the LpxC series. That join is the cheapest route from 406 to a real
   structure-activity argument.

---

*Task 962 — 962-drug-discovery-synthesis. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

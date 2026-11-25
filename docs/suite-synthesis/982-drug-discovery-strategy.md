# Drug-discovery strategy

## The asset

Three leads with recovered structures, one target with 406 measured actives, and a QSAR that predicts at
CV5 R² 0.7226. That is a real, if small, discovery platform.

| lead | SMILES source | MW | logP | TPSA | Lipinski viol. | Veber | Ghose |
|---|---|---:|---:|---:|---:|---|---|
| TRM-ABX-220 | pose REMARK | 260.25 | 0.38 | 86.71 | 0 | ok | FAIL |
| TRM-ABX-219 | pose REMARK + 3D SDF (agree) | 399.43 | 1.16 | 126.64 | 0 | ok | ok |
| TRM-ABX-082 | pose REMARK + 3D SDF (agree) | 329.32 | -0.24 | 125.14 | 0 | ok | ok |

## The strategy, in order of evidentiary strength

**1. Defend 219.** It is in-domain for the model (pKi 8.88), passes Lipinski/Veber/Ghose, and its structure is
corroborated by two independent files. Its nearest competitor claim (220 at pKi 5.22) is an extrapolation that
the model itself flags.

**2. Give 220 and 082 chemistry, not models.** 19 and 24 heavy atoms against a training median of 386 Da means
the model has no basis for an opinion. The honest next step is measured: synthesise and assay, or find
close analogs in ChEMBL-37 (2.9M) to build a local series.

**3. Do not quote docking until the control passes.** The current redock fails at RMSD 13.82 Å, the shipped
receptor overlaps the ligand, and the one artifact on disk claiming −10.170 kcal/mol has 324 atoms where the
ligand has 36. Every affinity number in circulation is therefore unfalsifiable.

**4. Use the corpus for what it uniquely gives.** BindingDB (3.24M) + PDBbind (4,404) + Tox21 (7,831) +
CARD (6,463) can turn a small lead series into a defensible argument about potency, liability and resistance
risk — none of which requires a single new experiment.

## What would change the strategy

A returned MIC panel. The platform predicts; it has never measured a bacterium. Until an assay returns a
number, the strategy is a plan for prioritising experiments, and should be described that way to any partner
or funder.

---

*Task 982 — 982-drug-discovery-strategy. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

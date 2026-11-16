# TRM-ABX-220 Provenance Reconciliation
**Generated:** 2026-09-20T19:08:33.621254+00:00  **Lead:** TRM-ABX-220  **Revision:** 2 (supersedes `0fd85de141eb12f13197099dff101b133f90dee503a50d99c11817eec5d0003a`)  **Status:** `REVISED_UNRESOLVED_PENDING_STRUCTURE_AND_EXPERIMENT`
## 1. Why this reconciliation was necessary
TRM-ABX-220 was withheld from the wet-lab dispatch: it has no structure on record, yet was circulated with a specific sub-nanomolar affinity and a 'Confirmed' verdict. This report derives what can honestly be said from the canonical ledgers.
## 2. Findings
### 1. INVALID_CONFIDENCE_INTERVAL *(HIGH)*

Declared ci_95 [0.1, 0.12] excludes its own mean (0.0833 nM). A 95% CI must contain the mean. Recomputing from the same values (n=3, sd=0.0351, t=4.303) gives [-0.0039, 0.1706] nM; the lower bound is <= 0, so the sub-nanomolar claim is not statistically resolvable from these three points at all.
### 2. UNSOURCED_VERDICT_NUMBER *(HIGH)*

Verdict 'VALIDATED_POTENT_LPXC_INHIBITOR (Ki ~ 4.2 nM Confirmed)' cites 4.2 nM, which appears nowhere in this ledger's data ([0.05, 0.08, 0.12] nM). 4.2 nM is the ESKAPE-ledger mean of the LpxC organisms [3.6, 3.8, 4.1, 4.5, 4.8] (mean 4.16 nM) - a figure spliced in from a different ledger and labelled 'Confirmed'.
### 3. ELECTROSTATIC_TERM_DOUBLE_COUNTED *(HIGH)*

delta_g_calibrated adds electrostatic_dg = -4.0 kcal/mol on top of AutoDock Vina's base score, but Vina's own scoring function already includes an electrostatic term. The constant is byte-identical across all 3 structures, indicating a hardcoded correction rather than a per-structure computed value. A -4.0 kcal/mol spurious bonus alone equals a factor of ~856 in affinity.
### 4. PIPELINE_SCALE_MISMATCH *(HIGH)*

This ledger places TRM-ABX-220 at 0.083 nM (mean); the ESKAPE ledger places the same compound on the same target at ~4.16 nM - a ~50x discrepancy. The ground-truth-calibrated pipeline is the tie-breaker: its validated leads match ESKAPE exactly (TRM-ABX-219 -> 14.2 nM, TRM-ABX-082 -> 18.7 nM) and TRM-ABX-220 is ABSENT from it. The sub-nanomolar scale is the outlier, not the ESKAPE scale.
## 3. What the numbers actually say
| Quantity | Value | Source |
|---|---|---|
| Ledger Kd values | [0.05, 0.08, 0.12] nM | trm_abx_220_recalibration.json |
| Mean (as published) | 0.08 nM | declared |
| Mean (recomputed) | 0.0833 nM | this report |
| CI declared | [0.1, 0.12] nM | declared - invalid |
| CI recomputed | [-0.0039, 0.1706] nM | this report |
| ESKAPE LpxC Kds | [3.6, 3.8, 4.1, 4.5, 4.8] nM | eskape_spectrum_full.json |
| ESKAPE mean | 4.16 nM | eskape_spectrum_full.json |
| Discrepancy | ~50x | this report |
| TRM-ABX-219 (validated) | 14.2 nM | docking_calibration_benchmark.json |
| TRM-ABX-082 (validated) | 18.7 nM | docking_calibration_benchmark.json |
| TRM-ABX-220 in calibration of record | no | docking_calibration_benchmark.json |

## 4. Defensible position
- TRM-ABX-220 is an **in-silico LpxC hypothesis**, not a characterised lead.
- Its pipeline-consistent potency is **3.6 - 4.8 nM** (mean 4.16 nM) - the same calibrated scale as the validated leads.
- The sub-nanomolar figures are **retracted** as pipeline artifacts.
- No structure exists; **no synthesis or assay RFP can cover it** until one is deposited.
## 5. Required to resolve
1. A defined structure (SMILES/InChI) with a canonical `sha256` lock.
2. Re-running the recalibration pipeline **without** the hardcoded electrostatic term.
3. Experimental Ki against LpxC, and a check against the ESKAPE scale.

## 6. Structure provenance audit (added after the docking-protocol investigation)

Two further defects were found once the docking harness was validated, both reinforcing why ABX-220 must stay withheld.

**6.1 The three "structures" cited by this ledger do not exist here.** `structures_evaluated` lists PDB ids `3P3G`, `2XCT` and `1S16`. None of those files are present in the repository (only `3TYZ`, `5U39` and `5U3B` are). Those entries are **receptor** structures (LpxC from three organisms) — they are not ABX-220's molecule, and they cannot be re-examined. Note also that all three carry the identical `electrostatic_dg: -4.0`, `hbond_dg: 0.0`, `torsional_dg: 1.4`, the hardcoded constants behind the potency inflation already documented in §2.

**6.2 The benchmark was docking a different molecule under ABX-220's name.** `run_federation_realworld_benchmark.py` listed a compound called `TRM-ABX-220` with the SMILES `CN1CCN(CC1)C2=CC=C(C=C2)C3=CC(=NO3)C(=O)NO` (C15N4O3, a piperazine/oxadiazole) while actually docking `TRM-ABX-219-H1.pdbqt`, whose own `REMARK SMILES` header declares `CC1=C(C(=O)NO)C(=O)N(Cc2ccccc2)C1=O` (C13N2O4, a benzyl succinimide hydroxamate). **One name, two different molecules, neither of them a verified ABX-220 structure.**

That entry has been relabelled to `TRM-ABX-219-H1` — the molecule actually present in the file — and `TRM-ABX-220` is now recorded in the report's `excluded_compounds` with reason `NO_MOLECULAR_STRUCTURE_ON_RECORD`. The exclusion is stated rather than silently backfilled with an analog.

**Conclusion unchanged and now better supported:** ABX-220 has no SMILES, no InChI and no ligand coordinates anywhere in the canonical record. It is an affinity hypothesis attached to a name, not a characterisable compound. To bring it into scope it needs a defined molecular structure; until then no figure may be published under its name, and no analog may be presented as it.

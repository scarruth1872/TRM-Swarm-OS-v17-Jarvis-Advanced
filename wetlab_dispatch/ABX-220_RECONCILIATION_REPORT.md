# TRM-ABX-220 Provenance Reconciliation
**Generated:** 2026-09-10T11:02:20.973881+00:00  **Lead:** TRM-ABX-220  **Revision:** 2 (supersedes `0fd85de141eb12f13197099dff101b133f90dee503a50d99c11817eec5d0003a`)  **Status:** `REVISED_UNRESOLVED_PENDING_STRUCTURE_AND_EXPERIMENT`
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

## 6. Related artifacts
- `DRUG_DISCOVERY_AND_SECURITY_ADVANCEMENTS.md` — two corrupted structure locks (219, 082) replaced with the canonical values.
- `docking_calibration_benchmark.json` — unchanged; it is the calibration of record, and it contains **no** TRM-ABX-220 entry.
- `eskape_spectrum_full.json` — unchanged; its TRM-ABX-220 values are pipeline-consistent and are the basis of the retained estimate.
- `hybrid_intelligence_6_tier_60_tasks_ledger.json` — a **synthetic demonstration corpus** (100% pass rate, placeholder signature). Its TRM-ABX-220 affinity figure is illustrative and is superseded by this reconciliation. Left unmodified so its declared merkle root stays valid; flagged here instead.
- `wetlab_outreach_packages/`, `wetlab_dispatch/` — TRM-ABX-220 remains **WITHHELD**.

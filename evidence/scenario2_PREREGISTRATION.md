# PROOF SCENARIO 2 - PREREGISTRATION (written BEFORE docking)

Timestamp: 2026-09-28T19:13:25+00:00

## What is being tested

Whether the vina docking pipeline PREDICTS measured potency on molecules it has
never seen. This is a prospective test, not a re-analysis: the holdout molecules
were selected and this rule fixed BEFORE any of them were docked.

## Corpus

- Target: P. aeruginosa LpxC (CHEMBL3855). Receptor 5U39 (LpxC, EC 3.5.1.108,
  residues 2-288, 1.75 A), the same receptor used for the training set.
- Training set: 32 molecules already docked (22 actives + 10 measured weak) -
  these produced the -9.0 threshold used below.
- HOLDOUT: 38 molecules sampled from the 477 ChEMBL LpxC molecules with clean
  exact-relation nM measurements that were NEVER docked. Stratified:
  15 potent (pChEMBL>=8), 10 mid (6-8), 8 weak (4-6), 5 inactive (<4).
- Labels are written to scenario2_holdout.json and will NOT be read until after
  docking completes. The prediction below is fixed now.

## The prediction rule (FIXED BEFORE SEEING ANY RESULT)

Derived from the training set, where the best measured-inactive scored -10.360:
a single-threshold rule cannot work, so the training analysis concluded
"NO PHOENIX target may be reported as a binder on this evidence."

Therefore the PRIMARY preregistered metric is NOT accuracy. It is the
CORRELATION between docking score and measured potency:

    H1: Spearman rho(score, pChEMBL) over the 38 holdout molecules is
        NEGATIVE and |rho| >= 0.3     -> the pipeline carries real signal
    H0: |rho| < 0.3                    -> the pipeline does not rank these
                                          molecules usefully

Secondary, preregistered: using a cut at -9.0 kcal/mol (no training decoy passed
it), report sensitivity and specificity for detecting pChEMBL >= 7. Because the
training set showed a measured inactive at -10.36, I PREDICT this cut will show
POOR specificity - if it does not, that contradicts the training evidence.

## What would falsify the claim

- If rho >= 0 or |rho| < 0.3, the docking score does not rank potency.
- If specificity at -9.0 is high, my earlier "no threshold works" conclusion is
  wrong for this holdout and must be revised.
- If docking fails to produce scores for a majority of the 38, the pipeline is
  not applicable at this scale.

## Method commitments made in advance

- Same receptor file, same box as the training docks (5U39, centre
  -8.126/-13.520/21.157, size 22x24x25), exhaustiveness 8, seed 42.
- All 38 docked regardless of intermediate results; no early stopping.
- The reported rho is computed on whatever the docks return, not on a subset.
- If the result is negative, it is reported as negative.

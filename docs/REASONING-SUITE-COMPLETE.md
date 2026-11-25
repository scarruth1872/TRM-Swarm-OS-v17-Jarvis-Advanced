# The complete reasoning suite — measured, not claimed (task 579)

Generated from `/reason/suite` and the harnesses it names. Every row says what exists, where it lives, what
proves it, and **which number it carries**. A row with no number is a row with no evidence.

## Status

| task | module | endpoint | measured result | verdict |
|---|---|---|---|---|
| **571** forward chaining | `reasoning_engine.rs` | `POST /reason/forward` | 406/406 agreement with RDKit on two profiles | **works** |
| **572** backward chaining | `reasoning_engine.rs` | `POST /reason/prove` | derivation tree to facts; UNKNOWN names the premise | **works** |
| **573** case-based | `case_base.rs` | `POST /reason/case` | LOO k=5 RMSE **0.9181** vs mean **1.6259** (R² 0.6796) | **works** |
| **574** analogical | `analogical_reasoning.rs` | `POST /reason/analogy` | LOO RMSE **1.1984** vs zero-delta **1.0235** — **17% worse** | **REFUTED** |
| **575** abductive | `abduction.rs` | `POST /reason/abduce` | **27/27** exact on real Ghose cases; no size-1 explanation of a 2-premise rule | **works** |
| **576** default reasoning | `reasoning_engine.rs` | `POST /reason/forward` | defeat by exception; UNKNOWN assumption named in `assumptions` | **works** |
| **577** temporal | `temporal_reasoning.rs` | `POST /reason/temporal` | 400/400 pairs, 300/300 triples sound on real ledger timings | **works** |
| **578** spatial | `spatial_reasoning.rs` | `POST /reason/spatial` | 13,288 contacts, checksum identical to numpy | **works** |
| **579** complete suite | `reason_routes.rs` | `GET /reason/suite` | 10/10 modules present on disk, 7/7 endpoints answer 200 | **works** |
| **580** explainable reports | `reasoning_engine.rs` | `POST /reason/prove` | every conclusion carries a derivation to its evidence | **works** |

Eight of ten work. One is refuted. One is the index.

## 574 is refuted, and that is the honest result

The proposal to the council pre-registered its acceptance test: *analogy must beat zero-delta (copying the
nearest neighbour's outcome), and the random-outcome control must show no such gain*. Measured on leave-one-out
over 406 real ChEMBL LpxC actives:

```
analogy RMSE     1.1984
zero-delta RMSE  1.0235     <- copying the neighbour is BETTER
mean RMSE        1.6259
R²               0.4541
clamped          3.9% of predictions lay beyond the observed transformation
control (seed 42): analogy 2.5739 vs zero-delta 2.2641  (no gain either)
```

The mechanism is implemented, tested (10 unit tests), and exposed — but it **does not do what it was proposed to
do**. Extending a measured change along the best-aligned transformation adds more noise than the change it
carries, because the descriptor→pKi relationship is weak to begin with (the best model on this cohort is a GBM
at CV R² 0.7226, and the CBR gets 0.6796 by not extrapolating at all).

The endpoint reports this itself: `worse_than_zero_delta: true`. Nothing in the platform now claims analogical
reasoning is a working capability; it is a tested hypothesis that lost. Improving it after seeing these numbers
would be tuning on the test set, so it is left as measured.

## 575 works, and its minimality was checked independently

- **27 of 27** compounds whose RDKit heavy-atom count violates the Ghose rule were explained by exactly the
  assumption matching which side of the range they fell on — no mismatches.
- **Minimality**: for a conclusion whose rules require two premises, no single assumption explains it (0 of 5
  tried). The size-2 answer is therefore minimal, not merely the first one found.
- **Alternatives are published**: the size-2 abduction reports `alternatives: 9` — nine other two-assumption
  sets explain the same observation equally cheaply. An abduction that showed only its favourite would be
  hiding information.

## What the suite deliberately cannot do

- **No first-order reasoning anywhere.** Atoms are opaque strings; there are no variables and no unification.
  A rule about "any compound with logP > 5" must be written per compound.
- **No learning of rules.** Every rule base here is authored. The engine derives from what it is given.
- **No negation-as-failure, no counting, no arithmetic.** Stated in `reasoning_engine.rs` and enforced by the
  tests that refuse to fire on an UNKNOWN premise.

## Reproduce everything

```
python tools/reasoning_verify.py            # 571/572/576/580 vs RDKit (406 compounds)
python tools/reasoning_cbr_verify.py        # 573 leave-one-out
python tools/reasoning_analogical_verify.py # 574 — exits 1, because it is refuted
python tools/reasoning_abduction_verify.py  # 575 real cases + minimality
python tools/reasoning_temporal_verify.py   # 577 real ledger intervals
python tools/reasoning_spatial_verify.py    # 578 vs numpy on PDBQT coordinates
python tools/reasoning_suite_verify.py      # 579 the index itself
```

`reasoning_analogical_verify.py` returning non-zero is the suite working: a harness that can only pass is not a
harness.

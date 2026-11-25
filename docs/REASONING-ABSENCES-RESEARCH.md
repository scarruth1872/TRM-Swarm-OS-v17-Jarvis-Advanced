# The five reasoning absences — what could ground each, and what would verify it

Researched 2026-11-25 against what is on this machine. Each row names the asset, the method, and — the part
that matters — **how a built capability could be proved wrong**. A capability that cannot fail a check is not
being measured.

| task | asset on this host | method | verification that could fail |
|---|---|---|---|
| **573** case-based reasoning | **406** ChEMBL LpxC actives with SMILES, 8 RDKit descriptors and measured pKi (`evidence/chembl_lpxc_actives.json`, `data/suite_lpxc_descriptors.json`) | k-nearest cases in descriptor space, then **delta adaptation**: predict the query by transferring the neighbour's measured pKi and correcting for the descriptor difference | leave-one-out RMSE must beat the mean predictor; and the adapted prediction must beat the un-adapted neighbour |
| **574** analogical reasoning | the same 406 compounds, as **structures** (SMILES), not just vectors | matched molecular pairs: find a shared transformation between two compounds and transfer the property delta along it | held-out pair prediction must beat a zero-delta baseline; a random-pair control must not |
| **575** abductive reasoning | `data/eskape_candidates.json`, `data/eskape_proteomes.json` as a hypothesis space; the propositional engine from G249 as the entailment check | **minimal abduction**: find the smallest set of assumptions that, added to the rule base, derives the observation | exhaustive enumeration on small rule bases must confirm the returned set is minimal; a larger returned set is a failure |
| **577** temporal reasoning | `data/council_ledger.json` — **1,586** proposals with unix timestamps, phases and durations | Allen's 13 interval relations, composed algebraically | the algebra's answer must equal the arithmetic comparison on the real timestamps for every sampled pair, and durations must reconcile |
| **578** spatial reasoning | `5U39_receptor.pdbqt` (**4,477** atoms) and `3TYZ_receptor.pdbqt` (**8,372** atoms) — real 3D coordinates | contact / containment relations over atom sets (RCC-8-style topology plus distance queries) | the contact graph must match an independent distance computation exactly; a query "residues within 4 Å of the ligand" must match the same query done with numpy |

## Cost, honestly

- **577 and 578 are pure functions.** No new data, no model, no network. Cheap, and fully checkable — the
  strongest candidates.
- **573 and 574 need RDKit** (present in Python 3.12 only) and produce measurable prediction error. They are
  cheap to build and easy to falsify, but they will not beat the existing GBM (CV R² 0.7226); their value is a
  different capability — a reasoning trace a chemist can argue with, not a better number.
- **575 needs a hypothesis space that is real.** `eskape_candidates.json` gives one, but the abduction will
  inherit the weakness of that file: if the candidate list is thin, the abduction is thin. Worth building only
  if the minimality check is enforced, since that is the part that cannot be faked.

## What is still missing after any of this

None of these turn the platform into a general reasoner. All five are **propositional or numeric**:
- no first-order variables or unification (573-578 included) — a rule mentioning "any compound with logP > 5"
  is written per-atom, not quantified;
- no temporal projection beyond interval relations (577 says "A before B", not "B will occur");
- no spatial simulation (578 says "these sets touch", not "this ligand moves here");
- no learning of rules — the rule base is authored, and an authored rule base is only as good as its author.


---

# Council decisions, and what was built

Five proposals were put to the council with their evidence and their acceptance test (proposal ids 2587-2591 in
`data/council_ledger.json`). The verdicts are the platform's own, enforced by the distinct-voter quorum added in
G246 — `min_approve` is 10 out of 14:

| task | proposal | verdict | approve / abstain / reject | built? |
|---|---|---|---|---|
| **577** temporal | 2590 | **ENACTED** | 14 / 0 / 0 | **yes** |
| **578** spatial | 2587 | **ENACTED** | 10 / 4 / 0 | **yes** |
| **573** case-based | 2591 | **ENACTED** | 10 / 4 / 0 | **yes** |
| **574** analogical | 2588 | **NO_CONSENSUS** | 8 / 4 / 2 | **no — below threshold** |
| **575** abductive | 2589 | **UNDECIDED** | 8 / 6 / 0 | **no — below threshold** |

An 8-approval proposal does not reach `min_approve` 10, so 574 and 575 were **not built**. Their research stands
in the table above; the council declined to spend the work, and that decision is recorded rather than overridden.

## 577 — temporal reasoning (`src/temporal_reasoning.rs`)

Allen's 13 relations over half-open `[start, end)` intervals, plus composition answered as the *set* of possible
relations. Composition is computed by constructing **witness intervals** on a bounded grid, not by transcribing a
169-cell table — a table that large is an artefact waiting to drift.

Verified on **real** ledger timings:

```
ledger entries 1591 | distinct timestamps 652 | windows 651
pair check : 400 pairs | agree 400 | disagree 0     (vs an independent Python implementation)
triple check: 300 triples | sound 300 | violations 0 (relation(A,C) ∈ compose(r(A,B), r(B,C)) on real data)
RESULT: PASS
```

The triple check tests the algebra against the data: a composition set that admits too few relations fails there.
Note 652 distinct timestamps among 1,591 entries — most seeded records share a timestamp, so the windows are
formed from distinct real timings and the report says so.

## 578 — spatial reasoning (`src/spatial_reasoning.rs`)

Contact graphs and proximity queries over real coordinates, with squared-distance predicates (boundary included,
`d² ≤ cut²`) so no epsilon can be tuned until a test passes. `no_std` has no `sqrt`, so one is implemented by
Newton's iteration — and the contact decision never uses it.

Verified against numpy on `5U39_receptor.pdbqt`:

```
parsed 4478 atoms
kernel : contacts 13288 | checksum 29114098492237
numpy  : contacts 13288 | checksum 29114098492237
near query: kernel 98 | numpy 98 | match True
RESULT: PASS
```

The checksum (`Σ i·1000003 + j`) is the stronger half of that: a count alone would miss a single swapped pair.

## 573 — case-based reasoning (`src/case_base.rs`)

Two of the four CBR phases are implemented and two are declared absent: **retrieve** (k-NN under z-scored
Euclidean distance) and **reuse** (distance-weighted interpolation across the retrieved cases). **Revise** and
**retain** are not, and the header says so.

Verified by leave-one-out inside the kernel over 406 real compounds:

| k | RMSE | mean-predictor RMSE | R² |
|---|---|---|---|
| 3 | 0.9509 | 1.6259 | 0.6563 |
| **5** | **0.9181** | 1.6259 | **0.6796** |
| 9 | 0.9278 | 1.6259 | 0.6728 |

Beats the baseline, and is **not** as good as the GBM's CV R² 0.7226 — reported here rather than omitted. A
held-out query (case removed from the base first) predicts 7.38 against an actual 11.05, an error of 3.67: the
highest-potency compound in the set is an outlier, and the tool says so instead of hiding it behind the mean.

## What remains absent after this

| task | why it is still absent |
|---|---|
| **574** analogical | council: no consensus (8/4/2). Assets exist (406 structures, matched pairs); the decision was not to spend on it |
| **575** abductive | council: undecided (8/6/0). The hypothesis space is real but thin, which the proposal itself flagged |
| 579 complete suite | 7 of 10 members now exist; the umbrella is still only as real as its parts |

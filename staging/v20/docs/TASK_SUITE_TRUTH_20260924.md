# Truth in the 1000-task suite: what it actually contains, and what gates it (2026-09-24)

Source: `C:\Users\shawn\Downloads\# TRM Swarm OS  Complex Multi.txt`, parsed to
`scratch/task_suite_parsed.json` and classified per task in
`scratch/task_suite_domain{1..5}_classified.json` (467 rows, verified: no gaps, no duplicates,
no schema violations).

## Claims vs contents

| the file says | what it holds |
|---|---|
| "1000 Tasks = 10 Domains x 100" | **467 tasks**, 5 domain headers, truncated mid-sentence at Task 467 |
| "TRM-LAP-3070 (Primary) + TRM-DESK-6700XT (Federation)" | one host; no 6700XT node is present or reachable |
| "no mocks, no simulations... all tasks produce verifiable, real artifacts" | artifacts exist for **143/467**, and those were generated (seeded RNG, hardcoded metrics) |
| tasks are self-contained | **zero explicit cross-task references**; dependency is implicit by domain logic |

## Census by domain (per-task reading, not keywords)

| domain | tasks | locally satisfiable | gated | dominant gate |
|---|---|---|---|---|
| 1 Drug Discovery & Biomedical | 100 | 21 | 79 | EXT_DATA / ANALYSIS / TRAINED / ENGINE |
| 2 Genomics & Variant Analysis | 100 | **0** | 100 | EXT_DATA (78) |
| 3 Network & Systems Engineering | 100 | 92 | 8 | IMPL (86) |
| 4 Security & Cryptography | 100 | 72 | 28 | IMPL (64) |
| 5 Data Science & ML (truncated) | 67 | 41 | 26 | IMPL (39) |
| **total** | **467** | **226** | **241** | |

## The finding that matters: artifacts do not track feasibility

| | locally satisfiable | knowledge-gated |
|---|---|---|
| has artifacts | 71 | **72** |
| no artifacts | 155 | 169 |

Artifact rate **31% vs 30%** - statistically indistinguishable. Output was produced at a
constant rate whether or not the task could be done. Two consequences:

- **72 artifacts claim results for tasks that cannot be completed on this host** - they are
  fabrications, not completions.
- **155 locally-satisfiable tasks have no artifact at all** - the platform's real capacity was
  left unused while impossible tasks were "answered".

## The catalyst: 241 gated tasks reduce to one question - can the missing knowledge be obtained?

| | tasks | examples |
|---|---|---|
| **acquirable here** (download / install / train) | **147** | public omics corpora (24), model weights (24), gnomAD/1000G (15), docking engine (7), PharmGKB/CPIC (7), ClinVar (6) |
| **not acquirable by a machine** (samples / labs / hardware / consent) | **55** | patient/cohort NGS (19), malware corpus in an isolated lab (13), security hardware TPM/HSM (7), second federation node (7), mass spectrometer (4) |
| **held only by the operator** | ~39 | the identity of **TRM-ABX-220** (no SMILES or structure anywhere on disk); LpxC has no structure here |

## Truths that fell out of the validation

- **The target does not exist on this host.** The 9 local PDB structures are PI3Kgamma
  (1E8X, 2A4Z), EGFR (1M17, 1XKK), PI3Kalpha (2RD0) and tau PHF (5O3L) - **none is LpxC**, the
  target of the entire first domain. `1M17_prot`, `2A4Z_prot` and `receptor_clean` are duplicates.
- **The lead compound is undefined.** TRM-ABX-220 - named in the first 100 tasks - has no SMILES,
  structure or measured property anywhere on disk. Only TRM-ABX-219 and TRM-ABX-082 have SMILES.
- **Knowledge, not capability, is the only thing gating this suite.** Every gated task's blocker is
  a named missing artefact, engine, dataset or physical object - never an algorithmic unknown.
- **The one irreducible catalyst**: of the 241, 147 are unblocked by acquisition the platform can
  perform itself, 55 require the physical world, and ~39 need one input only the operator holds.

# The 1000-task suite against this machine — what actually runs

Source: `~/Downloads/TRM Swarm OS — 1000 Complex Multi-Step Task Suite.txt` (1,000 tasks parsed, 100 per
domain, no gaps, no duplicate ids). Classifier: `tools/suite_classify.py`. Per-task record with the rule that
fired: `data/suite_classification.json`. Probes: `tools/suite_probe_lpxc.py`, `tools/suite_probe_qsar.py`,
`data/suite_probe_lpxc.json`, `data/suite_probe_docking.json`.

Measured 2026-11-24 on TRM-DESK-6700XT, with TRM-LAP-3070 reachable over SSH (`100.123.215.119`).

## Verdict

**89 of the 1,000 tasks can be run here and checked** — 3 already delivered, 32 that compute from data on
disk, 54 that are code you can build and test. **74 are blocked at the molecule** they name, 42 at missing
data, 29 at missing hardware, 10 at missing scale. **677 produce a document that nothing verifies.**

That last number is the finding that matters. The suite's closing claim — *"Every task produces verifiable
artifacts… K3 truth state… the knowledge will grow"* — holds for about 9% of it as written. The other 68% are
writing tasks whose correct output is a file that exists, and no part of the platform can tell a good one from
a bad one.

## Classes

| class | meaning | count |
|---|---|---|
| **P** | proven on this host (built and verified) | 3 |
| **A1** | a measurement computable from data on disk | 32 |
| **A2** | an implementation buildable here, verifiable by `cargo build` + test | 54 |
| **B** | a tool runs but its control fails | 2 |
| **C** | artifact only — a document; existence is the only check | 677 |
| **N** | normative — no measurement can settle it | 77 |
| **L** | blocked at the molecule: TRM-ABX-220 has no SMILES | 74 |
| **D** | blocked: dataset not on this host | 42 |
| **E** | blocked: hardware not on this host | 29 |
| **F** | blocked: scale not present | 10 |

## Per domain

| domain | P | A1 | A2 | B | C | N | L | D | E | F |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 Drug Discovery & Biomedical | 0 | 3 | 0 | 0 | 21 | 1 | **71** | 4 | 0 | 0 |
| 2 Genomics & Variant Analysis | 0 | 0 | 0 | 0 | 64 | 4 | 0 | **32** | 0 | 0 |
| 3 Network & Systems Engineering | 2 | 0 | 28 | 0 | 65 | 0 | 0 | 0 | 1 | 4 |
| 4 Security & Cryptography | 0 | 0 | 7 | 0 | 86 | 2 | 0 | 0 | 4 | 1 |
| 5 Data Science & Machine Learning | 0 | 8 | 13 | 2 | 69 | 1 | 0 | 6 | 1 | 0 |
| 6 Knowledge Engineering | 0 | **21** | 2 | 0 | 77 | 0 | 0 | 0 | 0 | 0 |
| 7 Hardware & Embedded Systems | 1 | 0 | 0 | 0 | 77 | 0 | 0 | 0 | **22** | 0 |
| 8 Federation & Distributed Systems | 0 | 0 | 3 | 0 | 92 | 0 | 0 | 0 | 0 | 5 |
| 9 Humanistic & Ethical Reasoning | 0 | 0 | 0 | 0 | 32 | **68** | 0 | 0 | 0 | 0 |
| 10 Creative & Generative Work | 0 | 0 | 1 | 0 | **94** | 1 | 3 | 0 | 1 | 0 |
| **total** | **3** | **32** | **54** | **2** | **677** | **77** | **74** | **42** | **29** | **10** |

## What is actually on this host (each line measured)

**Data** — 33 shard manifests, **10,824,989 records**: ChEMBL-37 2,897,805 (+ descriptors), BindingDB
3,243,585, ClinicalTrials 612,025, FAERS 2026Q3 421,404, ARC 291,778, SMPDB 147,932, Orange Book 48,990,
ChEMBL real 44,401, TPTP 28,346, ESKAPE 25,888, NCBI-AMR 8,143, Tox21 7,831, CARD 6,463, PDBbind 4,404,
**ChEMBL LpxC actives 406**.

**Tools present** — RDKit 2026.03.6 (py3.12), numpy 1.26.4 / scipy 1.10.1 / pandas 2.0.2 / scikit-learn
1.3.2 / networkx 3.4.2 / torch 2.5.1+cpu (py3.11), Biopython 1.88 (py3.12), AutoDock Vina **1.2.7**,
the Rust toolchain (141 tests pass), llama.cpp serving gpt-oss-20b on `:8587` and embeddings on `:8593`.

**Tools absent** — OpenMM / MDAnalysis / mdtraj (no MD), meeko / openbabel (no ligand prep in Python),
statsmodels, any OWL/SPARQL reasoner, any vision weights, any protein-folding weights.

**Hardware** — desktop: Ryzen 9 3900X, RX 6700 XT (Vulkan only, **no CUDA**), 32 GB. Laptop: Ryzen 9 5900HX,
RTX 3070 Laptop, 15 GB, **nvcc absent**. `Get-Tpm`: **TpmPresent False**. No FPGA, SDR, muon detector, robot
or IoT device.

**Scale** — `/federation/stats`: 3 configured peers, 3 active, 14 mesh nodes, monad anchor
`0x4D4F4E4144000001` (matches the suite's anchor). Not 584 agents, not 10 nodes.

**One interpreter fact worth carrying**: py3.12 has RDKit but its scikit-learn is built against numpy 1.x while
numpy 2.0.2 is installed (`numpy.dtype size changed`). Any task mixing RDKit and sklearn needs the
two-interpreter chain used below — a capability that only works that way is recorded that way.

## Proofs run today (not planned — executed)

**Task 9 class — drug-likeness, real molecules.** RDKit on the 406 ChEMBL LpxC actives: 406/406 parsed
(0 unparseable), **97.78%** with zero Lipinski violations, **95.32%** Veber-pass, **86.95%** Ghose-pass,
median MW 386.47. This is the computation Task 9 asks for; it runs here.

**Task 10 class — QSAR.** Gradient boosting on 8 descriptors → pKi, 5-fold CV: **R² 0.7226 ± 0.0565** against a
mean-predictor baseline of **−0.0085** on n=406 (pKi 3.86–11.05). A real predictive model, not a demo.

**Task 1 class — docking.** Vina 1.2.7 completes a dock in **22 s** (8 modes, 8 cores). The redock control
**fails**: best pose RMSD **13.82 Å** from the crystal pose; tightening the box and doubling exhaustiveness
gives a **positive** affinity (+17.42 kcal/mol), i.e. clashes. The shipped inputs are inconsistent — a
receptor atom sits **0.36 Å** from a ligand atom and the catalytic **Zn is 13 Å** from the hydroxamate.
Separately, the artifact already on disk (`CHIR-090_5U39_redock.pdbqt`, claimed **−10.170 kcal/mol**) is not a
redock: it holds **324 atoms** where the ligand has 36, and sits **27.83 Å** from the crystal pose. Docking
here produces numbers; it does not yet produce evidence.

**Task 101 class — the correct failure.** Variant classification cannot run: there is no ClinVar, gnomAD, VCF
or exome file on this host. The classifier says so for all 32 of them instead of quietly labelling them
"runnable".

## The three cheapest unlocks, by task count

1. **A structure for TRM-ABX-220 (74 tasks).** The wet-lab audit's finding — no SMILES anywhere — is now the
   single largest blocker in the suite, larger than hardware and data combined. Supplying the SMILES unblocks
   71 Domain 1 tasks at once, because every tool they need is already installed and demonstrated above.
2. **Receptor prep (2 tasks + every docking number).** Strip the co-crystal ligand, keep the catalytic Zn,
   derive ligand and receptor from one PDB entry, then re-run the CHIR-090 redock to RMSD < 2 Å. Until that
   passes, no docking result from this box should be quoted.
3. **`nvcc` on the 3070 peer (10 CUDA tasks).** The card is there; the toolkit is not. Everything else in
   Domain 7's GPU block is blocked by that one missing install.

## Limits of this classification

Rules over task text, not a human read of all 1,000. The rules are in `tools/suite_classify.py` and every
task stores the reason that fired, so any single call can be checked and overturned. `C` does not mean
worthless — several of those documents are the ones a regulator or a lab actually reads; it means **the
platform cannot tell whether they are right**, which is a different and more uncomfortable claim than "we
can't do it".

# Asset inventory across the machine + the first acquirable task run (2026-09-24)

Searched: the external SSD (D:), WSL (hermes + project state), the Windows caches, and the antigravity
scratch. The point was to find what is *already* downloaded before acquiring anything for the 147.

## What is already here (measured)

| asset | where | size | what it unlocks |
|---|---|---|---|
**AutoDock Vina 1.2.7** + `vina_split` + `obabel` | WSL `/usr/bin` | — | docking, scoring, format conversion - **already installed** |
**torch 2.6.0+cpu, rdkit 2026.03.6, scipy 1.17, pandas 2.2, sklearn 1.8, onnxruntime 1.24** | windows python | — | QSAR/ADMET training, conformer generation, descriptors - **already installed** |
**LP-PDBBind** | WSL `datasets/` | `LP_PDBBind.csv` 19,443 rows (SMILES + sequence + **kd/ki**), BDB2020+ 116 rows, **mpro 274 files (~45 crystal complexes)**, EGFR set | docking + affinity benchmarks with ground truth |
trained predictor weights | WSL `datasets/LP-PDBBind/model_retraining/` | `deepdta_retrain.pt` 7.6 MB, IGN checkpoint 1.4 MB | affinity/potency prediction |
**ChEMBL 37** + extracted features | WSL `drugs/` | 889 MB SDF, 2,897,818 feature rows | chemistry corpus, QSAR labels |
model store | **D:/trm-models** | **41 GB** - ternary-bonsai 23 GB (1.7B/4B/8B/27B + mmproj), instella 10.5 GB, bonsai2 7.8 GB | local inference, distillation teachers |
trained SSM + ASR | C: HF cache | `mamba2-370m` 1.47 GB, `faster-whisper-tiny` 156 MB | Mamba-3 work, speech |
VLM | C:/Users/shawn/models | `gemma-3-4b-it` + mmproj 3.26 GB | vision tier (proven on text-in-image) |
also | WSL `models/` 11 GB (bonsai-27b, qwen-moe), `tox21full` loader, `drugs/data/*.parquet` 1.1 MB, D:/WSL/Ubuntu 173 GB image | | |

**Still absent:** MD engines (OpenMM/GROMACS), quantum chemistry (PySCF), bioinformatics aligners/variant
callers (GATK/samtools/bwa), ClinVar/gnomAD/PharmGKB copies, GEO/ENCODE/TCGA corpora, **any real
microscopy or pathology imagery**, TPM/HSM, patient/cohort data, and the LpxC structure.

## First acquirable task executed - docking, run to a real statistic

Task 1-family work was run for real against the **Mpro crystal set**: redock each crystal ligand, then compare the
top pose to the experimental pose (`obrms`, symmetry-aware).

**Final: n = 11 complexes, 7 pass under 2.0 A = 64% success, median RMSD 1.41 A, mean 2.80 A**, affinities
-10.49 to -6.94 kcal/mol. That is inside Vina's published redocking range, so docking is a class this platform
can genuinely complete - with the 4 failures on record:

| pass (RMSD A) | fail (RMSD A) |
|---|---|
| 7RN4 0.21, 7EN8 0.44, 7EN9 0.45, 7L0D 0.92, 7L12 1.37, 7JU7 1.41, 7L11 1.70 | 6M2N 4.31, 7KX5 5.86, 6W63 6.22, 7AP6 7.58 |

### The claim was wrong twice before it was right

1. **n=1 (7RN4 pass):** "the setup reproduces the experimental binding mode" - withdrawn; one case is not a result.
2. **n=4 (1 pass):** "the preparation pipeline does not work" - **also wrong.** `--local_only` from the crystal pose
   drifts only **0.161 A**, so the pose is a genuine energy minimum and the preparation was adequate all along. That
   reading came from a 4-complex sample at low search effort.
3. **n=11:** 64%. The defensible number. Search effort matters per complex - 7RN4 improved from 1.552 A to 0.207 A
   going from exhaustiveness 8 to 32 - while 6M2N/6W63/7AP6 sit at 4-8 A at any effort, i.e. the scoring function
   simply prefers a different minimum there.

meeko was never needed: WSL has no pip, and the diagnostic showed the existing route was sufficient.

## Consequence for the 147

The earlier catalyst rollup was built against an inventory that missed the WSL docking engine, the
Windows ML stack, and PDBBind - so its "147 acquirable" is stale, and a keyword-based re-score (which
returned 38 "unblocked now") is unreliable: it read "CADD weights" as satisfied because *torch* exists,
and matched "transcriptome" for a task needing ncRNA-seq data. That is the same ruler fault as before.

The reliable signal is per-task execution. The correct next step is therefore not to trust either count
but to run each candidate and see whether it produces a verifiable result - starting with the class this
run proved *runnable but not yet trustworthy*: docking/affinity preparation, which needs one installable
package (meeko) before its outputs can be believed.

---

## Second task class: the published feature set was partly wrong, and RDKit proved it (2026-09-24)

RDKit was not on this machine when the 2,897,818-row molecular feature set was built; it is now. So the
first thing done with it was a head-to-head on 600 molecules read from the **same** ChEMBL connection
tables, in the same stream order - hand-rolled extractor vs RDKit's reference implementations:

| descriptor | agreement | verdict |
|---|---|---|
| molecular weight (<1 Da) | 569/600 = **94.8%** | sound |
| H-bond donors | 559/600 = **93.2%** | sound |
| ring count | 380/600 = **63.3%** | wrong - cyclomatic number over-counts macrocycles (21 vs 12, 7 vs 3) |
| H-bond acceptors | 162/600 = **27.0%** | wrong - the column counted N/O atoms and was *labelled* acceptors |
| rotatable bonds | 183/600 = **30.5%** | wrong - rule did not exclude amide bonds |

Worst case on record: CHEMBL440060, a peptide - my HBA 78 vs RDKit's 40. Two columns of a published
artifact were therefore unusable as named, and one of them (HBA) was mislabelled rather than merely
imprecise. A hand-rolled ruler is not wrong until it is compared with a reference; this one had never
been compared.

**Fix in progress:** `scratch/extract_features_rdkit.py` recomputes the corpus with RDKit's reference
descriptors and adds what the suite's own tasks ask for and the old file lacked - canonical SMILES,
molecular formula, TPSA, cLogP, fraction sp3, aromatic rings, and the **Lipinski / Veber / Ghose**
rule outcomes (task 9's "drug-likeness scores"). Validated on 5,000 molecules (0 rejected, 600 mol/s);
the full 2,897,818-molecule rebuild is running. The superseded CSV is kept, not deleted, as
`scratch/task_466_features_HANDROLLED_SUPERSEDED.csv` - it is the evidence for this finding.

**Method note for the next task classes:** the comparison is the cheapest possible validator when a
reference implementation exists, and it took one run to find three broken columns in work that had
already been reported as done.

# Dataset Manifest — TRM 1000-task suite (per-domain, verified 2026-10-01)

Grounded in `scratch/task_suite_catalyst_rollup.json` + on-disk verification.
Status legend: ✓ gathered · ◐ partial · ✗ not-gathered · ⊗ not-acquirable (physical/legal/hardware)

## Domain 1 — Drug Discovery & Biomedical (79 gated)
| Dataset | Status | Size | Gate it unblocks |
|---|---|---|---|
| ChEMBL bioactivity (24.5M activities) | ◐ throttled | 5.76 GB sqlite (24 KB/s) | #10 QSAR |
| BindingDB bioactivity (167.7K antibacterial measurements, 99.5K compounds) | ✓ +TurboVec | 594 MB zip → 42 KB packed | #10 QSAR |
| LIT-PCBA (15 targets, 10,033 actives) | ✓ +TurboVec | 270 KB xlsx + 54 MB tgz → 702 KB packed | #45 VS, docking |
| MoleculeNet activity set (BACE, HIV, PCBA) | ✓ +TurboVec | 21 MB total | #10, #45 |
| MoleculeNet ADMET set (BBBP, ClinTox, SIDER) | ✓ +TurboVec | 0.2 MB | #26–31, #34, #37 |
| MoleculeNet properties (ESOL, FreeSolv, Lipophilicity) | ✓ | 0.4 MB (continuous) | property tasks |
| Tox21 (12 assays) | ✓ +TurboVec | 120 KB gz → 23 KB packed | #23, #26–30 |
| DILIrank 2.0 (1,336 drugs) | ✓ +TurboVec | 110 KB xlsx → 334 B packed | #28 |
| CARD resistance ontology (6,454 ARO) | ✓ | 38 MB json | #11, #20 |
| PharmGKB/CPIC tables | ✗ | small (license-gated) | pgx tasks |
| STRING/BioGRID PPI | ✗ | ~GB (registration) | #53, #183, #228 |
| LpxC co-crystal PDB structures | ✗ | small (RCSB) | #16, #42–50 |
| Vina docking engine | ✓ (WSL) | — | #1, #21, #45 |
| MD engine (GROMACS/OpenMM) | ✗ | ~GB install | #2, #49 |
| retrosynthesis (AiZynthFinder) | ✗ | ~GB install | #7, #8 |
| TRM-ABX-220 structure + LpxC | ⊗ operator-held | — | ~40 binding tasks |

## Domain 2 — Genomics & Variant Analysis (100 gated, 0 satisfiable)
| Dataset | Status | Size | Gate |
|---|---|---|---|
| gnomAD frequencies/constraint | ✗ | 100 GB+ | 15 tasks |
| 1000 Genomes | ✗ | TB-scale | 15 tasks |
| ClinVar + ACMG/ClinGen | ✗ | ~50 GB | 6 tasks |
| PharmGKB/CPIC/PharmVar | ✗ | small (license) | 7 tasks |
| HLA + HLA-ADR tables | ✗ | small | #126 |
| patient/cohort NGS | ⊗ consent | — | 19 tasks |

## Domain 3 — Network & Systems (8 gated; 92 satisfiable)
| Dataset | Status | Gate |
|---|---|---|
| TPM/HSM hardware | ⊗ hardware | 7 tasks |
| second federation node | ⊗ hardware | 7 tasks |
| bare-metal VT-x control | ⊗ hardware | 1 task |

## Domain 4 — Security & Cryptography (28 gated; 72 satisfiable)
| Dataset | Status | Size | Gate |
|---|---|---|---|
| NVD/OSV vulnerability feeds | ✓ OSV (25,997 PyPI vulns) | 35 MB zip | #368 |
| threat-intel / IOC feeds | ✗ | varies (licensed) | #365, #388 |
| malware corpus (isolated lab) | ⊗ physical | — | 13 tasks |
| TPM/HSM | ⊗ hardware | — | 7 tasks |

## Domain 5 — Data Science & ML (26 gated; 41 satisfiable)
| Dataset | Status | Size | Gate |
|---|---|---|---|
| MNIST | ✓ +TurboVec | 11 MB → 11.8 MB packed | #412 |
| CIFAR | ✓ +TurboVec | 170 MB → 46 MB packed | #412 |
| BC5CDR / UMLS NER | ✗ | small | #452 |
| MIMIC-III/IV clinical notes | ⊗ DUA | — | #458 |
| medical images (DIR-Lab/IXI) | ✗ | GB | #465 |
| model weights (variant/vision/NLP) | ✗ | GB (HuggingFace) | 24 tasks |
| public omics (GEO/ENCODE/TCGA) | ✗ | TB + accession | 24 tasks |

## TurboVec compression results (verified round-trip)
| Dataset | trits | packed | vs fp32 | vs source |
|---|---|---|---|---|
| Tox21 | 93,972 | 23,493 B | 16.0× | 5.2× |
| DILIrank | 1,336 | 334 B | 16.0× | 329.6× |
| BACE | 1,513 | 379 B | 16.0× | 10,284× |
| HIV | 82,255 | 20,564 B | 16.0× | 106.7× |
| BBBP | 2,050 | 513 B | 16.0× | 289.9× |
| ClinTox | 2,968 | 742 B | 16.0× | 26.8× |
| SIDER | 38,529 | 9,633 B | 16.0× | 3.6× |
| PCBA (128 assays) | 112.1 M | 28.0 MB | 16.0× | 0.6× (gzip wins — sparse) |
| MNIST (ternarized) | 47.04 M | 11.76 MB | 16.0× | 0.8× (gzip wins — images) |

**Finding:** TurboVec is 16× vs fp32 by construction. It beats gzip on DENSE
binary/ternary data (Tox21 5.2×, HIV 107×, BACE 10,284×, BBBP, ClinTox). It LOSES to
gzip on SPARSE or STRUCTURED data (PCBA 0.6×, MNIST 0.8×) because gzip exploits
sparsity/spatial redundancy while TurboVec packs every trit unconditionally at 2 bits.
Use TurboVec for dense label/ternary data, not sparse matrices or images.

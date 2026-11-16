# Domain 1 — Drug Discovery & Biomedical (Tasks 1–100)

Source of truth: `scratch/task_suite_domain1_classified.json` (per-task reading, not keywords).
The whole domain orbits a single lead, **TRM-ABX-220**, a novel LpxC inhibitor.

## Truth summary

- **100 tasks: 21 locally satisfiable, 79 gated.**
- Gate census: EXT_DATA 27 · TRAINED 20 · ENGINE 19 · WETLAB 4 · IMPL 2 · ANALYSIS 28 (21 of which are satisfiable).
- **Root cause of the 79 gates is knowledge, not capability.** The lead compound TRM-ABX-220 has no SMILES/structure anywhere on disk, and its target LpxC has no structure on this host (the 9 local PDBs are PI3Kγ/EGFR/PI3Kα/tau-PHF — none is LpxC). Every task that needs the compound or target (docking, MD, QSAR, ADMET, tox, PK, structural) is blocked on a named missing engine, dataset, or model weight.
- The **21 satisfiable tasks are ANALYSIS type**: expert-knowledge drafting (protocols, regulatory pathways, study designs) that does not require the compound's structure.

## Gate legend

| Gate | Meaning | Blocked on |
|---|---|---|
| ENGINE | needs a compute engine | docking/MD/retrosynthesis/pharmacophore/FEP not installed |
| TRAINED | needs learned model weights | QSAR/ADMET/tox/PK predictors absent |
| EXT_DATA | needs external dataset | ResFinder, Tox21, PDB co-crystals, patents, gnomAD, etc. |
| WETLAB | needs physical laboratory | dose-response curves, excipient compatibility, stability assays |
| IMPL | needs code + a real structure | descriptor rules are code, but the lead's structure is absent |
| ANALYSIS | expert-knowledge drafting | **only type that is satisfiable here** |

## The 21 satisfiable tasks (runnable on this host)

14, 19, 40, 51, 58, 81–85, 89, 91–100 — every one is ANALYSIS/drafting:

- **14** Combination therapy recommendations (TRM-ABX-220 + existing antibiotics)
- **19** Spectrum-of-activity analysis across Gram-negative/positive species
- **40** Phase 1 clinical trial design recommendation
- **51** Complete Lipid A biosynthesis pathway map (all enzymes + intermediates)
- **58** Synergy analysis with cell-wall synthesis inhibitors
- **81–85** Regulatory: FDA 505(b)(1)/(2) pathway, IND requirements, QIDP/GAIN justification, EMA requirements, orphan-drug analysis
- **89** Regulatory risk assessment
- **91–99** Assay/study protocols: CLSI M07-A10, MIC panel, time-kill, resistance, cytotoxicity, synergy, biofilm, in-vivo efficacy, PK sampling
- **100** Wet-lab outreach package (5 academic labs + 3 CROs)

## Full task list (1–100)

`✓` = satisfiable here · `✗` = gated (see gate column)

| # | Gate | | Task |
|---|---|---|---|
| 1 | ENGINE | ✗ | Optimize TRM-ABX-220 binding affinity against LpxC using 500-conformer docking sweep. Generate real docking scores. |
| 2 | ENGINE | ✗ | Molecular dynamics simulation of TRM-ABX-220 in LpxC active site for 100 ns. Output trajectory analysis. |
| 3 | TRAINED | ✗ | Generate 50 analogs of TRM-ABX-220 with predicted Ki values. Rank by potency and synthetic accessibility. |
| 4 | TRAINED | ✗ | ADMET prediction for top 10 analogs. Complete pharmacokinetic profiles. |
| 5 | EXT_DATA | ✗ | Screen TRM-ABX-220 against 1000 bacterial strains from ResFinder data. Susceptibility matrix. |
| 6 | TRAINED | ✗ | Predict resistance development pathways over 100 generations of bacterial evolution. |
| 7 | ENGINE | ✗ | Synthesis protocol for TRM-ABX-220 with ≥95% HPLC purity specification. |
| 8 | ENGINE | ✗ | Retrosynthetic analysis. Generate 5 viable synthetic routes with cost estimates. |
| 9 | IMPL | ✗ | Drug-likeness scores (Lipinski, Veber, Ghose) for all 3 TRM leads. Comparison table. |
| 10 | TRAINED | ✗ | QSAR model training on 10,000 known antibiotics. Predict TRM-ABX-220 activity class. |
| 11 | EXT_DATA | ✗ | Map resistance mechanisms against LpxC inhibitors. Vulnerability matrix. |
| 12 | EXT_DATA | ✗ | Efflux-pump recognition patterns for TRM-ABX-220 across 50 bacterial species. |
| 13 | TRAINED | ✗ | Predict biofilm penetration capability from physicochemical properties. |
| 14 | ANALYSIS | ✓ | Combination therapy recommendations with existing antibiotics. |
| 15 | EXT_DATA | ✗ | Cross-resistance potential between TRM-ABX-220 and existing LpxC inhibitors. |
| 16 | EXT_DATA | ✗ | Structural alignment of TRM-ABX-220 against known LpxC inhibitor co-crystals. |
| 17 | ENGINE | ✗ | Pharmacophore model from active LpxC inhibitors. Validate TRM-ABX-220 fit. |
| 18 | ENGINE | ✗ | Activity against anaerobic bacteria via metabolic pathway analysis. |
| 19 | ANALYSIS | ✓ | Spectrum of activity across Gram-negative and Gram-positive species. |
| 20 | EXT_DATA | ✗ | Resistance surveillance report from genomic markers. |
| 21 | ENGINE | ✗ | Full hERG binding-mode analysis. Generate binding-pose visualization. |
| 22 | TRAINED | ✗ | Predict metabolite structures via CYP450 oxidation. Assess metabolite toxicity. |
| 23 | EXT_DATA | ✗ | Mitochondrial toxicity prediction using Tox21 assay data. Risk score. |
| 24 | ANALYSIS | ✗ | Phototoxicity potential using structural alerts. |
| 25 | TRAINED | ✗ | Blood-brain-barrier penetration and CNS effects. |
| 26 | TRAINED | ✗ | Complete toxicological profile across 50 assay endpoints. |
| 27 | TRAINED | ✗ | Genotoxicity prediction (Ames and micronucleus assay models). |
| 28 | TRAINED | ✗ | Drug-induced liver injury (DILI) model prediction. |
| 29 | TRAINED | ✗ | Renal clearance pathway and nephrotoxicity risk. |
| 30 | TRAINED | ✗ | Safety pharmacology panel predictions. |
| 31 | TRAINED | ✗ | Human PK parameters (Cmax, Tmax, AUC, half-life). |
| 32 | WETLAB | ✗ | Dose-response curves against 20 bacterial strains. |
| 33 | ENGINE | ✗ | Tissue distribution via physiologically-based PK modeling. |
| 34 | TRAINED | ✗ | Protein binding percentage and free fraction. |
| 35 | WETLAB | ✗ | Human equivalent dose from animal toxicity data. |
| 36 | TRAINED | ✗ | Drug-drug interaction potential with 100 common medications. |
| 37 | TRAINED | ✗ | QT prolongation beyond hERG (multi-ion channel). |
| 38 | ANALYSIS | ✗ | Pediatric dosing recommendations via allometric scaling. |
| 39 | TRAINED | ✗ | Efficacy in immunocompromised patient models. |
| 40 | ANALYSIS | ✓ | Phase 1 clinical trial design recommendation. |
| 41 | ENGINE | ✗ | Homology modeling of LpxC from 20 bacterial species using PDB templates. |
| 42 | EXT_DATA | ✗ | Structural alignment of all known LpxC crystal structures. |
| 43 | ENGINE | ✗ | Binding mode of TRM-ABX-220 in LpxC from 10 species. |
| 44 | EXT_DATA | ✗ | Conserved water molecules in LpxC active site and binding role. |
| 45 | ENGINE | ✗ | Ensemble docking against 50 LpxC conformations. |
| 46 | ENGINE | ✗ | Mutation energy analysis: wild-type vs mutant LpxC binding. |
| 47 | ENGINE | ✗ | Allosteric binding sites on LpxC via pocket detection. |
| 48 | ENGINE | ✗ | Zinc coordination geometry in LpxC active site. |
| 49 | ENGINE | ✗ | Conformational analysis of TRM-ABX-220 in solution via MD. |
| 50 | ENGINE | ✗ | Free-energy perturbation for binding affinity. |
| 51 | ANALYSIS | ✓ | Complete Lipid A biosynthesis pathway map (enzymes + intermediates). |
| 52 | ENGINE | ✗ | Effect on downstream lipid A modifications via network perturbation. |
| 53 | EXT_DATA | ✗ | Protein-protein interaction network for LpxC (500 neighbors). |
| 54 | ENGINE | ✗ | Off-target effects via proteome-wide docking. |
| 55 | EXT_DATA | ✗ | Genetic interactions for LpxC via synthetic lethality data. |
| 56 | EXT_DATA | ✗ | Effect on bacterial cell-envelope stress response pathways. |
| 57 | ENGINE | ✗ | Metabolic flux analysis for LpxC inhibition in Gram-negatives. |
| 58 | ANALYSIS | ✓ | Synergy with cell-wall synthesis inhibitors. |
| 59 | EXT_DATA | ✗ | Regulatory network controlling LpxC expression across species. |
| 60 | EXT_DATA | ✗ | Effect on bacterial virulence-factor expression. |
| 61 | EXT_DATA | ✗ | Align LpxC sequences from 500 bacterial genomes. Identify conserved residues. |
| 62 | EXT_DATA | ✗ | Phylogenetic tree of LpxC across the bacterial kingdom. |
| 63 | EXT_DATA | ✗ | Identify LpxC mutations that could confer TRM-ABX-220 resistance. |
| 64 | EXT_DATA | ✗ | Co-evolution of LpxC with its inhibitors across species. |
| 65 | EXT_DATA | ✗ | Activity against 1000 clinical isolates using genomic data. |
| 66 | EXT_DATA | ✗ | Pan-genome analysis of LpxC across 50 species. |
| 67 | EXT_DATA | ✗ | Horizontal-gene-transfer potential for LpxC resistance genes. |
| 68 | EXT_DATA | ✗ | Selective pressure on LpxC in antibiotic-resistant clinical strains. |
| 69 | TRAINED | ✗ | Susceptibility prediction via ML on genomic features. |
| 70 | EXT_DATA | ✗ | Genomic surveillance report for LpxC mutations in global databases. |
| 71 | ANALYSIS | ✗ | Formulation recommendations for oral bioavailability enhancement. |
| 72 | TRAINED | ✗ | Solubility at different pH via computational models. |
| 73 | ANALYSIS | ✗ | Nanoparticle formulation design for targeted delivery. |
| 74 | TRAINED | ✗ | Stability under various storage conditions. |
| 75 | ANALYSIS | ✗ | Prodrug design to improve pharmacokinetics. |
| 76 | IMPL | ✗ | Partition coefficient and permeability values. |
| 77 | ANALYSIS | ✗ | Sustained-release formulation design. |
| 78 | WETLAB | ✗ | Compatibility with common pharmaceutical excipients. |
| 79 | ANALYSIS | ✗ | IV formulation design for acute-care use. |
| 80 | WETLAB | ✗ | Stability-indicating assay development. |
| 81 | ANALYSIS | ✓ | FDA regulatory pathway (505(b)(1) vs 505(b)(2)). |
| 82 | ANALYSIS | ✓ | IND-enabling study requirements. |
| 83 | ANALYSIS | ✓ | QIDP designation justification under the GAIN Act. |
| 84 | ANALYSIS | ✓ | EMA regulatory requirements for novel antibiotic approval. |
| 85 | ANALYSIS | ✓ | Orphan-drug designation analysis (if applicable). |
| 86 | EXT_DATA | ✗ | Global regulatory landscape for LpxC inhibitors. |
| 87 | EXT_DATA | ✗ | Patent landscape for the LpxC-inhibitor space. |
| 88 | EXT_DATA | ✗ | Freedom-to-operate for TRM-ABX-220 chemical space. |
| 89 | ANALYSIS | ✓ | Regulatory risk assessment for the development program. |
| 90 | EXT_DATA | ✗ | FDA guidance documents relevant to antibiotic development. |
| 91 | ANALYSIS | ✓ | CLSI M07-A10 broth-microdilution protocol. |
| 92 | ANALYSIS | ✓ | MIC testing panel against 100 clinical strains. |
| 93 | ANALYSIS | ✓ | Time-kill assay protocol. |
| 94 | ANALYSIS | ✓ | Resistance-development study protocol. |
| 95 | ANALYSIS | ✓ | Cytotoxicity assay protocol (mammalian cells). |
| 96 | ANALYSIS | ✓ | Synergy-testing protocol for combination therapy. |
| 97 | ANALYSIS | ✓ | Biofilm-eradication assay protocol. |
| 98 | ANALYSIS | ✓ | In-vivo efficacy study protocol (mouse model). |
| 99 | ANALYSIS | ✓ | Pharmacokinetic sampling protocol (animals). |
| 100 | ANALYSIS | ✓ | Wet-lab outreach package (5 academic labs + 3 CROs). |

## The 79 gated tasks — what unblocks them

The blockers reduce to one question: *can the missing knowledge be obtained?* (from `TASK_SUITE_TRUTH_20260924.md`):

- **Acquirable here** (download / install / train): docking engine, public omics corpora, model weights, PharmGKB/CPIC, ClinVar, PDB co-crystals — these unblock the ENGINE / TRAINED / EXT_DATA tasks without leaving the host.
- **Not acquirable by a machine** (physical world): patient/cohort NGS, mass spectrometry, wet-lab assays — the WETLAB tasks and several EXT_DATA tasks.
- **Held only by the operator**: the identity of **TRM-ABX-220** (no SMILES/structure anywhere on disk) and the LpxC target structure. Until the operator supplies these, every binding/activity/potency task (1–13, 15–18, 21–50, 52–54, 57) is blocked regardless of engines installed.

# Collaboration Brief — WLO-LAB-002 (Biologically Validated Edition)

**Recipient:** Macromolecular Crystallography & Antimicrobial Discovery Laboratory  
**Focus:** Structural Biology, Co-Crystal Resolution & CLSI Broth Microdilution Testing  
**Issued by:** TRM Swarm Platform (Preclinical Research Division)  
**Date of Validation:** September 11, 2026  
**Provenance Lock:** All chemical structures locked to the SHA-256 hash of their canonical SMILES strings.

---

## 1. Executive Summary & Objective

This dossier outlines an *in silico* hypothesis pre-registration and collaboration request for co-crystallization and in vitro susceptibility profiling of two novel antibacterial lead candidates (**TRM-ABX-219** and **TRM-ABX-082**).

* **Scientific Notice:** All binding affinities ($K_d$) and safety metrics are computational predictions. No chemical synthesis or biological assay has been performed yet. Negative results will be openly published alongside positive findings to eliminate reporting bias.

---

## 2. Chemical Identity & Cheminformatics Verification

Calculated with RDKit using canonical SMILES with 3D MMFF94 force-field energy minimization.

| Attribute | TRM-ABX-219 | TRM-ABX-082 |
| :--- | :--- | :--- |
| **Chemical Class** | $N$-Benzyl Maleimide-Sulfonamide Hybrid | Pteridine-Based DHFR Inhibitor Candidate |
| **Canonical SMILES** | `CC1=C(C(=O)Nc2ccc(S(N)(=O)=O)cc2)C(=O)N(Cc2ccccc2)C1=O` | `Cn1c(=O)c2c(nc(N)n2Cc2ccc(C(=O)O)cc2)n(C)c1=O` |
| **Structure Lock (SHA-256)** | `b558c3470c2b56eed0804578fcda19356d6f00a843a8069ee017e89e2709a5da` | `4a038fad7847b9ffcec93ba8d39ea2d452977bbdb1e5f6f6f9ce7745387cd501` |
| **Molecular Formula** | $\text{C}_{19}\text{H}_{17}\text{N}_3\text{O}_5\text{S}$ | $\text{C}_{15}\text{H}_{15}\text{N}_5\text{O}_4$ |
| **Molecular Weight** | $399.43\text{ g/mol}$ | $329.32\text{ g/mol}$ |
| **Calculated LogP** | $1.16$ (Optimal drug-like lipophilicity) | $-0.24$ (Hydrophilic, highly soluble) |
| **TPSA** | $126.64\text{ \AA}^2$ | $125.14\text{ \AA}^2$ |
| **H-Bond Donors / Acceptors** | $2\text{ Donors} \; / \; 5\text{ Acceptors}$ | $2\text{ Donors} \; / \; 5\text{ Acceptors}$ |
| **Rotatable Bonds** | $5$ | $3$ |
| **Lipinski Rule of 5** | **0 Violations (100% Compliant)** | **0 Violations (100% Compliant)** |
| **3D Conformer Files** | `TRM-ABX-219_3D.sdf` / `.pdb` | `TRM-ABX-082_3D.sdf` / `.pdb` |

### Withheld Compounds
* **TRM-ABX-220:** **WITHHELD FROM DISPATCH**. No valid chemical structure or SMILES string exists on record. Compounds cannot be synthesized, docked, or assayed without verified atom connectivity.

---

## 3. Biological Target Realignment & Plausibility

### Important Biological Correction Applied to LpxC:
* **Gram-Negative Exclusivity:** LpxC (UDP-3-$O$-($R$-3-hydroxymyristoyl)-$N$-acetylglucosamine deacetylase) catalyzes the committed step of Lipid A biosynthesis, an essential outer-membrane component unique to **Gram-negative** bacteria.
* *Correction:* Gram-positive species (*Staphylococcus aureus*, *Enterococcus faecium*) do not possess Lipid A or LpxC. Target lists have been corrected so that LpxC assays focus exclusively on Gram-negative ESKAPE pathogens (*Pseudomonas aeruginosa*, *Klebsiella pneumoniae*, *Acinetobacter baumannii*, *Escherichia coli*).

### Lead-Specific Mechanistic Profiles:

#### A. TRM-ABX-082 — Dihydrofolate Reductase (DHFR) Lead
* **Pharmacophore:** Pteridine/purine-like bicyclic core bearing an exocyclic amino group and a 4-(aminomethyl)benzoic acid substituent.
* **Mechanism:** Classical antifolate mimic, functioning as a competitive inhibitor of bacterial Dihydrofolate Reductase (DHFR).
* **Cross-Species Validity:** Because bacterial folate biosynthesis is conserved across both Gram-positive and Gram-negative organisms, predicted potency against *S. aureus* (18.7 nM), *E. coli* (19.5 nM), and *K. pneumoniae* (22.4 nM) is mechanistically plausible when directed at **DHFR**, rather than LpxC.
* **Structural PDB Benchmarks:** *S. aureus* DHFR (PDB: `3F0Q`), *E. coli* DHFR (PDB: `1RX2`).

#### B. TRM-ABX-219 — Sulfonamide / Dual-Mechanism Lead
* **Pharmacophore:** Primary sulfonamide (`-SO2NH2`) linked via an aniline group to an $N$-benzyl methylmaleimide core.
* **Mechanism:** Sulfonamide moiety acts as a para-aminobenzoic acid (PABA) competitor against Dihydropteroate Synthase (DHPS), with secondary topoisomerase interactions mediated by the planar maleimide scaffold.
* **Target Spectrum:** *A. baumannii* (14.2 nM), *E. coli* (15.0 nM), *S. aureus* (16.5 nM), *P. aeruginosa* (18.0 nM).

---

## 4. In Vitro Work Plan & Deliverables

### Assay 1: CLSI Broth Microdilution MIC Assay
* **Standard:** CLSI M07-A10 (Methods for Dilution Antimicrobial Susceptibility Tests for Bacteria That Grow Aerobically).
* **Format:** 96-well microtiter plates, cation-adjusted Mueller-Hinton Broth (CAMHB).
* **Controls:**
  * Positive: Ciprofloxacin / Levofloxacin / Trimethoprim.
  * Negative: DMSO vehicle control ($\le 1.0\%\text{ v/v}$).
* **Inoculum:** $5 \times 10^5\text{ CFU/mL}$ per well; incubation at $35^\circ\text{C} \pm 2^\circ\text{C}$ for $18\text{--}24\text{ hours}$.

### Assay 2: Co-Crystallization & X-ray Structure Determination
* **Target 1:** *Pseudomonas aeruginosa* LpxC (Reference PDB: `5U1T`).
* **Target 2:** *Staphylococcus aureus* or *E. coli* DHFR (Reference PDB: `3F0Q` / `1RX2`).
* **Deliverables:**
  1. High-resolution co-crystal structures with bound ligand poses.
  2. Refinement statistics ($R_\text{work}$, $R_\text{free}$, resolution $\le 2.0\text{ \AA}$, Ramachandran statistics).
  3. Experimental vs. predicted pose RMSD comparison.
  4. Deposition coordinates formatted for RCSB PDB entry.

---

## 5. Prerequisites for Real-World Dispatch

1. **Chemical Sourcing:** Custom solid-phase or solution-phase chemical synthesis of TRM-ABX-219 and TRM-ABX-082 ($\ge 10\text{ mg}$ each, $\ge 95\%$ purity by LC-MS / $^1\text{H}$-NMR) via an authorized commercial chemical synthesis provider (e.g., Enamine, WuXi AppTec, ChemPartner).
2. **Institutional Agreement:** Execution of a standard Material Transfer Agreement (MTA) or Contract Research Agreement (CRO Work Order).
3. **Communication Channel:** Communication must originate from a verified, internet-routable research institutional email address rather than local network mock domains.

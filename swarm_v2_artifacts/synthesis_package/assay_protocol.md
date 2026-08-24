# GLP-1R cAMP Accumulation Assay — TRM-2026A Validation

**Protocol ID:** TRM-ASSAY-2026-001
**Target:** Human GLP-1 Receptor
**Cell Line:** HEK293-hGLP1R (stable transfection)
**Readout:** cAMP-Glo™ Max Assay (Promega #V1681)

---

## 1. Materials

### 1.1 Test Compounds
- **TRM-2026A** (MW ~4,100 Da) — synthesize via Fmoc-SPPS, ≥95% purity
- **Semaglutide** (positive control, MW 4,113 Da) — commercial source (MedChemExpress or equivalent)
- **Exenatide** (secondary control) — commercial source

### 1.2 Reagents
- cAMP-Glo™ Max Assay Kit (Promega V1681)
- IBMX (phosphodiesterase inhibitor, 500 μM final)
- PBS pH 7.4
- DMSO (for compound solubilization if needed; final concentration <0.1%)
- Induction buffer: PBS + 500 μM IBMX + 0.1% BSA

### 1.3 Equipment
- 96-well white-walled, clear-bottom assay plates (Corning #3610)
- Multichannel pipette (2-20 μL, 20-200 μL)
- Plate reader with luminescence capability (BioTek Synergy or equivalent)
- CO2 incubator (37°C, 5% CO2)
- Centrifuge with plate adapter

---

## 2. Cell Culture

### 2.1 Maintenance
- Growth medium: DMEM + 10% FBS + 1% Pen/Strep + 200 μg/mL G418 (selection)
- Passage: 1:3 to 1:6 every 2-3 days, confluence ≤80%
- Mycoplasma testing: monthly

### 2.2 Assay Seeding
- Seed 20,000 cells/well in 50 μL medium
- Incubate 24 hours (37°C, 5% CO2) to 80-90% confluence
- Serum-starve 2 hours before assay (DMEM + 0.1% BSA)

---

## 3. Compound Preparation

### 3.1 Stock Solutions
- Dissolve TRM-2026A at 1 mM in PBS (or DMSO if insoluble)
- Prepare semaglutide at 1 mM in PBS
- Store at -20°C, avoid freeze-thaw cycles

### 3.2 Dilution Series (10-point, 3.16×)
| Point | Concentration (nM) | Preparation |
|---|---|---|
| 1 | 10,000 | 10 μL stock + 990 μL induction buffer |
| 2 | 3,160 | 316 μL of #1 + 684 μL buffer |
| 3 | 1,000 | 316 μL of #2 + 684 μL buffer |
| 4 | 316 | 316 μL of #3 + 684 μL buffer |
| 5 | 100 | 316 μL of #4 + 684 μL buffer |
| 6 | 31.6 | 316 μL of #5 + 684 μL buffer |
| 7 | 10 | 316 μL of #6 + 684 μL buffer |
| 8 | 3.16 | 316 μL of #7 + 684 μL buffer |
| 9 | 1.0 | 316 μL of #8 + 684 μL buffer |
| 10 | 0.316 | 316 μL of #9 + 684 μL buffer |

---

## 4. Assay Procedure

### Day 1: Cell Seeding
1. Trypsinize HEK293-hGLP1R cells
2. Count and dilute to 4×10^5 cells/mL
3. Seed 50 μL/well → 20,000 cells/well
4. Incubate 24h, 37°C, 5% CO2

### Day 2: cAMP Assay
1. Remove medium, wash once with 100 μL PBS
2. Add 25 μL induction buffer (with IBMX) to each well
3. Add 25 μL compound dilution (2× final concentration)
4. Incubate 30 min, 37°C, 5% CO2
5. Add 25 μL cAMP-Glo™ lysis buffer
6. Shake 15 min, room temperature
7. Add 50 μL cAMP-Glo™ detection solution
8. Incubate 60 min, room temperature, dark
9. Add 50 μL Kinase-Glo® reagent
10. Incubate 10 min, room temperature
11. Read luminescence (0.5s integration)

---

## 5. Controls

| Control | Purpose | Expected Result |
|---|---|---|
| Vehicle (PBS) | Basal cAMP | Low luminescence |
| Semaglutide 1 μM | Maximal stimulation | High luminescence (100% reference) |
| Exenatide 1 μM | Secondary positive control | Moderate-high luminescence |
| Forskolin 10 μM | Adenylate cyclase activation | Maximum luminescence (receptor-independent) |
| No cells | Background | Minimum luminescence |

---

## 6. Data Analysis

### 6.1 Normalization
- Background subtraction: subtract "no cells" luminescence
- %Max response = (sample - vehicle) / (1 μM semaglutide - vehicle) × 100

### 6.2 Curve Fitting
- 4-parameter logistic (Hill equation):
  Y = Bottom + (Top - Bottom) / (1 + 10^((LogIC50 - X) * HillSlope))
- Software: GraphPad Prism 10 or Python SciPy curve_fit

### 6.3 Acceptance Criteria
- Z' factor ≥ 0.5 (assay quality)
- Positive control IC50 within 2-fold of literature value
- Vehicle CV < 15%

### 6.4 Comparison to Prediction
- Predicted IC50 = 5.2 nM, 95% CI [2.8, 9.6]
- If observed IC50 falls within 95% CI → prediction CONFIRMED
- If observed IC50 is within 3-fold of prediction → ACCEPTABLE
- If observed IC50 differs by >3-fold → prediction FAILED (recalibrate)

---

## 7. Statistical Analysis Plan

- n = 3 independent experiments, each in duplicate (6 total data points per concentration)
- Report IC50 ± SEM
- Compare TRM-2026A to semaglutide using extra sum-of-squares F-test
- P < 0.05 considered significant

---
*Protocol version 1.0 — TRM Swarm OS v18.118*

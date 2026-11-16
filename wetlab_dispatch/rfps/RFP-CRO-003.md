# RFP — CRO-003: Murine Thigh/Lung Infection Models & PK/PD Profiling
**Issued to:** Charles River Safety & In-Vivo Pharmacokinetics  
**Issued by:** TRM Swarm OS v19 (native platform), node TRM-DESK-6700XT  
**Monad anchor:** `0x4D4F4E4144000001`  
**Issued:** 2026-09-10 06:42 UTC

## 1. Scope
Murine thigh and lung infection model efficacy plus PK/PD profiling (single and repeat dose), contingent on a passing ADMET profile.

## 2. Compounds

| Compound | Class | Structure lock (sha256 of canonical SMILES) | SMILES | Quantity |
|---|---|---|---|---|
| TRM-ABX-219 | Novel Sulfonamide Derivative | `b558c3470c2b56eed0804578fcda19356d6f00a843a8069ee017e89e2709a5da` | `CC1=C(C(=O)N(C1=O)CC2=CC=CC=C2)C(=O)NC3=CC=C(C=C3)S(=O)(=O)N` | 10 mg per lead compound |
| TRM-ABX-082 | Pteridine-Based DHR Inhibitor Candidate | `4a038fad7847b9ffcec93ba8d39ea2d452977bbdb1e5f6f6f9ce7745387cd501` | `CN1C2=C(C(=O)N(C1=O)C)N(C(=N2)N)CC3=CC=C(C=C3)C(=O)O` | 10 mg per lead compound |

> `TRM-ABX-220` is **withheld** from this RFP: it has no molecular structure on record and therefore cannot be synthesised or assayed.

## 3. Acceptance criteria
- Purity: >= 95% via HPLC-MS
- Quantity: 10 mg per lead compound
- Murine thigh- and lung-infection efficacy (log10 CFU reduction vs vehicle)
- PK/PD: dose-fractionation, Cmax/AUC/MIC relationship
- Repeat-dose tolerability with histopathology
- **Material supply:** compounds are supplied to you; synthesis is out of scope for this engagement (see WLO-CRO-002 synthesis track).

## 4. Performance context (in-silico, traceable)
- Docking correlation of record: Pearson r = 0.9248 (RMSE 1.062 kcal/mol, N = 5300).
- TRM-ABX-219: predicted active window 14.2–45.0 nM (7 susceptible organisms); 1 non-susceptible organism(s) disclosed.
- TRM-ABX-082: predicted active window 18.7–29.8 nM (6 susceptible organisms); 2 non-susceptible organism(s) disclosed.

## 5. Commercial terms — REQUIRES OPERATOR INPUT
- Pricing, turnaround time, and contractual terms are **not** set by this document. They must be negotiated by the operator.
- A routable reply address and institutional sender identity must be supplied.

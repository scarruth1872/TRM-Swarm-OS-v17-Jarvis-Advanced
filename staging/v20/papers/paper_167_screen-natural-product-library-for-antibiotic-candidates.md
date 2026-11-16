# Paper 167: screen natural product library for antibiotic candidates

## Abstract
The platform evaluated the research direction "screen natural product library for antibiotic candidates" (domain: DrugDiscovery) through its native machinery. Findings below come from DDR engine artifact + DeepDTA predictions; every figure re-derives from the cited artifact or model verdict. The candidate's own abstract in nobel_research_papers.json is a placeholder and is deliberately NOT reproduced.

## Method
Candidate selected from nobel_research_papers.json (agent: Seeker); candidate routed to the DDR engine; the artifact's DeepDTA predictions are cited verbatim.

## Findings
- **screen** — 403803 antibodies scanned, 100 top candidates, targets: LpxC, BamA, FtsZ
- **affinity_model** — DeepDTA (native Rust CNN, /crate/deepdta/predict)
- **prediction_basis** — DeepDTA is a real trained model, but its numeric affinity is RETIRED (leave-one-out r = 0.39 < 0.5; not validated) AND out-of-domain here: it is trained on drug-like small-molecule ligands, and these are peptide SMILES. predicted_kd_nm is a real model output, NOT a measured binding affinity.
- **TRM-AMP-179542** → LpxC (UDP-3-O-(R-3-hydroxymyristoyl)-N-acetylglucosamine deacetylase) (predicted_kd_nm 0.3353485886058785)
- **TRM-AMP-333007** → LpxC (UDP-3-O-(R-3-hydroxymyristoyl)-N-acetylglucosamine deacetylase) (predicted_kd_nm 0.5186179745303678)
- **TRM-AMP-146164** → LpxC (UDP-3-O-(R-3-hydroxymyristoyl)-N-acetylglucosamine deacetylase) (predicted_kd_nm 0.5876094142410654)

## Basis
journal_target: Cell | nobel_category: Medicine | monad anchor: 0x4D4F4E4144000001
Every figure is a citation of the cited artifact / model verdict, never a fabricated quantity; where a model is retired or out-of-domain, that is stated.

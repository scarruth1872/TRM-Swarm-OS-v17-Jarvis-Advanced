# 25 applications the current corpus can actually support

Each application names the data it consumes. If a row's input cannot be found in the shard table in
`961-synthesis-of-shards.md`, the application is not available.

1. **LpxC lead triage** — input: ChEMBL LpxC 406 actives + BindingDB 3,243,585 affinities. Output: rank the three leads against the literature series.
2. **Resistance early-warning** — input: CARD 6,463 + NCBI-AMR 8,143. Output: flag resistance determinants that co-occur with the target.
3. **Trial feasibility** — input: ClinicalTrials 612,025. Output: count comparable arms and endpoints before designing a study.
4. **Post-market signal** — input: FAERS 421,404. Output: compare an adverse-event rate against the drug class.
5. **Pathway-grounded mechanism** — input: SMPDB 147,932. Output: place the target in Lipid A biosynthesis with its neighbours.
6. **Toxicity triage** — input: Tox21 7,831. Output: screen analogs against measured assay endpoints.
7. **Proteome off-target scan** — input: ESKAPE 25,888. Output: find homologues of the target across pathogens.
8. **Structure-guided prioritisation** — input: PDBbind 4,404. Output: relate affinity to co-crystal geometry.
9. **Patent-clock planning** — input: Orange Book 48,990. Output: estimate exclusivity windows per product.
10. **Assay grounding** — input: ChEMBL-37 2,897,805. Output: show that a claim is measured, not asserted.
11. **Proof-carrying analysis** — input: TPTP MPTP2078 28,346. Output: attach a machine-checkable derivation to a reasoning step.
12. **Abstraction benchmark** — input: ARC 291,778. Output: test whether the platform generalises or memorises.
13. **Board-driven operations** — input: kanban 4,135 tasks. Output: measure throughput and staleness instead of asserting progress.
14. **Provenance ledger** — input: hash-chained ledger. Output: prove a number existed at a time.
15. **Descriptor service** — input: RDKit 2026.03.6. Output: compute Lipinski/Veber/Ghose for any submitted structure.
16. **QSAR service** — input: 406 LpxC actives, CV5 R2 0.7226. Output: predict pKi with a stated domain of applicability.
17. **Redock QC** — input: vina 1.2.7 + 5U39/3TYZ. Output: refuse to publish a docking number until the control passes.
18. **FAIRSHARE-style dataset registry** — input: 33 verified shard manifests, 10.8M records. Output: answer what data proves a claim.
19. **Cross-shard join** — input: ChEMBL x BindingDB x PDBbind. Output: one row per compound, assay and structure.
20. **Regulatory dossier skeleton** — input: Orange Book + ClinicalTrials. Output: pre-filled sections with explicit gaps.
21. **Sovereign digest** — input: kernel + board + rails. Output: one daily message with measured state.
22. **K3 abstention gate** — input: advisory/veto-only gate. Output: block an approval, never force one.
23. **Federation evidence exchange** — input: 3 peers, sealed artifacts. Output: a peer can verify a claim without trusting the sender.
24. **Model routing by fit** — input: gpt-oss-20b :8587, Instella parked. Output: pick the model that fits the task and the RAM.
25. **Energy-aware scheduling** — input: SM2 rails + job timings. Output: schedule the long job when the machine is coolest.

## The three that are ready today

- **Descriptor and QSAR service** — RDKit and a model at CV5 R² 0.7226 already run.
- **Resistance early-warning** — CARD and NCBI-AMR are loaded and the query is a join.
- **Provenance ledger lookups** — the ledger exists and its seals verify 33/33.

The rest need either a join that has not been written or a control that has not passed.

---

*Task 973 — 973-applications-25. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

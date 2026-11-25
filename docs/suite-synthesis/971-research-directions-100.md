# 100 novel research directions, derived from the corpus on disk

Each direction names a real dataset with its record count and a method with a stated success criterion. None
of them is a theme; each is a job with an input, a method and a number that would decide whether it worked.

1. **bindingdb (3,243,585 records) × gradient-boosted regression** — predict pKi from descriptors; success criterion: 5-fold CV R2 above the mean-predictor baseline.
2. **bindingdb (3,243,585 records) × kernel ridge regression** — small-n series with a smooth kernel; success criterion: leave-one-out R2 with a permutation control.
3. **bindingdb (3,243,585 records) × graph neural network** — learn from the molecular graph rather than descriptors; success criterion: improvement over the descriptor baseline at matched budget.
4. **bindingdb (3,243,585 records) × conformal prediction** — intervals that hold their stated coverage; success criterion: empirical coverage at 90% nominal.
5. **bindingdb (3,243,585 records) × active learning** — choose the next 100 compounds to assay; success criterion: label efficiency at fixed accuracy.
6. **bindingdb (3,243,585 records) × abstention model** — refuse compounds outside the training range; success criterion: risk-coverage curve with ABSTAIN on out-of-domain.
7. **bindingdb (3,243,585 records) × multi-task model across targets** — share the chemistry signal across LpxC and its neighbours; success criterion: per-target R2 versus single-task baselines.
8. **bindingdb (3,243,585 records) × transfer from a large ChEMBL pretraining** — test whether 2.9M records help 406; success criterion: R2 gain at equal label budget.
9. **bindingdb (3,243,585 records) × ensemble with decomposed uncertainty** — separate model error from data noise; success criterion: calibration of the predicted interval.
10. **bindingdb (3,243,585 records) × retrieval baseline** — nearest-neighbour potency from fingerprints; success criterion: R2 versus the learned model.
11. **clinicaltrials (612,025 records) × transformer fine-tune** — extract endpoints and arms; success criterion: F1 on a hand-labelled sample.
12. **clinicaltrials (612,025 records) × survival analysis** — time-to-completion by design features; success criterion: concordance index on held-out trials.
13. **clinicaltrials (612,025 records) × causal inference** — effect of a design choice on completion; success criterion: stability across two identification strategies.
14. **clinicaltrials (612,025 records) × retrieval over protocol text** — find comparable arms before designing a study; success criterion: recall@10 on a curated set.
15. **clinicaltrials (612,025 records) × design-clone detection** — spot trials that duplicate an existing design; success criterion: precision at fixed recall.
16. **clinicaltrials (612,025 records) × endpoint harmonisation** — map heterogeneous endpoints to one ontology; success criterion: agreement on a labelled sample.
17. **clinicaltrials (612,025 records) × eligibility mining** — extract inclusion criteria as structured rules; success criterion: F1 against manual extraction.
18. **faers (421,404 records) × Bayesian signal detection** — disproportionality per event/drug pair; success criterion: posterior predictive check plus a labelled control set.
19. **faers (421,404 records) × survival analysis** — time-to-onset by reaction class; success criterion: concordance index.
20. **faers (421,404 records) × case/non-case design** — validate the signal against a control set; success criterion: odds ratio stability.
21. **faers (421,404 records) × temporal clustering** — detect reporting surges that are not safety signals; success criterion: false-alarm rate.
22. **faers (421,404 records) × class-effect comparison** — compare against the therapeutic class; success criterion: signal stability across quarters.
23. **faers (421,404 records) × serious-outcome stratification** — separate seriousness classes; success criterion: calibration by class.
24. **arc (291,778 records) × program synthesis** — derive a transform from few examples; success criterion: exact-match rate on held-out tasks.
25. **arc (291,778 records) × active learning** — choose demonstrations that generalise; success criterion: sample efficiency at fixed accuracy.
26. **arc (291,778 records) × curriculum from solved tasks** — order tasks by abstraction depth; success criterion: transfer rate to unseen tasks.
27. **arc (291,778 records) × program induction with a DSL** — search a small transform space; success criterion: exact-match rate on held-out tasks.
28. **arc (291,778 records) × compositional generalisation probe** — hold out a task family, test transfer; success criterion: accuracy drop on the held-out family.
29. **smpdb (147,932 records) × graph traversal** — place the target in Lipid A biosynthesis; success criterion: path completeness against the curated map.
30. **smpdb (147,932 records) × retrieval** — find pathway neighbours from free text; success criterion: recall@10.
31. **smpdb (147,932 records) × flux feasibility check** — is the proposed inhibition flux-consistent; success criterion: agreement with curation.
32. **smpdb (147,932 records) × intermediate enumeration** — list every intermediate for the pathway; success criterion: completeness against curation.
33. **smpdb (147,932 records) × enzyme annotation** — map enzymes to reactions without curation; success criterion: F1 on a curated subset.
34. **orange_book (48,990 records) × survival analysis** — exclusivity windows per product; success criterion: agreement with published expiry on a sampled set.
35. **orange_book (48,990 records) × regression** — predict exclusivity from product features; success criterion: MAE against a held-out year.
36. **orange_book (48,990 records) × patent-family linkage** — join products to their patent families; success criterion: linkage coverage.
37. **orange_book (48,990 records) × exclusivity-clock model** — date the first generic entry per moiety; success criterion: MAE in months.
38. **orange_book (48,990 records) × label-change tracking** — link safety label revisions to trial outcomes; success criterion: event recall.
39. **chembl (44,401 records) × gradient-boosted regression** — predict pKi from descriptors; success criterion: 5-fold CV R2 above the mean-predictor baseline.
40. **chembl (44,401 records) × kernel ridge regression** — small-n series with a smooth kernel; success criterion: leave-one-out R2 with a permutation control.
41. **chembl (44,401 records) × graph neural network** — learn from the molecular graph rather than descriptors; success criterion: improvement over the descriptor baseline at matched budget.
42. **chembl (44,401 records) × conformal prediction** — intervals that hold their stated coverage; success criterion: empirical coverage at 90% nominal.
43. **chembl (44,401 records) × active learning** — choose the next 100 compounds to assay; success criterion: label efficiency at fixed accuracy.
44. **chembl (44,401 records) × abstention model** — refuse compounds outside the training range; success criterion: risk-coverage curve with ABSTAIN on out-of-domain.
45. **chembl (44,401 records) × multi-task model across targets** — share the chemistry signal across LpxC and its neighbours; success criterion: per-target R2 versus single-task baselines.
46. **chembl (44,401 records) × transfer from a large ChEMBL pretraining** — test whether 2.9M records help 406; success criterion: R2 gain at equal label budget.
47. **chembl (44,401 records) × ensemble with decomposed uncertainty** — separate model error from data noise; success criterion: calibration of the predicted interval.
48. **chembl (44,401 records) × retrieval baseline** — nearest-neighbour potency from fingerprints; success criterion: R2 versus the learned model.
49. **tptp (28,346 records) × proof search** — machine-checkable derivations for a claim; success criterion: fraction of steps the prover closes.
50. **tptp (28,346 records) × transformer fine-tune** — premise selection; success criterion: recall of required premises.
51. **tptp (28,346 records) × abstention model** — refuse problems beyond the fragment; success criterion: coverage/precision on MPTP2078.
52. **tptp (28,346 records) × lemma retrieval** — find the premises a proof needs; success criterion: recall@k of required lemmas.
53. **tptp (28,346 records) × fragment verification** — check each step in a decidable fragment; success criterion: steps verified per proof.
54. **tptp (28,346 records) × proof-length analysis** — does the platform reason or retrieve; success criterion: length versus a retrieval baseline.
55. **eskape (25,888 records) × protein language embedding** — homologue detection for the target; success criterion: recall of known homologues at 1% FDR.
56. **eskape (25,888 records) × clustering** — group proteomes by resistance profile; success criterion: silhouette against a labelled subset.
57. **eskape (25,888 records) × structure-aware embedding** — fold recognition rather than sequence; success criterion: recall of known folds.
58. **eskape (25,888 records) × domain-level alignment** — find the target domain across pathogens; success criterion: domain hit rate.
59. **eskape (25,888 records) × taxonomic stratification** — does homology hold across Gram groups; success criterion: per-group recall.
60. **eskape (25,888 records) × off-target scan** — count human homologues above a similarity bar; success criterion: paralogue count at a stated threshold.
61. **ncbi_amr (8,143 records) × contrastive embedding** — place resistance determinants in a shared space; success criterion: recall@10 against a curated query set.
62. **ncbi_amr (8,143 records) × graph model** — co-occurrence of determinants across organisms; success criterion: held-out edge prediction AUC.
63. **ncbi_amr (8,143 records) × rule mining** — determinant combinations that predict phenotype; success criterion: precision at fixed recall.
64. **ncbi_amr (8,143 records) × phylogenetic correction** — remove lineage confounding from resistance association; success criterion: effect stability across trees.
65. **ncbi_amr (8,143 records) × Bayesian hierarchical model** — pool across organisms with partial sharing; success criterion: posterior predictive check.
66. **ncbi_amr (8,143 records) × motif/domain search** — locate the determinant in unannotated proteomes; success criterion: hit recall at 1% FDR.
67. **ncbi_amr (8,143 records) × network propagation** — spread resistance phenotype across a gene network; success criterion: precision@50.
68. **tox21 (7,831 records) × random forest classification** — label compounds per Tox21 endpoint; success criterion: PR-AUC on a held-out fold.
69. **tox21 (7,831 records) × multi-task neural network** — share signal across the 12 assay endpoints; success criterion: mean PR-AUC versus per-endpoint baselines.
70. **tox21 (7,831 records) × conformal prediction** — per-endpoint intervals; success criterion: coverage at 90% nominal.
71. **tox21 (7,831 records) × graph attention model** — learn assay-relevant substructures; success criterion: PR-AUC versus descriptors.
72. **tox21 (7,831 records) × structural-alert rules** — compare learned signal against medicinal-chemistry rules; success criterion: agreement rate per alert.
73. **tox21 (7,831 records) × probability calibration study** — are the probabilities usable for triage; success criterion: ECE on held-out data.
74. **tox21 (7,831 records) × assay transfer** — predict one endpoint from another endpoint labels; success criterion: PR-AUC gain over single-task.
75. **tox21 (7,831 records) × dose-response curve fitting** — recover AC50 with intervals; success criterion: coverage of the reported interval.
76. **card (6,463 records) × contrastive embedding** — place resistance determinants in a shared space; success criterion: recall@10 against a curated query set.
77. **card (6,463 records) × graph model** — co-occurrence of determinants across organisms; success criterion: held-out edge prediction AUC.
78. **card (6,463 records) × rule mining** — determinant combinations that predict phenotype; success criterion: precision at fixed recall.
79. **card (6,463 records) × phylogenetic correction** — remove lineage confounding from resistance association; success criterion: effect stability across trees.
80. **card (6,463 records) × Bayesian hierarchical model** — pool across organisms with partial sharing; success criterion: posterior predictive check.
81. **card (6,463 records) × motif/domain search** — locate the determinant in unannotated proteomes; success criterion: hit recall at 1% FDR.
82. **card (6,463 records) × network propagation** — spread resistance phenotype across a gene network; success criterion: precision@50.
83. **pdbbind (4,404 records) × gradient-boosted regression** — predict pKi from descriptors; success criterion: 5-fold CV R2 above the mean-predictor baseline.
84. **pdbbind (4,404 records) × kernel ridge regression** — small-n series with a smooth kernel; success criterion: leave-one-out R2 with a permutation control.
85. **pdbbind (4,404 records) × graph neural network** — learn from the molecular graph rather than descriptors; success criterion: improvement over the descriptor baseline at matched budget.
86. **pdbbind (4,404 records) × conformal prediction** — intervals that hold their stated coverage; success criterion: empirical coverage at 90% nominal.
87. **pdbbind (4,404 records) × active learning** — choose the next 100 compounds to assay; success criterion: label efficiency at fixed accuracy.
88. **pdbbind (4,404 records) × abstention model** — refuse compounds outside the training range; success criterion: risk-coverage curve with ABSTAIN on out-of-domain.
89. **pdbbind (4,404 records) × multi-task model across targets** — share the chemistry signal across LpxC and its neighbours; success criterion: per-target R2 versus single-task baselines.
90. **pdbbind (4,404 records) × transfer from a large ChEMBL pretraining** — test whether 2.9M records help 406; success criterion: R2 gain at equal label budget.
91. **pdbbind (4,404 records) × ensemble with decomposed uncertainty** — separate model error from data noise; success criterion: calibration of the predicted interval.
92. **pdbbind (4,404 records) × retrieval baseline** — nearest-neighbour potency from fingerprints; success criterion: R2 versus the learned model.
93. **chembl_lpxc (406 records) × gradient-boosted regression** — predict pKi from descriptors; success criterion: 5-fold CV R2 above the mean-predictor baseline.
94. **chembl_lpxc (406 records) × kernel ridge regression** — small-n series with a smooth kernel; success criterion: leave-one-out R2 with a permutation control.
95. **chembl_lpxc (406 records) × graph neural network** — learn from the molecular graph rather than descriptors; success criterion: improvement over the descriptor baseline at matched budget.
96. **chembl_lpxc (406 records) × conformal prediction** — intervals that hold their stated coverage; success criterion: empirical coverage at 90% nominal.
97. **chembl_lpxc (406 records) × active learning** — choose the next 100 compounds to assay; success criterion: label efficiency at fixed accuracy.
98. **chembl_lpxc (406 records) × abstention model** — refuse compounds outside the training range; success criterion: risk-coverage curve with ABSTAIN on out-of-domain.
99. **chembl_lpxc (406 records) × multi-task model across targets** — share the chemistry signal across LpxC and its neighbours; success criterion: per-target R2 versus single-task baselines.
100. **chembl_lpxc (406 records) × transfer from a large ChEMBL pretraining** — test whether 2.9M records help 406; success criterion: R2 gain at equal label budget.

## How to read this list

The first 12 directions (ChEMBL) are cheapest: the data is clean, the model class is proven on 406 actives at
CV5 R² 0.7226, and the binding affinity join (BindingDB) is one SQL join away. The directions involving
ClinicalTrials and FAERS are next, because both are large enough to support survival and signal-detection
work without new sources. The TPTP and ARC directions are the only ones that test **reasoning** rather than
fitting, and they are the ones most likely to fail honestly.

Two directions are conspicuously absent because the data is absent: any variant-level genomics direction, and
any imaging direction. Adding them to this list without the data would be the same error the suite makes in
Domain 2.

---

*Task 971 — 971-research-directions-100. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

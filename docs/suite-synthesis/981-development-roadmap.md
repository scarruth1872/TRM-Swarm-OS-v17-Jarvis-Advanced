# TRM development roadmap

## Where the platform is, measured

- kernel: **HEALTHY**, build `3a63c34+dirty`, uptime 506 min
- board: 27 cards — 6 todo / 0 doing / 0 review / 14 done / 7 retired
- federation: 3 of 3 peers active, mesh nodes 14, anchor `0x4D4F4E4144000001`
- DDR: 439 scans, mitigation `SOFTWARE_PARITY_SCRUB_AND_GATE`
- AMR node: **REFUTED** (3 refuted / 2 undetermined)

| domain | P | A1 | A2 | B | C | N | L | D | E | F |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Drug Discovery & Biomedical | 0 | 10 | 0 | 6 | 72 | 8 | 0 | 4 | 0 | 0 |
| Genomics & Variant Analysis | 0 | 0 | 0 | 0 | 64 | 4 | 0 | 32 | 0 | 0 |
| Network & Systems Engineering | 2 | 0 | 28 | 0 | 65 | 0 | 0 | 0 | 1 | 4 |
| Security & Cryptography | 0 | 0 | 7 | 0 | 86 | 2 | 0 | 0 | 4 | 1 |
| Data Science & Machine Learning | 0 | 8 | 13 | 2 | 69 | 1 | 0 | 6 | 1 | 0 |
| Knowledge Engineering | 0 | 21 | 2 | 0 | 77 | 0 | 0 | 0 | 0 | 0 |
| Hardware & Embedded Systems | 1 | 0 | 0 | 0 | 77 | 0 | 0 | 0 | 22 | 0 |
| Federation & Distributed Systems | 0 | 0 | 3 | 0 | 92 | 0 | 0 | 0 | 0 | 5 |
| Humanistic & Ethical Reasoning | 0 | 0 | 0 | 0 | 32 | 68 | 0 | 0 | 0 | 0 |
| Creative & Generative Work | 0 | 0 | 1 | 0 | 97 | 1 | 0 | 0 | 1 | 0 |
| **total** | **3** | **39** | **54** | **8** | **731** | **84** | **0** | **42** | **29** | **10** |

## Phase 0 — make the numbers falsifiable (now)
1. **Redock control**: CHIR-090 into 5U39/3TYZ, RMSD < 2.0 Å. Blocks every docking claim (8 tasks in the
   classification plus every affinity figure in circulation).
2. **Structure registry**: canonical SMILES for 3 leads, hashed, with the source file recorded (done as
   evidence, not yet as a registry).
3. **Claim register**: every document sentence cites a measurement id or is marked judgement. 731 tasks are
   documents; this is the only mechanism that makes any of them checkable.
4. **One science environment**: RDKit and sklearn in one interpreter, pinned (today they are split by a numpy
   ABI break).

## Phase 1 — exploit what is already here (weeks)
5. BindingDB × ChEMBL-LpxC × PDBbind join; the 406-actives model becomes a 3.5M-affinity model.
6. Tox21 endpoints joined to the lead series; measured liability instead of predicted.
7. CARD/NCBI-AMR co-occurrence for the target's resistance neighbourhood.
8. ClinicalTrials feasibility queries for any proposed study design.

## Phase 2 — close the hardware ceiling (months, gated by purchase)
9. CUDA toolkit or prebuilt kernels on the 3070 peer (unlocks 10 tasks).
10. TPM or a software attestation substitute (unlocks 3 tasks honestly, or renames them).
11. OpenMM on CPU for a 1 ns smoke test, so MD tasks fail with evidence rather than silence.

## Phase 3 — the parts that need a decision, not a build
12. Genomics data (ClinVar/gnomAD) — 32 tasks, licence and size decisions.
13. Scale honesty: either grow past 3 nodes or stop writing 584-agent objectives.
14. Federation attestation: can a peer verify a claim without trusting the sender? The sealed shards say yes;
    the ledger integration says not yet.

## What the roadmap deliberately does not contain

No item whose success criterion is "more agents", "a larger council" or "an improved architecture". Every item
above has a number that would show it failed.

---

*Task 981 — 981-development-roadmap. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

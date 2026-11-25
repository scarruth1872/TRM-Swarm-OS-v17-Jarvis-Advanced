# TRM Swarm OS capability report

## Answer

**96 of 1000 tasks can be executed and checked on this
deployment.** The rest are documents (731), normative claims (84), or blocked by data
(42), hardware (29) or scale (10). Docking (8) runs but
its control fails.

## Capabilities, with the evidence that backs each

| capability | evidence |
|---|---|
| compute descriptors and drug-likeness | RDKit 2026.03.6 on 406 actives, 406/406 parsed |
| predict potency with an error bar | GBM, CV5 R² 0.7226 ± 0.0565 vs baseline −0.0085 |
| recover lead chemistry from artifacts | 3 leads, SMILES from pose REMARK, 2 corroborated by 3D SDF |
| hold and verify a data corpus | 33 manifests, 16 distinct shards, 7,793,536 records, seals 33/33 |
| boot a custom UEFI kernel | image byte-identical to the boot media |
| run a 3-node federation | `/federation/stats`, 3 active |
| refuse an approval safely | gate veto-only, ABSTAIN on unknown |
| score docking | **engine yes, control no** — RMSD 13.82 Å |

## Capabilities the suite assumes that do not exist here

CUDA kernels, a TPM, 584 agents, 10 nodes, genomic variant data, imaging, mass spectrometry, OpenMM, a
validated docking pipeline, and any wet-lab measurement whatsoever.

## Live state

- kernel: **HEALTHY**, build `3a63c34+dirty`, uptime 506 min
- board: 27 cards — 6 todo / 0 doing / 0 review / 14 done / 7 retired
- federation: 3 of 3 peers active, mesh nodes 14, anchor `0x4D4F4E4144000001`
- DDR: 439 scans, mitigation `SOFTWARE_PARITY_SCRUB_AND_GATE`
- AMR node: **REFUTED** (3 refuted / 2 undetermined)

## Closing statement

This report was generated from measurements taken in the same process that wrote it, and it is itself class C:
the platform cannot verify its own prose. What it *can* verify it has: hashes, counts, cross-validated scores,
a failed control recorded. The distance between those two statements is the honest measure of this system
today — and closing it is cheaper than any hardware purchase on the roadmap.

---

*Task 1000 — 1000-capability-report. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

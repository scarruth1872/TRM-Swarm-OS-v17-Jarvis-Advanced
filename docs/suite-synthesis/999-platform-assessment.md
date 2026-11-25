# Complete platform assessment

## Against the 1000-task suite

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

Interpreted:

- **3 proven**, 39 computable today, 54 buildable and testable.
- **8 produce numbers whose control fails** — docking.
- **731 are documents the platform writes and cannot verify.**
- **84 are normative claims no measurement can settle.**
- **42 blocked on data, 29 on hardware, 10 on scale.**

## Against its own claims

| claim the platform has made | verdict |
|---|---|
| "verifiable artifacts for every task" | **false for 731 of 1000** |
| "584 agents" | false; 3 peers, 14 mesh nodes |
| "10-node federation" | false; 3 active |
| "hardware root of trust" | false; no TPM |
| "docking validates the leads" | **unfounded**; redock RMSD 13.82 Å |
| "TRM-ABX-220 has no SMILES" | false; recovered from pose REMARK + corroborated SDFs |
| "field-usable AMR node in Kisumu" | **REFUTED** by its own measured steps |
| "seals verify" | true; 33/33 payload hashes |

## The assessment

The platform is **honest where it measures and unreliable where it narrates**. Its engineering is real:
bootable kernel, sealed fabric, live endpoints, a model with an error bar, a board that can no longer be
closed silently. Its narratives — dossiers, deployments, swarms — have repeatedly outrun its measurements,
and each overrun cost credibility that took a measurement to recover.

The single most valuable change is not capability. It is refusing to publish a sentence that has no
measurement behind it, which is feature 9 of 50 and the only feature this assessment would fund first.

---

*Task 999 — 999-platform-assessment. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

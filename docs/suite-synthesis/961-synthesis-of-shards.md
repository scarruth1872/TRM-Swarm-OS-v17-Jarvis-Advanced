# Comprehensive synthesis of the shard corpus

## What is actually held

**16 distinct shards derived from 33 verified manifests, 7,793,536 records.** The remaining
17 manifests are benchmark and refactor builds of the same data and are not counted twice. Every shard body
was re-hashed today: the
manifest's `seal_sha256` covers the payload *after* a 128-byte `TURBOVEC` header, and **16 of 16**
match on that basis. (An earlier check that hashed the whole file reported 0/33 — that was the wrong test, not
a bad shard. The correction matters: a seal check that fails everything is a broken check, not broken data.)

| dataset | records | size | seal (payload) |
|---|---:|---:|---|
| arc | 291,778 | 56.0 MB | `5904b4fca7a01138…` verified |
| bindingdb | 3,243,585 | 622.8 MB | `1ec99469bfd3ac98…` verified |
| card | 6,463 | 1.2 MB | `b33e148f4eb7fb91…` verified |
| chembl | 44,401 | 8.5 MB | `ae79a8d4c6255314…` verified |
| chembl37 | 2,897,805 | 556.4 MB | `52a30b7eea062f07…` verified |
| chembl_lpxc | 406 | 0.1 MB | `d9835c7b7c136500…` verified |
| clinicaltrials | 612,025 | 117.5 MB | `3601b5badf29a1ac…` verified |
| eskape | 25,888 | 5.0 MB | `029d57f14865b4a8…` verified |
| faers | 421,404 | 80.9 MB | `40a1848dbc3c0094…` verified |
| ncbi_amr | 8,143 | 1.6 MB | `5e19d9c11020edc7…` verified |
| orange_book | 48,990 | 9.4 MB | `879d8ad31e9c9885…` verified |
| pdbbind | 4,404 | 0.8 MB | `9e0e203dc8f34460…` verified |
| smpdb | 147,932 | 28.4 MB | `a500dfa100b6ba2a…` verified |
| tox21 | 7,831 | 1.5 MB | `7d9f00fcbf8ae017…` verified |
| tptp | 28,346 | 5.4 MB | `22ecc748e125c6b5…` verified |

## What the corpus can and cannot answer

| question | answerable | by which shard |
|---|---|---|
| Is this compound measured against LpxC? | yes | ChEMBL LpxC (406) |
| What is its recorded affinity, and to what? | yes | BindingDB (3.24M), PDBbind (4,404) |
| Does a resistance determinant exist for the target? | yes | CARD (6,463), NCBI-AMR (8,143) |
| Is there a pathway context? | yes | SMPDB (147,932) |
| Has this been tried in humans, and did it fail? | yes, at trial level | ClinicalTrials (612,025) |
| What adverse events were reported? | yes, one quarter | FAERS 2026Q3 (421,404) |
| What is the patent/exclusivity clock? | approximately | Orange Book (48,990) |
| Is the claim logically derivable? | partially | TPTP MPTP2078 (28,346) |
| Any genomic variant, expression or imaging data? | **no** | nothing on this host |
| Any wet-lab result produced here? | **no** | none — no assay has run on this machine |

## The honest summary

This corpus is a **chemistry and regulatory** corpus with a resistance layer. It is not a genomics corpus and
it contains no wet-lab measurement produced locally. Synthesis that treats it as broader than that is where
the platform's past fabrications came from: the eCTD dossier, the ionosphere mesh and the Kisumu deployment
were all written as if data existed. The defensible synthesis is narrower and more useful: 7,793,536
records that can ground a claim about a molecule, a target, a resistance gene, a trial or an adverse event —
and nothing about a patient.

---

*Task 961 — 961-synthesis-of-shards. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

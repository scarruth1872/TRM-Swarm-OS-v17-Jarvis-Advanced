# Genomics knowledge synthesis — what exists, what does not

## The finding

**There is no genomics dataset on this host.** No ClinVar, no gnomAD, no VCF, no exome, no expression matrix,
no methylation or Hi-C file. A sweep of `F:`, `C:\Users\shawn`, and the mounted `D:`, `E:` and `G:` volumes
returned nothing; WSL cannot start on this machine (virtualisation disabled), so the 178 GB `ext4.vhdx` that
might hold such data is unreachable rather than empty.

What looks genomic in the corpus is **resistance genetics**, which is a different thing:

| dataset | records | size | seal (payload) |
|---|---:|---:|---|
| card | 6,463 | 1.2 MB | `b33e148f4eb7fb91…` verified |
| chembl_lpxc | 406 | 0.1 MB | `d9835c7b7c136500…` verified |
| eskape | 25,888 | 5.0 MB | `029d57f14865b4a8…` verified |
| ncbi_amr | 8,143 | 1.6 MB | `5e19d9c11020edc7…` verified |

## What can honestly be said

| question | status |
|---|---|
| Which genes confer resistance to this antibiotic class? | answerable from CARD (6,463) and NCBI-AMR (8,143) |
| Which pathogens carry homologues of the target? | answerable at protein level from ESKAPE (25,888) |
| Does this variant of uncertain significance cause disease? | **not answerable** — no variant data |
| Is this gene under selection in clinical isolates? | **not answerable** — no genomes |
| What is the population allele frequency? | **not answerable** — no gnomAD |

## The structural point

32 of the suite's 100 genomics tasks need variant-scale data. The platform's habit is to answer such tasks
with a document that *sounds* like an analysis; that is precisely the failure mode the audit found in the eCTD
card. The correct output for all 32 is a refusal with the missing input named — which is what
`tools/suite_classify.py` now produces for each of them.

## What would change it

A ClinVar release (variant + assertion), a gnomAD subset (frequencies by population), and one reference genome
would move ~30 tasks from blocked to runnable. Each is a public download; none is present today; each requires
an explicit decision about licence and size before it is fetched.

---

*Task 963 — 963-genomics-synthesis. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N

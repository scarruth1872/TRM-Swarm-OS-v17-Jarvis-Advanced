> **UPDATE 2026-09-23**
> SUPERSEDED for the record counts by STRUCTURED_INDEX_FULL / CHEMBL_REFETCH_COMPLETE: the index now holds
> 2,897,819 molecules (complete release), not 250,000. The method and the three format corrections below
> still stand as written.
>
> **UPDATE 2026-09-26 (block 161)** — the span decision went to the council and was realized; the full record
> is `docs/STRUCTURED_INDEX_SPAN_20260926.md`. What the store holds now: `structure` **20** rows (was 9 — entry
> 6, derived 3, ligand 6, receptor 5, with no duplicate `pdb_id`), `binding` **19,443** rows (LP-PDBBind's
> affinity table, which the store could not see before), `span` **11** declared trees (6 INGESTED, 5
> OUT-OF-SPAN with reasons), `molecule_features` owned by the generator again, and the ChEMBL provenance row
> now carrying the **full-file** sha256 `2cdf4d33c5f42613…` with `hash_span = full` (it was a 200 MB prefix
> hash). The shipped store is **1,125,068,800 B**, sha256 `c347a05036d66b8d…`, built in 249.7 s with the
> derived structures written into the rows during the build (2,897,805 of 2,897,819 = 0.999995); the same
> content before that input measured 754,692,096 B, i.e. 344.8 MB *smaller* than the file it replaces, and the
> shipped store is 25.6 MB larger only because it carries 2.7 M more structures.

# Structured index for the WSL data (2026-09-23)

Decision taken: **the fabric stays for prose; structured records get a SQLite index.** Embeddings of
100k near-duplicate molecules would degrade the fabric's already-weak short-query discrimination
(33-47% top-1), and a molecule wants an exact lookup, not a cosine. Built with stdlib only - no new
dependency, no kernel surface, offline tooling as the doctrine requires.

Generator: `swarm_v2/core/build_structured_index.py` (kept beside the index so it is reproducible).

## What is in it

```
ethics      95,847 rows   5 subsets, EVERY row with text (virtue 28,245 | justice 21,791 |
                          deontology 18,164 | commonsense 13,910 | utilitarianism 13,737)
molecule   250,000 rows   250,000/250,000 with a real ChEMBL id
clinical   201,572 rows   40 distinct measurements, 400 patients, NaN skipped
prose        1,633 rows   Shakespeare / Law of One / wisdom texts, 4,000-char chunks
structure        9 rows   PDB entries with resolution + atom counts
                                                    index 112,627,712 B  sha256 40c72aaac1ca444a...
```

Every row carries `source_path` + `source_sha256`, so a lookup traces to its record.

## Three formats I mis-modelled first, and how it showed

The first build reported **1,000,000 molecules** - and every one was empty. `chembl_id` and `smiles`
were NULL throughout. Row counts looked like success; querying the *content* is what exposed it. Two
more empty-content cases surfaced the same way. The actual formats:

| source | actual | assumed |
|---|---|---|
| ChEMBL SDF | the record's **first line** is the id (`CHEMBL153534`) | a `> <ChEMBL_ID>` tag that does not exist |
| clinical `.psv` | **transposed**: the header is the measurement list (`HR\|O2Sat\|...\|SepsisLabel`), rows are timepoints | a (patient, itemid, value) table |
| ETHICS CSVs | five different schemas: `input` (commonsense), `scenario` (deontology/justice/virtue), `baseline`+`less_pleasant` (utilitarianism, and it has **no label column**) | one column named `text` |

The clinical one was a design error, not a typo: the format is a timepoint matrix, so the index now
stores it long (`patient_id, timepoint, measurement, value`) with NaN skipped, and
`WHERE measurement='HR'` returns real vitals. The ethics fix keeps the columns it does not index in a
`meta` JSON field rather than dropping them.

Rule this reinforces: **assert on content, never on row count.** A table of N empty rows passes every
count-based check.

## Span, decided from measurement rather than a round number

Marginal ingest rate, measured on this host: 250k records in 6.9 s, 1M in 37.4 s - the rate **degrades
with size** (SQLite insert + index growth), so the honest figure is ~25,000 records/s at scale, not the
136,000/s a small run suggests. Index cost is **~145 bytes/molecule**.

| span | time | index |
|---|---|---|
| 250k (current) | ~7-11 s | 112.6 MB |
| 1,000,000 | ~37 s | 171.6 MB (measured) |
| all ChEMBL (~2.4M) | ~100 s | ~320 MB (extrapolated at 145 B/record) |

**Recommendation: ingest all of ChEMBL.** The binding constraint is disk, not time, and C: has 119 GB
free - so the "bound" is not needed for this table. Everything except ChEMBL is already ingested in full
(the ethics CSVs are 23 MB total). If a bound is wanted anyway, the current 250k is a reasonable
committed default and costs nothing to raise.

For contrast, putting this same data through the fabric encoder would have been ~169 docs/s (1.6 hours
for 1M) and would have made retrieval worse. The index does it in 37 seconds and answers exactly.

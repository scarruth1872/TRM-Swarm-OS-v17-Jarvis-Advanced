> **UPDATE 2026-09-23**
> SUPERSEDED for the ingest decision by CHEMBL_REFETCH_COMPLETE: the WSL ChEMBL file was a TRUNCATED
> download (78.8% present), so 'bounded sample' framing was hiding a data-integrity fault. The release was
> re-fetched, checksum-verified, and fully ingested (2,897,819 records). No bound is needed - disk was never
> the constraint.

# WSL reconciliation: the last two items (2026-09-23)

## The unclassified directories, classified

| dir | size | newest | verdict |
|---|---|---|---|
| `relational` | 234 MB | 2026-08-13 | **HARVEST** - real training corpora: `ethics_commonsense.csv`, `ethics_deontology.csv`, `ed_train.csv`, `enwikiquote.xml.bz2` |
| `hermes-archive` | 39 MB | 2026-08-16 | **HARVEST as reference** - prior Hermes sessions (`session-*.jsonl/.md`) + `PROJECT_CONTEXT.md` |
| `llama.cpp` | 997 MB | 2026-07-15 | reference - upstream checkout, superseded by the runtime's b10964 |
| `mamba-server` | 964 MB | 2026-06-03 | superseded - the SSM now lives in the Rust `swarm-ssm` crate |
| `voicebox` | 6.9 GB | 2026-08-13 | other workstream (voice), not council material |
| `anythingllm` | 134 MB | 2026-06-02 | third-party app storage, not ours |
| `Agent-Q-master` / `-old` | 3.1 GB | 2026-08-24 | other workstream (Agent-Q / Hera7), not council material |

`relational` is the find: it is exactly the "wisdom / philosophy corpora" row of the pre-ingestion plan, and
it sits un-ingested on the WSL side next to ChEMBL, PDBbind and the clinical records.

## The ingestion design (the plan's open question)

**The constraint.** `build_kernel_knowledge_index.py` embeds **text** (768-dim, 2 bits/dim, 180 docs in
1.07 s = ~169 docs/s). The WSL payload is mostly **structured**: `chembl_37.sdf.gz` (701 MB), `LP-PDBBind`,
400 clinical `.psv` records, 29 PDB structures, and these new CSVs. None of those is a text document, so
"ingest 48 GB" is not an encoder run.

**The decision: convert to text documents and reuse the existing pipeline - do not build a parallel store.**
The doctrine prefers wiring existing machinery over new surface, and a second store type would need its own
query path, its own gate and its own parity story. Instead: one **document per record**, rendered as text
with its provenance, fed through the encoder that already works.

Shape per record (the encoder sees only text, so the text must carry the meaning):

```
source: chembl_37.sdf.gz#CHEMBL12345   (sha256 of the source archive recorded once)
molecule: <SMILES>
properties: mw=..., logp=..., hbd=..., hba=...
targets: <names>
notes: <assay summary, first N chars>
```

**Provenance is not optional.** Each document carries `source file` + the archive's sha256, so a retrieval
can be traced back to the record it came from - the same rule the ledger already applies to stores.

**Scale, honestly.** At ~169 docs/s, 10,000 records is ~1 minute, 100,000 is ~10 minutes, 1,000,000 is
~1.6 hours. ChEMBL alone is millions of records. So the first pass should be a **bounded, stated sample**
(e.g. 100k across sources, randomised with a fixed seed) rather than "everything", and the summary must say
which fraction was ingested. An unbounded ingest is an unbounded runtime with no receipt.

**What this does NOT do.** It does not make the fabric "know chemistry" in a way that can be checked by
retrieval accuracy - the fabric's short-query discrimination is already weak (33-47% top-1, from the 2-bit
store). Adding 100k near-duplicate molecules will make that worse, not better, because every record will
resemble every other. If the goal is queryable chemistry, the honest first step is a **structured** index
(exact substructure/property filters) with the fabric used for prose only. That is a design fork worth
deciding explicitly rather than discovering later.

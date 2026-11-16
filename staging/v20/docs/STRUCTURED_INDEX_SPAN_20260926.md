# Structured-index span: the council's decision, and the store rebuilt to it (2026-09-26)

## The engine that decides had to be fixed before it could decide

`trm_uefi_kernel/host_council_deliberation` declared its own roster four ways at once: the banner said
**15** agents, the doc comment **14**, the sealed report wrote `"total_agents": 14` **and**
`"pbft_threshold": 10`, while the vote path actually compared `approve >= 9`. The code path was the correct
one (PBFT needs N >= 3f+1, so with N=15 f=4 and the quorum is 9); three of the four declarations were not,
and they disagreed with each other. Worse, the vote table carried **seven arms for `LogicMaster`** — an agent
absent from its own `AGENTS` table — so those arms, each carrying a binding abstention condition, were
unreachable code.

Fixed in the engine, not in the report after it:

- `LogicMaster` added to `AGENTS` at the reasoning-peer weight (oracle `agent_intelligence_bridge.AGENT_MODELS`
  loss 0.212 against Logic's 0.224), which makes the seven arms live.
- `TOTAL_AGENTS`, `BYZANTINE` and `PBFT_QUORUM` are now derived from that one table, and the banner, the
  per-motion threshold line, the summary text, the ranked-direction strings and the JSON all read from them.
  The count can no longer drift from the roster.

The sealed Sep-10 session (7 motions) is preserved at
`swarm_v2_artifacts/council_strategic_deliberation_20260910_7motions.json` and
`backups/council_engine_pre/council_strategic_deliberation_20260910.json` (sha256 `60f388016c3d0c2c…`).

## The verdict

`council_strategic_deliberation.json` — 15 agents, 10 motions, all enacted, anchor `0x4D4F4E4144000001`,
`total_agents: 15`, `pbft_threshold: 9`, `byzantine_tolerance: 4`.

| motion | tally | score | binding abstentions (conditions, not neutrality) |
|---|---|---|---|
| MOTION-008 index span | 12/3/0 | +0.679 | **Shield**: provenance must identify the artifact it names — the recorded sha256 is a 200 MB prefix hash and cannot. **Verify**: record the full-file sha256 and a hash span before any row claims to be verified. **Archi**: a pdb_id taken from a filename is not an identifier. |
| MOTION-009 structures | 13/2/0 | +0.718 | **Devo**: measure the pass rate on 5,000 records first — rdkit is absent from the WSL interpreter. **Verify**: every derived smiles must cite the source sha256 that produced it. |
| MOTION-010 reachability | 13/2/0 | +0.687 | **Devo**: no live consumer calls for a route today; wire the seam when a task path does. **Archi**: a SQLite binding inside the kernel is a dependency the narrow waist does not need. |

## What the measurement found (and one thing it corrected)

| claim | measured 2026-09-26 |
|---|---|
| provenance can identify the ChEMBL source | **No** — `provenance.sha256` held `348d9f31647dd40f…`, the builder's `cap=200MB` **prefix** hash, while the file's real hash is `2cdf4d33c5f426130c610b8627c3702804e6f10d149d76b8a1bea674fa605a77` |
| the ChEMBL source is truncated | **No longer** — the builder's docstring still said so; re-measured: `gzip -t` OK, 931,156,420 B, trailer declares 3,703,817,902 uncompressed, full-file sha256 **matches the release's own checksum**, 2,897,819 records to EOF. The docstring was stale and is corrected. |
| `structure.pdb_id` is an identifier | **No** — it was `name[:4].upper()`, so `receptor_clean.pdb` became `RECE` and `1M17_prot.pdb` collided with the real `1M17` |
| LP-PDBBind is indexed | **No** — `datasets/LP-PDBBind/dataset/LP_PDBBind.csv`, 10,897,207 B and 19,443 labelled protein-ligand rows, had **zero** rows in the store |
| the docking ligands are indexed | **No** — 11 `.pdbqt` poses plus the receptors were invisible to the store |
| the feature table has one owner | **No** — `molecule_features` was created by an out-of-band scratch script, so the generator could not reproduce the store it claims to own |
| the store is 507 MB | **No** — 1,099,448,320 B live, and it held `molecule.smiles` for 199,997 of 2,897,819 rows (6.90%) |
| the builder's docstring describes this build | **No** — it still said the ChEMBL source is truncated (re-measured: `gzip -t` OK, sha256 = the release checksum, the stream ends cleanly) and declared an `--enwiki-mb` input that no code defines and no ingest branch reads |

## What changed, in the generator

`swarm_v2/core/build_structured_index.py` — the store's one owner — now carries:

- **`provenance.hash_span`**, and the ChEMBL row hashes the **full file**; a capped hash would have to declare
  itself (`prefix:<n>B`) rather than pass as the artifact's identity.
- **`structure.kind` / `structure.parent`** — `entry` (id from the HEADER record), `derived` (parent from its
  filename), `receptor`, `ligand` — with a unique index on `path`, so a filename fragment is never mistaken
  for a PDB entry again.
- **`binding`** — LP-PDBBind's 19,443 rows. The source header is shifted one cell against its own data
  (measured: index 2 is the SMILES, 5 the resolution, 13 the kd/ki, 14 the value — exactly where the DATA puts
  them), so columns are taken by index and each row is also kept verbatim in `payload`.
- **`span`** — 12 trees as of block 169, each measured at build time: 7 `INGESTED`, 5 declared `OUT-OF-SPAN`
  with a reason (`tox21full.csv.gz` is **INGESTED** — the Tox21 assay matrix, stored long, one row per
  measured cell — while `tox21full` stays OUT-OF-SPAN because it is the *generator*: a 352 KB git checkout
  with no data files; the LP-PDBBind archives are
  model code and weights while the affinity table beside them *is* ingested; `hw_kb`/`rust_kb` are the
  platform's own sources).
- **`molecule_features`** — the schema's one owner again, loaded from `drugs/molecule_features.csv`. The CSV
  carries 12 columns and 8 are published; `hba` is an N+O count (verified on 4,000 molecules) and `hbd` matched
  RDKit on 91.7%, so neither is published under a Lipinski name — P5's decision, kept rather than re-litigated.
- **derived structures are an input to the build, not a patch after it** (`--structures`,
  `swarm_v2_artifacts/chembl_structures_full.jsonl`). The join is a single lazy cursor: the pass walks the SDF
  in source order and so does this ingest, and a record whose structure is absent makes the cursor **wait**
  rather than scan — scanning forward would walk past every later match and quietly empty the column after the
  first record the parser skipped. Matching 2,000/2,000 with 0 skipped in the sandbox, and the sampled
  molecule's `smiles` and `props` are byte-equal to the file's. This is why the separate
  `scratch/fill_structures_full.py` is **not** what built the final store: it exists for a store built without
  the JSONL, and running it after the fact is exactly the post-hoc UPDATE churn that left the old file with
  495.9 MB of B-tree for 146.1 MB of rows.

## The structures (MOTION-009), and the abstention it answered

The motion's condition came from an abstention — **Devo**: "measure the pass rate on 5,000 records first; rdkit
is absent from the WSL interpreter the ingest runs under". Both halves were true and both were handled:
rdkit **is** absent from WSL's default `python3`, so the pipe keeps WSL for the stream (`zcat`) and runs the
chemistry on the Windows side under `py -3` with **rdkit 2026.03.6**; and the rate was measured before the full
run, exactly as asked:

| run | records | rate | extrapolation |
|---|---|---|---|
| 5,000 | 5,000 | 164.59 rec/s (includes a fixed cold-start cost) | — |
| 50,000 | 50,000 | 1,229.5 rec/s (marginal 1,297) | 39.3 min for the full table |

The full pass then produced **2,897,805 structures from 2,897,819 records (14 unparsable) in 2,430.65 s at
1,192.2 rec/s** — 40.5 min against the 39.3 predicted, so the rate held rather than drifting.

- `swarm_v2_artifacts/chembl_structures_full.jsonl` — **535,069,297 B, 2,897,805 lines**, sha256 `e2a11aa66015b2af…`
- `swarm_v2_artifacts/chembl_structures_full_receipt.json` — `fraction_of_store: 0.999995`, method
  `MolFromMolBlock(sanitize=True, removeHs=True) → MolToSmiles`, props from `MolWt`, `MolLogP` (Crippen),
  `NumHBD`, `NumHAcceptors`, `HeavyAtomCount`
- the 14 records the parser could not read stay **without** structures and are counted, not interpolated
- the earlier 6.90% was a *deliberate* P4 bound for a chemistry query, not a parser limit: this closes it to
  99.9995%

**Verify's** abstention — "every derived smiles must cite the source sha256 that produced it" — is answered at
the altitude the store actually works at: the `molecule` table's provenance row carries the ChEMBL source's
**full-file** sha256 with `hash_span = full`, the JSONL is an artifact with its own hash, and the store records
which path built it, so a derived structure traces source-hash → script → JSONL-hash → store row.

## The rebuild, and the swap

Built BESIDE the live store and swapped only after verification, never edited in place. Two builds, because the
derived structures became a build input mid-flight — the second is the one that ships:

| build | seconds | bytes | sha256 | structures in the rows |
|---|---|---|---|---|
| without the structures input | 185.5 | 754,692,096 | `261636511082407f…` (from that run's stdout; the file was replaced) | 0 |
| with it (**shipped**) | 249.7 | 1,125,068,800 | `c347a05036d66b8d…` (verified at the live path after the swap) | 2,897,805 |
| + the tox21 table (**shipped**, block 168) | 313.8 | 1,249,710,080 | `6d2ab79de28600dd…` (verified at the live path after the swap) | 2,897,805 |
| + the span declaration (**shipped**, block 169) | 312.9 | 1,249,710,080 | **`521ff968a5cb2d8c…`** — the live store now | 2,897,805 |

```
ethics 95,847 | clinical 201,572 | structure 20 | prose 1,633 | binding 19,443 | tox21 577,987
molecule 2,897,819 (full stream to EOF, 174.9 s, [2897805 structures]) | molecule_features 2,897,818
span 12 | provenance 436        <- as of block 169; span 11 / provenance 435 was the block-161 build
structure kinds: entry 6, derived 3, ligand 6, receptor 5   —   duplicate pdb_id: none, duplicate path: none
```

**The bytes, measured with `dbstat` against the file it replaced:**

| object | new | live (replaced) | delta |
|---|---|---|---|
| `molecule` | 730.7 MB | 385.7 MB | **+345.0 MB** — 2,897,805 structures where the old file held 199,997 |
| `molecule_features` | 146.1 MB | 495.9 MB | **−349.8 MB** — the old copy's B-tree was ~3.4x its data |
| `binding` | 29.7 MB | 0.0 MB | +29.7 MB — the LP-PDBBind table the store could not see |
| **file** | **1,125.1 MB** | **1,099.4 MB** | **+25.6 MB** |

Read that honestly: the store is **25.6 MB larger** and carries **2.7 million more structures**. For the same
content — before the structures input — the rebuild measured 754,692,096 B, i.e. **344.8 MB smaller** than the
file it replaces. The row counts *and* the row byte sums (`sum(length(formula))` = 31,304,959 B) are identical in
both files, so the features delta is page-fill and not content; neither file has free pages (`freelist_count` = 0
in both). Content was verified from the file at the live path, not from the build log: `integrity_check` ok,
coverage **0.999995** — which is the pass receipt's own `fraction_of_store` to six digits — props json-valid for
every one of the 2,897,805, and three ids read out of the store matching the JSONL on **both** `smiles` and
`props`.

**The swap** (`scratch/swap_structured_index.py`, sandbox-proven first against a fake live path): the old store
is kept, not deleted — `backups/structured_index_pre_span_20260926/structured_index.db`, sha256
`212b0b1b5f937a42…` — and the file now at the live path re-reads as the candidate, sha256 `c347a05036d66b8d…`,
with every table count matching (`verified_from_live_path: true`). Receipt:
`swarm_v2_artifacts/structured_index_swap.json`.

## Reachability (MOTION-010): offline tooling, with its consumers named

Kept offline. The store has **no kernel surface and no kernel dependency**; its consumers are named so the
decision is a record rather than an absence:

| consumer | what it does |
|---|---|
| `swarm_v2/core/build_structured_index.py` | the store's one owner: rebuilds every table and the span declaration |
| `scratch/fill_structures_full.py` | applies the derived structures and asks the store a chemistry question |
| `scratch/p4_fill_structures.py`, `scratch/p5_wire_features.py`, `scratch/p5_hba_definition.py` | the earlier fill/wiring steps, kept as their own receipts |
| `scratch/validate_platform_capabilities.py` | reads the store when validating platform capabilities |
| `docs/STRUCTURED_INDEX_20260923.md` | the store's own doc |

Archi's abstention stands as the boundary: when a task path genuinely needs a resolver seam, it gets one —
the fabric stays for prose and the exact store stays for exact lookups.

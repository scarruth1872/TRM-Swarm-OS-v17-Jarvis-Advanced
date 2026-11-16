# WSL Reconciliation: investigation and dev plan (2026-09-23)

## What was found

Two trees, one lineage — **not** two peers:

| | WSL `/home/shawn` | Windows `Downloads/TRM-Swarm-OS-v19-…` |
|---|---|---|
| version | **v18.233 Enterprise** (2026-08-23) | **v19.001** (current, active) |
| Rust | `trm-rust/` — **35 crates**, ~2.8k lines, 1 squashed commit, dirty | `trm_uefi_kernel` (183 modules, 16.4 MB EFI) + `swarm_v2_rust_kernel` (the DLL) |
| toolchain | **none** (no cargo anywhere) → cannot build | cargo + `x86_64-unknown-uefi`, builds in ~13 s |
| fabric | none found | `.k3` 153,523,394 trits live on :8022 |
| knowledge | `rust_kb/`, `hw_kb/`, `datasets/` (LP-PDBBind), `clinical/` (400 `.psv`), `drugs/` (chembl_37.sdf.gz 701 MB), `docking/` (29 PDB), `tox21full/` | **2,180** indexed text docs × 768 dims (live `semantic_index.json`, 53 MB) **plus a 1.25 GB SQLite store this plan did not know about**: molecule 2,897,819 / molecule_features 2,897,818 / binding 19,443 / ethics 95,847 / clinical 201,572 / prose 1,633 / tox21 577,987 / structure 20 |
| models | `models/` 4.6 GB (`bonsai-27b-q1_0.gguf`, `instella-gguf/`, `qwen-moe-2.4b`) + `prism-ml-llama.cpp/` (the g64 fork) | `D:\trm-models` (the `_g64` set) |
| live service | none | :8022 + :5183 + :8587 |

The knowledge bases are the headline: **the pre-ingestion plan's categories B, C and D are already
downloaded and organised in WSL** — they were simply never ingested into the v19 fabric. That is the
reconciliation's real payload, and it is data, not code.

## The dev plan, in dependency order

**Step 0 — Freeze the WSL side first (highest priority; nothing else is urgent but this is).**
`trm-rust` carries **untracked** work with no other copy: `swarm-ssm/src/fractal_mamba3.rs`,
`research/MOTION_A_RUST_COUNCIL_ABSORPTION.md` (a council-motion design doc), `hard_problems_results.json`,
`known_knowledge_results.json`, `swarm_impossible_results.json`, `voltage_node`, `voltage_offload`.
The tracked diff is only 1 line; the untracked set is the risk. Commit + archive the repo out of WSL and
hash-manifest it **before** any cleanup or dedupe touches that filesystem.

> **VERIFIED DONE 2026-09-26 — already performed on 2026-09-23, so it was checked rather than repeated.**
> Commit `89d2d75855a18056a000bcb6b704d81fd40cc819` (2026-09-23 18:02:03) captures all seven files, and each
> now reads back as **tracked**; `git status` on `~/trm-rust` is clean. Archive:
> `backups/wsl_freeze_20260923_180202/trm-rust_source.tar.gz`, **446,867,446 B, sha256 `a2b3f9a737affc90…`,
> 3,134 entries**, plus `FREEZE_MANIFEST.json` / `home_inventory.tsv` (1,260 rows) /
> `knowledge_sha256.txt` (509 files) / `models_manifest.tsv`, and a 3.7 GB `wsl_crates/trm-rust` copy out of
> WSL. The archive's own manifest hash matches the bytes on disk.

**Step 1 — Make "reconciled" a measured claim.** Hash-manifest both trees; classify every WSL artifact as
*harvest* / *superseded* / *duplicate*, each with its evidence (a crate v19 already implements is
superseded; a 701 MB ChEMBL archive v19 lacks is harvest).

> **CLOSED 2026-09-26; counts advanced twice since (blocks 165 → 168, see “Since the plan was written”).**
> `swarm_v2_artifacts/wsl_reconciliation_manifest.json` (22,383 B) classifies **all
> 55** groups the freeze inventories describe: **6 harvested, 3 harvested-prose, 3 harvest, 13 superseded,
> 24 archive-only, 5 ephemeral, 1 never-harvest** (`.ssh` — credentials, excluded from every archive by
> policy). "Harvested" is not prose: the live store's `provenance` table is the ledger — **436 sources,
> 1,179,565,334 B, 6,490,967 records** — so `drugs` (molecule 2,897,819 + features 2,897,818), `datasets`
> (binding 19,443), `clinical` (201,572 rows from 400 `.psv`), `relational` (ethics 95,847 + prose 1,633),
> `docking` (20 receptor rows) and **`tox21full.csv.gz` (577,987 measured assay cells)** are in the platform.
> As first sealed, `rust_kb`/`hw_kb`/`tox21` were provably absent (no tox table existed at all); all three
> have since been harvested, and prose gets its own class (`harvested-prose`) checked against the knowledge
> index rather than the store, because prose never enters provenance.
> `scratch/build_wsl_reconciliation.py` refuses to write unless every group is classified, every `harvested`
> appears in provenance, every `harvest` does not, and every `superseded` counterpart path exists — it passed
> on all four rules. Note the third class came up **empty**: no WSL artifact measured as a byte-duplicate of a
> v19 one, and the two near-twins in the model set are a different quant (q1_0 vs `_g64`), so nothing is
> claimed as duplicate without a hash to show for it. Full write-up:
> `docs/WSL_RECONCILIATION_MANIFEST_20260926.md`.

**Step 2 — Harvest the knowledge bases, but the fabric's ingestion path does not fit them.** ← the
non-obvious finding. `build_kernel_knowledge_index.py` embeds **text** (180 docs × 768 dims, 2 bits/dim).
`chembl_37.sdf.gz`, `LP-PDBBind`, the `.psv` clinical records and the PDB structures are **structured
data**. So "ingest 48 GB" is not an encoder run — it needs a structured-store design (or a text-conversion
step) decided first. This is a design decision, not a data-movement task.

> **CLOSED 2026-09-26 (block 161).** The decision had already been taken in the store's own docstring on
> 2026-09-23 — the fabric stays for prose, structured records go to SQLite ("a molecule wants an exact
> lookup, not a cosine") — so what remained was to take it to the council and close the gap between what the
> store DECLARED and what it MEASURED. Measured: `provenance.sha256` held a 200 MB **prefix** hash while the
> file's real hash is `2cdf4d33c5f42613…` = the release's own checksum; `structure.pdb_id` was
> `name[:4].upper()`, so `receptor_clean.pdb` became `RECE` and `1M17_prot.pdb` collided with the real
> `1M17`; LP-PDBBind's **19,443** affinity rows were on disk and invisible to the store; the feature table
> was owned by an out-of-band scratch script; the store was cited at 507 MB while measuring 1,099,448,320 B;
> and the docstring claimed an `--enwiki-mb` input that no code defines. Rebuilt by its one owner (185.5 s),
> verified against the sources, and swapped in — see `docs/STRUCTURED_INDEX_SPAN_20260926.md` and
> `swarm_v2_artifacts/structured_index_span.json`.

**Step 3 — Harvest then retire the 35 crates.** v18.233 is superseded by v19, but `swarm-brain` (338 lines),
`swarm-intel` (263), `swarm-docking` (226), `swarm-drug` (176), `swarm-fit` (136) and `swarm-proof` (134)
may hold unique logic. Port them with the discipline already running in this campaign — port → wire →
receipt — then retire the workspace, never before the archive exists.

**Step 4 — Decide what WSL is *for*.** With no toolchain it cannot build Rust, so it should not be a second
Rust host. Recommended role: **data/archive + the `.so` build host** — `swarm_v2/core/build_kernel_ops.sh`
exists precisely to produce `libswarm_kernel_ops.so` with g++ on Linux, which is the artifact the PHOENIX
report names and which is absent.

**DONE 2026-09-26 (block 160).** The recommendation was confirmed by measurement rather than adopted:
`command -v cargo` → **MISSING** (and `~/.profile` points at a `~/.cargo/env` that does not exist), while
`g++` (15), `nm` and `python3` are present; WSL `/` holds **808 GB free** against `C:`'s 119 GB. So the role is
**data/archive + the Linux build host for `.so` artifacts** — explicitly *not* a Rust host. The artifact now
exists: `swarm_v2/core/libswarm_kernel_ops.so`, **21,632 B, sha256 `7aff4be98f8889c1…`, 8 exports**, ruled
**29/0** against the Windows DLL (same source, identical inputs; mamba3 max deviation **5.2e-08** over 100
steps, gate ids/active/weights exact) and exercised through its own consumer
(`swarm_v2/core/native_ops_bridge.py`, which raises `FileNotFoundError` without exactly this file and now
returns `mamba3 step 0.03200000151991844`). Producing it required fixing a real defect:
`swarm_kernel_ops.h` used `size_t` without `<cstddef>`, which GCC 15 rejects — the Windows DLL predates that
toolchain, so the `.so` build script could never have run here. Details, limits and the full proof chain:
`docs/WSL_STEP4_KERNEL_OPS_20260926.md`.

**Step 5 — Models: dedupe opportunistically, watch C:.** WSL `models/` holds `bonsai-27b-q1_0.gguf`
(3.8 GB — a *different quant* from the Windows `_g64` set), `instella-gguf/` and a Qwen MoE. WSL `/` has
**808 GB free**; `C:` has **119 GB free at 88% used**. The Windows side is the space constraint for any
large ingestion, not WSL.

**Step 6 — Re-derive the plan's current-state numbers.** From the validation of the same document:
85,532,109 trits → **153,523,394**; "360x" → **158.6×**; "612K tasks/s" is a parameter, not a measurement;
"4 silicon fabrics" → 4 *stores* / 18 shards.

> **CLOSED 2026-09-26, re-derived from the runtime rather than from prose.** Every figure was taken live off the
> running kernel and the files on disk:
> - `swarm_v2_nand_fabric.k3` = **76,761,697 B × 2 trits/byte = 153,523,394 trits** — an exact match, and
>   `/k3/status` states the same rule itself (`"encoding": "one nibble per trit, 2 trits/byte (the store's own
>   rule)"`) together with the honest scope of its own audit (`"scope": "first 4 KiB only - not a whole-store
>   audit"`, `illegal_pct 0.0`).
> - **158.6×** confirmed: 153,523,394 / 968,000 = 158.6. The "360x" in
>   `A_MONTH_OF_CONTINUOUS_WORKLOADS_HARVEST.json` is ~2.27x too high, and that file is a **historical record**,
>   so the correction lives in `docs/VALIDATION_PREINGESTION_PLAN_20260923.md`, not in the record itself.
> - **612K tasks/s** confirmed as an input: it is `task_throughput_per_sec` inside `execution_parameters` of that
>   same harvest JSON — a parameter, and never a measurement to cite.
> - **4 *stores*, not 4 silicon fabrics**: `swarm_v2_nand_fabric.{k3,tercode,turbovec,text.jsonl}` are four
>   formats on one basename, **78,391,807 B** in total (k3 76,761,697 / text.jsonl 1,538,828 / turbovec 45,829 /
>   tercode 45,453). The kernel has already stopped asserting the shard count: `/memory` returns
>   `"shards": null` with `"no shard count is measured here: the fabric is one .k3 file plus a tercode store and
>   the IPC ring"`, and files **18** under `design_figures` with its source (`docs/NAND_K3_FABRIC_SPEC.md`). The
>   same endpoint reports `ddr_antibody_count: 27` beside `ddr_antibody_count_previously_asserted: 1801`.
>
> One thing the live read surfaced rather than settled: `/k3/status` reports `monad_anchor.byte_identical:
> false`, `differing_bytes: [2]`, with the engine's own explanation — the anchor is stored as its **trit
> projection**, and `0x4E` carries the 2-bit group `0b10` that the engine's map cannot produce, so that byte
> reads back `0x4D`. That is a stated design consequence, not a silent mismatch.

## Checkpoint of record

WSL `trm-rust` HEAD **`89d2d75`** (`freeze: capture uncommitted v18.233 work before Windows-side
reconciliation`), branch `master` — the seven at-risk files are now **tracked**, which is what step 0
recorded; the earlier bare `29cb510` with "1 tracked + 9 untracked" is superseded by that freeze.
WSL disk `/` 808 GB free. No WSL service is running, so nothing there is load-bearing yet — which is
exactly why step 0 is a cheap safe copy and no step after it is blocked on anything.

## Since the plan was written (blocks 164–168)

What closed after the plan's own steps, each sealed in `swarm_v2_artifacts/monad_ledger.jsonl` (now **171
blocks, 0 broken links**; the only UNVERIFIED remain the pre-existing 97–101):

- **164 — tier 2 got a second model root.** `discover_tier2_generator()` globbed only `D:/trm-models`, a
  removable disk that held the *running* tier's model. It now globs two roots **primary first** (the external
  set stays the model of record; ties resolve in glob order) with the fallback derived from the module's own
  location and seeded with the **verified** 8B g64 (2,310,125,920 B, sha256 `e17b298d84ee7879…`), not the 4B
  the manifest lists `not_yet_verified`. Proven against three cases in one harness: real FS, primary hidden,
  both hidden.
- **166 — the four voltage instruments are preserved.** `prism-ml-llama.cpp/examples/{voltage_node,
  voltage_node_offload,capture_hidden,capture_layers}.cpp` were **untracked with no other copy**, and the
  2026-09-23 freeze covered `trm-rust` only — so they had no preservation at all. Harvested to
  `swarm_v2/tools/voltage_instruments/` with a per-file sha256 receipt, their 10 reference captures, and a
  second archive (`backups/wsl_freeze_20260926_instruments/prism-ml-untracked.tar.gz`). Source-only: no
  `g++`/`cl` here and the runtime ships `llama.dll` without headers.
- **167 — the hardware/bare-metal corpora are in the knowledge index.** `rust_kb` (5 crate archives → 92
  `.rs`) and `hw_kb` (18 hardware references) copied to `swarm_v2/knowledge/` and indexed. A query for
  "x86_64 GDT descriptor and segmentation" now returns `gdt.rs` 0.4761 where the pre-harvest index had **no
  knowledge doc in its top-3 at all** for any of three queries.
- **168 — Tox21 is in the store**, closing the last domain the platform held *no* record of: 577,987 rows,
  one per **measured** cell (8,734 compounds × 75 assays), blanks skipped rather than stored as 0.0, verified
  by an independent `awk` re-count and a swap that moved provenance by exactly one row.

Numbers this plan should not be re-quoted from now on:

- The knowledge index is **2,180 docs live** (53 MB) and **336 docs** in the harvested candidate
  (`kernel_knowledge_index_harvest.json`); the "180 text docs" figure above is stale in both directions.
- Its build rate measures **2.1 docs/s** (336 docs / 163.2 s). Any "163 docs/s" figure comes from an old
  artifact's self-reported meta, not from a measurement — a declared rate is not a measured one.
- The store measures **1,249,710,080 B, sha256 `521ff968a5cb2d8c…`** (block 169) with `tox21` among its
  tables and `span` declaring 12 trees. Three kept generations sit on disk, each the predecessor of the
  next: `pre_span` `212b0b1b…` → `pre_tox21` `c347a050…` → `pre_spandecl` `6d2ab79d…` → live `521ff968…`.
- **Remaining harvest: 3 groups** — `models` (public GGUFs whose *load evidence* is already harvested),
  `prism-ml-llama.cpp` (the fork itself; its instruments are in), and `trm-rust` (**step 3's six crates**).

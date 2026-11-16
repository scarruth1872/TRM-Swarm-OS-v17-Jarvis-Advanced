# WSL reconciliation, step 1 — "reconciled" as a measured claim (2026-09-26)

Plan: `docs/WSL_RECONCILIATION_PLAN_20260923.md`, step 1. Artifact: `swarm_v2_artifacts/wsl_reconciliation_manifest.json`
(20,180 B, 55 groups). Verdicts live in `swarm_v2_artifacts/wsl_verdicts_part{1,2}.json`; the builder is
`scratch/build_wsl_reconciliation.py` and it **checks every verdict** before writing.

## Step 0 was already done — verified, not assumed

The freeze the plan asks for exists and is sound: commit `89d2d75855a18056a000bcb6b704d81fd40cc819`
("freeze: capture uncommitted v18.233 work before Windows-side reconciliation", 2026-09-23 18:02:03),
archive `backups/wsl_freeze_20260923_180202/trm-rust_source.tar.gz` — **446,867,446 B, sha256
`a2b3f9a737affc90aca35de63eb73107ac239481be5ba348df9682ccf6606ea5`**, 3,134 entries, and it really carries
the seven files the plan named (`fractal_mamba3.rs`, `MOTION_A_RUST_COUNCIL_ABSORPTION.md`, the three
`*_results.json`, `voltage_node`, `voltage_offload`). Each of them is now **tracked** in the repo, not
untracked: the seven-file risk is closed, and `git status` on `~/trm-rust` is clean. The same directory holds
`FREEZE_MANIFEST.json`, `home_inventory.tsv` (1,260 rows), `knowledge_sha256.txt` (509 hashed files) and
`models_manifest.tsv`; a 3.7 GB `wsl_crates/trm-rust` copy sits out of WSL.

## The method: the store's own provenance table is the harvest ledger

Three freeze inventories measure what is on the WSL side (`home_inventory.tsv`, `knowledge_sha256.txt`,
`models_manifest.tsv`). What the platform **holds** is not taken from prose: it is read from the live
store's `provenance` table — 435 sources, 1,179,412,577 B, 5,912,980 records — so "harvested" means *the
store names that source*, and nothing else does. The builder enforces four rules and refuses to write on any
failure: every inventoried group must be classified; `harvested` must appear in provenance; `harvest` must
**not**; and every `superseded` verdict must point at a counterpart path that exists.

## The verdicts (55 / 55 classified, every check passed)

| verdict | n | meaning |
|---|---|---|
| harvested | 6 | the content is inside the platform, with the table that proves it |
| harvested-prose | 3 | the content is in the knowledge index, and the index proves it |
| harvest | 3 | valuable and absent — the real remaining work |
| superseded | 13 | v19 carries the counterpart (path asserted to exist) |
| archive-only | 24 | historical; the freeze bundle preserves it, nothing reads it |
| ephemeral | 5 | regenerable, 0–12 KB; never deleted, only moved with a manifest |
| never-harvest | 1 | `.ssh` — credentials, excluded from every archive by policy |

**Harvested (5).** `clinical` → `clinical` table **201,572 rows** from 400 `.psv`; `datasets` → `binding`
**19,443 rows** = the LP-PDBBind affinity table; `docking` → 20 receptor rows in `structure`;
`drugs` → `molecule` **2,897,819** + `molecule_features` **2,897,818** + 2,897,805 derived structures from
1,127,883,975 B across 2 files; `relational` → `ethics` **95,847 rows** (`ethics_commonsense.csv`) + `prose`
1,633 chunked rows. Live table census: binding 19,443 · clinical 201,572 · ethics 95,847 · molecule
2,897,819 · molecule_features 2,897,818 · prose 1,633 · **tox21 577,987** · provenance 436 · span 12 · structure 20.

**Harvest (2) — what is genuinely left** (it was 7 when first sealed, then 3 after the corpora and Tox21, and 2
after the crate triage -- `trm-rust` moved to `superseded` when block 172 measured all six of its candidate
crates already wired and computing, 6/6 answering 200 on `:8022`): `models` (public GGUFs; the load evidence
is already harvested) and `prism-ml-llama.cpp` (the fork itself; its instruments are in). `rust_kb`, `hw_kb`,
`tox21full` and `tox21full.csv.gz` were harvested earlier — prose into the knowledge index, the matrix into
the store.
As first sealed, this section read: `rust_kb` and `hw_kb` (prose corpora that appear in the 509-file
knowledge manifest and in **none** of the store, the fabric text store or provenance — the fabric's text path
is what fits them, which is exactly the asymmetry step 2 identified); `tox21full` + `tox21full.csv.gz` (there
is **no tox table** in the store — toxicology is the one domain the platform holds no record of);
`models` (`qwen-moe-2.4b-q4_k_m` 964,652,928 B, mainline-loadable but unverified here; `bonsai-27b-q1_0`
3,803,452,480 B, a *different quant* from the `_g64` set; `instella Q2_K` 6,514,704,512 B, fork-only arch
measured to fail load on mainline b10964); `prism-ml-llama.cpp` (four **untracked** instruments with no other
copy — `examples/{voltage_node,voltage_node_offload,capture_hidden,capture_layers}.cpp`, the voltage
doctrine's own measuring tools); and `trm-rust` (the workspace is superseded, but six crates may hold unique
logic — swarm-brain 338 L, swarm-intel 263, swarm-docking 226, swarm-drug 176, swarm-fit 136, swarm-proof 134
— step 3 ports before retiring).

**Superseded (13).** The Agent-Q line (`swarm_v2_artifacts/agent_q_advanced_modules_report.json`), the
Jarvis line (`absorbed_jarvis_metatron_matrix.json`, `jarvis_complex_task_report.md`), the aura line,
`mamba3step_gpu_port.json`, and the second `llama.cpp` source checkout — superseded by the prebuilt runtime
this host already ships (`AppData/Local/hermes/runtimes/llamacpp/b10964` and `b10679`).

**Two things this manifest deliberately does not do.** It does not call `voicebox`, `aipro2` or
`start-anythingllm.sh` superseded — no v19 artifact names them, so they are `archive-only` with that reason
stated rather than a verdict that sounds decisive and is unverified. And it touches no secret: `.ssh` is
classified `never-harvest`, and no secret value was read while building any of this.

## What "reconciled" now means, precisely

Every WSL group the freeze inventories describe has a verdict and an evidence string, and the verdicts a
machine can check were checked: 5 harvested (each named in provenance), 7 harvest (each absent from it), 13
superseded (each with an existing counterpart). "The WSL side is reconciled" is now falsifiable — re-run
`scratch/build_wsl_reconciliation.py` and it either reproduces this or fails loudly.

## Closed since: tox21, the last empty domain (block 168)

`tox21full` was the one group where the platform held *no* record of any kind — no table, no provenance row,
no indexed document. It is now a table: **577,987 rows**, one per cell the source actually *measured*
(8,734 compounds × 75 assays; 551,824 inactive / 26,163 active). Blanks are skipped rather than written as
0.0, because an empty cell is not a negative result and storing it as one would have fabricated ~76k labels.
The count is not taken on trust: an independent `awk` pass over the gzip agrees, and the swap was verified
against the pre-swap store kept at `backups/structured_index_pre_tox21_20260926/` — every other table
identical, provenance up by exactly one row. Verdicts are now **6 harvested / 3 harvested-prose / 3 harvest**.

Two corrections the work produced, both recorded rather than smoothed over:

- The generator's README claims "**64 different assays**"; the file it produces has **75** columns. The file
  is the measurement and it wins — a declared number is not a measured one.
- **No codebook for the 75 assay codes exists anywhere in the preserved corpus** (`elg271`, `p53823`,
  `mmp146`, …). The store holds the identifiers verbatim; inventing meanings for them would be exactly the
  fabrication the doctrine forbids.

Also fixed while rebuilding the knowledge index: `build_kernel_knowledge_index.py` re-walked the **entire
repo** on every build (`os.walk(ROOT)`), which on a WSL `/mnt/c` mount holding multi-GB artifacts never
returned — and its parser fallback matched keys by `stem[:32]` prefix *anywhere in the tree*, which can bind
a key to an unrelated file. The fallback is now bounded to the harvested corpus: builds finish in 163 s
instead of hanging, and 6 keys that were being resolved by bare filename now correctly report as missing.

## And the store now declares it (block 169)

Block 168 put the matrix *in* the store; the store's own `span` table still said `tox21full` was out of span
with "no data files" and never named the CSV that had just been ingested — a declaration contradicting its
own contents, which is the declared-vs-measured defect running in reverse. `record_spans()` (the generator,
never a hand-edit) now declares **12 trees, 7 `INGESTED` / 5 `OUT-OF-SPAN`**, with `tox21full.csv.gz`
INGESTED at its measured 152,757 B and `tox21full` still OUT-OF-SPAN because it is the *generator*, not data.

The store measures **1,249,710,080 B, sha256 `521ff968a5cb2d8c…`**, and its own history is checkable rather
than described: `pre_span 212b0b1b…` → `pre_tox21 c347a050…` → `pre_spandecl 6d2ab79d…` → live, each kept
store exactly the one its successor replaced.

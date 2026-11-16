# FULL PLAN — every artifact, how the platform proves them, and what setup remains (2026-09-24)

Built on what the project already documents, not on assumptions: `docs/WSL_RECONCILIATION_PLAN_20260923.md`
(Steps 0–6), `docs/WSL_INGESTION_DESIGN_20260923.md` (the ingestion decision + provenance rule),
`docs/STRUCTURED_INDEX_FULL_20260923.md` (the provenance field, already implemented),
`docs/WSL_STEP1_CLASSIFICATION_20260923.md`, `docs/ASSET_INVENTORY_AND_FIRST_TASK_20260924.md`,
`docs/TASK_SUITE_TRUTH_20260924.md`. Where this plan contradicts those, it says so.

## 1. The artifact inventory (measured, not assumed)

**Windows checkout** `Downloads/TRM-Swarm-OS-v19-…` (v19.001 when this inventory was taken; **v19.002 live as of 2026-09-28**): `trm_uefi_kernel` (183 modules,
16.4 MB EFI, 56 NEEDS-LOGIC), `swarm_v2_rust_kernel` (the DLL), `swarm_v2_artifacts/native_voltage_strips`
(549 binaries, 130-marker MZ gate PASS), `swarm_v2_artifacts/structured_index.db` (1,099,448,320 B live;
rebuilt to 754,692,096 B on 2026-09-26 with the same rows — see `docs/STRUCTURED_INDEX_SPAN_20260926.md`),
`swarm_v2/skills` (35 modules), `scratch/` (**214 scripts** — the de-facto executor layer),
`docs/` (30 files, 24 of them in the last 48 h), `papers/`, the fabric (76,761,697 B = 153,523,394 trits),
ledger (117 blocks then; **203 blocks as of 2026-09-28**), `backups/` (74 files in the window).

**WSL** `/home/shawn` (v18.233, **no toolchain**: `cargo`/`rustc` absent, `~/.cargo` absent):
`trm-rust/` — **36 crates, ~7,000 LOC, one squashed commit (HEAD `29cb510`), dirty, 9 untracked files**
including `swarm-proof`, `swarm-docking` (226), `swarm-drug` (176), `swarm-health` (109),
`swarm-services` (132), `swarm-mesh` (80), `swarm-ssm` (429), `swarm-voltage` (495), `swarm-kernel` (943),
`swarm-nand` (265), `swarm-shard` (304), `swarm-council` (370), plus three BINs (`swarm-boot`,
`swarm-runtime`, `swarm-server`); `datasets/LP-PDBBind` (128 MB: `LP_PDBBind.csv` 19,443 rows with kd/ki,
BDB2020+ 116, mpro 274 files, EGFR set, and trained predictors `deepdta_retrain.pt` 7.6 MB + IGN 1.4 MB);
`drugs/` (ChEMBL 37 SDF 889 MB + 2,897,818 feature rows); `clinical/` (400 `.psv` = **PhysioNet 2012** —
40 measurements × 400 patients, not PharmGKB as its name suggests); `docking/` (29 PDB: 1E8X, 1M17, 1XKK,
…) and `docking_work/` (281 files incl. `batch.py`, `mpro_sweep.py`); `relational/` (234 MB ethics/wisdom
corpora); `hw_kb/` (18 files), `rust_kb/`, `tox21full/`; `models/` 11 GB; `Agent-Q-master` (3.1 GB —
**the AURA engine, 1,200+ metaconcepts, port 8001** — this is what `/aura/*` belongs to), `aipro2`
(836 MB), `JarvisShawn` (35 MB), `anythingllm`, `hermes-archive`; **engines: `/usr/bin/vina` (AutoDock
Vina 1.2.7), `/usr/bin/vina_split`, `/usr/bin/obabel` — already installed**.

**Other stores**: `D:/trm-models` **41 GB** (ternary-bonsai 23 GB across 1.7B/4B/8B/27B + mmproj,
instella 10.5 GB, bonsai2 7.8 GB); C: HF cache (`mamba2-370m` 1.47 GB, `faster-whisper-tiny`);
`C:/Users/shawn/models/gemma-3-4b-it` (3.26 GB); **antigravity scratch: 36 brain dirs, the newest
(`e019a966…`, 09-19) holding 13,425 files / 1.5 GB** — 6,147 `output.txt`, 2,178 `.json`, 1,722 `.jsonl`,
948 task logs, 603 `.py`, 543 `.o`, 195 `.md`. The `task_*` artifacts of the suite live there.

## 2. The modules are not junk — the documented reason each exists

| refused family / module | the reason it exists | how it should be set up |
|---|---|---|
| `/aura/*` (5 sites) | **Agent-Q-master's AURA engine** (1,200+ metaconcepts, port 8001); WSL classifies Agent-Q as "other workstream, not council material" | run Agent-Q on 8001 and let it own those routes; do not port them into the kernel |
| docking / affinity tasks | Vina + PDBBind + mpro crystals **in WSL**, and the platform already ran a real redocking study (n=11, 7<2.0 Å, median 1.41 Å) | `swarm-docking` crate + the WSL vina path; meeko only if preparation changes |
| `/api/mobile-sync/*` (2) | companion-app transport | needs the app/peer; binary evidence of the peer, else stays refused |
| `/thz/resonance/sync`, `/swarm/spatial/*` | THz/spatial engines in `swarm-thsm`/`swarm-cosmos` | build those crates and expose them; today they cannot build |
| `/system/modules/*` engines (7) | each module owns an engine (chain-of-verification, GAN, MCTS, monad absorb, neural wall, small diffusion, zero-human test) | port → wire → receipt, one per batch (the campaign's existing discipline) |
| infra watchdog / secrets / evolution / research / artifacts / DDR / discovery / humanistic (22) | Python engines that own live state; the refusal names each owner | port to Rust **or** keep the refusal and run the offline engine — never re-stub. **UPDATE 2026-09-28:** the infra watchdog is no longer in this class — its FSM now exists in Rust (`trm_uefi_kernel/src/self_healing_infra.rs`, linked by `#[path]`; the same implementation `strip_self_healing_infra.dll::strip_eval_service_health` exports) and `/infra/health`, `/infra/history`, `/infrastructure/status` are served, so "no FSM exists for Rust to serve" is no longer true |
| `/verification/*`, `/testing/*`, `/models` | the platform's own proof surfaces (9 theorems, 11 test runs, crc32 **over bytes [0..28)** declared "intact") | extend, don't replace: full-payload integrity + provenance verdicts |

## 3. Letting the platform prove artifacts

What already exists to build on: the structured-index parser **records `provenance` per artifact**
("TRUNCATED SOURCE: … re-fetch to complete it" — the artifact carries its own caveat), the ledger chains
`prev_hash` blocks, `/verification/stats` reports 9/9 theorems, and `swarm-proof` is a real proof-tree
checker (axiom / modus ponens / ∨-intro / ∧-elim). What is missing is a verdict on **work artifacts**.

New mechanism (`scratch/prove_artifacts.py`, run now): provenance-by-producer — one pass indexes every
quoted identifier in the repo, traces each artifact to the script that writes it, and reads that
producer to decide:

| verdict | meaning | receipt |
|---|---|---|
| COMPUTED | producer reaches a real engine/tool (rdkit, torch, vina, obabel, scipy, sklearn, pandas, Bio, subprocess) and the record fields are not drawn | the engine line |
| GENERATED | fields come from `random.*` / `seeds` / `uuid4` with no engine call | the RNG line |
| STATIC | a producer exists but shows neither engine nor RNG → **not proven by this rule** | the producer path |
| UNTRACED | no producer on disk → nothing vouches for it | the search |

**Result on 150 suite artifacts: COMPUTED 0 · GENERATED 80 · STATIC 49 · UNTRACED 21.** The GENERATED
receipts are literal `random.seed(N)` lines in the band runners; the UNTRACED set includes the imaging
tasks (no producer anywhere). This is the first artefact-level proof the platform has, and it says
plainly: **none of the 150 artifacts is shown to be a computation's output.** Next steps, in order:
1. Serve it: `GET /verification/artifacts` (same shape as `/testing/stats`) so the dashboard shows the
   verdict per artifact, with the receipt link.
2. Per-family contracts for the STATIC set (a convergence run's loss must fall; a docking pose's RMSD
   must be measured against the crystal; a feature table must reproduce from its source) — a contract
   per family, not a value to freeze.
3. Gate the claim: an artifact may only be reported as a completion once its family contract passes.

## 4. Acquisition list — present vs still absent (measured today)

**Present and usable now:** Vina 1.2.7 + vina_split + obabel (WSL) · Windows ML stack (torch 2.6.0,
rdkit 2026.03.6, scipy 1.17, pandas 2.2, sklearn, onnxruntime, biopython) · LP-PDBBind + trained affinity
weights · ChEMBL 37 + 2,897,805-row RDKit feature table · structured index (408.7 MB, 5 categories:
ethics 95,847 / molecule 2,238,290 / clinical 201,572 / prose 1,633 / structure 9) · 41 GB model store ·
mamba2-370m + whisper · gemma-3-4b VLM · PhysioNet 2012 clinical records · 29 PDB structures ·
`relational/` ethics corpora (234 MB, **still un-ingested**) · the 36 Rust crates.

**Still absent (from `ASSET_INVENTORY` + verified today):** MD engines (OpenMM/GROMACS), PySCF, genomic
aligners/variant callers (GATK/samtools/bwa), ClinVar/gnomAD/PharmGKB copies, GEO/ENCODE/TCGA corpora,
any real microscopy/pathology imagery, TPM/HSM, patient/cohort data, and the LpxC structure and the
TRM-ABX-220 identity.

**Ruler correction carried into this plan:** the "147 acquirable" figure is stale — it was derived before
anyone found Vina, PDBBind and the ML stack in WSL. A keyword re-score returned 38 and was itself
unreliable (it read "CADD weights" as satisfied because *torch* exists). Per the inventory doc's own
conclusion, the only reliable signal is **per-task execution**, which is why §3's prover is the first
step and not the last.

## 5. The plan, in dependency order

| # | step | acceptance (what makes it done) |
|---|---|---|
| **P0** | **Freeze the WSL side** (the reconciliation plan's step 0) | **DONE, verified 2026-09-24:** `backups/wsl_freeze_20260923_180202/trm-rust_source.tar.gz` (447 MB) holds 3,134 entries and carries all nine at-risk items — `fractal_mamba3.rs`, `MOTION_A_RUST_COUNCIL_ABSORPTION.md`, `hard_problems_results.json`, `known_knowledge_results.json`, `swarm_impossible_results.json`, `voltage_node`, `voltage_offload` — plus 34 crate manifests |
| **P1** | **Artifact proof, served** (§3) | **DONE 2026-09-24:** `GET /verification/artifacts` serves the census *and re-checks it* — tallies counted from the entries, each receipt re-read from its producer at request time. Live: 150 examined, `COMPUTED 0 / GENERATED 80 / STATIC 49 / UNTRACED 21`, receipts **80/80 reproduce**, classes sum == examined. No census → `501 census_absent` with the command; non-GET → `405`. Deployed `8dc6113c037bdbfc`. **Second half DONE 2026-09-24:** `scratch/verify_artifact_contracts.py` + a `contracts` block on the same route — over the 70 STATIC+UNTRACED artifacts: **19 PASS** (range consistency only, weak), **1 FAIL**, **50 NO_CONTRACT** (the library needs the ML/docking record shapes). The FAIL is real and decisive: `task_113_gnomad_5000_gene_constraints` — `mis_z` vs its own recomputed `(obs-exp)/sqrt(exp)` gives **r = 0.0162 over 5000 rows** (drawn independently, not derived), while its coarse pLI/LOEUF coupling passes 5000/5000. The kernel re-runs the range contract itself: **rechecked 19, agreeing 19, disagreed 0**. Deployed `0128da2cf9ae4e04` |
| **P2** | **Wire the crates that already exist** | **DONE 2026-09-24 (first crate serving; 34 built, unwired):** the frozen tree extracted to `wsl_crates/trm-rust` — a 35-crate cargo workspace that **did not build here at all**: every failure was in one crate, `swarm-nand`, from 5 Unix-only positional-I/O call sites (`read_exact_at`/`write_all_at`) plus a `libc flock`. Fixed as a class with a cfg'd `pio` shim in that same file (unix syscalls untouched; windows rebuilt from `seek_read`/`seek_write` + std `try_lock`; no new dependency). Result: **31 of 35 crates build** — 31 rlibs, 14 binaries, all 8 target crates among them. The only failure is `swarm-boot` (`LNK1561`, UEFI-only, needs `--target x86_64-unknown-uefi` — as its own manifest says; not patched around). **`swarm-proof` is wired for real:** a path dependency serving `POST /verification/proof`, where the crate's own `ProofNode::verify` decides and the kernel adds no rule. Image grew 2,213,888 → 2,583,040 B (+369,152) so the code is linked, not declared. Ruler `scratch/verify_p2_wired_proof.py` **9/9**, the load-bearing probe being "same tree, conclusion rewritten" flipping `sound` true→false. Deployed `abc900982f0e5724`. A probe found a real message bug (the MP rejection named the *antecedent* as the conclusion); fixed, with the crate's own 3 tests still passing. **Remaining:** 34 crates built but unexercised, and the 43 `ENGINE_PORT` refusals are still refused — this step made the port mechanically possible, it did not port the engines. **UPDATE 2026-09-28:** the `ENGINE_PORT` set is no longer exactly 43 — the infrastructure watchdog FSM has since been implemented in Rust (`trm_uefi_kernel/src/self_healing_infra.rs`) and `/infra/health`, `/infra/history`, `/infrastructure/status` are served; the remainder of the set is unchanged |
| **P3** | **Run the docking class for real** | the n=11 redocking result is reproduced from a scripted path with the `swarm-docking` crate, with the 4 failing complexes recorded |
| **P4** | **Ingest `relational/` + the structured stores** (the ingestion design's decision) | one text document per record, each carrying `source file` + archive sha256, through the existing encoder; a stated, bounded sample with the fraction recorded; chemistry queries go to the structured index, not the fabric |
| **P5** | **Acquire only what a task demonstrably needs** | per-task: run it, and if it fails on a named missing artifact, acquire that artifact — the 7 docking tasks first (engine present), then gnomAD/ClinVar/PharmGKB, then aligners |
| **P6** | **The 55 physical + ~39 operator tasks** | LpxC structure and TRM-ABX-220 identity from the operator; the rest stay blocked and are reported as blocked, with the reason |
| **P7** | **Retire what is superseded** (reconciliation steps 3–4) | **Step 1 DONE 2026-09-25 (block 150):** all 36 crates classified by REACHABILITY, not by name — **0 SUPERSEDED** (10 LIVE path-deps, 22 HARVEST, 4 BIN_HARVEST), which **refutes** `WSL_STEP1_CLASSIFICATION_20260923.md`'s "mostly superseded — v19 has Rust equivalents": those crates *are* the equivalents. First harvest `swarm-drug` wired (`/native/drug/*`, ruler 11/0). **Step 2 DONE 2026-09-25 (block 151) — NOTHING RETIRED, on evidence:** the duplicate census found 13,888 mirror files byte-identical to an outside twin, but `swarm_v2_integrated/` is a *live destination* (the artifact pipeline's `integrated_path`, the dashboard's monitored directory, `module_registry.json`, an import in `nexus_usage_example.py`), so byte-identity is not redundancy. **Harvest continues:** `swarm-docking` (block 152) and `swarm-intel` (block 153) wired, 9 modules / 11 operations live — see `docs/NATIVE_HARVEST_20260925.md`. Remaining harvest candidates: `swarm-brain`, `swarm-fit`. The v18 crates are retired **only after** their unique logic is ported or proven redundant **Harvest COMPLETE 2026-09-26** (blocks 152-155: drug, docking, intel, fit and brain wired as path dependencies, each with a ruler and a sealed block; /native/modules = 13). **LCARS tab DONE 2026-09-26**: `/native` and `/sram` are forwarded and tab 31 renders the 13-module census plus the 26-operation table (block 156). **host_tranche classified 2026-09-26** (block 158): skeleton tranche, never compiled, superseded as source of record and kept; the port's remaining gap is measured (123/232 names, 6 residual candidate groups) -- `docs/P7_HOST_TRANCHE_CENSUS_20260926.md`. **Residual groups 1&2 CLOSED 2026-09-26 (block 159):** the load/fuzz modules were ported to the *source's* own rules and routed — `/native/modules` = **15**, `/native/routes` = **37**, parity ruler **65/0** (`swarm_v2_artifacts/load_fuzz_parity_ruler.json`); the other four residual groups were naming/design differences, not missing behaviour. **WSL step 4 CLOSED 2026-09-26 (block 160):** the WSL side's role was decided by measurement (no `cargo` there; `g++`, `nm`, `python3` present; 808 GB free vs `C:`'s 119 GB) — **data/archive + the Linux build host for `.so` artifacts, not a Rust host** — and the artifact the PHOENIX report named now exists: `swarm_v2/core/libswarm_kernel_ops.so` (21,632 B, `7aff4be98f8889c1…`, 8 exports, ruled **29/0** against the Windows DLL: mamba max deviation 5.2e-08, gate ids/active/weights exact), exercised through the consumer that requires exactly this file (`swarm_v2/core/native_ops_bridge.py`). It required fixing a real source defect: `swarm_kernel_ops.h` used `size_t` without `<cstddef>`, which GCC 15 rejects — the DLL predates that toolchain, so the `.so` build script could never have run. **P7 IS COMPLETE.** |

## 6. What would make this plan wrong

- If a producer I called GENERATED turns out to compute with pure math (my classifier only recognises
  engine calls, so a numpy-free computation reads as STATIC) → the per-family contracts in P1 correct it;
  the verdict tables are receipts, not verdicts of record.
- If the 36 crates are an abandoned v18 attempt rather than live logic → P2 finds it within one crate
  and the rest are archived instead of built.
- If the fabric's weak short-query discrimination (33–47% top-1, from the ingestion doc) means retrieval
  quality is the real bottleneck → P4's structured-index fork becomes the priority over any fabric ingest.
- If an artifact store I have not walked (older brain dirs, `code_tracker`, `annotations`) holds computed
  work → the prover's scope is a parameter; the census is over 150 of 13,425 files, and that fraction is
  stated rather than implied.


# P7 — retiring what is superseded: the classification, with evidence

**Block 150 companion.** P7 is defined as *"every WSL artifact classified harvest/superseded/duplicate with its
evidence; the v18 crates retired only after their unique logic is ported or proven redundant"*. This is that
classification, and it is a **negative result**: nothing in the vendored workspace may be retired yet.

## Step 2 (block 151): the duplicate class — nothing retired, on evidence

The mirror census found `swarm_v2_integrated/` = 15,410 files with **13,888 byte-identical to a copy outside**
(820 MB; 765 MB of that is the single `_peerpath_inventory.json`), 1,522 files with no twin, and 62 live files
naming the tree. The retire rule (identical twin **and** the path named by no live file) selected 13,888 files
— and then the consumers were read: the artifact pipeline's `integrated_path` in `artifacts_status.json`, the
dashboard's Monitored Directory (`ModuleMatrix.jsx`), `module_registry.json`, and an import in
`nexus_usage_example.py` all point at that tree. **Byte-identity is not redundancy when the directory is a live
destination.** Nothing was moved; `scratch/retire_mirror_subset_v2.py` exists but was not applied.

## Step 2 continued (blocks 152–153): the harvest, not the retirement

`swarm-docking` and `swarm-intel` are now wired (`/native/docking/*`, `/native/intel/*`), taking the native
surface to **9 modules / 11 operations**; `docs/NATIVE_HARVEST_20260925.md` records the rulers and the findings
— including that `council_vote` can only ever return ACT.

## The crates: 0 superseded

`scratch/census_wsl_artifacts.py` → `swarm_v2_artifacts/wsl_artifact_census.json`, over all 36 crates of
`wsl_crates/trm-rust`. Verdicts are mechanical: **LIVE** = reachable from `host_council_server` through the path
dependency graph (transitively); **HARVEST** = defines `pub fn` symbols that exist nowhere in the live tree;
**SUPERSEDED** = every symbol it defines already exists live.

| verdict | crates | evidence |
|---|---|---|
| LIVE | **10** | `swarm-proof`, `swarm-k3`, `swarm-instella`, `swarm-turbovec`, `swarm-ssm`, `swarm-council`, `swarm-nand`, `swarm-gguf`, `swarm-git`, `swarm-paths` are path dependencies of the kernel server |
| HARVEST | **22** | 0–11 of their `pub fn` names exist live; e.g. `swarm-voltage` 0/11, `swarm-vitals` 0/5, `swarm-shard` 3/11 |
| BIN_HARVEST | **4** | binaries with no exported surface; compared by every fn/type/const name (`swarm-kernel` 7/46, `swarm-server` 2/23) |
| SUPERSEDED | **0** | — |

**This refutes `docs/WSL_STEP1_CLASSIFICATION_20260923.md`**, which calls the crates *"mostly superseded — v19 has
Rust equivalents: `swarm-council` ↔ `council_multi_intelligence_deliberation_engine.rs`, `swarm-k3`/`swarm-nand`/
`swarm-turbovec` ↔ v19's K3/fabric/TurboVec"*. Measured: those four crates **are** live path dependencies of the
kernel — they are how v19 has those equivalents. That verdict was a name-level assertion where reachability is the
measurement; retiring on it would have deleted the equivalents themselves.

Per the plan's own condition, therefore: **no crate retires from this repo**. Retiring the WSL-side *original*
tree stays open, and it needs the archive-manifest comparison against
`backups/wsl_freeze_20260923_180202/trm-rust_source.tar.gz` (P0's 447 MB freeze) — a separate, provable step:
the WSL side has no cargo toolchain at all, so that copy cannot build.

## Step 3's first harvest: `swarm-drug`, wired

Of the six crates the plan flags as possibly holding unique logic (`swarm-brain`, `swarm-intel`, `swarm-docking`,
`swarm-drug`, `swarm-fit`, `swarm-proof`), `swarm-proof` was already wired in P2. `swarm-drug` is the most
self-contained: a streaming gzipped-SDF parser and a pure K3 triage.

It is wired the same way as `swarm-proof` — **as a path dependency, so the crate's own function decides** and the
kernel adds no rule, no clamp and no default (`trm_uefi_kernel/host_council_server/src/drug_handlers.rs`):

- `POST /native/drug/triage` — `swarm_drug::triage(&Molecule, tau)` per molecule; the label and detail strings are
  the crate's own.
- `POST /native/drug/sdf` — `swarm_drug::parse_sdf(path, limit)`; the path is the caller's, and a path that is not
  there is refused with a reason rather than answered with invented molecules.
- `GET /native/modules` now reports **6** modules, the crate's source measured at request time (5,986 B).

**Receipt: 11 checks, 0 failed** (`scratch/verify_drug_wiring.py` → `swarm_v2_artifacts/drug_wiring_ruler.json`).
The load-bearing parts: a 144-case boundary grid (150/500, 15/40, 20/80 and the ±2.0 sums) where every case equals
a **from-source Python oracle** in both verdict and the crate's own detail string, with the grid's verdict coverage
reported (36 drug-like / 48 blocked / 60 analyze) so a one-verdict grid could not pass; the crate's 3 tests; and
that `/native/modules`'s measured size matches the file on disk.

**Recorded, not smoothed:** the crate's `tau` parameter is **never read**. The compiler says so
(`warning: unused variable: tau`) and the ruler proves it — four taus (0.0 / 0.35 / 0.99 / 1.0) answer identically
over the same cases. The route reports the tau it was handed rather than pretending it mattered.

## Open, and named as open

- **The supervisor daemon is down.** It was already gone before the 14:10 reload (the reload stops by the exact
  process name, so it did not kill the supervisor); starting it again — with `py -3` and with the
  `Python313` interpreter that `start_platform.bat` names — exits before its first log line, and
  `logs/supervisor_run.log`'s newest entry is from 2026-09-20. The platform is up and serving because the image
  was launched directly. This needs its own diagnosis.
- **The duplicate census** (every byte-identical cluster whose twin is live) walks 141,142 files and had not
  finished when this block was sealed; its artifact is `swarm_v2_artifacts/p7_duplicate_census.json`.
- The remaining five harvest crates stay exactly where they are until each is wired or reviewed.

# License Audit — TRM-Swarm-OS (2026-10-19)

Audit of every LICENSE/COPYING file present in the tree. Every row is read from disk, not assumed.

## Root + vendored repos

| path | license | copyright | verdict |
|---|---|---|---|
| `LICENSE` (repo root) | **MIT** | © 2025 **Samsung Electronics Co., Ltd.** | inherited from SwarmOS origin — attribution required |
| `SwarmOS_v6_Snapshot_extracted/LICENSE` | **MIT** | © 2025 Samsung Electronics | same Samsung origin |
| `Jarvis-Advanced-main/` | **none found** | — | **UNLICENSED — gap** |
| `gemma_swarm_adapter/` | **none found** | — | **UNLICENSED — gap** |

## Data + research licenses

| path | license |
|---|---|
| `swarm_v2/knowledge/tox21full/LICENSE` | Apache 2.0 (tox21 assay data) |
| `research/alphagenome_research/LICENSE` | Apache 2.0 |
| `.licenses/uniprot_database_LICENSE.txt` | UniProt terms |
| `.licenses/pubchem_database_LICENSE.txt` | PubChem terms (U.S. gov) |
| `.licenses/clinvar_database_LICENSE.txt` | ClinVar (NCBI) |
| `.licenses/clinical_trials_database_LICENSE.txt` | ClinicalTrials.gov (U.S. gov) |
| `.licenses/alphafold_database_fetch_and_analyze_LICENSE.txt` | AlphaFold (CC BY 4.0) |

## Vendored Rust crates (`swarm_v2/knowledge/rust_kb/`)

`x86_64-0.15.5`, `volatile-0.6.1`, `spin-0.12.2`, `bootloader-0.11.17` — **MIT** and/or **Apache-2.0** (permissive).

## What this means

1. **The base is Samsung's SwarmOS, MIT.** The root LICENSE (© Samsung Electronics 2025) proves the
   platform descends from Samsung's open SwarmOS. MIT permits the fork + commercial reuse + the
   proprietary additions, **with the Samsung copyright notice retained** — a compliance obligation,
   not a blocker.

2. **TRM's additions are the proprietary layer.** On top of the MIT base sit the Rust microkernel
   (v20.0.0), the 36 trm-rust intelligence crates, the 15-agent Monad council, the turbovec HDC
   engine, the NAND fabric, and the drug-discovery pipeline. These are TRM's own IP and are the
   license-out assets — they are **not** covered by the Samsung MIT grant, so they stay proprietary.

3. **Two unlicensed gaps to close before any external release:** `Jarvis-Advanced-main` and
   `gemma_swarm_adapter` carry no LICENSE. Each must either (a) be attributed to its source under its
   actual license, (b) be removed, or (c) have a license written if it is TRM's own work.

4. **Dataset obligations** (UniProt/PubChem/ClinVar/ClinicalTrials.gov/AlphaFold/tox21) are
   attribution/redistribution terms, not our IP — they ride along as a `THIRD_PARTY_NOTICES`.

## Action items (before any public or commercial release)

- [ ] Write `Jarvis-Advanced-main/LICENSE` (or remove/vendor-document the repo).
- [ ] Write `gemma_swarm_adapter/LICENSE` (or remove/vendor-document the repo).
- [ ] Produce a ship-ready `THIRD_PARTY_NOTICES` bundling Samsung MIT + the 5 dataset licenses +
      the vendored crates' MIT/Apache terms.
- [ ] Confirm the Samsung MIT copyright notice is retained in every distributed copy (MIT §"include
      this permission notice").

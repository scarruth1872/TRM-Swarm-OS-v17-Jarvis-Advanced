# CHECKPOINT — 2026-10-19 (pre-reboot)

Safe to reboot. All session work is committed to git (`Quantum-Nexus`, root `C:/Users/shawn`).
This records the state so nothing is lost across the reboot.

## Runtime state (must be verified after reboot)

| facet | now | after reboot |
|---|---|---|
| Monad ledger | **436 blocks**, tip `0354cf928f317adf` | file: `swarm_v2_artifacts/monad_ledger.jsonl` — persisted, nothing to do |
| Council | **44/44** motions enacted | verdicts in `council_strategic_deliberation.json` |
| Kernel | **v20.0.0 HEALTHY** on :8022 | restart with `python swarm_supervisor.py` (background/persist) |
| Dashboard | :5183 | restart alongside the kernel |
| Recall P@1 | **0.9487** (234 queries, all stores ≥ 0.909) | artifact: `swarm_v2_artifacts/fused_fusion_test.json` |

## This session — completed and committed

- **Recall arc closed**: 0.692 → 0.9487 P@1 over 234 queries (five measured fixes).
- **Drug-name catalyst mitigated**: name → ChEMBL id + SMILES via the voltage-doctrine sidecars
  (`chembl_name_index.json`, `chembl_smiles.json`) — derived, not fetched.
- **Exact-ID bug fixed**: O(1) byte-offset index (`chembl_id_offset_index.json`, 2,897,805 ids)
  replaces the broken sorted-scan early-exit.
- **Class of finding**: a reader must verify the data's shape before depending on it.
- **License audit**: Samsung MIT base (attribution required); `Jarvis-Advanced-main` +
  `gemma_swarm_adapter` UNLICENSED (gaps); 5 dataset licenses in `.licenses/`.
- **Marketing + pricing + cost-to-reproduce + pitch deck**:
  `docs/MARKETING_STRATEGY_20261019.md`, `docs/DIFFERENTIAL_PRICING_20261019.md`,
  `docs/COST_TO_REPRODUCE_20261019.md`, `docs/TRM_Swarm_OS_Pitch_Deck.html` (9-slide HTML deck).

## Two remaining gaps (unchanged)

1. chembl SMILES→bioassay overlap (2/234 queries).
2. Docking blocked on 3D conformers.

## Commits this session (latest first)

`fd3daad` deck · `cb3d711` cost-to-reproduce · `4ae7edb` pricing · `ff667fc` license+marketing ·
`d2aeb25` checkpoint+inventory · `19be621`/`10cf74a` exact-ID fix · `bde945b`/`31640cb` drug-name ·
`6e4943a`/`2e5a261` bioassay gap · `54284fb`/`4403802` fabric papers.

## Reboot procedure

1. Reboot the host.
2. `cd C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main`
3. `python swarm_supervisor.py` (background/persist) — brings up the kernel on :8022 + dashboard on :5183.
4. Verify: `GET http://127.0.0.1:8022/health` → `"status":"HEALTHY"`.

## Notes (do not touch)

- The uncommitted files in `git status` are runtime state (daemon JSON, kanban, findings) + unrelated
  projects under the home repo — they regenerate on boot, leave them uncommitted.
- Ollama stays stopped except embedder ingestion.
- Native Git path: `C:/Program Files/Git/cmd/git.exe` (MSYS shim throws "Permission denied" post-reboot).

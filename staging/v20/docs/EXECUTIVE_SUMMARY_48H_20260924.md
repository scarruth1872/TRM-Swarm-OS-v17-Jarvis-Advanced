# Executive Summary — the last 48 hours (window: 2026-09-22 19:23 → 2026-09-24 19:23 local)

Method: everything below is read off artifacts on disk — the ledger (`monad_ledger.jsonl`, 116 blocks),
backup directories, file mtimes and sizes, and the verification scripts actually executed. Nothing is
carried over from notes without a file behind it.

## 1. What was done

| when | what | evidence |
|---|---|---|
| 09-23 morning | **EFI port campaign** to 68 modules / 56 NEEDS-LOGIC | `docs/EFI_PORT_CAMPAIGN_LOG.md` (25 KB, 10:30) |
| 09-23 day | **30+ checkpoints**, WSL workspace frozen, structured index built | `backups/checkpoint_20260923_*` (31 dirs), `backups/wsl_freeze_20260923_180202/` (447 MB Rust source), `swarm_v2_artifacts/structured_index.db` (507 MB, 19:21) |
| 09-23 evening | **council roster 14 → 15**, USB boot drive cut | `backups/preroster15c_20260923_184258/`, `backups/usb_pre_r15_20260923_192506/` (30.9 MB) |
| 09-24 07:39 | previous feature table retired — **rebuilt from scratch** | `scratch/task_466_features_HANDROLLED_SUPERSEDED.csv` (196.7 MB) |
| 09-24 10:55–11:45 | **task suite truth**: 467 tasks parsed + classified, 5 domain slices | `docs/TASK_SUITE_TRUTH_20260924.md`, `scratch/task_suite_parsed.json` (467), `scratch/task_suite_domain{1..5}_classified.json` |
| 09-24 11:46–12:38 | **ChEMBL 37 decompressed (8.0 GB) and re-run through RDKit**: 2,897,805 molecules, 912 mol/s, 52m56s, 200/200 spot-check reproduce | `scratch/chembl_37_decompressed.sdf` (7,998.8 MB), `scratch/task_466_features_rdkit.csv` (425.0 MB) |
| 09-24 12:12–12:25 | **IPC/ring + monad anchor + corpus**: mailbox ring backed up, fabric pre-anchor snapshot, monad anchor sealed at idx 0..31, corpus 186 docs | `backups/mailbox_ring_20260924_121237/`, `backups/fabric_pre_anchor_20260924_122329/`, `docs/IPC_TRANSPORT_AND_K3_NAMESPACE_20260924.md`, `docs/MONAD_ANCHOR_SEAL_AND_CORPUS_20260924.md` |
| 09-24 13:18 | **checkpoint + clean shutdown for reboot**, then restart-and-resume | `backups/checkpoint_20260924_131803_REBOOT/`, ledger block **110** |
| 09-24 16:47–19:13 | **dashboard backend rollout, batches 1 → 7d** and `/brain/reason` closed — 15 ledger blocks | blocks **102–116**; `docs/DASHBOARD_DATA_PATH_AUDIT_20260924.md` |

## 2. The rollout, measured

| batch | subject | sealed | deploy | assertions |
|---|---|---|---|---|
| 1–5 | pillars · discovery/humanistic · continuum/microkernel · ModuleMatrix · architect/mobile-sync | blocks 104–108 | — | — |
| 6 | `/mesh` family (7 prefix sites answered one hardcoded two-node topology) + council propose ignoring its body | 109 | `00977fa82019d44d` | 39/39 |
| 7a | `/services/catalog` (asserted Φ_V 0.9115, 219/219 modules, 14/14 quorum, 2,167 antibodies) + the client's fake-key minting | 111 | `e3dbeda07fa54aec` | — |
| 7b | `/aura/*` (a **separate product**, `aipro2`) + 6 research actions the client reported as success | 112 | `920ae61caa264c57` | 47/0 |
| 7c | 12 engine actions + the asserted payloads no stub detector sees (`14/14/0`, `ddr_antibody_count 1801` vs store **27**, two invented messages) | 113 | `c411f5446dde7492` | 67/0 |
| 7d | six arms answering **200** with unmeasured values + `/brain/reason` (hardcoded `sample_context`, `confidence 0.94` for any body) | 114, 115, 116 | `fd6b2828beea62b1` | 60/0 |

Ruler (`scratch/scan_dashboard_sites_v2.py`): **167 rows — REAL 96 / REFUSAL 65 / STUB 0 / ERROR 5 /
UNREACHABLE 1**. STUB went 71 → 0 across the rollout.

## 3. What was corrected, not just added

- **The ruler itself**: refusals with long messages were unparseable (body truncated before
  classification), so they were filed as OTHER; fixed.
- **Deployed-image hash**: `0977fa8…` was a mis-transcription of `00977fa8…`; block 110 corrected.
- **Telemetry is now measured**: the accept loop times every request; `throughput_ops_sec` and
  `avg_latency_ns` are this process's own quotient over its own uptime (4–8 ms mean live) instead of
  the literals 4,251,552 and 235.2.
- **Two derivations replaced after review**: a lead id that rode along as a 100-char filename fragment,
  and `__init__.py` counted as a skill (block 115 supersedes the image block 114 named).

## 4. What is NOT done (stated plainly)

- The **147 acquirable tasks** need acquisition the platform does not currently perform itself.
- **No Rust cheminformatics engine**: descriptor computation still leaves for Python (offline).
- `/memory/write` has **no Rust write path**; the ledger has exactly one writer, by design.
- The federation is still **not synced from this host** (peers need `deploy_peer.bat` or
  `tailscale up --ssh` run on them); vision tiers 461/462/464/465/467 have **no executor** (4B VLM
  cannot share 8 GB VRAM with the 8B tier-2 that occupies `:8587`).
- **The task-runner layer generates artifacts rather than computing them** — see
  `docs/REFUSAL_TRIAGE_AND_MODULE_BACKLOG_20260924.md` §4: `run_tasks_441_445_optimization.py` filled
  200 "gradient descent convergence runs" from `random.uniform()` in 10 ms.
- Pre-existing and untouched: ledger blocks **97–101 unreproducible**, 56 EFI NEEDS-LOGIC modules,
  109 of 183 modules never referenced from the entry surface.

## 5. State at this moment

`:8022` Rust kernel **HEALTHY** (`fd6b2828beea62b1`, 2,134,016 B) · `python_active: false` · 15 council
members / quorum 9 · fabric 76,761,697 B = 153,523,394 trits · `:5183` dashboard up · `:8587`
**`llama-server` running Ternary-Bonsai-8B-Q2_0_g64** (this is the tier-2 model, not a generator
service) · `:8021` absent (correct) · `:11434` Ollama stopped (by design) · ledger **116 blocks**, 0
broken `prev_hash` links.

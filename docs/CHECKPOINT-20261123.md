# Checkpoint — 2026-11-23 (G227 → G238)

Measured state at the time of this checkpoint. Every figure below was read from the running platform or a
command, not carried forward from an earlier document.

## Provenance

| item | value |
|---|---|
| HEAD | `f228e28` (G237), tag `checkpoint-20261123-g237` |
| `build_commit` from `/health` | **`f228e28`** — clean, exactly HEAD |
| kernel version | `v19.1.0-sovereign`, uptime 146 s at read time |
| tests | `cargo test --release` → **122 passed, 0 failed** (110 + 4 + 6 + 1 + 1) |
| binary | `target/release/host_council_server.exe`, 19:35:07 |

## Live services (all HTTP 200)

`8022` kernel · `3000` Jarvis · `3001`/`3002` companion bridges · `5183` LCARS dashboard · `8587` gpt-oss-20b
(MXFP4, the only resident model) · `8593` embeddings.

| endpoint | measured |
|---|---|
| `/system/health_matrix` | **OPTIMAL**, `anomaly_count: 0` |
| `/kanban/stats` | 19 cards — todo 4 / done 6 / stale 13 (the 13 seeds) |
| `/amr/deployment` | `verdict REFUTED`, 3 refuted / 2 undetermined, 7 serialisers found |
| `/federation/stats` | 3 active of 5 configured, 2 offline |
| `/ddr/stats` | `scans_performed: 0` (per-process counter, reset by today's restarts), rowhammer mitigation `NOT_IMPLEMENTED` |
| `/health voltage_rails` | `measured: true`, 417 readings, HWiNFO64 SM2, v_core 1.488 V, v_soc 1.03125 V |

Disk `F:` 107 GB free of 465 GB.

## What this arc changed

- **G233/G234** — the kanban board became an instrument: `tools/cron/kanban_post.py` posts one card per
  measured defect (deduplicated by title, because the kernel numbers cards from `store.len()+1`), and
  `tools/cron/kanban_close.py` closes a card only when the same endpoint that opened it says the defect is
  gone. Its self-test caught a bug before it fired: a missing `anomalies` field would have CLOSED every
  anomaly card. Board went 13 seeded / 0 organic → 13 seeded / 6 organic.
- **G235** — runnability matrix for the 13 seed cards: 5 run (01 attestation, 04 DDR scan, 05 rails, 06 AMR,
  10 wet-lab audit), 1 already running (07), 1 half-blocked (12), 6 blocked by absent hardware/data/models.
  Erratum: the earlier claim that the TRM-ABX lead IDs exist only in the board was false — a grep scoped to
  `data/` and `docs/`.
- **G236** — second sweep: article 183 is the *sentinel/neural-bridge* article, `CP-ANOMALY-001` has no
  ledger record, ionosphere has zero implementation.
- **G237** — council was never stuck: proposal 1439 completed `REJECTED` with 14 votes and 1440 took
  **38.1 s**; the 0-vote reading was taken 6 s in. Real defect fixed: `"poll"` pointed at the list, not
  `/council/proposal/{id}`, and the response now states `deliberation_expected_seconds: 40`. Card
  `TASK-USR-0018` closed as **REFUTED, not fixed**. AMR node replaced with `src/amr_surveillance.rs` +
  `GET /amr/deployment`, which measures instead of narrating.

## Cron (11 jobs, all `no_agent` for the report path)

`0591e50a5893` hourly heartbeat **ok** · `ee126bdaf4ae` daily health · `ccb4dab59fe5` weekly security ·
`027715a380c5` weekly evolution · `6b89bac701cb` 6-hourly learning **ok** · `f08126777b8a` weekly skills
**ok** · `federation-registry-tick` every 5 min **ok** · `7164ea587c04` kanban check **ok** ·
`e12f823da647` kanban poster **ok** · `c6625c567d2e` kanban closer **ok** · `5f1c6f58249c` wisdom.

The four jobs whose `last_status` is `error` carry a **stale `HTTP 402`** from their last LLM run on
2026-11-23 05:00–07:00, before G226 repointed them at scripts. A provider-balance failure cannot recur with
no provider in the path, so `kanban_post.py` skips those as history and names them rather than posting them.

## Known open items (not defects of this checkpoint)

- `TASK-USR-0015` DDR rowhammer mitigation `NOT_IMPLEMENTED` — 7 rules loaded, none enforcing.
- `TASK-USR-0016`/`0019` PBFT quorum unreachable: threshold **67**, active peers **3**.
- `TASK-USR-0017` wet-lab provenance: 5 defects, incl. TRM-ABX-220 verdict ~50× inconsistent and no SMILES.
- AMR steps 4 and 5 undetermined (no MIC/EUCAST parser, no drill); steps 1–3 refuted on this host.
- Six seed cards blocked on hardware/data this box does not have (02, 03, 08, 09, 11, 13).

## Two limits worth knowing

1. **The poster's dedupe reads every column, including DONE.** A defect that regresses after its card was
   closed will not re-open, because the title still exists on the board. Correct for "do not duplicate",
   wrong for "recurrence".
2. **The DDR counter is per process.** `scans_performed` resets to 0 on every kernel restart, so a restart
   makes the "0 scans" condition true again — which is why the counter is reported with `last_scan` and
   `counter_window` rather than as an absolute.

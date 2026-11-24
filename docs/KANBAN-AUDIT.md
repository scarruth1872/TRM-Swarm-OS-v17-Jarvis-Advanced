# Kanban audit: all 13 seeded cards, checked against the platform (2026-11-23)

The board was seeded on 2026-08-27 and never moved. This audit decides, card by card, whether the claim
is backed by something in this repo or on this machine. Nothing was deleted to produce this document.

**Method.** For each card, the specific claim was searched for: the artefact on disk, the endpoint that
serves it, or the code that implements it. Three of the numbers on the board — `13,147`, `1,420`, `60` —
are the same literals G161 removed from `/kanban/verify` because they were invented there; their
reappearance here is evidence about the seed, not about the platform.

| card | claim | found | verdict |
|---|---|---|---|
| `TASK-CAD-01` Monthly UEFI Kernel & Boot Media Re-Flash | "28th monthly re-flash, G:\ attestation under SHA-256 seal" | `trm_uefi_kernel.efi` **exists** (17,488,896 B, 2026-11-13); `G:` drive exists; **no flash or attestation script anywhere** | **KEEP, restate** — image real, the monthly ritual is not instrumented |
| `TASK-CAD-02` FDA eCTD Module 5 Dossier | "230-patient Phase 2 report, differential privacy ε=0.45" | No patient data, no trial, no eCTD tooling in the repo | **RETIRE** — fiction |
| `TASK-CAD-03` 100-Node Byzantine Health Audit | "67/100 quorum across all 6 tiers" | `/council/federation/nodes`: **2 active**, 5 configured, 3 offline, threshold 67, fault tolerance 33 | **KEEP, restate** — the audit is real at 2 nodes; "100-node" is scale fiction, though 67/100 is a real threshold |
| `TASK-CAD-04` Muon Flux & 6s Scrubbing | "13,147 daily cosmic muons, SEU mitigation on DDR" | Muon appears only as **prose in route text**; `/ddr/stats`: `scans_performed: 0`, `rowhammer_mitigation: NOT_IMPLEMENTED`, `last_scan: on request` | **RETIRE** — no detector; the DDR half is a real *finding*, now posted as real work |
| `TASK-CAD-05` Hourly Voltage Harmonization | "lock 15.53 THz optical phonon mode to substrate PLL" | Voltage **is** measured (HWiNFO SM2, φ_v, 417 readings); **no voltage tool or hourly job exists**; the phonon claim has no code | **RETIRE** — φ_v is real, the card describes something else |
| `TASK-CAD-06` AMR Node in Kisumu | "1,420 MIC samples on Raspberry Pi 4, Z=3.42" | No Pi node, no Kisumu deployment; `1,420` is one of the G161 invented literals. ESKAPE proteomes (25,888) exist but are not this | **RETIRE** — fiction |
| `TASK-CAD-07` Anomaly Sentinel Telemetry Radar | "3s polling across all 5 core ports with auto-healing" | **Real**: guard polls 6–7 listeners every **2 min** (was PT1M), mutex-guarded, per-service auto-healing verified today (bridge pid 39796 → restarted) | **KEEP, restate** — 2 minutes and 7 listeners, not 3 s and 5 |
| `TASK-CAD-08` 6-Tier 60-Task Certification | "100.0% pass rate across Instella, Bonsai, DeepSeek-R1, Qwen-Coder, Gemma3, TRM Native" | `/learning/skills` explicitly notes it used to return **literal** capabilities; Bonsai's ternary GGUFs are structurally unservable; measured gate results are 95.5 % / 91.7 %, not 100 % | **RETIRE** — "a metric that can only return 100 %" |
| `TASK-CAD-09` Ionosphere F2 MUF/OWF Protocol | "planetary skywave mesh synced to 15.92 fs, ML-DSA-87" | Appears only as a **tier description string** in route text | **RETIRE** — fiction |
| `TASK-CAD-10` 11 Preclinical Leads Locked | "TRM-TBX-001 InhA Kd=2.4 nM, TRM-ABX-220, TRM-NTD-CRZ-01" | The IDs appear **only** in `data/kanban_board.json` and its backup — nowhere in shards, code or docs | **RETIRE** — the records do not exist |
| `TASK-CAD-11` Neural Bridge Persona Routing | "direct Ollama gemma3:1b socket across 14 personas" | `gemma3:1b` appears only in `docs/port/laptop-base/intelligence_router.rs`; **Ollama is forbidden on this box**; `route_agent_intelligence()` is keyword-templated despite its docstring | **RETIRE** — contradicts doctrine and the "activated" claim is false |
| `TASK-CAD-12` Monad Journal Article 183 | "unanimous +0.978 PBFT consensus, CP-ANOMALY-001 enacted" | `/journal`: **180 papers, ids 101–281, article 183 present**, `articles_verified: true`; `CP-ANOMALY-001` appears only in `docs/index.html` | **KEEP, restate** — the article is real; the consensus/enactment claim is unverified |
| `TASK-CAD-13` 54-Asset State Sync | "desktop and laptop in lockstep, zero divergence" | 2 active / 5 configured / **3 offline** peers | **RETIRE** — "zero divergence" is contradicted by its own federation stats |

**Result: 4 keep (all needing restatement), 9 retire.**

The four that survive are the ones where a real artefact exists and only the *description* is inflated —
which is the difference between a stale card and a fabricated one. The nine that go describe clinical
trials, hardware that is not here, and figures that were already caught as inventions once.

## Real work posts instead

From now on the board is fed by measurement, not prose: `tools/cron/kanban_post.py` reads
`/system/health_matrix`, `/ddr/stats`, `/federation/stats` and the Hermes job list, and posts one card per
**current** defect, deduplicated by title so a re-run adds nothing. It posts findings, never progress:
a card appears because something is wrong now, and it is closed by moving it, not by the poster deciding
it is fixed.

## Verified after implementation

- `kanban_post.py --dry-run` first reported **7** cards, 4 of which were the pre-G226 `HTTP 402` history of
  jobs that are now `no_agent` scripts. A provider-balance error cannot recur with no provider in the path,
  so those are skipped and named, not posted — otherwise the board would fill with archaeology and call it
  work.
- Live post created `TASK-USR-0014/0015/0016` (DDR 0 scans, DDR rowhammer NOT_IMPLEMENTED, 3 of 5 peers
  offline). A second identical run posted **0** and reported `already present 3`, so the hourly job cannot
  duplicate the board.
- Board state after: **16 cards — 13 seeded, 3 organic**, newest update 0.0 days ago, verdict **STALE**
  instead of **FROZEN**. The seed is still there; only it is still 89 days old.

Jobs: `7164ea587c04` check 06:45 daily, `e12f823da647` poster hourly at :20, both `no_agent`.

## The verified closer (job `c6625c567d2e`, hourly at :35)

`tools/cron/kanban_close.py` closes a card only when the *same* endpoint that opened it says the defect is
gone, and prints the evidence:

```
TASK-USR-0014: CLOSED -> DONE — /ddr/stats scans_performed = 1 (> 0)
TASK-USR-0015: STILL OPEN — /ddr/stats rowhammer_mitigation = NOT_IMPLEMENTED
TASK-USR-0016: STILL OPEN — /federation/stats offline_nodes = 3
closed 1, still open 2, left alone 9 (seeded or no rule)
```

That close is a real end-to-end proof rather than a dry run: `POST /ddr/scan` was executed on the live
kernel and moved `scans_performed` 0 → 1, which is precisely the condition the card named. The card
`TASK-USR-0014` had been created from the 0-scan state and closed from the 1-scan state.

**A bug the self-test caught, before it closed anything in production:** the anomaly rule defaulted a
missing `anomalies` field to an empty list, so an unreadable `/system/health_matrix` would have CLOSED
every anomaly card — absence of data treated as absence of the defect. `--self-test` now covers 12 cases in
both directions (`12/12`), including `{"hm": {}}` → left open, and every rule is exercised both resolving
and not resolving. Cards with no rule and all 13 seeded cards are named as unverifiable and left untouched.

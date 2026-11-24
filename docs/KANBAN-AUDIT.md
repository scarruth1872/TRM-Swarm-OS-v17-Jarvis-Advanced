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
| `TASK-CAD-02` FDA eCTD Module 5 Dossier | "230-patient Phase 2 report, differential privacy ε=0.45" | `ectd` appears in `autonomous_cadence_council_deliberation.json`, `autonomous_periodic_task_cadence_ledger.json` and two archived plan docs — **prose and deliberation only**; no patient data, no trial, no eCTD tooling | **BLOCKED** — no data exists to run it |
| `TASK-CAD-03` 100-Node Byzantine Health Audit | "67/100 quorum across all 6 tiers" | `/council/federation/nodes`: **2 active**, 5 configured, 3 offline, threshold 67, fault tolerance 33 | **KEEP, restate** — the audit is real at 2 nodes; "100-node" is scale fiction, though 67/100 is a real threshold |
| `TASK-CAD-04` Muon Flux & 6s Scrubbing | "13,147 daily cosmic muons, SEU mitigation on DDR" | Muon appears only as **prose in route text**; `/ddr/stats`: `scans_performed: 0`, `rowhammer_mitigation: NOT_IMPLEMENTED`, `last_scan: on request` | **RETIRE** — no detector; the DDR half is a real *finding*, now posted as real work |
| `TASK-CAD-05` Hourly Voltage Harmonization | "lock 15.53 THz optical phonon mode to substrate PLL" | Voltage **is** measured (HWiNFO SM2, φ_v, 417 readings); **no voltage tool or hourly job exists**; the phonon claim has no code | **RETIRE** — φ_v is real, the card describes something else |
| `TASK-CAD-06` AMR Node in Kisumu | "1,420 MIC samples on Raspberry Pi 4, Z=3.42" | No Pi node, no Kisumu deployment; `1,420` is one of the G161 invented literals. ESKAPE proteomes (25,888) exist but are not this | **RETIRE** — fiction |
| `TASK-CAD-07` Anomaly Sentinel Telemetry Radar | "3s polling across all 5 core ports with auto-healing" | **Real**: guard polls 6–7 listeners every **2 min** (was PT1M), mutex-guarded, per-service auto-healing verified today (bridge pid 39796 → restarted) | **KEEP, restate** — 2 minutes and 7 listeners, not 3 s and 5 |
| `TASK-CAD-08` 6-Tier 60-Task Certification | "100.0% pass rate across Instella, Bonsai, DeepSeek-R1, Qwen-Coder, Gemma3, TRM Native" | `/learning/skills` explicitly notes it used to return **literal** capabilities; Bonsai's ternary GGUFs are structurally unservable; measured gate results are 95.5 % / 91.7 %, not 100 % | **RETIRE** — "a metric that can only return 100 %" |
| `TASK-CAD-09` Ionosphere F2 MUF/OWF Protocol | "planetary skywave mesh synced to 15.92 fs, ML-DSA-87" | Appears only as a **tier description string** in route text | **RETIRE** — fiction |
| `TASK-CAD-10` 11 Preclinical Leads Locked | "TRM-TBX-001 InhA Kd=2.4 nM, TRM-ABX-220, TRM-NTD-CRZ-01" | **ERRATUM (see below): the IDs DO exist** — `wet_lab_bridge_package.json`, `trm_abx_220_recalibration.json`, `100k_resolution_and_strategic_expansion.json`, `audit_wetlab_provenance.py` | **KEEP** — runnable, and its own audit returns 5 defects |
| `TASK-CAD-11` Neural Bridge Persona Routing | "direct Ollama gemma3:1b socket across 14 personas" | `gemma3:1b` appears only in `docs/port/laptop-base/intelligence_router.rs`; **Ollama is forbidden on this box**; `route_agent_intelligence()` is keyword-templated despite its docstring | **RETIRE** — contradicts doctrine and the "activated" claim is false |
| `TASK-CAD-12` Monad Journal Article 183 | "unanimous +0.978 PBFT consensus, CP-ANOMALY-001 enacted" | `/journal`: **180 papers, ids 101–281, article 183 present**, `articles_verified: true`. Its real title is *"Neural Bridge Dynamic Cognition & Real-Time Platform Anomaly Sentinel: Autonomous 5-Service Telemetry, 14-Agent Dialogue Engine & One-Click Self-Healing"* — i.e. it is the article about cards 07 and 11, not about a PBFT consensus. `CP-ANOMALY-001` appears **only** in `data/kanban_board.json` and its backup, in no ledger | **KEEP, restate** — article real; the enactment claim has no record anywhere |
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


---

# ERRATUM — the card-10 claim in the table above was wrong

The audit above said the TRM-ABX lead IDs appear "only in `data/kanban_board.json` and its backup".
**That is false**, and the reason is a search that could not have found them: the grep was scoped to
`data/` and `docs/`. A repo-wide search finds:

```
wet_lab_bridge_package.json                  PRE_REGISTRATION_MANIFEST (2026-08-23)
trm_abx_220_recalibration.json               recalibration ledger (2026-08-27, REVISED_UNRESOLVED)
docking_calibration_benchmark.json           calibration of record (pearson_r 0.9248)
eskape_spectrum_full.json                    ESKAPE spectrum (2026-08-27)
100k_resolution_and_strategic_expansion.json
audit_wetlab_provenance.py                   the read-only provenance auditor for the dispatch
autonomous_periodic_task_cadence_ledger.json
```

Card 10 is not fiction. It is a **runnable task whose own auditor reports it is not ready**, which is a far
more useful state than "fabricated". The corrected verdict is KEEP, and the audit output below is the
evidence.

# Runnability: can each card's task actually execute on this machine?

The question changed from "is this card honest" to "will it run". A card is RUNNABLE when the inputs it
needs exist here (hardware, ledgers, models, services); it is BLOCKED when something measurable is missing.
Running the runnable ones is the only way to answer it.

| card | runs? | evidence |
|---|---|---|
| 01 UEFI re-flash + attestation | **RAN (attestation)** | Built `trm_uefi_kernel.efi` and `G:/EFI/BOOT/BOOTX64.EFI` are **byte-identical**: both `504c41ee647ff45f8bb07b38da6c6f288f1f83a556eec8b8501ad367818308b3`, 17,488,896 B. Boot media is real: `G:` FAT32, label **SWARM_OS**, 14.6 GB. The *re-flash* step is destructive and was not run. |
| 02 FDA eCTD dossier | **BLOCKED** | Needs patient-level trial data and eCTD tooling; `ectd` appears only in a council deliberation JSON and an archived plan doc. No trial data exists here. |
| 03 100-node Byzantine audit | **RAN, and cannot pass** | `/council/federation/nodes`: threshold **67**, fault tolerance 33; `/federation/stats`: **2 active** of 5 configured. 2 < 67, so quorum is arithmetically unreachable — the audit's honest result is "cannot form quorum", not a pass. |
| 04 Muon flux + 6s scrubbing | **SPLIT** | DDR half **RAN**: `POST /ddr/scan` → `scans_performed` 0→1, 0 findings, 6/6 taxonomy IDs verified. Muon half cannot run: no detector, and `muon` appears only as prose in route text. |
| 05 Voltage harmonization | **RAN (measurable half)** | `/health` → `voltage_rails.measured=true`, **417 readings**, source HWiNFO64 SM2, v_core 1.488 V, v_soc 1.03125 V. The "15.53 THz phonon PLL lock" half has no code. |
| 06 Kisumu AMR node | **RUNS, BUT MEASURES NOTHING** | `scratch/kernel194/src/hospital_amr_surveillance_node.rs` has `execute_amr_deployment()`, and every step is annotated by its own author: *"Nothing verifies these steps: the metrics beside them are authored strings"*, `k3_verification: Unknown`. No Raspberry Pi on this machine. |
| 07 Anomaly sentinel | **RUNNING NOW** | Guard: 7 listeners, 183 ms healthy path, auto-heal proven today. |
| 08 6-tier 60-task cert | **BLOCKED (1 of 6 tiers)** | `/props`: only `:8587` answers, serving `gpt-oss-20b-MXFP4.gguf`; 8595/8596/8597 return nothing; Bonsai's ternary GGUFs are structurally unservable. A 1-tier eval runs (95.5 %/91.7 % measured). |
| 09 Ionosphere F2 MUF/OWF | **BLOCKED** | No SDR/ionosonde code or hardware; MUF/OWF/skywave appear only in a tier description string. |
| 10 11 leads locked | **RAN — 5 DEFECTS** | `audit_wetlab_provenance.py` completed after a one-line fix (it read `confidence_interval["validation_verdict"]`; the key is at the ledger's top level). Output: TRM-ABX-220 verdict **INCONSISTENT (~50× apart)**, three potency windows **UNDERSTATED/INCONSISTENT** (219/082/220), and **TRM-ABX-220 has no SMILES anywhere in the repo**. |
| 11 Ollama persona routing | **MUST NOT RUN** | Ollama is forbidden on this box; `gemma3:1b` appears only in a laptop port doc. |
| 12 Article 183 / CP-ANOMALY-001 | **HALF RUNS** | Article 183 exists (`/journal`: 180 papers, manifest-verified). The enactment path accepts work — `POST /council/propose` created **proposal 1439**, `min_approve: 10` — but it sits at `deliberation_mode: pending` with **0 votes**, so nothing can be enacted. |
| 13 54-asset state sync | **BLOCKED AS WRITTEN** | 2 active / 5 configured / **3 offline**. A 2-node sync does run (federation tick, every 5 min, `ok`). |

**Score: 5 run (or partly run) on this machine — 01, 04, 05, 06, 10 — 1 already running (07), 1 half-blocked
by a non-deliberating council (12), and 6 genuinely blocked by absent hardware, data or models (02, 03, 08,
09, 11, 13).** The blocked ones are blocked for stated, checkable reasons rather than because nobody tried.

## Evidence pass, second sweep (same day)

A repo-wide grep re-run confirmed two details the first pass stated too narrowly:

- **Article 183 is the sentinel/neural-bridge article**, so `TASK-CAD-12` describes an article that exists
  about two *other* cards. The `CP-ANOMALY-001` id it cites appears **only** in `data/kanban_board.json`
  and its backup — no ledger, no council record, so "enacted" has nothing behind it.
- **`ectd` also appears in the two autonomous cadence ledgers**, not just an archived plan. That is still
  prose and deliberation: no patient data, no submission tooling, so card 02 stays blocked on data rather
  than on honesty.
- **Ionosphere (`MUF`/`OWF`/`skywave`) returned zero hits** outside tier-description strings — card 09 has
  no implementation, confirmed rather than assumed.

---

# ERRATUM — `TASK-USR-0018` was wrong, and the reason matters

`TASK-USR-0018` said *"Council cannot enact: proposal 1439 sits at deliberation_mode=pending with 0 votes
after being accepted"*. **That claim was false.** Re-measured:

```
proposal 1439  phase=REJECTED  deliberation_mode=ssm+llm:8587
               approve 3 / abstain 5 / reject 6   weighted -0.1486
               deliberations 14  votes 14  evidence_used 2
proposal 1440  completed in 38.1 s   approve 3 / abstain 4 / reject 7   weighted -0.2057
```

The council deliberates. The "0 votes" reading was taken **6 seconds** into a deliberation that takes
**38.1 s** with 14 agents at concurrency 4 against `:8587`. The card is closed as **REFUTED, not fixed** —
the record is corrected here rather than deleted.

## The real defect the misreading exposed

The propose response published `"poll": "/council/proposals"` — the **list**, not `/council/proposal/{id}`.
A caller following it can never see its own proposal's progress, and `EVALUATING` + `deliberation_mode:
pending` with no stated expectation reads naturally as "stuck". My own reading is the proof that this
misleads. Fixed in `main.rs`:

```
"poll": format!("/council/proposal/{}", id),
"deliberation_expected_seconds": 40,
"deliberation_note": "EVALUATING + deliberation_mode=pending are the normal transient state;
                      measured completion at 38.1 s for 14 agents. A reading taken before that is not a verdict."
```

Verified live on a fresh proposal: `id 2520 | poll /council/proposal/2520 | expected_s 40`.

# Card 06 — the AMR node now measures

`GET /amr/deployment` (new, `src/amr_surveillance.rs`) replaces the scratch module that emitted authored
strings. Measured on this host: `verdict REFUTED | passed 0 | refuted 3 | undetermined 2`, with arch
`x86_64` / 24 cores, **7** serialisers in `tools/`, sha256 `062b0cf3449094a6…` of the ESKAPE file (3 of 7
named genes present), and a 0.244 ms real ingest. REFUTED is the correct verdict: the deployment target is
aarch64. Details in `docs/AMR-SURVEILLANCE.md`.

# Cron ownership: the platform writes its own reports

**Change (2026-11-23): all eight Hermes cron jobs are now `no_agent` script jobs.** No LLM, no cloud
provider, no balance dependency. The platform owns its reports.

## Why

Five of the eight jobs were "GET an endpoint on :8022, then narrate the JSON". That put a language
model between a measurement and its reader — it could drift, it could invent a value, and on
2026-11-23 five jobs failed with `HTTP 402: Insufficient Balance` while the platform they describe was
healthy and answering. A health report that cannot run because an unrelated account is out of credit
is a report about the wrong system.

## What runs now

| job | schedule | entry point | what it does |
|---|---|---|---|
| Swarm Hourly Heartbeat | hourly | `trm_report_heartbeat.py` | `/health` + `/system/health_matrix` → kernel, anomaly count, service states |
| Swarm Daily Health & Sync | 06:00 | `trm_report_daily.py` | adds mesh topology, experts, verification, skill count |
| Swarm Weekly Security Scan | Mon 05:00 | `trm_report_security.py` | security stats, threat rows, DDR counters, flags non-zero |
| Swarm Weekly Evolution | Mon 07:00 | `trm_report_evolution.py` | genome, proposal phases, sentinels, council status |
| Swarm Learning Feedback | every 6 h | `trm_report_learning.py` | learning registry, skills, genome |
| Swarm GitHub Skill Acquisition | Mon 08:00 | `trm_report_skills.py` | reads the registry; acquires nothing (no such native endpoint) |
| wisdom-pipeline | 06:00 | `trm_wisdom_pipeline.py` | runs `uqi-platform/scripts/harvest_wisdom_v2.py` (stdlib + curl, no LLM) |
| federation-registry-tick | every 5 m | `federation_tick_hysteresis.py` | unchanged |
| **Swarm Kanban Board Check** | **06:45 daily** | **`trm_kanban_check.py`** | **board counts, stale %, seeded vs organic, verify status, store mtime** |

## The kanban job (added 2026-11-23, job `7164ea587c04`)

The board endpoints have existed since G161 (`/kanban/board`, `/kanban/stats`, `/kanban/verify`) but
nothing ever called them on a schedule. Measured the moment the check first ran:

```
cards: 13 total — todo 0 / in_progress 7 / review 2 / done 4
stale: 13/13 (100.0%) — nothing moved for more than 7.0 days
provenance: 13 seeded card(s), 0 changed organically
age: newest update 7.4 days ago, oldest 89.0 days ago
verification: UNVERIFIED (not instrumented) — no per-card evidence is instrumented
store file: last written 2026-11-16 13:18 UTC (7.4 days ago)
verdict: FROZEN
```

So "the kanban board hasn't been running" is doubly true: no job ran, and no card has moved since the
seed. The check reports both facts and **moves nothing** — inventing a board transition would manufacture
progress that never happened. Two honest exits from here, both the user's call: post real work through
`POST /kanban/cards/{id}/move`, or retire the seeded cards (they describe an FDA eCTD dossier, a muon-flux
scrubber and a 100-node federation audit — seed fiction, not a backlog).

It also prints `UNVERIFIED (not instrumented)` verbatim rather than converting a missing verification into
a tick, and reports `unmeasured` for any field the kernel does not instrument. An unreachable kernel exits
3 with "no values reported — the board was not measured, not found empty".

### Note on the four jobs still showing HTTP 402

`wisdom-pipeline`, Daily Health, Weekly Security and Weekly Evolution still carry `last_error: HTTP 402:
Insufficient Balance` in `cronjob_manage list`. Those are **pre-G226 records**: their last LLM run was
05:00–07:00 on 2026-11-23, before the jobs were repointed to `no_agent`. Heartbeat and Learning, which
have fired since, show `ok`. The next fires are script-only and cannot 402.

### Layering, and why it is split

- **Implementation lives in the platform repo**: `tools/cron/platform_report.py` (one module, one
  report per subcommand) and `tools/cron/job_<name>.py` (one-line entry points). Versioned with the
  platform it describes.
- **Hermes requires scripts under its own directory** — `cronjob_manage` rejects an absolute path
  ("Script path must be relative to `%LOCALAPPDATA%\hermes\scripts\`"). So `~/AppData/Local/hermes/
  scripts/trm_*.py` are four-line shims that put the repo on `sys.path` and call the real module. The
  duplication is a filename, not logic.

### Reporting rules encoded in the module

- Every value is a field the server returned **in this run**. A missing field prints `unmeasured`;
  nothing is filled in, defaulted, or summed in prose.
- A non-200 endpoint is named in the output (`Endpoints: 5/6 HTTP 200 — failed: {...}`), never hidden.
- If the council is unreachable the report says `COUNCIL UNREACHABLE` and reports **no** values. It does
  not start the service (that is the guard's job) and does not exit non-zero by default, because
  `no_agent` maps a non-zero exit to an engine error rather than to the report. `--strict` exits 3.
- `genome.fitness_score` is reported as `UNMEASURED` with its basis string, because this kernel
  computes no fitness — the published `0.99956` was removed as a fabricated constant.

Verified output of a real run (`cron/output/0591e50a5893/2026-11-23_11-22-10.md`):

```
Kernel: **HEALTHY** — Native Rust Sovereign Kernel v19.1.0-sovereign build `9aabc88`, uptime 13946s,
python_active false
Anomalies: **1** — k3 +1_GOLD, PBFT 14/14
  - LCARS Dashboard :5183 LOW — LCARS dashboard unreachable on port 5183
Services: 5 probed, 1 not ONLINE (lcars_dashboard=OFFLINE)
Voltage: measured=true phi_v=0.444 rails={'measured': False, 'error': 'HWiNFO shared memory not enabled…'}
```

## Addendum — the rail reading is not stable, so neither was my first claim about it

The sample above was captured at 16:21 UTC, when `/health` reported
`voltage_rails.measured = false` with `error: "HWiNFO shared memory not enabled (Settings → Shared
Memory Support)"`. **One minute later the same job, same endpoint, reported `measured = true` with 417
SM2 readings** and real rails:

```
v_core 1.488 V (HWiNFO64 SM2: Vcore SVI2) | v_gpu 1.1768 V | v_soc 1.03125 V
v_ddr 1.2 V — WMI Win32_PhysicalMemory.ConfiguredVoltage, i.e. SPD configured, NOT a live rail
phi_v 0.8096
```

Two corrections, both recorded rather than deleted:

1. I first told the owner the heartbeat's "rails unmeasured" prose was wrong and the endpoint should be
   trusted. That was wrong — the endpoint agreed with the prose at that moment.
2. I then said the prose was right and the rails were unmeasured. That was true at 16:21 and false at
   16:22.

The lesson for the report path: **HWiNFO SM2 availability is a runtime state, not a build property.**
A single sample of `voltage_rails` cannot support "voltage is unmeasured on this host" — read
`measured` and `source` on every run (the report does, since it prints the whole field), and treat a
transition from unmeasured to measured as an event worth noting rather than a contradiction to argue
about.

## SwarmKernel guard: the firing fixed

`tools/swarm_kernel_guard.ps1` supervises five listeners (kernel 8022, companion proxy 3001, companion
bridge 3002, tailnet relay 8022, tailnet companion relay 3001). It ran **three `Get-CimInstance
Win32_Process` enumerations on every tick**, 1,440 times a day, healthy or not.

| | before | after |
|---|---|---|
| healthy-path runtime | **442–962 ms** | **14 ms** |
| WMI enumerations (healthy) | 3 | **0** |
| trigger interval | PT1M | PT2M |
| overlapping runs | raced | named mutex, second run exits |
| start failure | aborted the remaining services | isolated per service, logged, continues |

The fast path probes the five ports and exits if all answer — the check now measures the service
itself rather than enumerating every process on the machine. WMI is only reached for a service whose
port is dead. The tailnet probes are skipped when `tools/tailnet_bridge.py` is absent.

**Two real bugs found while testing the slow path** (both would have left a service permanently dead):

1. `Start-Process -ArgumentList ''` throws in PowerShell — the companion bridge start failed every time
   and the throw was swallowed by the outer `catch`, which **also skipped the kernel start that follows
   it**. Fixed by building the parameter set conditionally and reporting `START FAILED <tag>`.
2. A failure in any one service aborted the rest. Each start is now isolated.

Verified end-to-end: the companion bridge was force-stopped (pid 225828), the guard restarted it
(pid 93972), and `:3002` and `:3001` both answer 200. Recovery is real, not assumed.

## Still open

- **`:5183` LCARS dashboard** and **`:8588` ARC routes** are down. The kernel reports 5183 as a LOW
  anomaly; it does not probe 8588 at all, so that gap is invisible to the platform's own health matrix.
- Delivery: every job still writes `local`. Pointing them at Telegram is one field per job
  (`deliver: telegram:<chat_id>`), not wired in this change.

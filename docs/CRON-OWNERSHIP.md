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

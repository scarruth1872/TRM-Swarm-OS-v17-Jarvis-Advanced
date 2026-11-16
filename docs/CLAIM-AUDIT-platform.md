# CLAIM AUDIT — what the platform says about itself vs what is true

Method: for each self-reported number, find what would falsify it, then test that
independently. Sources: the live endpoints, the server source, the real test suite
on disk, host hardware APIs, and the WSL + antigravity reference trees.

## 1. FABRICATED — values the platform reports but cannot measure

**`/testing/stats` is a static fixture, not a test report.**

```
served : total_tests 60 | passed 60 | failed 0 | accuracy 100.0%
         coverage "100.0%" | k3_consensus UNANIMOUS_PLUS_1_GOLD | pqc_seal ML-DSA-87
real   : tests/ collects 31 tests -> 28 passed, 3 ERRORS   (90.3%)
         full repo: 510 py files / 11,703 test functions
                  + 409 rs files /  5,046 #[test]      = 16,749 test functions
         run artifacts on disk: 0 (no junit xml, no .coverage, no pytest_cache)
```

The endpoint serves a hardcoded array of six "runs" (TIER-1..TIER-6) whose
timestamps are all the identical `2026-09-03T13:36:06Z`, durations are string
literals like `"18.5 us"`, and `stats_obj` carries `"accuracy_percent": 100.0`.
The 3 real errors are a genuine integration break:
`Client.__init__() got an unexpected keyword argument 'app'`.

Origin check, done first as instructed: `TIER-5-TRAN` / `UNANIMOUS_PLUS_1_GOLD` /
`ML-DSA-87` appear in **exactly one file** — `host_council_server/src/main.rs`.
Not in the antigravity brain (1,133 files scanned), not in the WSL home, not in
WSL bash history (entirely UEFI/boot work), not elsewhere in the repo. It is not a
stale copy of a real plan. It is invented where it is served.

**`/infra/health` reports a CPU temperature and fan speed the host cannot produce.**

```
served : cpu_temp_c 44.5   fan_rpm 1200   status OPTIMAL   running true
host   : MSAcpi_ThermalZoneTemperature  - class unavailable
         Win32_Fan                       - class unavailable
         Win32_TemperatureProbe          - class unavailable
         OpenHardwareMonitor WMI         - class unavailable
         LibreHardwareMonitor WMI        - class unavailable
source : main.rs:2131  "cpu_temp_c": 44.5,  "fan_rpm": 1200,   <- literals
         main.rs:576   "...CPU 44.5C | GPU 51.0C | Fan 1200 RPM | 3.8 GHz..."
```
`cpu_temp_c` returned 44.5 on every call while the real benchmarks vary run to
run. A health endpoint is the worst place for an invented number.

## 2. CONTRADICTORY — endpoints disagree on the same platform

| item | endpoint A | endpoint B |
|---|---|---|
| `voltage_phi_v` | /health 0.6778, /system/status 0.6778, /system/voltage 0.6778 | **/system/resources 0.9115** |
| `active_nodes` | /federation/stats **100** ("100.0000% integrity", merkle_root, 6 tiers) | /swarm/telemetry **2** |
| journal `latest_article` | /journal/status names **Article 189** | /journal/articles serves ids to **197** |

0.9115 is the value the voltage doctrine says must never be hardcoded; one
endpoint still serves it while three others report the measured 0.6778.
The federation endpoints assert a 100-node, 6-tier, 100%-integrity network while
telemetry on the same process reports 2 nodes.

## 3. HARDCODED METRIC LITERALS — 76 assignments across 45 names

Scanned `host_council_server/src/main.rs` for `"metric": <number>` inside `json!`:

```
cpu_temp_c 44.5 | fan_rpm 1200 | accuracy_percent 100.0 | success_rate_24h 100
scheduler_latency_us 0.42 | retrieval_latency_us 0.85 | harmony_index 0.99
fitness_score 0.99956 | total_verified 100006 | completed_tasks 100000
active_nodes 100 | data_integrity 100.0000 | work_stealing_efficiency 99.98%
vram_allocated_mb 1120.0 | match_percentage 0.98/0.95/0.91 | duration_seconds ...
```

Not all of these are defects. Split honestly:

- **Legitimate static config** (hardware facts, not measurements):
  `memory_total_mb 32768`, `total_gb 12.0`, `l3_sram_kb`, `cpu_clock_mhz`,
  `real_modules_count`.
- **Fabricated measurements** (must be measured, are literals):
  `cpu_temp_c`, `fan_rpm`, `accuracy_percent`, `scheduler_latency_us`,
  `retrieval_latency_us`, `harmony_index`, `fitness_score`, `completed_tasks`,
  `total_verified`, `duration_seconds`, `match_percentage`.

## 4. VERIFIED REAL — these claims survive

Not everything is hollow, and saying so matters:

- **`/k3/bench`** — 7 calls returned ns_per_op 0.8783, 0.8803, 0.8785, 0.8861,
  0.8803, 0.9269, 0.8864. **6 distinct values in 7 runs.** A real timing loop;
  the ~1.13e9 ops/sec figure is reproducible.
- **`/turbovec/bench`** — ns_per_triple varied 1441.99-1448.61 over 4 calls.
- **`/turbovec/femto`** — wall_ns_per_step varied 1190.68-1205.21.
- **`/k3/sram/bench`** — ns_per_store_plus_lookup varied 401.01-418.07.
- **`/system/voltage`** — phi_v 0.6778 MEASURED, with formula, rail values and
  source (`wmic CurrentClockSpeed + OS memory`) disclosed. The doctrine is
  honoured here; it is violated only by `/system/resources`.
- **`/research/stats`** — self-reports `"clinical_trial_hospitals": "RETRACTED - no
  clinical trial exists; TRM-ABX-220 has no molecular structure on record"`. The
  platform is honest about its worst claim.
- **`/k3/sram/patterns`** — carries the note *"Article 118 claims 235.2 ns across
  all 15"* beside per-pattern values running 0.0 to 1888.342: the endpoint itself
  exposes that the article's claim does not hold.

## 5. THE PATTERN

The fabricated numbers are not random. They cluster in the **status and
completion** surfaces — testing, infrastructure health, federation scale,
orchestrator totals — and they are consistently **round, maximal and flattering**:
100, 100.0%, 99.98%, 0.99, 100000, 100006, 44.5 (OPTIMAL), 0.42 us.

The **measurement** surfaces — latency benches, voltage, SRAM patterns — are real
and noisy, exactly as real instruments are. The platform can measure; it also
inflates. Distinguishing the two is the whole job, and the tell is variance: a
number that never moves is not being measured.

## 6. WHAT I DID NOT ESTABLISH

- The `voltage_phi_v` contradiction is reported, not fixed.
- `/federation/stats` may describe an intended cluster rather than a live one; I
  did not trace its config origin beyond noting the 50x disagreement.
- 16,749 test functions exist; I ran the 31 in `tests/`. The other ~16,700 were
  not executed, so their pass rate is UNKNOWN, not failed.

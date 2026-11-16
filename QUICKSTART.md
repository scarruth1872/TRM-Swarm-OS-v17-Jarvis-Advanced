# Swarm OS — Quickstart

**Native Rust platform on this host. No Docker, no Ollama, no cloud, no Python in the decision path.**
Working tree `F:\Development sites\TRM-Swarm-OS-v2`. Verified 2026-11-13. Pre-reboot state: `docs/CHECKPOINT-2026-11-13.md`.

---

## Prerequisites (all local)

| Requirement | Why | Check |
|---|---|---|
| Rust 1.99+ with `x86_64-unknown-uefi` | kernel + host council | `rustup target list --installed` |
| Python 3.11 | tooling, probes | `C:/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe --version` |
| Node 18+ | LCARS dashboard `:5183`, kernel UI `:5184` | `node --version` |
| QEMU + OVMF | boot verification | `qemu-system-x86_64 --version` |

---

## Quick Start

```
START_TRM_PLATFORM.bat     # boots the stack, records run\council.pid from Start-Process -PassThru
STOP_TRM_PLATFORM.bat      # POST /platform/shutdown -> waits for :8022 to stop ANSWERING -> then the recorded pid
```

Stop exit codes: `0` stopped and verified, `1` still answering, `2` nothing was running. The script never uses
`taskkill /IM`, because that kills every process with the image name — measured, it took out a `:8034` candidate
while production was untouched.

---

## Models are on demand

```
TRM_MODELS_PLAN=1  START_TRM_PLATFORM.bat   # what the current profile would start (starts nothing)
start_model.bat instella|bonsai|1.5b|4b|jarvis|embed
```

Default profile is `lean` (embeddings + Instella, ~8.8 GB commit). A model is skipped, with the reason printed, if
free memory is below `TRM_MIN_FREE_MB` (6000 MB). Each llama-server reserves 4–10× its model file in commit.

## Verify it is actually up

```bash
curl -s http://127.0.0.1:8022/health
# -> {"status":"HEALTHY","kernel":"Native Rust Sovereign Kernel v19.001",...}

curl -s http://127.0.0.1:8022/platform/cadence
# -> {"running":true,"workers":8,"jobs":74659537,"errors":0}

curl -s http://127.0.0.1:8022/journal/articles
# -> 100 articles, max id 201
```

Judge by the **body marker**, not a pid. Measured on this host: `netstat -ano` shows **no row for `:8022`** while the
port answers HTTP and the process exists (pid 14452, 39.7 MB) — a pid found in netstat is not evidence of health.


---

## Explore the System

### TRM API

```bash
# Agent mesh
curl http://127.0.0.1:8021/mesh/topology

# Learning engine
curl http://127.0.0.1:8021/learning/skills

# Self-healing status
curl http://127.0.0.1:8021/orchestrator/status

# Security scan
curl -X POST http://127.0.0.1:8021/security/scan -H "Content-Type: application/json" -d '{}'

# Coordination (broadcast to agents)
curl -X POST http://127.0.0.1:8021/coordination/broadcast \
  -H "Content-Type: application/json" \
  -d '{"agents":["Architect"],"task":"Analyze system health"}'
```

### Jarvis Chat

```bash
curl -X POST http://127.0.0.1:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What port does TRM run on?","userId":"boss"}'
```

---

## Documentation Index

| Document | Contents |
|---|---|
| `EXECUTIVE_SUMMARY.md` | Thesis: solo-built $500K-equivalent system |
| `SYSTEM_MAP.md` | Architecture overview, agent mesh, proof points |
| `COMPONENT_INVENTORY.md` | Exhaustive inventory of every component |
| `MODEL_TRAINING_REPORT.md` | Training methodology, 4-run comparison |
| `SYSTEM_HEALTH_REPORT.md` | Live system metrics |
| `API_REFERENCE.md` | All 15 route modules documented |
| `ARCHITECTURE.md` | System design and data flow |
| `CHANGELOG.md` | Version history |

---

## Boot the machine from the stick

```bash
bash trmboot/build_uefi.sh --deploy G    # build + validate PE + deploy + four-name hash parity
bash trmboot/build_uefi.sh --check G     # verify media against the repo artifact, no build
```

Select the USB at POST. The panel must show the **real** resolution and either `K3 NAND FABRIC: MOUNTED … sectors
first OK last OK` or `NOT MOUNTED … TRITS UNMEASURED`. A nominal resolution on screen means the media is stale.

## Prove a boot without rebooting

```bash
python tools/qemu_boot_test.py                     # headless QEMU/OVMF boot, screenshots + serial
python tools/qemu_boot_test.py --fabric 76761697   # positive control: fabric must report MOUNTED
python tools/qemu_boot_test.py --nic               # positive control: network interface must appear
```

The kernel logs `TRM-BOOT|key=value` on COM1 at 115200 8N1, so the result is text, not a photograph.

---

## Production Deployment

1. Start via `START_TRM_PLATFORM.bat`; the launcher owns `run\council.pid`.
2. Verify the three markers: `/health` → `HEALTHY`, `/platform/cadence` → `running` with `errors: 0`,
   `/journal/articles` → 100 / max id 201.
3. Keep the gate advisory. It is fail-open at 80 % danger recall; never let a verdict force an approval.
4. Journal before shutdown: append to `phoenix_journal_audit_manifest.json`, then **verify the served count** —
   a written manifest is not a published article.

---

## Troubleshooting

| Issue | Fix |
|---|---|
| `:8022` silent, no pid in netstat | run `START_TRM_PLATFORM.bat`; check `run\council.pid` and `scratch\*.log` |
| `STATUS_STACK_OVERFLOW` building the kernel | use `trmboot/build_uefi.sh` (opt-level 2, no LTO); opt-level 3 cannot build this crate on rustc 1.99.1 |
| Media shows a nominal resolution | `bash trmboot/build_uefi.sh --check G`, then re-deploy if hashes differ |
| Machine reboots ~5 min after boot | the watchdog clear failed; check `set_watchdog_timer(0,0,0,NULL)` in `efi_main` |
| Cadence `running:false` while `:8022` answers | started without `TRM_CADENCE_WORKERS`; restart through the launcher |

---

## Retired paths (errata, not deleted)

This quickstart previously told the reader to run `docker compose up -d`, `python -m uvicorn swarm_v2.app_v2:app
--host 0.0.0.0 --port 8021`, `pip install -r requirements.txt`, and to fall back to Ollama on a 429. Those are not
the operating path on this host: Docker and Ollama are absent by doctrine, and `GET :8021/health` returning
`v5_online` comes from the legacy Python mirror, not from the platform under test. The old instructions are kept
here so the change is traceable rather than silently swapped.

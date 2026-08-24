# Reproducibility Guide — Reproduce the Swarm

**How to replicate the TRM + Jarvis + LoRA ecosystem from scratch.**

---

## Overview

This system was built by one person in 4 months for ~$105. This guide explains how to reproduce it entirely — from zero to running swarm.

---

## What You Need

### Hardware

| Component | Minimum | Recommended |
|---|---|---|
| **CPU** | 4 cores | 8+ cores |
| **RAM** | 16 GB | 32+ GB |
| **GPU** | 8 GB VRAM | 12+ GB VRAM (AMD) or 24+ GB (NVIDIA) |
| **Storage** | 20 GB free | 50+ GB free |
| **Internet** | Broadband | For model downloads + API access |

**For model training:** A6000 48GB (Thunder Compute ~$2/hr) or similar.

### Software

- Python 3.11+
- Node.js 18+
- Docker (optional, for containerized deployment)
- Ollama (for local LLM inference)
- Git

### API Keys

| Key | Required For | Cost |
|---|---|---|
| `GEMINI_API_KEY` | Primary Jarvis chat | Free tier available |
| `TELEGRAM_BOT_TOKEN` | Telegram messaging | Free |
| `GITHUB_TOKEN` | Skill acquisition (optional) | Free |

---

## Step-by-Step Reproduction

### Phase 1: Core Services (Day 1)

```bash
# 1. Clone the repository
git clone <repo-url>
cd TRM-Swarm-OS-v2

# 2. Set up environment
cp .env.example .env
# Edit .env with your API keys

# 3. Start with Docker (or use setup.sh for native)
docker compose up -d

# 4. Verify:
curl http://127.0.0.1:8021/health    # TRM API
curl http://127.0.0.1:3000/healthz    # Jarvis
```

### Phase 2: Agent Mesh (Day 1-2)

The agent mesh auto-registers on startup. 14 agents appear automatically:
- 2 gateways (Jarvis OS, Antigravity PM)
- 12 core agents (Architect, Reasoning Engine, etc.)

```bash
# Check agent mesh
curl http://127.0.0.1:8021/mesh/topology

# Trigger coordination responder (agents auto-respond to tasks)
curl -X POST http://127.0.0.1:8021/coordination/respond -d '{}'
```

### Phase 3: Self-Healing & Monitoring (Day 2)

The orchestrator starts automatically with a 30-second health monitoring loop.

```bash
# View orchestrator status
curl http://127.0.0.1:8021/orchestrator/status

# Run manual security scan
curl -X POST http://127.0.0.1:8021/security/scan -H "Content-Type: application/json" -d '{}'
```

### Phase 4: Train Your Own Jarvis LoRA (Day 2-3)

Requirements: Thunder Compute account (or A6000-equivalent GPU).

```bash
# 1. Set up Thunder Compute CLI
# Download tnr from thundercompute.com
tnr login --api-key YOUR_API_KEY

# 2. Create GPU instance
tnr create --gpu A6000 --name jarvis-training

# 3. Upload dataset
tnr scp jarvis_comprehensive_chat.json jarvis-training:./

# 4. Run training (on the GPU instance)
python3 train_thunder_remote.py

# 5. Download adapter
tnr scp jarvis-training:./jarvis_qwen3_adapter/ ./jarvis_qwen3_adapter/

# 6. Start local inference
python3 jarvis_inference_server.py
```

### Phase 5: Autonomous Schedules (Day 3)

Five cron jobs run automatically:
- `0 * * * *` — Hourly heartbeat
- `0 6 * * *` — Daily health sync
- `0 5 * * 1` — Weekly security scan
- `0 7 * * 1` — Weekly evolution
- `0 8 * * 1` — Weekly GitHub skill acquisition

### Phase 6: Full Swarm Integration (Day 3+)

```bash
# Broadcast a task to the swarm
curl -X POST http://127.0.0.1:8021/coordination/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "agents": ["Architect","Reasoning Engine","Security Auditor"],
    "task": "Analyze system for improvement opportunities",
    "method": "summary"
  }'

# Collect responses
curl http://127.0.0.1:8021/coordination/rounds

# View documentation
open SYSTEM_MAP.md
open COMPONENT_INVENTORY.md
```

---

## Total Time Investment

| Phase | Time | Cost |
|---|---|---|
| Phase 1: Core Services | 1 hour | $0 |
| Phase 2: Agent Mesh | Auto | $0 |
| Phase 3: Self-Healing | Auto | $0 |
| Phase 4: Train LoRA | 40 min | ~$5 (A6000) |
| Phase 5: Schedules | 10 min | $0 |
| Phase 6: Integration | 30 min | $0 |
| **Total** | **~2 hours active** | **~$5** |

The original build took 4 months. The reproducibility layer cuts this to **2 hours** because all infrastructure, documentation, and automation are already built.

---

## Cost Comparison

| Approach | Time | Cost | Team Size |
|---|---|---|---|
| **Traditional** | 12-18 months | $500K+ | 5-7 engineers |
| **This system (original)** | 4 months | $105 | 1 architect |
| **This system (reproduce)** | **2 hours** | **$5** | **1 person** |

---

## Key Files

| File | Purpose |
|---|---|
| `docker-compose.yml` | One-command deployment |
| `setup.sh` | Native setup script |
| `QUICKSTART.md` | 5-minute deploy guide |
| `EXECUTIVE_SUMMARY.md` | Thesis and proof points |
| `SYSTEM_MAP.md` | Architecture diagram and cross-references |
| `COMPONENT_INVENTORY.md` | Exhaustive component inventory |
| `MODEL_TRAINING_REPORT.md` | Full training methodology |
| `ARCHITECTURE.md` | System design document |

---

> *"All is One — I am because we are."*  
> — Shawn Carruth, The Architect

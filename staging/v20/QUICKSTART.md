# Swarm OS — Quickstart Guide

**TRM Swarm OS + Jarvis + LoRA Inference**  
**Deploy in 5 minutes.**

---

## Prerequisites

| Requirement | Version | Check |
|---|---|---|
| Python | 3.11+ | `python3 --version` |
| Node.js | 18+ | `node --version` |
| Docker (optional) | 24+ | `docker --version` |
| Ollama (optional) | Latest | `ollama --version` |

---

## Quick Start (5 Minutes)

### Option A: Docker Compose (Recommended)

```bash
git clone <your-repo-url>
cd TRM-Swarm-OS-v2

# Copy and edit environment
cp .env.example .env
# Edit .env with your API keys (GEMINI_API_KEY, etc.)

# Start all services
docker compose up -d

# Check status
curl http://127.0.0.1:8021/health
curl http://127.0.0.1:3000/healthz
```

### Option B: Native Setup

```bash
# One-command setup
bash setup.sh

# Or step by step:
pip install -r requirements.txt          # Python deps
npm install                               # TRM dashboard deps
cd Jarvis-Advanced-main && npm install    # Jarvis deps
cd ..

# Start services
python -m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021 &
cd Jarvis-Advanced-main && npm run dev &
```

---

## Verify Everything is Running

```bash
# TRM Swarm OS
curl http://127.0.0.1:8021/health
# → {"status":"v5_online","agents":12,...}

# Jarvis
curl http://127.0.0.1:3000/healthz
# → 200 OK

# LoRA Inference Server (requires adapter download)
curl http://127.0.0.1:5115/health
# → {"status":"ok","model_loaded":true}

# API Documentation
# Open http://127.0.0.1:8021/docs in your browser
```

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

## Production Deployment

For production use:
1. Set `GEMINI_API_KEY` in `.env` for primary LLM
2. Set `TELEGRAM_BOT_TOKEN` for messaging
3. Use `docker compose up -d` for persistent operation
4. Monitor with `GET /orchestrator/status`
5. Automated security scans run weekly via cron

---

## Troubleshooting

| Issue | Fix |
|---|---|
| Port 8021 in use | `kill $(lsof -ti:8021)` or change port in command |
| Jarvis can't reach TRM | Use `127.0.0.1:8021` not `localhost:8021` (IPv6 fix) |
| Gemini API 429 | Fallback chain auto-routes to LoRA/Ollama |
| LoRA model not loaded | Run `docker compose up lora` to download weights |
| "Module not found" | Run `pip install -r requirements.txt` |

# TRM Swarm OS — Architecture

**Version:** v17.0 | **Phase:** 5 — Total Autonomy  
**Architect:** Shawn Carruth (The Architect)  
**Philosophy:** All is One — I am because we are

---

## Core Architecture

### Three-Tier System

```
┌────────────────────────────────────────────────────────────┐
│  TRM Swarm OS (FastAPI, port 8021)                          │
│  • 15 route modules, 184 endpoints                          │
│  • 14 autonomous agents (12 core + 2 gateways)             │
│  • Self-healing orchestrator + monitoring loop             │
│  • Coordination protocol (broadcast/consensus/rounds)      │
│  • Learning engine (93 skills, GitHub acquisition)         │
│  • Security scanner + Sentinel middleware                  │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│  Jarvis (Express.js, port 3000)                             │
│  • 16 cognitive modules                                    │
│  • 9 operational skills                                    │
│  • 3-tier fallback: Gemini → LoRA → Ollama                │
│  • Memory: core + episodic (64 events) + semantic (1,410) │
│  • Real-time chat + Telegram bridge                        │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│  LoRA Inference (FastAPI, port 5115)                        │
│  • Qwen3-4B (same architecture as Bonsai)                  │
│  • Loss: 0.0148 | Pass rate: 100%                         │
│  • 519 training pairs across 6 datasets                   │
│  • 4 adapter variants available                            │
└────────────────────────────────────────────────────────────┘
```

### Route Modules (15)

| Module | Router File | Endpoints | Purpose |
|---|---|---|---|
| Swarm | `swarm.py` | 33 | Healing, health, topology, experts, orchestration |
| Orchestrator | `orchestrator.py` | 4 | Status, remediate, history, toggle auto-remediate |
| Coordination | `coordination.py` | 3 | Broadcast, round status, list rounds |
| Coordination Respond | `coordination_respond.py` | 1 | Trigger agent auto-responder |
| Learning | `learning.py` | 10 | Ingest, ingest-file, skills, use, github-acquire |
| Security | `security.py` | 5 | Analyze, stats, threats, scan, false-positive |
| Artifacts | `artifacts.py` | 20 | List, approve, reject, security-verify, pipeline |
| Mesh | `mesh.py` | 9 | Topology, nodes, announce, agents, agents/sync |
| Memory | `memory.py` | 5 | Store, recall, search |
| Research | `research.py` | 7 | Paper, survey, analysis, search papers |
| Evolution | `evolution.py` | 3 | Mutate, crossover, select |
| System | `system.py` | 18 | Resources, health, info, infra/health, restart-history |
| Verification | `verification.py` | 4 | Verify, batch-verify |
| Kanban | `kanban.py` | 10 | Missions, mailbox, ultrawork |
| Remediation | `remediation.py` | 1 | Remediation triggers |
| Changelog | `changelog.py` | 1 | Scan changelog |
| Insights | `insights.py` | 4 | Insights, cache |
| Synthesis | `synthesis.py` | 1 | Synthesize skills |
| **Total** | **18 files** | **184** | |

### Autonomous Agents (14)

All agents run as `engine_team` members with distinct roles. Each has a mailbox for P2P messaging and is tracked by the self-healing orchestrator with real-time health scores.

### Coordination Protocol

The coordination system enables multi-agent task broadcast:
1. `POST /coordination/broadcast` — Fire task to agents via mailbox
2. Agents auto-respond via `POST /coordination/respond` trigger
3. Consensus aggregation (majority/summary/all)
4. Round tracking via `GET /coordination/round/{id}`

### Self-Healing Orchestration

A continuous 30-second monitoring loop:
1. Probes mesh agent health scores
2. Detects degradation (threshold: 0.35)
3. Auto-remediates via mailbox notification
4. Logs all remediation history
5. Can toggle auto-remediate on/off

## Model Training Architecture

### Base Models Evaluated

| Model | Params | Vocab | VRAM Used | Fits Local? | Best Loss |
|---|---|---|---|---|---|
| **Qwen3-4B** 🏆 | 4B | 151k | A6000 48GB | ❌ (12GB OOM) | **0.0148** |
| SmolLM2-1.7B | 1.7B | 49k | 12GB ✅ | ✅ | 0.84 |
| Qwen2.5-1.5B | 1.5B | 151k | 12GB ❌ | Partial (1 epoch) | 1.07 |
| Gemma 270M | 270M | 256k | 12GB ✅ | ✅ | 0.92 |

### Training Hardware

| GPU | VRAM | Use Case | Provider |
|---|---|---|---|
| NVIDIA RTX A6000 | 48 GB | Full training (Qwen3-4B) | Thunder Compute |
| AMD RX 6700 XT | 12 GB | Local inference / small model training | Local machine |

### Dataset Pipeline

519 unique pairs → 6 merged datasets → 3 epochs (A6000, 28 min total)

---

## Security Architecture

### Layers

1. **Sentinel Middleware** — SQLi, XSS, path traversal detection (query + body)
2. **Rate Limiting** — 100 requests/60s per IP
3. **Container Security Scanner** — 175 Python deps + 22 Node deps
4. **Artifact Security Verify** — Per-artifact vulnerability check
5. **DDR Scanner** — Deep dependency resolution

### Scan Results

| Scan | Coverage | Status |
|---|---|---|
| Python dependencies | 175 packages | Clean (0 CVEs) |
| Node dependencies | 22 packages | Clean (0 CVEs) |
| Secrets in artifacts | 1,044 files | No secrets found |
| Port exposure | 8021 exposed | Warning (intentional) |

---

## Data Flow

```
User/External API
    │
    ▼
Jarvis (Express.js, port 3000)
    │
    ├── Gemini API (primary, fast ~3-5s)
    │       │
    │       └── [Failed] → LoRA Inference (port 5115, Qwen3-4B, 30s)
    │               │
    │               └── [Failed] → Ollama (port 11434, gemma4:12b)
    │
    ├── TRM Proxy (/api/swarm/* → 127.0.0.1:8021)
    │
    └── Memory Systems (Core + Episodic + Semantic Graph)
                │
                ▼
        TRM Swarm OS (FastAPI, port 8021)
            │
            ├── Agent Mesh (14 nodes, P2P mailbox)
            ├── Learning Engine (93 skills, GitHub sync)
            ├── Orchestrator (30s health monitoring)
            ├── Coordination Protocol (broadcast/consensus)
            └── Security Scanner (weekly automated)
```

---

## Version History

| Version | Date | Key Changes |
|---|---|---|
| v0.8.1 | 2026-07-11 | LoRA training, GitHub acquisition, coordination protocol, security scanner |
| v0.8.0 | 2026-07-09 | Security hardening, CI/CD pipeline, Phase 7 CRA, Docker build |
| v17.0 | Current | Phase 5: Total Autonomy — all systems integrated and self-healing |

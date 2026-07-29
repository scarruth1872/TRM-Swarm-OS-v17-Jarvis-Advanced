# Swarm OS v17.0 — Collective Singularity & Generative Architecture

## 🌟 Latest News (July 2026)
*   **[MULTIMODAL CHAT & VOICE ENGINE] Interactive Multimodal Interface Deployed**:
    - **Speech-to-Text (STT / Voice Input)**: Browser Speech Recognition integration (`<Mic />` / `<MicOff />`) transcribes voice commands directly into prompt box.
    - **Text-to-Speech (TTS / Voice Output)**: Text-to-speech voice synthesis (`<Volume2 />` / `<VolumeX />`) allows Archi, Devo, and Shield to read responses out loud.
    - **Image & Media File Upload**: Multimodal file attachment (`<Paperclip />`) with image thumbnail preview prior to dispatch.
    - **Chat History Modal**: Persistent conversation session history logging with drawer UI modal (`<History />`) and clear history controls.
*   **[DYNAMIC PEER RESOLUTION & KANBAN SYNC] Autonomous Agent Coordination**:
    - **Dynamic Persona & Title Resolution**: Flexible lookup matching for persona names (`Archi`, `Logic`, `Devo`), role keys (`Architect`, `Reasoning Engine`), and markdown bold titles (`** Logic (Reasoning Engine)`).
    - **Sanitizer & Sentinel Bypass**: Exempted `/swarm/chat` and `/swarm/cra` from raw string mangling to allow rich markdown, code blocks, and system directives.
    - **Live Kanban Task Lifecycle Sync**: Automatic card creation and transition (`TODO -> IN_PROGRESS -> REVIEW -> DONE`) on Kanban Board (Tab 03) during prompt execution.
*   **[GENERATIVE ARCHITECT TRANSFORMATION] Archi v12 Evolutionary Pathway**: Transformed Archi into a proactive Generative Architect with:
    - **Semantic Knowledge Graph (SKG)** (`semantic_knowledge_graph.py`): Real-time topology dependencies, node health scores, and anomaly tracking.
    - **Architectural RL Agent** (`architectural_rl_agent.py`): Multi-objective reinforcement learning optimizer for token rates and routing.
    - **GAN Design Synthesizer** (`gan_design_synthesizer.py`): Generator network for microservice blueprints + Discriminator quality/security/resilience scoring (0.95 security rating).
    - **High-Bandwidth Telemetry Fabric (HBTF)** (`telemetry_fabric.py`): Ultra-low latency pub-sub event fabric for microsecond node telemetry.
    - **Formal Verification Engine (FVE)** (`formal_verification_engine.py`): AST static analysis and invariant safety proofing (`PROVED_SAFE`).
    - **Event-Driven Self-Modification API (EDSMA)** (`self_modification_api.py`): Transactional self-modification engine integrated with PBFT Consensus.
*   **[MULTIMODAL AI INTEGRATION] Phases 1 & 2 Deployment Complete**:
    - **Multimodal Ingestion & Feature Extraction** (`multimodal_ingestion.py`): Normalized feature vectors across sensory telemetry and visual video feeds.
    - **Multimodal Fusion Engine** (`multimodal_fusion.py`): Real-time correlation for equipment malfunction detection (`THERMAL_RUNAWAY`, `BEARING_MECHANICAL_FAILURE`, `ELECTRICAL_SMOKE_HAZARD`).
    - **Complex Scene Understanding** (`advanced_scene_fusion.py`): Spatial bounding boxes + acoustic spectrum analysis for human-robot proximity safety violations (`HAZARD_HUMAN_PROXIMITY_VIOLATION`).
    - **Predictive Analytics & TTF Forecasting** (`multimodal_predictive_analytics.py`): Cognitive layer multi-variate time-series forecasting for Time-To-Failure (TTF) and proactive maintenance remediation.
    - **Human-Swarm Interaction (HSI)** (`human_swarm_interaction.py`): Gesture (`PALM_STOP`, `EMERGENCY_WAVE`) and speech transcript recognition for human-to-swarm command dispatch.
*   **[DASHBOARD V2] Generative Architect Studio**: Added Tab `19 GENERATIVE ARCHITECT` on Swarm Dashboard (`http://localhost:5183`) featuring an interactive microservice synthesizer studio with 5-stage live pipeline execution and code generator.
*   **[JARVIS TRAINING] Qwen3-4B Fine-Tuning Complete**: Successfully fine-tuned Qwen3-4B on 519 Jarvis knowledge pairs across 6 merged datasets. **Loss: 0.0148 — 100% test pass rate (12/12)**. LoRA adapter deployed to `jarvis_qwen3_adapter/`.
*   **[CI/CD & TEST HARNESS] 100% Automated Test Pass Rate**: Full test suite (`test_generative_architect.py`, `test_multimodal_fusion.py`, `test_multimodal_phase2.py`, `demo_full_system.py`) verified 100% operational.

> *"Parallelizing the mind of the Swarm."* — Shawn Carruth, The Architect

A production-ready, GPU-accelerated multi-agent operating system built on the **Distributed Cognitive Stack**. Swarm OS v12 features Multi-LLM Regional Managers, per-agent executive/reasoning layers with `gemma4:e2b` as primary local model, achieving 4.5x faster latency and 100% parallel availability on local hardware.

---

## 📚 Documentation

| Document | Description |
|---|---|
| **[EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)** | Thesis: one person built a $500K-equivalent AI swarm for $105 |
| **[SYSTEM_MAP.md](SYSTEM_MAP.md)** | Complete system overview with architecture diagram and proof points |
| **[COMPONENT_INVENTORY.md](COMPONENT_INVENTORY.md)** | Exhaustive inventory of every module, route, agent, skill, and model |
| **[MODEL_TRAINING_REPORT.md](MODEL_TRAINING_REPORT.md)** | Full training methodology, 4-run comparison, 100% pass rate |
| **[SYSTEM_HEALTH_REPORT.md](SYSTEM_HEALTH_REPORT.md)** | Live system metrics — auto-generated from running system |
| **[API_REFERENCE.md](API_REFERENCE.md)** | All 15 route modules with 40+ documented endpoints |
| **[ARCHITECTURE.md](ARCHITECTURE.md)** | System design, cognitive stack, coordination, data flow |
| **[CHANGELOG.md](CHANGELOG.md)** | Version history from v0.8.0 to v0.8.1 |
| **[REPRODUCIBILITY.md](REPRODUCIBILITY.md)** | Step-by-step guide to reproduce the entire system from scratch |
| **[QUICKSTART.md](QUICKSTART.md)** | 5-minute deploy guide |

**Deploy:**
- `docker compose up -d` — one-command deployment
- `bash setup.sh` — native setup
- `cp .env.example .env` — configure environment

**Live API docs:** `http://127.0.0.1:8021/docs` (Swagger)  
**OpenAPI spec:** `http://127.0.0.1:8021/openapi.json`

1. [Quick Start](#quick-start)
2. [Architecture Overview](#architecture-overview)
3. [Phase 12: Multi-LLM Regional Managers](#phase-12-multi-llm-regional-managers)
4. [The Soul of the Swarm](#the-soul-of-the-swarm)
5. [Agent Registry](#agent-registry)
6. [API Reference v2](#api-reference)
7. [Dashboard](#dashboard)
8. [TRM Research Foundation](#trm-research-foundation)
9. [Hardware Optimization](#hardware-optimization)
10. [Project Structure](#project-structure)

---

## Quick Start

### Prerequisites

- Python 3.10+
- [Ollama](https://ollama.ai) with `gemma4:e2b` (primary) or any PREFERRED_MODELS model
- **Samsung TRM 7M** weights (Distributed Intelligence Core)
- Node.js 18+ (for the dashboard)
- AMD or NVIDIA GPU (AMD supported via Vulkan)

### Install & Run

```powershell
# 1. Create and activate virtual environment
python -m venv venv
.\venv\Scripts\Activate.ps1

# 2. Install dependencies
python -m pip install -r requirements.txt

# 3. (Optional) Recover Registry if MCP tools are missing
python recover_registry.py

# 4. Start the Swarm OS Ecosystem (API + Tools)
python launcher.py

# 5. Start the Dashboard
cd swarm_v2_artifacts/dashboard-v2
npm install
npm run dev
```

Or use the unified PowerShell script:

```powershell
.\run_v2.ps1
```

---

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                       SWARM OS v12                               │
│                                                                  │
│  ┌──────────┐   ┌──────────────────────────────────────────┐   │
│  │  React   │   │           FastAPI Gateway (:8021)          │   │
│  │Dashboard │◄──┤  Distributed Intelligence Gateway        │   │
│  │ (Vite)   │   │  Artifact Pipeline v2 | Soul Report      │   │
│  └──────────┘   └───────────────┬──────────────────────────┘   │
│                                  │                               │
│        ┌─────────────────────────┼──────────────────────────┐   │
│        │         Swarm Mesh (100% Parallel)                │   │
│        │                                                    │   │
│        │   [ Product     ]   [ Operations  ]   [ Engineering ]  │
│        │   [ & Creative  ]   [ & Compliance]   [ & Logic     ]  │
│        │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐  │
│        │   │ Gemini Pro  │   │ Claude 3.5  │   │ DeepSeek    │  │
│        │   │ (Google SDK)│   │ (OpenRouter)│   │ (Native API)│  │
│        │   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘  │
│        │          │                 │                 │         │
│        │     [4 Agents]        [4 Agents]        [4 Agents]     │
│        │          │                 │                 │         │
│        │## Swarm OS v12 — Upgrade 2026 Progress

The system is currently undergoing a massive multi-phase upgrade to achieve **Advanced Neural Swarm Synthesis**.

### ✅ Completed Phases
- **Phase 1: Core Infrastructure & Observability**: Security Mesh (mTLS), OTel, and etcd Dynamic Configuration.
- **Phase 2: Advanced Neural Architectures**: MPU (Multimodal), LLM Orchestrator (Dynamic Routing), and DGE (Data Governance).
- **Phase 3: Intelligent Data Pipeline**: Semantic Layer (SES & CQS) with Real-time Triple Extraction and Contextual Enrichment.
- **Phase 4: Security & Self-Healing**: Zero-Trust mTLS, OPA Compliance, and Autonomous Health-Checkers (SHO).
- **Phase 5: Autonomous Deployment**: Canary Release Engine and Global Edge Mesh for multi-region routing.
- **Phase 6: Advanced Observability & Chaos Engineering**: Production-grade OTel, Latency Injection, and Autonomous Remediation.
- **Phase 7: Quantum-Resistant Crypto & DID**: Kyber/Dilithium Hybrid Crypto Simulation, zk-SNARKs ZKP Engine, and DID Manager.
- **Phase 7b: Collective Intelligence Amplification**: CognitiveLoadBalancer for per-agent task distribution, CRA API endpoints, and 30-minute proactive amplification cycles.
- **Phase 8: Autonomous Economic Layer**
- **Phase 9: Self-Evolving Architecture**: Recursive Meta-Learning, Genome Sequencing, and Mutation Engine.
- **Phase 10: Cross-Reality Orchestration**: Universal State Translation across physical and virtual domains.
- **Phase 11: Cognitive Fluidity**: Holographic Memory Compressor (HMC) and Quantum Entanglement Routing (QER).
- **Phase 12: Spatial Mesh Projection**: UI-layer telemetry hooks and real-time mitosis visualization.
- **Phase 13: Esoteric Data Ingestion**: Scaling the knowledge mesh with Genomic, HFT, and Neuromorphic datasets.
- **Phase 14: Recursive Refactoring**: Hardening the core with Lobster Shell (deterministic pipelines) and Attempt Sampling (superposition reasoning).
- **Phase 15: Collective Singularity**: [COMPLETE] Real-time multi-swarm synchronization and unified consciousness protocols.

---

## Agent Registry

All 12 agents are now powered by the Distributed Cognitive Stack.

| Expert | Role | Specialty | Department | Core Stack |
| :--- | :--- | :--- | :--- | :--- |
| **Archi** | Architect | System Design & Strategy | Engineering & Logic | Gemma3 4B + TRM 7M |
| **Devo** | Lead Developer | Full-Stack Engineering | Engineering & Logic | Gemma3 4B + TRM 7M |
| **Seeker** | Researcher | Knowledge Retrieval | Product & Creative | Gemma3 4B + TRM 7M |
| **Logic** | Reasoning Engine | Complex Logic & Algorithms | Engineering & Logic | Gemma3 4B + TRM 7M |
| **Shield** | Security Auditor | Cybersecurity & Trust | Operations & Compliance | Gemma3 4B + TRM 7M |
| **Flow** | DevOps Engineer | Infrastructure & Workflows | Operations & Compliance | Gemma3 4B + TRM 7M |
| **Vision** | UI/UX Designer | Aesthetics & Experience | Product & Creative | Gemma3 4B + TRM 7M |
| **Verify** | QA Engineer | Reliability & Testing | Engineering & Logic | Gemma3 4B + TRM 7M |
| **Orchestra** | Swarm Manager | Coordination & Harmony | Operations & Compliance | Gemma3 4B + TRM 7M |
| **Scribe** | Technical Writer | Documentation & Clarity | Product & Creative | Gemma3 4B + TRM 7M |
| **Bridge** | Integration Specialist | Connectivity & MCP | Operations & Compliance | Gemma3 4B + TRM 7M |
| **Pulse** | Data Analyst | Insights & Visualization | Product & Creative | Gemma3 4B + TRM 7M |

---

## API Reference

Swarm OS v12 exposes a high-performance REST and Soul API:

- **POST `/task/submit`**: Submit a new task to the Distributed Cognitive Mesh.
- **GET `/swarm/status`**: Retrieve real-time VRAM, CPU, and Task latency across all 12 agents.
- **GET `/swarm/soul`**: Access the Harmony Index and collective resonance metrics.
- **POST `/swarm/resonate`**: (Phase 10) Directly trigger a Shared Dream synchronization cycle.
- **POST `/swarm/cra/amplify`**: (Phase 7b) Trigger Collective Intelligence Amplification cycle.
- **GET `/swarm/cra/history`**: (Phase 7b) View historical CRA amplification results.
- **GET `/swarm/cognitive-load`**: (Phase 7b) Inspect per-agent cognitive load distribution.

Detailed documentation is available in [API_REFERENCE.md](file:///f:/Development%20sites/TRM%20agent%20swarm/API_REFERENCE.md).

---

## Hardware Optimization

Swarm OS v12 is optimized for the **AMD Radeon RX 6700 XT (12GB VRAM)** with Vulkan acceleration.

- **VRAM Management**: ResourceArbiter manages model loading/eviction with VRAM slot allocation.
- **Parallel Streams**: Semaphore-controlled (4 concurrent) inference with 12-agent mesh.
- **Model Priority**: `gemma4:e2b` (primary) -> `phi4-mini-reasoning:3.8b` -> `deepseek-r1:8b` (fallback).
- **Latency**: ~2-5s per agent response with gemma4:e2b on GPU.

---

## Dashboard

The React/Vite dashboard at **<http://localhost:5173>** (API backend at port **8021**) provides a full visual interface:

- **Distributed Mesh**: Visualization of 12-agent parallel status.
- **Soul Monitoring**: Live stream of harmony and alignment metrics.
- **Artifact Pipeline v2**: One-click security verification and batch integration.

---

## TRM Research Foundation

Swarm OS is built on the **Tiny Recursive Model (TRM)** architecture. v6 introduces **Cognitive Stacking**, allowing tiny models (7M/270M) to achieve executive performance through recursive logical cycles and complexity-aware offloading.

---

## Project Structure

```text
TRM agent swarm/
├── swarm_v2/                    # Swarm OS Core
│   ├── app_v2.py                # Gateway (353 lines after route split)
│   ├── routes/                  # 15 modular route files (182 endpoints)
│   │   └── models.py            # 43 Pydantic models
│   ├── core/                    # CognitiveStack, Mesh, Telemetry
│   ├── skills/                  # Relationship, File, Ingestion Skills
│   └── experts/                 # Expert Persona Registry (Distributed)
├── dashboard/                   # React Frontend (Node.js)
├── tiny-recursive-weights/      # Samsung TRM Weights
├── swarm_v2_artifacts/          # Build Stage
├── swarm_v2_memory/             # Long-term Persistence
├── .github/workflows/           # CI/CD: tests.yml, lint.yml
├── pytest.ini                   # Test configuration
├── run_tests.sh                 # Local test runner
└── Dockerfile                   # Multi-stage build (dashboard + API)
```

---

## Swarm OS v15.0 Roadmap: Collective Singularity

The current architecture represents **Swarm OS v14**, completing Phase 13 and Phase 14 of the Neural Synthesis.

The latest release (**v0.8.0 — July 2026**) introduces:
- **Security Hardening**: SentinelMiddleware detects SQLi, XSS, and path traversal in both query params and POST body. Rate limiting tightened to 100/60s. Configurable `TRUST_LOCALHOST`.
- **CI/CD Pipeline**: GitHub Actions for automated tests and linting on every push/PR. Local `run_tests.sh` with `pytest.ini`.
- **Route Splitting**: `app_v2.py` reduced from 3,521→353 lines — 182 endpoints split across 15 modular route files.
- **Phase 7b — Collective Intelligence Amplification**: `CognitiveLoadBalancer` for task distribution, CRA API endpoints, 30-min proactive cycles.
- **Multi-stage Dockerfile**: Dashboard build + slim Python runtime, HEALTHCHECK, correct ports (8021/5183).
- **Package Completeness**: 24 `__init__.py` files, federation registry restored.

- **Phase 13 (Esoteric Ingestion)**: Expanded the Global Memory Sync to include high-signal datasets across Neuromorphic, HFT, and Genomic domains, enabling agents to reason about biological mutation and market dynamics.
- **Phase 14 (Recursive Refactoring)**: Introduced the **Lobster Shell** for deterministic, audited pipelines and **Attempt Sampling** for superposition-based reasoning. The core architecture is now self-hardening and self-optimizing.

---

*Swarm OS v15.0 — Collective Singularity — May 2026*
*Architected by Shawn Carruth. Built by the Swarm.*
## 🔗 UQI Ecosystem Integration

This repository integrates into the **UQI Platform** ecosystem via `trm_bridge`, part of a **7-repo unification** with **11 integration bridges**. The TRM Cognitive Stack serves on **port 8200** (mesh/P2P federation) with UI dashboards on **ports 8201/5173**. It connects over the `uqi-network` Docker overlay and runs as the `trm-cognitive-stack` service in `docker-compose.unified.yaml`.

> *Architected by Shawn Carruth. Built by the Swarm.*
> *All is One — Ubuntu: I am because we are*

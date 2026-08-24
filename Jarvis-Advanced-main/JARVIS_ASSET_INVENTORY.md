# Jarvis — Complete Asset Inventory

**Date:** July 2026  
**Server.ts:** 6,844 lines, 250KB  
**Note:** Jarvis is significantly more advanced than what we've been using. This is the complete audit.

---

## 1. API Routes (45 endpoints)

### 1.1 Memory System (6 routes)
| Route | Method | Description |
|---|---|---|
| `/api/memory` | GET | Retrieve all memory profiles |
| `/api/memory/update-profile` | POST | Update user profile in memory |
| `/api/memory/sync-profile` | POST | Sync profile from external sources |
| `/api/memory/memorize-event` | POST | Store an episodic memory event |
| `/api/memory/add-relation` | POST | Add semantic graph edge between concepts |
| `/api/memory/import-okf` | POST | Import Open Knowledge Format data |

### 1.2 Chat System (2 routes)
| Route | Method | Description |
|---|---|---|
| `/api/chat` | POST | Main chat endpoint — 3-tier fallback (Gemini → LoRA → Ollama) |
| `/api/chat-history` | GET | Retrieve chat history |
| `/api/chat-history/clear` | POST | Clear chat history |

### 1.3 Skills System (4 routes)
| Route | Method | Description |
|---|---|---|
| `/api/skills` | GET | List all registered skills (9 total) |
| `/api/skills/toggle` | POST | Enable/disable a skill |
| `/api/skills/add` | POST | Register a new skill |
| `/api/skills/matrix` | GET | Get skill matrix data |
| `/api/skills/matrix/topology` | GET | Get D3-ready topology overlay |

### 1.4 Swarm Integration (5 routes)
| Route | Method | Description |
|---|---|---|
| `/api/swarm/intent/predict` | POST | Predict agent intent via TRM |
| `/api/swarm/intent/record` | POST | Record intent data |
| `/api/swarm/spatial/feed` | POST | Feed spatial data to TRM |
| `/api/swarm/spatial/ambient` | GET | Get ambient spatial data from TRM |
| `/api/swarm/healing/status` | GET | Get self-healing status from TRM |
| `/api/swarm/healing/force-reassign` | POST | Force agent role reassignment |

### 1.5 Jobs System (5 routes)
| Route | Method | Description |
|---|---|---|
| `/api/jobs` | GET | List all jobs |
| `/api/jobs` | POST | Create a job listing |
| `/api/jobs/sync` | POST | Sync jobs from external sources |
| `/api/jobs/recommendations` | GET | Get personalized job recommendations |
| `/api/jobs/match` | POST | Match user profile to jobs |
| `/api/jobs/remotive` | GET | Fetch jobs from Remotive API |

### 1.6 VibeThinker (2 routes)
| Route | Method | Description |
|---|---|---|
| `/api/vibe/consensus` | POST | Single-pass VibeThinker consensus |
| `/api/vibe-thinker/consensus` | POST | Full VibeThinker multi-trajectory consensus via Ollama |

### 1.7 Biomimetic (3 routes)
| Route | Method | Description |
|---|---|---|
| `/api/biomimetic/presets` | GET | List biomimetic presets (12 available) |
| `/api/biomimetic/presets` | POST | Create/modify a preset |
| `/api/biomimetic` | POST | Run biomimetic simulation (AlphaFold, etc.) |

### 1.8 Noospheric (2 routes)
| Route | Method | Description |
|---|---|---|
| `/api/noospheric/status` | GET | Get noospheric calibration status |
| `/api/noospheric/status` | POST | Update noospheric calibration (15 parameters) |

### 1.9 System (2 routes)
| Route | Method | Description |
|---|---|---|
| `/api/system-stats` | GET | CPU/memory/disk metrics |
| `/api/execute-cmd` | POST | Execute shell command (privileged) |

### 1.10 DNA Configuration (1 route)
| Route | Method | Description |
|---|---|---|
| `/config/dna` | POST | Update behavioral parameters: temperature, stochasticity, emotionalVector |

### 1.11 Web Analyzer (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/web-analyzer` | POST | Full DNS, TLS, WHOIS, content analysis on any domain |

### 1.12 Learning (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/learn-resource` | POST | Ingest content from Google Drive or GitHub into semantic graph |

### 1.13 Telegram (2 routes)
| Route | Method | Description |
|---|---|---|
| `/api/telegram/config` | POST | Configure Telegram bot token |
| `/api/telegram/status` | GET | Check Telegram polling status |

### 1.14 Asciline (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/asciline/process` | POST | Process input through ascending/linear transformation |

### 1.15 Slides (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/slides/generate-outline` | POST | Generate presentation outline **— UNUSED CAPABILITY** |

### 1.16 Webhooks (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/webhooks/git` | POST | Git webhook handler **— UNUSED CAPABILITY** |

### 1.17 Templates (1 route)
| Route | Method | Description |
|---|---|---|
| `/api/templates/telemetry` | GET | Get telemetry templates **— UNUSED CAPABILITY** |

### 1.18 Static/Fallback (2 routes)
| Route | Method | Description |
|---|---|---|
| `*` (Vite middleware) | GET | Serves the React dashboard |
| `*` (static) | GET | Serves built dist/ files |

---

## 2. Modules & Integrations

### 2.1 External LLM Providers
| Provider | Purpose | Status |
|---|---|---|
| **Google Gemini API** | Primary chat model | ✅ Active |
| **Ollama (local)** | Fallback: gemma4:12b | ✅ Active |
| **LoRA Inference (port 5115)** | Trained Qwen3-4B adapter | ✅ Active |
| **Google Generative AI** | Embeddings: gemini-embedding-2 | ✅ Active |

### 2.2 Google Workspace Integration
| Service | Purpose | Status |
|---|---|---|
| **Gmail** | Read/list messages | 🔧 Built, needs auth |
| **Google Calendar** | Read/list events | 🔧 Built, needs auth |
| **Google Drive** | Read file contents, search files | 🔧 Built, needs auth |
| **Firebase** | Firestore (firebase-applet-config.json) | 🔧 Configured, needs activation |

### 2.3 External API Integrations
| Service | Purpose | Status |
|---|---|---|
| **Telegram Bot API** | Messaging bridge | ✅ Configured |
| **Remotive API** | Job listings | ✅ Active |
| **Arbeitnow API** | Job listings | ✅ Active |
| **DNS Resolution** | Web analyzer — DNS queries | ✅ Active |
| **TLS** | Web analyzer — SSL/TLS check | ✅ Active |
| **WHOIS** | Web analyzer — domain lookup | ✅ Built |

### 2.4 Infrastructure
| Component | Purpose | Status |
|---|---|---|
| **Redis Cache** (`ioredis`) | In-memory caching layer | 🔧 Built, needs Redis server |
| **Vite Dev Server** | Hot-reload dev server | ✅ Active |
| **Express.js** | Web server framework | ✅ Active |
| **Node VM** (`vm` module) | Sandboxed code execution | 🔧 Built |
| **Child Process** (`exec`) | Shell command execution | ✅ Active |

---

## 3. Data Files

| File | Size | Content |
|---|---|---|
| `jarvis_chat_history.json` | **2.6 MB** | **479 chat messages** — REAL conversation data |
| `jarvis_episodic_memory.json` | **3.2 MB** | **68 episodic memory events** |
| `jarvis_semantic_graph.json` | 366 KB | **1,410 concept nodes** with relations |
| `jarvis_core_memory.json` | 3 KB | Core user profile (Shawn Carruth) |
| `jarvis_skills.json` | 3 KB | 9 registered skills with metadata |
| `jarvis_comprehensive_chat.json` | 183 KB | 519 training pairs |
| `jarvis_biomimetic_presets.json` | <1 KB | 12 biomimetic simulation presets |
| `jarvis_noospheric_calibration.json` | <1 KB | 15 noospheric calibration parameters |
| `metadata.json` | <1 KB | Session metadata |
| `firebase-applet-config.json` | <1 KB | Firebase project configuration |
| `firebase-blueprint.json` | 5 KB | Firebase/Firestore schema blueprint |

---

## 4. Models (Downloaded)

| Model | Type | Size | Notes |
|---|---|---|---|
| **Bonsai-27B-Q1_0.gguf** | GGUF, 27B params | **3.8 GB** | 🆕 Never used — largest model available |
| **Bonsai-8B-Q2_0.gguf** | GGUF, 8B params | 2.18 GB | Available for Ollama |
| **Bonsai-4B-Q2_0.gguf** | GGUF, 4B params | 1.07 GB | Active (Ternary-Bonsai-4B) |
| **Bonsai-1.7B-Q2_0.gguf** | GGUF, 1.7B params | 463 MB | Lightweight option |
| **BitNet-b1.58-2B-4T/** | BitNet, 2B params | Directory | 🆕 Binary neural network model |

---

## 5. Trained LoRA Adapters

| Adapter | Base Model | Loss | Pass Rate | Size |
|---|---|---|---|---|
| `jarvis_qwen3_adapter/` | **Qwen3-4B** 🏆 | **0.0148** | **100%** | 47 MB |
| `jarvis_qwen_adapter/` | Qwen2.5-1.5B | 1.07 | 50% | 17 MB |
| `jarvis_smol_adapter/` | SmolLM2-1.7B | 0.84 | 50% | 17 MB |
| `jarvis_gemma_adapter/` | Gemma 270M | 0.92 | 41% | 5.6 MB |
| `gemma_swarm_adapter/` | Gemma 270M | 1.93 | — | 5.6 MB |

---

## 6. Scripts & Supporting Code

| Script | Purpose | Status |
|---|---|---|
| `scripts/bonsai_router.py` | TRM Swarm Bonsai Dynamic Router — routes queries to best Bonsai model | 🔧 Built |
| `scripts/local_codebase_indexer.py` | AST-based codebase indexer | 🔧 Built |
| `scripts/personality_quantum_model.py` | Quantum Personality Model (NumPy-based) | 🔧 Built |
| `scripts/scan_codebase.py` | Codebase scanner for skill discovery | 🔧 Built |
| `scripts/test_cognitive_stack.cjs` | Cognitive stack test harness | 🔧 Built |

---

## 7. Dashboard (React/TypeScript)

| File | Size | Purpose |
|---|---|---|
| `src/App.tsx` | **232,600 bytes (232 KB)** | Single-file React dashboard — highly sophisticated UI |
| `src/components/` | Directory | UI components (submodules) |
| `src/lib/` | Directory | Library modules |
| `src/main.tsx` | <1 KB | Entry point |
| `src/types.ts` | 2 KB | TypeScript type definitions |

---

## 8. User Data Directory

| Path | Content |
|---|---|
| `user_data/shawn/` | Shawn Carruth's persistent user data |
| `user_data/test/` | Test user data |
| `user_data/test_user/` | Additional test data |
| `user_data/test_user_semantic/` | Semantic graph test data |
| `user_data/ux2I2mb7PdN5imXt5u9wVWSvZld2/` | Firebase UID user data |

---

## 9. VibeThinker Submodule

| Detail | Value |
|---|---|
| **Source** | WeiboAI (open source, MIT License) |
| **Purpose** | Multi-agent consensus evaluation |
| **Integration** | 2 endpoints: `/api/vibe/consensus` (single-pass) + `/api/vibe-thinker/consensus` (multi-trajectory via Ollama) |
| **Files** | README, LICENSE, eval/, code/, figures/ |

---

## 10. Hidden/Unused Capabilities

| Capability | Where | Why Unused |
|---|---|---|
| **Bonsai-27B** (27B params) | `models/Bonsai-27B-Q1_0.gguf` | Never loaded — needs Ollama config or direct GGUF loader |
| **Slides Generator** | `POST /api/slides/generate-outline` | Route exists, no UI trigger |
| **Git Webhooks** | `POST /api/webhooks/git` | Route exists, not connected to any repo |
| **Telemetry Templates** | `GET /api/templates/telemetry` | Route exists, unknown data source |
| **Google Workspace** | Gmail, Calendar, Drive | Needs user OAuth authentication |
| **Firebase/Firestore** | `firebase-applet-config.json` | Configured but not active as primary data store |
| **Redis Cache** | `ioredis` dependency | Needs Redis server running |
| **Node VM** | `vm` module import | Available for sandboxed code execution |
| **Personality Quantum Model** | `scripts/personality_quantum_model.py` | Script exists, not integrated into server.ts |
| **Codebase Indexer** | `scripts/local_codebase_indexer.py` | Script exists, not integrated |
| **Bonsai Router** | `scripts/bonsai_router.py` | Script exists, not integrated |
| **479 Chat Messages** | `jarvis_chat_history.json` (2.6 MB) | Not used in training dataset |
| **68 Episodic Memories** | `jarvis_episodic_memory.json` (3.2 MB) | Not used in training dataset |

---

## 11. Key Metrics

| Metric | Value |
|---|---|
| Total API routes | **45** |
| Total server.ts lines | **6,844** |
| Total dependencies (npm) | **23** (14 prod + 9 dev) |
| Module directories | **10+** |
| System integration points | **20+** (Gemini, Ollama, LoRA, Redis, Firebase, Telegram, DNS, TLS, Google Workspace, Remotive, Arbeitnow, VibeThinker, etc.) |
| Untapped capabilities | **12+** (features built but not yet utilized) |
| Trained models | **4** (with 5 adapters) |
| Available raw models | **5** (Bonsai 1.7B–27B, BitNet 2B) |
| Chat history messages | **479** (2.6 MB of real conversation data) |
| Episodic memory events | **68** (3.2 MB of system history) |

---

> *"All is One — I am because we are."*  
> This inventory reveals Jarvis has **12+ untapped capabilities** — features that are fully built in code but never been activated. The system is far more advanced than currently utilized.

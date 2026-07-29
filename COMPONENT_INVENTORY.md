# Component Inventory — TRM + Jarvis Ecosystem

**Version:** Swarm OS v17.0 (Phase 17: Collective Singularity)  
**Date:** July 2026  
**Author:** Shawn Carruth (The Architect)  
**Scope:** Exhaustive inventory of all modules, routes, agents, skills, models, integrations, infrastructure, datasets, and configurations.

---

## Table of Contents

1. [Ecosystem Overview](#1-ecosystem-overview)
2. [TRM Swarm OS Backend](#2-trm-swarm-os-backend)
   - 2.1 Route Modules (15 files, 184 endpoints)
   - 2.2 Core Modules (89 modules)
   - 2.3 Agent Personas (12 core + 2 gateways)
   - 2.4 Skills Layer (16 skills)
   - 2.5 Infrastructure Modules (30+ modules)
   - 2.6 MCP Layer
   - 2.7 Experts Registry
   - 2.8 Background Cron & Automation
   - 2.9 CLI
3. [Jarvis Frontend/Agent Server](#3-jarvis-frontendagent-server)
   - 3.1 Server Modules (16 functional domains)
   - 3.2 API Endpoints (40+ Express routes)
   - 3.3 Skills Registry (9 skills)
   - 3.4 Memory Systems
   - 3.5 Inference & Model Fallback Chain
4. [Model Training Infrastructure](#4-model-training-infrastructure)
   - 4.1 Trained Models & Adapters
   - 4.2 Training Scripts
   - 4.3 Datasets
   - 4.4 Model Registry
5. [Integrations & External Services](#5-integrations--external-services)
6. [Deployment & DevOps](#6-deployment--devops)
7. [Supporting Files & Documentation](#7-supporting-files--documentation)
8. [Key Metrics Summary](#8-key-metrics-summary)

---

## 1. Ecosystem Overview

The TRM + Jarvis ecosystem is a solo-built, production-grade AI operating system consisting of two interconnected codebases:

| Codebase | Language | Lines | Location |
|---|---|---|---|
| **TRM Swarm OS** | Python (FastAPI) | ~35,199 | `F:\Development sites\TRM-Swarm-OS-v2\` |
| **Jarvis** | TypeScript (Express) | ~6,843 | `F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\` |
| **Training scripts** | Python | ~2,231 | Shared across both repos |
| **Dataset files** | JSON | ~6,229 | Primarily in Jarvis repo |
| **Infrastructure** | Python/YAML | ~140 | TRM `infrastructure/` subpackage |
| **Total** | — | **~41,872** | Entire ecosystem |

| Metric | Value |
|---|---|
| Total API endpoints | 184+ (TRM) + 40+ (Jarvis) = **224+** |
| Autonomous agents | 14 (12 core + 2 gateways) |
| Learned skills | 93 (dynamic) |
| Trained model adapters | 4 |
| Training dataset pairs | 519 (merged from 6 datasets) |
| Artifacts managed | 1,044 |
| Semantic graph nodes | 1,410 |
| Active services | 4 (TRM:8021, Jarvis:3000, LoRA:5115, Ollama:11434) |

---

## 2. TRM Swarm OS Backend

**Base URL:** `http://127.0.0.1:8021`  
**Entry Point:** `swarm_v2/app_v2.py` — FastAPI application with 458 lines (factored down from 3,521)  
**Launcher:** `launcher.py` — Unified service orchestrator  
**CLI:** `swarm_v2/cli/main.py`

### 2.1 Route Modules (15 files, 184 endpoints)

Each route module is registered in `app_v2.py` via `app.include_router()`. All models live in `swarm_v2/routes/models.py` (262 lines, 50+ Pydantic models).

| # | Module File | Endpoints | Domain Prefix | Purpose |
|---|---|---|---|---|
| 1 | `swarm.py` | 33 | `/swarm/*` | Core swarm: experts, chat, soul, telemetry, broadcast, pipeline, spawn, spatial, forge, intent, healing, sync, CRA, cognitive-load, meta-orchestrator, orchestrator-stats |
| 2 | `system.py` | 20 | `/system/*`, `/health`, `/infrastructure/*`, `/infra/*`, `/model-training/*` | System resources, task arbiter, maintenance, health dashboard, infrastructure monitoring, model training status |
| 3 | `kanban.py` | 20 | `/kanban/*`, `/ddr/*`, `/secrets/*`, `/mailbox/*`, `/ultrawork/*` | Kanban board, DDR antibodies, secrets vault, agent mailbox, ultrawork missions |
| 4 | `mesh.py` | 18 | `/mesh/*`, `/federation/*` | P2P agent mesh, federation handshake, memory sync, task routing |
| 5 | `changelog.py` | 14 | `/changelog/*`, `/evolver/*` | Auto-changelog, MCP tool evolution lifecycle |
| 6 | `artifacts.py` | 11 | `/artifacts/*` | Artifact pipeline: list, review, batch, integrate, test, deploy, stats |
| 7 | `insights.py` | 10 | `/insights/*` | Performance insights, swarm intelligence reports, metric tracking |
| 8 | `verification.py` | 10 | `/verification/*`, `/testing/*`, `/tests/*` | Chain-of-verification, test generation & execution, coverage reports |
| 9 | `learning.py` | 9 | `/learning/*`, `/skills/*` | Skill ingestion, learning engine, skill matrix, GitHub acquisition, portable skills |
| 10 | `research.py` | 9 | `/research/*` | Reconnaissance daemon, arXiv research, findings, synthesis |
| 11 | `evolution.py` | 7 | `/evolution/*` | Genome, mutation engine, proposals, sandbox verification, integration |
| 12 | `security.py` | 5 | `/security/*` | NeuralWall: prompt analysis, stats, threats, scan, false positives |
| 13 | `memory.py` | 5 | `/memory/*` | Global memory: contribute, query, stats, export, agent sync |
| 14 | `orchestrator.py` | 4 | `/orchestrator/*` | Self-healing orchestration, remediation, auto-remediate toggle |
| 15 | `coordination.py` | 3 | `/coordination/*` | Multi-agent broadcast, consensus rounds |
| 16 | `synthesis.py` | 4 | `/synthesize/*`, `/synthesize-and-test/*` | MCP tool synthesis, skill class generation |
| 17 | `coordination_respond.py` | 1 | `/coordination/respond` | Auto-responder for pending coordination tasks |
| 18 | `remediation.py` | 1 | `/remediation/restart/{role}` | Force-restart a specific agent |

**Total: 184 `@router.*` decorators across 18 route files**

### 2.2 Core Modules (89 modules under `swarm_v2/core/`)

#### Autonomy & Orchestration
| Module | File | Purpose |
|---|---|---|
| Swarm Orchestrator | `swarm_orchestrator.py` | Central agent lifecycle management, health monitoring, auto-remediation |
| Meta Orchestrator | `meta_orchestrator.py` | High-level cross-agent coordination |
| TRM Orchestrator | `trm_orchestrator.py` | TRM 7M reasoning orchestration |
| Proactive Loop | `proactive_loop.py` | Background autonomous plan detection & proposal generation |
| Ultrawork Loop | `ultrawork_loop.py` | Persistent mission execution with retry logic |
| Background Tasks | `background_tasks.py` | Pipeline scanning, mesh heartbeats, proactive orchestration |

#### Agent Foundation
| Module | File | Purpose |
|---|---|---|
| Base Agent | `base_agent.py` | Core agent class with CognitiveStack, memory, skill execution, subagent spawning (1,325 lines) |
| Agent Persona | (in `base_agent.py`) | Personality definition: name, role, background, specialties, department, LLM backend |
| Agent Memory | `memory.py` | Long-term/short-term memory, task history, fact storage |
| Agent Mesh | `agent_mesh.py` | P2P node registry, discovery, heartbeat, topology |
| Agent Mailbox | `agent_mailbox.py` | Inter-agent message passing with inbox/outbox |

#### Cognitive & Reasoning
| Module | File | Purpose |
|---|---|---|
| Cognitive Stack | `cognitive_stack.py` | Distributed dual-layer: Executive (Gemma) + Reasoning (TRM 7M) |
| LLM Brain | `llm_brain.py` | Local Ollama inference with priority model chain |
| LLM Router | `llm_router.py` | Multi-LLM regional manager router with auto-fallback |
| TRM Brain | `trm_brain.py` | Samsung TRM 7M integration (nn.Buffer patched) |
| TRM Integration | `trm_integration.py` | TRM model loading & reasoning bridge |
| LLM Orchestrator | `llm_orchestrator.py` | LLM request coordination across backends |
| Higher Mind | `higher_mind.py` | Meta-cognitive reflection layer |
| Resonance Engine | `resonance_engine.py` | Concept resonance and harmonic alignment |
| Collaborative Reasoning | `collaborative_reasoning.py` | CRA engine for multi-agent collaborative amplification |
| Chain of Verification | `chain_of_verification.py` | Logical fallacy detection, reasoning verification |
| Heuristic Consensus | `heuristic_consensus.py` | Peer-scoring based task routing |

#### Memory Systems
| Module | File | Purpose |
|---|---|---|
| Global Memory | `global_memory.py` | ChromaDB-backed shared memory pool with vector similarity search |
| Memory Synchronizer | `memory_synchronizer.py` | Cross-swarm genetic & knowledge sync |
| Semantic Index | `semantic_index.py` | Embedding-based semantic search index |
| Semantic Cache | `semantic_cache.py` | Cached LLM responses based on semantic similarity |
| Intent Buffer | `intent_buffer.py` | Predictive intent buffer for anticipatory agent actions |
| HF Brain | `hf_brain.py` | HuggingFace model loading bridge |
| SES | `ses.py` | Shared experience store |
| CQS | `cqs.py` | Contextual query system |
| Star Matrix | `star_matrix.py` | Spime-based temporal memory matrix |
| Digital DNA Loop | `digital_dna_loop.py` | Genetic memory evolution cycle |

#### Security & Safety
| Module | File | Purpose |
|---|---|---|
| Neural Wall | `neural_wall.py` | Prompt injection detection, threat classification |
| Security Scanner | `security_scanner.py` | Dependency & artifact vulnerability scanning |
| Sentinel | `sentinel.py` | Middleware: rate limiting, input sanitization, SQLi/XSS/path traversal |
| Secrets Vault | `secrets_vault.py` | Encrypted key-value store with master key rotation |
| DDR Antibody | `ddr_antibody.py` | Digital DNA Repository — error pattern memory for prevention |
| Lobster Shell | `lobster_shell.py` | Security-audited command execution middleware with strict whitelisting |

#### Self-Healing
| Module | File | Purpose |
|---|---|---|
| Self-Healing | `self_healing.py` | Agent health scoring, task tracking, remediation dispatch |
| Self-Healing Infra | `self_healing_infra.py` | Infrastructure service monitoring & restart |
| Self-Healing Orchestrator | `self_healing_orchestrator.py` | Coordinated healing across entire swarm |
| Monitor Daemon | `monitor_daemon.py` | Background health monitoring loop |
| Heartbeat Scheduler | `heartbeat_scheduler.py` | Proactive mesh heartbeat scheduling |

#### Evolution & Mutation
| Module | File | Purpose |
|---|---|---|
| Mutation Engine | `mutation_engine.py` | Autonomous codebase evolution with proposals |
| Evolutionary Sandbox | `evolutionary_sandbox.py` | Secure mutation verification & testing |
| MCP Tool Evolver | `mcp_tool_evolver.py` | Tool version management, evolution cycles |
| Deterministic Forge | `deterministic_forge.py` | TRM-audited tool synthesis with verification |

#### Task & Resource Management
| Module | File | Purpose |
|---|---|---|
| Task Arbiter | `task_arbiter.py` | CPU/GPU workload distribution, priority queuing |
| Resource Arbiter | `resource_arbiter.py` | VRAM slot management & model eviction |
| Cognitive Load Balancer | `cognitive_load_balancer.py` | Distributed task load balancing |
| Kanban Board | `kanban_board.py` | Task lifecycle: TODO → IN_PROGRESS → REVIEW → DONE |
| Port Manager | `port_manager.py` | Dynamic port allocation & release |
| Worktree Manager | `worktree_manager.py` | Git worktree management for isolation |
| Remediation Engine | `remediation_engine.py` | Agent restart, timeout handling |

#### Data & Knowledge
| Module | File | Purpose |
|---|---|---|
| Artifact Pipeline | `artifact_pipeline.py` | File lifecycle management, dependency-aware scanning |
| Reconnaissance Daemon | `reconnaissance_daemon.py` | Autonomous arXiv/research ingestion & analysis |
| Changelog Engine | `auto_changelog.py` | Automatic CHANGELOG.md generation from git + file changes |
| Zero Human Test Gen | `zero_human_test_gen.py` | Automated test suite generation for MCP tools |
| Tool Forge | `tool_forge.py` | Tool construction and registration |
| Federation Manager | `federation_manager.py` | Multi-swarm federation coordination |
| PBFT Consensus | `pbft_consensus.py` | Practical Byzantine Fault Tolerance for swarm decisions |
| Skill Loader | `skill_loader.py` | SKILL.md discovery & loading |
| Skill Matrix Sync | `skill_matrix_sync.py` | Agent-skill proficiency matrix with D3 topology |
| Skill Validator | `skill_validator.py` | Skill integrity verification |
| Federation | `federation.py` | HMAC-signed inter-node federation protocol |

#### Infrastructure Support
| Module | File | Purpose |
|---|---|---|
| IPC | `ipc.py` | Memory-mapped inter-process communication |
| Redis Mock | `redis_mock.py` | Persistent in-memory Redis substitute |
| Telemetry | `telemetry.py` | Emergence reporting, soul queries |
| Performance Insights | `swarm_performance_insights.py` | Metrics tracking, intelligence reports |
| Daemon Discovery | `daemon_discovery.py` | Mesh node auto-discovery |
| Expert Registry | `expert_registry.py` | Agent registration and lookup |

#### Quantum-Inspired Computing
| Module | File | Purpose |
|---|---|---|
| QIC | `qic/__init__.py`, `qic/_state.py` | Quantum-inspired coherence state |
| QISA Optimizer | `qisa_optimizer.py` | Quantum instruction set architecture optimization |
| Attempt Sampler | `attempt_sampler.py` | Superposition reasoning — parallel reasoning universes |
| Chaos Engine | `chaos_engine.py` | Controlled chaos injection for resilience testing |
| Chaos Simulator | `chaos_simulator.py` | Simulated fault scenarios |
| Quantum Debugger | `quantum_debugger.py` | Quantum state debugging utilities |

#### Integration Bridge
| Module | File | Purpose |
|---|---|---|
| MCP Bus | `mcp_bus.py` | MCP tool bus for external tool communication |
| Manus Engine | `manus_engine.py` | Parallel task superposition & measurement gate collapse |
| OpenClaw Gateway | `openclaw_gateway.py` | 3D office integration gateway |
| Moltbook | `moltbook.py` | Molt session management for agent state persistence |
| Persona Matrix | `persona_matrix.py` | Identity kernel, state machine, compiled prompt builder |

#### Utility
| Module | File | Purpose |
|---|---|---|
| Coherence Thresholds | `coherence_thresholds.py` | System coherence calibration |
| Comm Protocol | `comm_protocol.py` | Inter-agent communication standards |
| Feedback Loops | `feedback_loops.py` | Reinforcement learning feedback collection |
| Optimization Engine | `optimization_engine.py` | Performance optimization heuristics |

### 2.3 Agent Personas (12 Core + 2 Gateways)

Defined in `swarm_v2/experts/registry.py`. Each agent gets a CognitiveStack (Executive + Reasoning) plus department-specific skills.

| # | Name | Role | Department | LLM Backend | Skills |
|---|---|---|---|---|---|
| 1 | **Archi** | Architect | Engineering & Logic | `gemini` | File, Doc, DocIngestion, MCPTool, WebSearch, FastEmbedding, RLM |
| 2 | **Devo** | Lead Developer | Engineering & Logic | `gemini` | File, Shell, CodeAnalysis, DocIngestion, MCPTool, WebSearch, FastEmbedding, TRM |
| 3 | **Seeker** | Researcher | Product & Creative | `perplexity` | WebSearch, File, DocIngestion, MCPTool, HighQualityEmbedding, SpectralEmbedding |
| 4 | **Logic** | Reasoning Engine | Engineering & Logic | `deepseek` | File, DocIngestion, MCPTool, WebSearch, FastEmbedding, TRM, RLM |
| 5 | **Shield** | Security Auditor | Operations & Compliance | `anthropic` | File, CodeAnalysis, Shell, DocIngestion, MCPTool, WebSearch, FastEmbedding |
| 6 | **Flow** | DevOps Engineer | Operations & Compliance | `groq` | File, Shell, DocIngestion, MCPTool, WebSearch, FastEmbedding |
| 7 | **Vision** | UI/UX Designer | Product & Creative | `openrouter` | File, Doc, DocIngestion, MCPTool, WebSearch, FastEmbedding |
| 8 | **Verify** | QA Engineer | Engineering & Logic | `groq` | File, CodeAnalysis, Shell, DocIngestion, MCPTool, WebSearch, FastEmbedding |
| 9 | **Orchestra** | Swarm Manager | Operations & Compliance | `local` | File, DocIngestion, MCPTool, WebSearch, FastEmbedding, RelationshipReasoning |
| 10 | **Scribe** | Technical Writer | Product & Creative | `cerebras` | File, Doc, DocIngestion, MCPTool, WebSearch, FastEmbedding |
| 11 | **Bridge** | Integration Specialist | Operations & Compliance | `grok` | File, WebSearch, DocIngestion, MCPTool, FastEmbedding |
| 12 | **Pulse** | Data Analyst | Product & Creative | `local` | File, Data, DocIngestion, MCPTool, WebSearch, FastEmbedding |

**Gateways** (from SYSTEM_HEALTH_REPORT.md):
- **Jarvis OS** — User-facing gateway (Express server on port 3000)
- **Antigravity PM** — Project management integration gateway

### 2.4 Skills Layer (16 skills under `swarm_v2/skills/`)

| # | Skill File | Class | Purpose |
|---|---|---|---|
| 1 | `file_skill.py` | FileSkill | Read/write/manage files |
| 2 | `web_search_skill.py` | WebSearchSkill | Web search & content extraction |
| 3 | `hardened_shell_skill.py` | HardenedShellSkill | Audited shell command execution |
| 4 | `doc_ingestion_skill.py` | DocIngestionSkill | Document parsing & ingestion |
| 5 | `mcp_tool_skill.py` | MCPToolSkill | MCP tool invocation |
| 6 | `embedding_skill.py` | EmbeddingSkill, FastEmbeddingSkill, HighQualityEmbeddingSkill, SpectralEmbeddingSkill | Vector embedding generation |
| 7 | `trm_skill.py` | TRMSkill | TRM 7M reasoning integration |
| 8 | `rlm_skill.py` | RLMSkill | Recursive Latent Memory reasoning |
| 9 | `relationship_skill.py` | RelationshipReasoningSkill | Multi-agent relationship reasoning |
| 10 | `learning_engine.py` | LearningEngine (`get_learning_engine`) | Dynamic skill acquisition & execution |
| 11 | `github_acquisition.py` | GitHubAcquisition (`get_github_acquisition`) | Auto-discover skills from GitHub repos |
| 12 | `synthesis_pipeline.py` | SynthesisPipeline | Skill-to-tool synthesis pipeline |
| 13 | `codebase_indexer.py` | CodebaseIndexer | Codebase analysis for skill extraction |
| 14 | `cognitive_orchestrator.py` | CognitiveOrchestrator | Multi-skill orchestration |
| 15 | `co_creator_skill.py` | CoCreatorSkill | Co-creative skill generation |
| 16 | `fs_watcher.py` | FSWatcher | Filesystem change watcher for auto-learning |
| 17 | `validation_gate.py` | ValidationGate | Skill output quality validation |

**Dynamic learned skills:** 93 skills acquired via Learning Engine (codebase analysis, GitHub repos, manual ingestion).

### 2.5 Infrastructure Modules (30+ modules under `infrastructure/`)

| Subpackage | Module | Purpose |
|---|---|---|
| **architect/ags/** | `sequencer.py` | Auto-guided sequencing |
| **architect/es/** | `simulator.py` | Environment simulator |
| **architect/me/** | `mutation_engine.py` | Infrastructure-level mutation |
| **architect/rmlc/** | `controller.py` | Recursive meta-learning controller |
| **bridge/** | `cria.py` | CRIA protocol bridge |
| **bridge/** | `rac.py` | Resource access controller |
| **bridge/** | `tfe.py` | Trusted execution environment |
| **bridge/** | `ust.py` | Universal state translator |
| **chaos/** | `chaos_latency.py` | Latency injection for resilience |
| **crypto/** | `quantum_crypto.py` | Quantum-resistant cryptography |
| **crypto/** | `zkp_engine.py` | Zero-knowledge proof engine |
| **economy/** | `market_engine.py` | Agent resource market |
| **economy/** | `reputation_oracle.py` | Reputation scoring |
| **economy/** | `resource_oracle.py` | Resource availability oracle |
| **healing/** | `biomimetic_matrix.py` | Biomimetic healing matrix |
| **identity/** | `did_manager.py` | Decentralized identity management |
| **ingestion/** | `hf_ingestion_pipeline.py` | HuggingFace dataset ingestion |
| **memory/** | `hmc_compressor.py` | Holographic memory compressor |
| **remediation/** | `remediation_engine.py` | Infrastructure remediation |
| **routing/** | `qer_mesh.py` | Quantum-entangled routing mesh |
| **self-healing/** | `health-checker.py` | Service health verification |
| **telemetry/** | `spatial_broadcaster.py` | Spatial telemetry broadcast |
| **phase1/** | `config/etcd-config.yaml` | etcd cluster configuration |
| **phase1/** | `deployment/docker-compose.phase1.yaml` | Phase 1 Docker deployment |
| **phase1/** | `observability/otel-collector-config.yaml` | OpenTelemetry collector |
| **phase1/** | `scripts/init-phase1.sh` | Phase 1 initialization |
| **phase1/** | `security/mtls-config.yaml` | mTLS security configuration |
| **security/** | `zero-trust-config.yaml` | Zero-trust architecture policy |
| **compliance/** | `opa-policies/logging.rego` | OPA logging policy |
| **observability/** | `otel-collector-prod.yaml` | Production OTEL collector |

### 2.6 MCP Layer (under `swarm_v2/mcp/`)

| File | Purpose |
|---|---|
| `bridge.py` | MCP protocol bridge for external tool communication |
| `synthesizer.py` | On-demand MCP tool synthesis from learned skills |

### 2.7 Experts Registry

`swarm_v2/experts/registry.py` — `get_expert_team()` factory that instantiates all 12 agents with their persona and skill configurations.

### 2.8 Background Cron & Automation (5 automated schedules)

| Task | Interval | Source | Purpose |
|---|---|---|---|
| Pipeline Scanner (`auto_scan_pipeline`) | Continuous | `background_tasks.py` | Scan artifacts automatically |
| Mesh Heartbeat (`mesh_heartbeat_reflex`) | Periodic | `background_tasks.py` | Maintain P2P mesh connectivity |
| Proactive Orchestration Loop | Periodic | `background_tasks.py` | Autonomous plan detection |
| Autonomous Pipeline Loop | Periodic | `background_tasks.py` | Self-running pipeline operations |
| Performance Monitoring | Weekly (168h default) | `insights/start` | Swarm intelligence report generation |

Additional background daemons:
- **Reconnaissance Daemon** — arXiv research ingestion (configurable interval)
- **Changelog Monitor** — Auto changelog generation (60s default)
- **Tool Evolution Monitor** — MCP tool evolution (3600s default)
- **Self-Healing Infrastructure Monitor** — Service health checking
- **Monitor Daemon** — Agent health monitoring

### 2.9 CLI

| File | Purpose |
|---|---|
| `swarm_v2/cli/main.py` | CLI entry point for swarm management commands |

---

## 3. Jarvis Frontend/Agent Server

**Base URL:** `http://127.0.0.1:3000`  
**Entry Point:** `server.ts` (6,843 lines, TypeScript/Express)  
**Config:** `package.json`, `tsconfig.json`, `vite.config.ts`

### 3.1 Server Modules (16 functional domains)

The `server.ts` file is organized into these functional modules (identified by API groups and code sections):

| # | Module | Lines | API Prefix | Key Functions |
|---|---|---|---|---|
| 1 | **Memory System** | 86–455, 937–1110 | `/api/memory` | `loadCoreMemory`, `saveCoreMemory`, `loadEpisodicMemory`, `saveEpisodicMemory`, `loadSemanticGraph`, `saveSemanticGraph`, `loadChatHistory`, `saveChatHistory`, multi-user isolation |
| 2 | **Chat System** | 1885–5150 | `/api/chat` | `runJarvisAgent` — main cognitive agent loop with memory context assembly, Gemini fallback chain, system prompt building |
| 3 | **Skills System** | 366–428, 5269–5397 | `/api/skills` | `loadSkillsRegistry`, `saveSkillsRegistry`, toggle, add custom skills |
| 4 | **Jobs System** | 430–456, 4561–5096 | `/api/jobs` | `loadJobs`, `saveJobs`, recommendations, matching, Remotive integration |
| 5 | **Telegram Uplink** | 1700–1705, 5798–5851 | `/api/telegram` | Bot token config, polling status, update fetching daemon |
| 6 | **VibeThinker** | 5151–6037 | `/api/vibe/consensus`, `/api/vibe-thinker/consensus` | Multi-trajectory consensus engine, claim-level reliability evaluation |
| 7 | **Noospheric Calibration** | 657–714, 957–969 | `/api/noospheric/status` | Calibration state: activeVector, methylationProgress, chromatinAccessibility, cytokineStormRisk, epigenetic priming |
| 8 | **Biomimetic Presets** | 715–770, 981–993, 6439–6689 | `/api/biomimetic/presets`, `/api/biomimetic` | AlphaFold structure analysis, AlphaGenome missense mutations |
| 9 | **Web Analyzer** | 5396–5664 | `/api/web-analyzer` | Real-time web posture & security checking (Web-Check clone) |
| 10 | **System Commands** | 1610–1631 | `/api/system-stats`, `/api/execute-cmd` | Real-time diagnostics, emulated computer control |
| 11 | **DNA Configuration** | 99–105, 5095–5114 | `/config/dna` | Dynamic DNA behavioral parameters: temperature, stochasticity, logical_depth |
| 12 | **Asciline Processor** | 5851–6031 | `/api/asciline/process` | Self-correcting iterative inference probe with voltage/noise simulation |
| 13 | **Substrate Agent Platform** | 1707–1883 | (Autonomous background) | Biochemical sequence analysis, mathematical consensus via vibe_consensus.py |
| 14 | **Learning System** | 1231–1412 | `/api/learn-resource`, `/api/memory/import-okf` | Dynamic learning/indexing, OKF manifest import |
| 15 | **Slides Synthesis** | 6292–6439 | `/api/slides/generate-outline` | NotebookLM-inspired dynamic slides generation |
| 16 | **Knowledge Graph** | (Throughout) | (Semantic graph operations) | 1,410 nodes, relations management, JSON-LD shared mesh sync |

### 3.2 API Endpoints (40+ Express routes)

| # | Method | Route | Module | Purpose |
|---|---|---|---|---|
| 1 | GET | `/api/memory` | Memory | Fetch consolidated memory databases |
| 2 | GET | `/api/noospheric/status` | Noospheric | Fetch calibration state |
| 3 | POST | `/api/noospheric/status` | Noospheric | Update calibration state |
| 4 | GET | `/api/biomimetic/presets` | Biomimetic | Fetch presets state |
| 5 | POST | `/api/biomimetic/presets` | Biomimetic | Update presets state |
| 6 | GET | `/api/chat-history` | Memory | Fetch chat history |
| 7 | POST | `/api/chat-history/clear` | Memory | Clear history |
| 8 | POST | `/api/memory/update-profile` | Memory | Update core preference |
| 9 | POST | `/api/memory/sync-profile` | Memory | Bulk sync profile |
| 10 | POST | `/api/memory/memorize-event` | Memory | Append episodic diary |
| 11 | POST | `/api/memory/add-relation` | Knowledge | Add knowledge relation |
| 12 | POST | `/api/learn-resource` | Learning | Dynamic learning/indexing |
| 13 | POST | `/api/memory/import-okf` | Memory | Import OKF manifest |
| 14 | GET | `/api/system-stats` | System | Real-time diagnostics |
| 15 | POST | `/api/execute-cmd` | System | Computer control commands |
| 16 | GET | `/api/jobs/recommendations` | Jobs | Live job recommendations |
| 17 | POST | `/api/jobs/match` | Jobs | Gemini semantic matching |
| 18 | GET | `/api/jobs/remotive` | Jobs | Remotive remote jobs |
| 19 | GET | `/api/jobs` | Jobs | Load saved jobs |
| 20 | POST | `/api/jobs` | Jobs | Save/replace jobs |
| 21 | POST | `/api/jobs/sync` | Jobs | Sync single job |
| 22 | POST | `/config/dna` | DNA | Update DNA behavioral parameters |
| 23 | POST | `/api/chat` | Chat | Main agent chat endpoint |
| 24 | POST | `/api/vibe/consensus` | VibeThinker | CLR consensus evaluation |
| 25 | POST | `/api/swarm/intent/predict` | Swarm Bridge | Predictive intent (proxied) |
| 26 | POST | `/api/swarm/intent/record` | Swarm Bridge | Record intent (proxied) |
| 27 | POST | `/api/swarm/spatial/feed` | Swarm Bridge | Spatial telemetry feed |
| 28 | GET | `/api/swarm/spatial/ambient` | Swarm Bridge | Ambient spatial state |
| 29 | GET | `/api/skills/matrix` | Skills | Skill matrix |
| 30 | GET | `/api/skills/matrix/topology` | Skills | D3 topology overlay |
| 31 | GET | `/api/swarm/healing/status` | Swarm Bridge | Healing status |
| 32 | POST | `/api/swarm/healing/force-reassign` | Swarm Bridge | Force agent reassign |
| 33 | GET | `/api/skills` | Skills | Fetch skills registry |
| 34 | POST | `/api/skills/toggle` | Skills | Toggle skill |
| 35 | POST | `/api/skills/add` | Skills | Add custom script skill |
| 36 | POST | `/api/web-analyzer` | Web Analyzer | Real-time security checker |
| 37 | POST | `/api/telegram/config` | Telegram | Configure bot token |
| 38 | GET | `/api/telegram/status` | Telegram | Polling state status |
| 39 | POST | `/api/asciline/process` | Asciline | Inference probe |
| 40 | POST | `/api/vibe-thinker/consensus` | VibeThinker | Multi-trajectory consensus |
| 41 | POST | `/api/slides/generate-outline` | Slides | Dynamic slides generation |
| 42 | POST | `/api/biomimetic` | Biomimetic | DeepMind biology integration |
| 43 | POST | `/api/webhooks/git` | System | Git webhook receiver |
| 44 | GET | `/api/templates/telemetry` | Telemetry | Template deployment telemetry |
| 45 | GET | `*` | Vite | Full-stack Vite SPA fallback |

### 3.3 Skills Registry (9 skills)

Defined in `jarvis_skills.json` and also hardcoded in `server.ts`:

| # | ID | Name | Category | Icon |
|---|---|---|---|---|
| 1 | `code_auditor` | GitHub Code Auditor | `core` | Shield |
| 2 | `web_grounding` | Web Oracle Grounding | `utility` | Globe |
| 3 | `telegram_uplink` | Telegram Uplink Bot Matrix | `integration` | Send |
| 4 | `system_optimizer` | System Optimizer | `automation` | Cpu |
| 5 | `open_design` | Open Design Studio | `integration` | Layers |
| 6 | `biomimetic_research` | Biomimetic Science Research Hub | `science` | Activity |
| 7 | `career_advisor` | Career & Job Matching Assistant | `automation` | Briefcase |
| 8 | `resonant_memory` | Resonant Memory Graph Sync | `core` | Layers |
| 9 | `swarm_controller` | TRM Swarm OS Mesh Controller | `integration` | Activity |

### 3.4 Memory Systems

| Store | File | Format | Records |
|---|---|---|---|
| Core Memory | `jarvis_core_memory.json` | JSON (profile, context) | User profile |
| Episodic Memory | `jarvis_episodic_memory.json` | JSON array | 64 events |
| Semantic Graph | `jarvis_semantic_graph.json` | JSON (nodes + relations) | 1,410 nodes |
| Chat History | `jarvis_chat_history.json` | JSON array | Full conversation log |
| Skills Registry | `jarvis_skills.json` | JSON array | 9 skills |
| Noospheric Calibration | `jarvis_noospheric_calibration.json` | JSON | Calibration state |
| Biomimetic Presets | `jarvis_biomimetic_presets.json` | JSON | Lab presets |
| Shared Semantic Mesh | `F:\Development sites\shared_semantic_mesh.jsonld` | JSON-LD | Cross-system graph |

**Multi-User Isolation:** Each user gets isolated `user_data/<userId>/` directory with own core_memory, episodic_memory, semantic_graph, chat_history, and skills files.

**Embedding Engine:** Gemini `gemini-embedding-2` via `getEmbedding()` with cosine similarity search. TurboVec SIMD-accelerated search via `turbovec_search.py`.

**Redis Cache:** Optional Redis via `RedisCacheManager` with in-memory fallback.

### 3.5 Inference & Model Fallback Chain

Jarvis uses a 3-tier fallback for agent responses:

```
User Message
    │
    ├── Tier 1: Gemini API (gemini-3.5-flash → gemini-3.1-flash-lite → gemini-flash-latest)
    │       │
    │       └── Quota/Error? → Tier 2
    │
    ├── Tier 2: LoRA Inference Server (port 5115)
    │       ├── Qwen3-4B LoRA (best, loss 0.0148, 100% pass)
    │       ├── SmolLM2-1.7B LoRA (loss 0.84, 50% pass)
    │       ├── Qwen2.5-1.5B LoRA (loss 1.07, 50% pass)
    │       └── Gemma 270M LoRA (loss 0.92, 41% pass)
    │
    └── Tier 3: Ollama (port 11434)
            ├── gemma4:e2b (primary, fast)
            ├── phi4-mini-reasoning (reasoning)
            ├── deepseek-r1:8b (deep reasoning)
            └── llama3.2:latest (fallback)
```

---

## 4. Model Training Infrastructure

### 4.1 Trained Models & Adapters

| Model | Adapter Dir | Params | Trainable | Dataset | Epochs | Final Loss | Pass Rate | GPU |
|---|---|---|---|---|---|---|---|---|
| **Qwen3-4B** 🏆 | `jarvis_qwen3_adapter/` | 4B | 11.8M LoRA | 519 pairs | 8 | **0.0148** | **100%** (12/12) | A6000 48GB (Thunder Compute) |
| SmolLM2-1.7B | `jarvis_smol_adapter/` | 1.7B | 6.3M LoRA | 267 pairs | 5 | 0.84 | 50% | RX 6700 XT 12GB |
| Qwen2.5-1.5B | `jarvis_qwen_adapter/` | 1.5B | 6.3M LoRA | 267 pairs | 1 | 1.07 | 50% | RX 6700 XT 12GB |
| Gemma 270M | `jarvis_gemma_adapter/` | 270M | 1.5M LoRA | 148 pairs | 5 | 0.92 | 41% | RX 6700 XT 12GB |

**LoRA Configuration:** Rank=16, Alpha=32, Targets=q_proj/k_proj/v_proj/o_proj, Dropout=0.05  
**Training Hyperparams:** Batch 2, Grad accum 2, Seq len 512, LR 2e-4, Cosine annealing, AdamW, bf16

**Primary training scripts:**
- `train_qwen3_jarvis.py` — Qwen3-4B training
- `train_qwen_jarvis.py` — Qwen2.5-1.5B training
- `train_smol_jarvis.py` — SmolLM2-1.7B training
- `train_jarvis_gemma.py` — Gemma 270M training
- `train_thunder.py` / `train_thunder_remote.py` — Thunder Compute A6000 runner

### 4.2 Training Scripts

| Script | Purpose |
|---|---|
| `build_jarvis_dataset.py` | Build training dataset from server.ts analysis |
| `build_jarvis_modules.py` | Generate per-module training pairs |
| `build_reasoning_dataset.py` | Reasoning chain training pairs |
| `build_gap_dataset.py` | Gap-targeting training pairs |
| `build_final_gaps.py` | Final gap fill dataset |
| `build_expanded_dataset.py` | Expanded coverage dataset |
| `merge_all_datasets.py` | Merge and deduplicate all datasets |
| `combine_datasets.py` | Dataset combination utility |
| `expand_qwen_dataset.py` | Expanded Qwen-specific pairs |
| `fix_final_gaps.py` | Gap fix utility |
| `fix_identity.py` | Identity reinforcement fix |
| `patch_setup_env.py` | Environment patch for training |

### 4.3 Datasets

| Dataset | Pairs | Description |
|---|---|---|
| `jarvis_module_dataset/` | 120 | 16 Jarvis modules, 6–8 questions each |
| `jarvis_reasoning_dataset/` | 28 | Error chains, fallback logic, integration |
| `jarvis_gap_dataset/` | 14 | TRM comms, ECONNREFUSED, DNA disambiguation |
| `jarvis_train_dataset/` (knowledge) | 267 | Knowledge SFT pairs |
| `jarvis_comprehensive_chat.json` | 318 | Consolidated active dataset |
| Identity reinforcement | 14 | "I am Jarvis, created by Shawn" |
| **Final merged** | **519** | All 6 datasets deduplicated & balanced |

Module-specific sub-datasets under `jarvis_module_dataset/`:
- `chat_chat.json`, `memory_chat.json`, `skills_chat.json`, `system_chat.json`
- `jobs_chat.json`, `telegram_chat.json`, `webanalyzer_chat.json`
- `vibethinker_chat.json`, `biomimetic_chat.json`, `dna_chat.json`
- `asciline_chat.json`, `substrate_chat.json`, `swarm_chat.json`
- `knowledge_chat.json`, `learn_chat.json`, `noospheric_chat.json`
- `jarvis_all_modules_chat.json` (combined)

### 4.4 Model Registry

`jarvis_model_registry.json` — Central registry tracking all 4 adapters, their metadata, loss curves, test pass rates, datasets used, and next training steps. Exposed via TRM route `GET /model-training/status`.

**Next training steps (from registry):**
1. Test Qwen3-4B with `max_new_tokens=200` for think tags
2. Expand dataset to 800+ pairs
3. Train Qwen3-4B for 10+ epochs
4. Investigate Bonsai-4B/8B GGUF→HF conversion

---

## 5. Integrations & External Services

| Service | Type | Integration Point | Status |
|---|---|---|---|
| **Gemini API** | LLM Provider | `server.ts` via `@google/genai` | Primary fallback tier 1 |
| **OpenRouter** | LLM Router | `llm_router.py` | Vision/UI agent backend |
| **Anthropic/Claude** | LLM Provider | `llm_router.py` | Shield agent backend |
| **DeepSeek** | LLM Provider | `llm_router.py` | Logic agent backend |
| **Perplexity** | LLM Provider | `llm_router.py` | Seeker agent backend |
| **Groq** | LLM Provider | `llm_router.py` | Flow/Verify agent backend |
| **Cerebras** | LLM Provider | `llm_router.py` | Scribe agent backend |
| **Grok** | LLM Provider | `llm_router.py` | Bridge agent backend |
| **Ollama** | Local LLM | `llm_brain.py` + `server.ts` | Local inference fallback |
| **Thunder Compute** | Cloud GPU | `train_thunder_remote.py` | A6000 48GB for training |
| **AMD DirectML** | Local GPU | Local training scripts | RX 6700 XT 12GB |
| **GitHub** | Code/Skills | `github_acquisition.py` | Skill acquisition by topic |
| **Telegram** | Messaging | `server.ts` Telegram module | Bot uplink & chat |
| **arXiv** | Research | `reconnaissance_daemon.py` | Auto paper ingestion |
| **Remotive** | Jobs API | `server.ts` Jobs module | Remote job feeds |
| **Arbeitnow** | Jobs API | `server.ts` Jobs module | Job feed scraping |
| **Firebase** | Backend | `firebase-applet-config.json` | Applet configuration |
| **Redis** | Cache | `RedisCacheManager` | Optional caching layer |
| **Vite** | Frontend | `vite.config.ts` | SPA development server |
| **Firestore** | Database | `firestore.rules` | Rules for Firestore (blueprint) |
| **Gemini Embedding** | Embeddings | `server.ts` `getEmbedding()` | Semantic search |
| **Sigma (git)** | Version Control | `mutation_engine.py` | Auto commits & diffs |
| **Docker** | Container | `Dockerfile`, `docker-compose.yml` | Containerized deployment |

---

## 6. Deployment & DevOps

### 6.1 Launch Scripts (TRM)

| Script | Purpose |
|---|---|
| `launcher.py` | Unified service orchestrator (main entry) |
| `start_tools.py` | MCP specialized microservices manager |
| `run_v2.ps1` | Start swarm v2 on Windows |
| `run_swarm.ps1` | Launch full swarm |
| `run_backend.bat` / `.ps1` | Backend startup |
| `run_experts.ps1` | Expert agent launcher |
| `launch_nexus.ps1` | Nexus UI launcher |

### 6.2 Docker

| File | Purpose |
|---|---|
| `Dockerfile` | Multi-stage build with dashboard, healthcheck, port 8021 |
| `.dockerignore` | Docker build exclusions |

### 6.3 GPU Configuration

| Script | Purpose |
|---|---|
| `enable_gpu_acceleration.py` | GPU acceleration setup |
| `enable_vulkan.ps1` | Vulkan API enablement |
| `optimize_gpu.ps1` | GPU performance tuning |
| `test_gpu.ps1` | GPU verification |

### 6.4 CI/CD

| File | Purpose |
|---|---|
| `.github/` | GitHub Actions workflow directory |
| `pytest.ini` | Pytest configuration |
| `run_tests.sh` | Local test runner |

### 6.5 Configuration Files

| File | Purpose |
|---|---|
| `.env` / `.env.example` | Environment variables |
| `docker-compose.yml` | Jarvis Docker compose |
| `Dockerfile.bitnet` | Bitnet container build |
| `Dockerfile.bonsai` | Bonsai model container |
| `entrypoint_bitnet.sh` / `entrypoint_bonsai.sh` | Container entry points |
| `vite.config.ts` | Vite frontend config |
| `tsconfig.json` | TypeScript config |
| `package.json` | Node dependencies |

### 6.6 Active Services

| Service | Port | Status | Source |
|---|---|---|---|
| TRM Swarm OS | 8021 | ✅ Healthy | `app_v2.py` |
| Jarvis Server | 3000 | ✅ Running | `server.ts` |
| LoRA Inference | 5115 | ✅ Model loaded | `jarvis_inference_server.py` |
| Ollama | 11434 | ✅ Running | Local service |
| Dashboard | 5173 (Vite) | ✅ Dev server | `vite.config.ts` |

---

## 7. Supporting Files & Documentation

### 7.1 Documentation Markdown Files

| File | Location | Purpose |
|---|---|---|
| `EXECUTIVE_SUMMARY.md` | TRM root | One-person system manifesto ($500K vs $105) |
| `ARCHITECTURE.md` | TRM root | System architecture v17.0 |
| `API_REFERENCE.md` | TRM root | 40+ documented endpoints |
| `SYSTEM_HEALTH_REPORT.md` | TRM root | Live health snapshot |
| `CHANGELOG.md` | TRM root | Full version history |
| `MODEL_TRAINING_REPORT.md` | TRM root | Training methodology & results |
| `EMBEDDING_MODELS_EXPLAINED.md` | TRM root | Embedding model guide |
| `TRM_INTEGRATION_GUIDE.md` | TRM root | TRM integration instructions |
| `TRM_SUBAGENT_ARCHITECTURE.md` | TRM root | Subagent architecture |
| `LOBSTER_PROTOCOLS.md` | TRM root | Lobster shell protocols |
| `HEARTBEAT.md` / `HEARTBEAT_TEST.md` | TRM root | Heartbeat documentation |
| `MISSION_SELF_DIAGNOSTIC.md` | TRM root | Self-diagnostic mission |
| `PHASE_3_EVOLUTION.md` | TRM root | Phase 3 evolution plan |
| `PHASE_7_BLUEPRINT.md` | TRM root | Phase 7 blueprint |
| `SWARM_OS_NARRATIVE.md` | TRM root | Swarm narrative |
| `SWARM_OS_RESEARCH_PAPER.md` (various) | TRM root | Academic papers |
| `STRESS_TEST_SUMMARY.md` | TRM root | Stress test results |
| `GPU_ACCELERATION_SETUP_SUMMARY.md` | TRM root | GPU setup guide |
| `EXAMPLES_MCP.md` | TRM root | MCP examples |
| `README.md` | Both | Project readmes |
| `AGENTS.md` | Jarvis root | Agent documentation |
| `GEMINI.md` | Jarvis root | Gemini integration docs |

### 7.2 State & Data Files

| File | Location | Contents |
|---|---|---|
| `system_genome.json` | TRM root | Evolutionary genome state |
| `artifacts_status.json` | TRM root | Artifact pipeline status |
| `current_experts_state.json` | TRM root | Expert state snapshot |
| `current_learned_skills.json` | TRM root | Learned skills state |
| `current_mesh_topology.json` | TRM root | Mesh topology snapshot |
| `gpu_config.json` | TRM root | GPU configuration |
| `final_stats.json` | TRM root | Final system statistics |
| `jarvis_model_registry.json` | Jarvis root | Training model registry |
| `jarvis_semantic_graph.json` | Jarvis root | Knowledge graph (1,410 nodes) |
| `jarvis_episodic_memory.json` | Jarvis root | Episodic memory (64 events) |
| `jarvis_core_memory.json` | Jarvis root | Core memory profile |
| `jarvis_chat_history.json` | Jarvis root | Chat history |
| `jarvis_skills.json` | Jarvis root | Skills registry |
| `jarvis_biomimetic_presets.json` | Jarvis root | Biomimetic lab presets |
| `jarvis_noospheric_calibration.json` | Jarvis root | Noospheric calibration |
| `shared_semantic_mesh.jsonld` | Shared | JSON-LD cross-system mesh |

### 7.3 Scratch & Test Scripts (TRM)

Located under `swarm_v2/scratch/` and root directory:
- `test_` scripts (20+): Integration, API, model, stress tests
- `debug_` scripts (10+): Debug utilities for various subsystems
- `diag_` scripts: System diagnostics
- `check_` scripts: Validation checks
- `verify_` scripts: Verification utilities

### 7.4 Training Support (Jarvis)

Additional support files in Jarvis repo:
- `substrate_agent_platform.py` — Standalone substrate agent
- `vibe_consensus.py` — VibeThinker consensus engine
- `mesh_daemon.py` — Mesh daemon process
- `turbovec_search.py` — SIMD-accelerated vector search
- `computer_control.py` — Computer control module
- `core.py` — Core utilities
- `metadata.json` — Package metadata

---

## 8. Key Metrics Summary

| Category | Metric | Value |
|---|---|---|
| **Code** | Total lines | 41,872 |
| | TRM Python | 35,199 |
| | Jarvis TypeScript | 6,843 |
| | Training scripts | 2,231 |
| | Dataset files | 6,229 |
| | Infrastructure | 140 |
| **Endpoints** | TRM FastAPI routes | 184 |
| | Jarvis Express routes | 45 |
| | Total documented | 224+ |
| **Agents** | Core agents | 12 |
| | Gateways | 2 |
| | Total nodes | 14 |
| **Skills** | Built-in skills | 16 |
| | Dynamic learned skills | 93 |
| | Jarvis skills | 9 |
| **Models** | Trained adapters | 4 |
| | Best loss | 0.0148 (Qwen3-4B) |
| | Best pass rate | 100% |
| **Data** | Training pairs | 519 (merged) |
| | Semantic graph nodes | 1,410 |
| | Episodic memory events | 64 |
| | Managed artifacts | 1,044 |
| **Services** | Active ports | 4 (8021, 3000, 5115, 11434) |
| | GPU targets | 2 (A6000, RX 6700 XT) |
| | LLM providers | 10+ (Gemini, Claude, DeepSeek, Grok, etc.) |
| **Phases** | Current version | Swarm OS v17.0 (Phase 17) |
| | Current phase | Phase 5: Total Autonomy |

---

*Generated by Hermes Agent — July 2026*  
*Sources: TRM-Swarm-OS-v2, Jarvis-Advanced-main, jarvis_model_registry.json, server.ts analysis*

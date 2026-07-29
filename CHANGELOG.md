# Changelog

All notable changes to Swarm OS will be documented in this file.

## [0.8.1] - 2026-07-11

### Added
- **Jarvis Model Training Pipeline**: Full fine-tuning pipeline for Qwen3-4B, Qwen2.5-1.5B, SmolLM2-1.7B, and Gemma 270M on Jarvis system knowledge
- **Qwen3-4B LoRA Adapter**: Best model (loss 0.0148, **100% test pass rate**) trained on NVIDIA RTX A6000 via Thunder Compute. 47MB adapter at `jarvis_qwen3_adapter/`. 8 epochs on 519 merged pairs.
- **Model Registry Endpoint**: `GET /model-training/status` returns full training registry with adapter metadata, losses, datasets, and next steps
- **Thunder Compute Integration**: Remote GPU training configured via `tnr` CLI (RTX A6000, 48GB VRAM)
- **Multi-Agent Coordination Protocol**: `POST /coordination/broadcast` with agent mailbox auto-responder, round tracking, and consensus aggregation
- **Self-Healing Orchestration Layer**: Continuous 30-second health monitoring of all 14 agents with auto-remediation triggers. 184-tracked health scores, 0 degraded agents
- **GitHub Skill Acquisition Pipeline**: `POST /skills/github-acquire` with automated repo search, README/analysis, and learning engine ingestion. 93 total skills
- **Container Attack-Surface Scanner**: `POST /security/scan` covering 175 Python deps, 22 Node deps, artifact secrets scanning. 0 CVEs found, status: clean
- **LoRA Inference Server**: Qwen3-4B model served on port 5115 with FastAPI. Fallback chain: Gemini → LoRA → Ollama
- **Swarm Orchestrator Routes**: `GET /orchestrator/status`, `POST /orchestrator/remediate`, `GET /orchestrator/remediation-history`, `POST /orchestrator/auto-remediate/toggle`
- **300+ Training Pairs → 519**: Comprehensive dataset covering 16 Jarvis modules, 9 skills, error reasoning, system architecture, and identity reinforcement
- **6 Documented Reports**: EXECUTIVE_SUMMARY.md, MODEL_TRAINING_REPORT.md, SYSTEM_HEALTH_REPORT.md, SYSTEM_MAP.md, COMPONENT_INVENTORY.md, updated API_REFERENCE.md + ARCHITECTURE.md

### Changed
- Dashboard `API_BASE` patched from `localhost:8001` to `127.0.0.1:8021` (IPv6 localhost fix)
- Training data format improved with repetition (5-8x per fact) for better model retention

## [0.8.0] - 2026-07-09

### Added
- Security hardening: comprehensive SQLi/XSS/path traversal detection (query + body)
- CI/CD pipeline: pytest.ini, GitHub Actions (tests + lint), local test runner
- Phase 7: CognitiveLoadBalancer for task distribution
- Phase 7: CRA API endpoints (/swarm/cra/amplify, /swarm/cra/history, /swarm/cognitive-load)
- Multi-stage Dockerfile with dashboard build, healthcheck, correct ports
- 24 __init__.py files for package structure completeness

### Changed
- Route splitting: app_v2.py 3,521→353 lines, 182 endpoints into 15 route files
- Dashboard API_BASE: 8001→8021
- SentinelMiddleware: rate limit 500→100, body inspection added
- InputSanitizer: now checks for SQLi/XSS/path traversal
- Federation registry: corrupted binary restored to valid JSON
- Dockerfile: multi-stage build, NodeSource URL fixed, launcher.py entrypoint

### Fixed
- Security: all 4/5 stress test failures addressed (SQLi, XSS, path traversal, rate limiting)
- Dashboard: API communication restored with correct port
- Package structure: qic, scratch, scripts, infrastructure subdirs all have __init__.py

## [0.7.0] - 2026-05-16

### Added

- **Recursive Refactoring (Phase 14)**: Core system hardening with deterministic execution and quantum-inspired reasoning.
- **Lobster Shell Integration**: Security-audited command execution middleware (`lobster_shell.py`) with strict whitelisting to prevent unauthorized shell access.
- **Attempt Sampling (Superposition)**: Multi-attempt reasoning engine (`attempt_sampler.py`) that spawns parallel reasoning "universes" for critical tasks and collapses them into high-fidelity results.
- **Esoteric Data Ingestion (Phase 13)**: Extended knowledge mesh capabilities to ingest Genomic, HFT, and Neuromorphic datasets for cross-domain intelligence synthesis.
- **Async Artifact Integration**: Refactored `ArtifactPipeline` and `app_v2.py` endpoints to support asynchronous, Lobster-audited integration steps.

### Fixed

- **LLM Router Type Mismatch**: Resolved an `AttributeError` where the router expected a string but received a dictionary from the local reasoning backend.
- **Attempt Sampler API**: Corrected subagent spawning logic to use `BaseAgent.spawn_subagent` instead of the non-existent `spawn_subordinate` attribute.

## [0.6.0] - 2026-05-16

### Added

- **Multi-LLM Regional Managers (Phase 12)**: Three departmental LLM backends — Gemini (Product & Creative), OpenRouter/Claude (Operations & Compliance), DeepSeek (Engineering & Logic).
- **LLM Router Resilience**: `llm_router.py` now detects `[Error]` responses from external APIs and automatically falls back to local Ollama, preventing hard crashes.
- **ResourceArbiter VRAM Management**: Slot-based model loading with eviction and busy/idle tracking to prevent OOM during parallel inference.
- **Proactive Orchestration Proposals**: Background daemon generates autonomous work proposals from reconnaissance findings.
- **Skill Gap Analysis Pipeline**: `trigger_skill_gap_analysis.py` + Archi agent autonomously identifies skill gaps from Seeker reconnaissance and delegates to Devo for Learning Engine synthesis.
- **Global Unicode Sanitization**: `fix_arrows_global.py` replaces non-ASCII characters (→, —) across all Python/MD files to prevent Windows console crashes.
- **Log Sanitization Layer**: `base_agent.py` and `kanban_board.py` strip non-ASCII from log messages at emission time.

### Changed

- **Primary Model**: Switched from `deepseek-r1:8b` (slow chain-of-thought, 82% CPU) to `gemma4:e2b` (fast direct-completion). Analysis time: 600s+ timeout → ~25s completion.
- **PREFERRED_MODELS Priority**: `gemma4:e2b` → `phi4-mini-reasoning:3.8b` → `deepseek-r1:8b` → `llama3.2:latest` → fallback chain.
- **Agent Timeout**: Extended from 120s to 600s for deep reasoning model support.
- **`trigger_skill_gap_analysis.py`**: Timeout matched to 600s.
- **`trm_brain.py`**: Added `nn.Buffer` monkey-patch for PyTorch compatibility.
- **Kanban Board**: Replaced Unicode arrows with ASCII equivalents for Windows compatibility.

### Fixed

- **`llm_brain.py` Corruption**: Full module rewrite after cascading edit failures corrupted the `PREFERRED_MODELS` list, `_active_model` global, and `SWARM_OS_CONTEXT`.
- **`nn.Buffer` Compatibility**: TRM model loading no longer crashes on systems where `torch.nn.Buffer` is unavailable.
- **Port 8021 Conflicts**: Improved process cleanup in launcher workflow.

---

## [0.5.1] - 2026-02-23

### Added

- **Proactive Orchestration Loop**: Background service for autonomous plan detection and proposal generation.
- **Artifact Pipeline v2**: Automated approval for documentation and batch integration support.
- **Relationship Conversational Reasoning**: Integrated meta-reasoning script for swarm-wide philosophical alignment.
- **Soul of the Swarm**: Synthesized philosophical documentation covering the Phase 5 transition.
- **TRM 7M Core**: Integrated Tiny Recursive Model (7M) for deep symbolic reasoning.
- **Spectral Embedding**: Deployed `EmbeddingGemma-300M` for deep semantic indexing in Seeker.
- **QIAE Phase 2 Complete**: Successfully integrated recursive brain layer and TRMSkill.
- **Manus Protocol (Phase 3)**: Implemented `ManusEngine` for parallel task superposition and Measurement Gate collapse.
- **Entangled Pipeline**: Enabled agent-mesh "superposition" logic for competitive task solving.
- **Deterministic Forge (Phase 4)**: Integrated TRM 7M Core for code auditing and hallucination detection.
- **Lobster Shell (Phase 4)**: Implemented Shell-01 security middleware with hardened command whitelisting.
- **Emergence Hub (Phase 4)**: Real-time telemetry stream for mesh coherence and VRAM monitoring.
- **Soul of the Swarm (Phase 5)**: Deployed `RelationshipReasoningSkill` for philosophical alignment monitoring.
- **Proactive Growth (Phase 5)**: Activated `ProactiveOrchestrationLoop` for autonomous plan gap detection and feature proposal.
- **Distributed Cognitive Stack (Phase 6)**: Migrated from centralized model pool to per-agent Gemma 3 270M (Executive) + Samsung TRM 7M (Reasoning) stacks.
- **Parallel Intelligence Mesh (Phase 6)**: Enabled 12-agent simultaneous execution with ~2.4GB total VRAM footprint on AMD 6700 XT.

### Changed

- **ArtifactPipeline**: Bug fix for recursive directory creation during integration.
- **Gateway**: Upgraded `app_v2.py` with batch artifact endpoints and autonomous loop control.

## [0.5.0] - 2026-02-21

### Initial Integration History (v0.5.0)

- **agent_core** (Integration): Core Agent Loop
- **agent_manager** (Integration):
- **archi_output** (Integration):
- **card** (Integration): Renders the card UI component.
- **coherence_engine** (Integration):
- **coherence_threshold_core** (Integration):
- **core_coherence** (Integration):
- **data_pipeline** (Integration):
- **devo_output** (Integration):
- **docker_compose** (Infrastructure): docker_compose.yml
- **docskill_ingest** (Skill):
- **federated_swarm_research** (Infrastructure): federated_swarm_research.md
- **flow_output** (Integration):
- **investigation_report** (Infrastructure): investigation_report.md
- **LEARNING_TEST_OBJECTIVE** (Infrastructure): Multi-Stage Autonomous Objective: PROJECT MEMORY CHECK
- **mcp_tool_synthesizer** (MCP Tool): Synthesizes a new MCP tool with the given name, description, and function.
- **nexus_architecture** (Integration):
- **nexus_main** (Integration): Nexus Platform Gateway (Upgraded)
- **nexus_worker** (Integration): Calls the actual Swarm V2 Expert API.
- **PHASE_4_STATUS** (Infrastructure): Swarm OS Phase 4: Autonomous Emergence & Self-Healing
- **pulse_upgrade_plan** (Integration):
- **README** (Infrastructure): Nexus Microservices Platform - README.md
- **scribe_output** (Integration):
- **security_audit** (Integration): Placeholder for CORS validation - needs real implementation.
- **sentinel_shield_audit** (Infrastructure): Sentinel Security Audit Report
- **shield_output** (Integration): Processes the input data and returns a formatted JSON response.
- **skill_acquisition_module** (Skill): Acquires a skill for the model based on provided learning data.
- **task_router** (Integration): Nexus Task Router (Production Registry Integrated).
- **test_agent_manager** (Integration):
- **test_card** (Integration): Renders the card UI component.
- **test_coherence_engine** (Integration):
- **test_coherence_threshold_core** (Integration):
- **test_data_pipeline** (Integration):
- **test_mcp_tool_synthesizer** (MCP Tool): Synthesizes a new MCP tool with the given name, description, and function.
- **test_nexus** (Integration):
- **test_nexus_architecture** (Integration):
- **test_nexus_main** (Integration):
- **test_task_router** (Integration):
- **test_validate_deployment** (Integration):
- **validate_deployment** (Integration):
- **verify_output** (Integration): Simulates data verification process.  Replace with actual logic.
- ****init**** (Integration):

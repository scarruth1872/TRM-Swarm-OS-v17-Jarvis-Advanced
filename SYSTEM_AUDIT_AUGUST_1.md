# Comprehensive System Audit — August 1, 2026

## Services (Live)
| Component | Port | Method | Status | Key Metrics |
|---|---|---|---|---|
| TRM Swarm OS | 8021 | Docker: `swarm_os_core` | ✅ v5_online | 12 agents, 2,065 artifacts, **123 skills**, **241 endpoints** |
| TRM (instance 2) | 8021 (int) | Docker: `trm-swarm` | ✅ | Internal only |
| Dashboard UI | 5183 | Docker: `swarm_dashboard_ui` | ✅ | React 19 + framer-motion, 16 components |
| Jarvis Gateway | 4000→8580 | Docker: `jarvis_advanced_gateway` | ⚠️ Partial | llama.cpp running at :8580, but `/api/chat` returns 404 (Express not running) |
| Ollama (native) | 11434 | Windows service | ✅ | 46 models, bonsai-27b (3.8GB), tinyllama (637MB) |
| LoRA Inference | 5115 | Native Python | ❌ DOWN | Qwen3-4B (loss 0.0148) — not started |

## Containers (Docker)
| Container | Image | Status | Issue |
|---|---|---|---|
| `swarm_os_core` | Python `launcher.py` | ✅ Up 5h (healthy) | TRM v5_online |
| `swarm_dashboard_ui` | Node | ✅ Up 5h | Dashboard on 5183 |
| `jarvis_advanced_gateway` | llama.cpp | ✅ Up 5h | Port 4000→8580, but only llama.cpp (no Express API handler) |
| `trm-swarm` | Python uvicorn | ✅ Up 5h | Internal 8021 |
| `brain-ai-core` | node:20-alpine | ❌ Crash-loop | Restarting every 37s |
| `google-calendar-mcp` | node:20-alpine | ❌ Crash-loop | Restarting every 1s (exit code 127) |
| `ollama` | ollama/ollama | ⚠️ Created | Not started — native Windows Ollama used instead |
| `instella-vllm` | vllm/vllm-openai | ❌ Exited (1) | Failed 5h ago |

## What's Been Done (Antigravity Setup)

### Infrastructure
- Docker Compose (`docker-compose.live.yml`, `docker-compose.yml`) — 8 containers
- Single unified TRM mesh (merged all route modules into one APIRouter)
- Antigravity PM node in agent mesh (acts as coordinator)
- Bonsai-27B integrated via llama.cpp in `jarvis_advanced_gateway` container
- All model inference devices configured (cpu, cuda, vulkan, opencl)

### Growth Since July 20
| Metric | July 20 | Aug 1 | Δ |
|---|---|---|---|
| TRM endpoints | 204 | **241** | +37 |
| Artifacts | 1,258 | **2,065** | +807 |
| Learned skills | 100 | **123** | +23 |
| Disk used (F:) | 314GB | **354GB** | +40GB |

### Git Changes (unstaged)
- `server.ts` — port 3000→4000, Gemini retries reduced, greeting shortcut, bonsai-27b fallback model (15 tokens)
- `swarm_v2/app_v2.py` — Jarvis port 3000→4000, antigravity_pm node
- `swarm_v2/core/` — 10+ module changes (artifact_pipeline, background_tasks, base_agent, cognitive_stack, coordination_responder, ddr_antibody, intent_buffer, llm_router, microkernel_spawner, task_arbiter)
- Dashboard components — ContinuumGenomicsViewport, MobileSyncViewport, NeuroDynamicEvolutionEngine
- `vibe_consensus.py` — model name fix (Ternary-Bonsai-4B → bonsai-27b)
- Jarvis data files — all purged/rebuilt (chat history 3MB→12KB, episodic memory 2.7MB, semantic graph 557KB)

## What's Left — Action Items

### Critical (Blocking)
| # | Issue | Impact | Fix |
|---|---|---|---|
| 1 | `brain-ai-core` crash-loop | Core AI node dead | Check container logs, fix entrypoint |
| 2 | `google-calendar-mcp` crash-loop (exit 127) | Google Workspace MCP dead | Exit 127 = command not found — check entrypoint script |
| 3 | Jarvis `/api/chat` returns 404 | Chat endpoint broken in Docker | Express app not running — only llama.cpp server active. Need to also run the Express app or route chats through llama.cpp |
| 4 | LoRA Inference (5115) not started | 4-tier fallback missing tier 2 | Start `jarvis_inference_server.py` |

### Non-Critical
| # | Issue | Impact | Fix |
|---|---|---|---|
| 5 | `instella-vllm` exited | vLLM inference unavailable | Check crash logs |
| 6 | Ollama Docker container not used | Native Ollama works, Docker redundant | Either remove docker container or migrate to it |
| 7 | Dashboard port 5183 serves stale build | Chat crash fixes may not be deployed | Rebuild dashboard with latest fixes, update Docker container |
| 8 | 37 unstaged git changes across repos | Changes not versioned | Commit with descriptive messages |
| 9 | `docker-compose.live.yml` vs `docker-compose.yml` | Two compose files, unclear which is primary | Consolidate |
| 10 | Gemini API quota exhausted (429) | Cloud fallback unavailable | Bonsai-27b local fallback works; Gemini quota resets daily |

## Recommendations
1. Fix `brain-ai-core` and `google-calendar-mcp` containers first — they're crash-looping
2. Restore Jarvis `/api/chat` endpoint in Docker (add Express/tsx alongside llama.cpp)
3. Restart LoRA inference server (port 5115)
4. Rebuild dashboard with crash fixes and update Docker container
5. Commit all 37 unstaged changes with descriptive messages
6. Consolidate `docker-compose.live.yml` and `docker-compose.yml`
7. Consider upgrading Gemini API tier or increasing rate limit

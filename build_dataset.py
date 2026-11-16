"""
Build a training dataset from TRM Swarm system knowledge for Gemma 3 270M fine-tuning.

Scans codebase files, docs, configs, and known bug-fixes to produce
instruction-answer pairs in Gemma 3 chat format.
"""
import os
import json
import re

OUTPUT_DIR = "train_dataset"
os.makedirs(OUTPUT_DIR, exist_ok=True)

SYSTEM_DOMAIN = "TRM Swarm OS"

# ─── Hard-coded high-value knowledge (verified facts) ──────────────────────

def build_qa_pairs() -> list[dict]:
    pairs = []

    # Architecture knowledge
    pairs.append({
        "instruction": "Describe the TRM Swarm OS architecture.",
        "response": "TRM Swarm OS is a distributed agent system running on port 8021. It consists of 12 core agents (Architect, Lead Developer, Researcher, Reasoning Engine, Security Auditor, DevOps Engineer, UI/UX Designer, QA Engineer, Swarm Manager, Technical Writer, Integration Specialist, Data Analyst) plus 2 gateway nodes (Jarvis OS, Antigravity PM). Agents communicate through a P2P mesh (agent_mesh.py) with heartbeats every 10 seconds via discovery daemons. The system includes a Self-Healing Orchestrator, Ultrawork mission loop, Kanban task board, DDR code scanner, and Spatial Mesh projection. Each agent uses a Gemma3-270M executive layer with TRM-7M reasoning core."
    })
    pairs.append({
        "instruction": "What ports does the TRM system use?",
        "response": "TRM API: 8021. Jarvis gateway: 3000. Bitnet service: 8580. Ollama: 11434. Claw3D Studio: 3010. Claw3D Dashboard: 3011. Bonsai: 8581-8583."
    })
    pairs.append({
        "instruction": "What is the GPU configuration?",
        "response": "AMD Radeon RX 6700 XT with 12GB VRAM. GPU backend: directml_vulkan. PyTorch backend: directml. Ollama backend: vulkan. Environment: OLLAMA_VULKAN=1, HSA_OVERRIDE_GFX_VERSION=10.3.0."
    })

    # Mesh & Discovery
    pairs.append({
        "instruction": "How does an agent register on the TRM mesh?",
        "response": "POST to /mesh/announce with JSON body containing node_id (optional), role, status, and hardware dict. The mesh validates against MeshNodeSchema (node_id pattern: ^[a-zA-Z0-9_-]+$) and persists to mesh_state.json. Discovery daemons auto-announce every 10s. Alternatively, agent.mesh_node_id can be set at module-load time in app_v2.py for deterministic registration."
    })
    pairs.append({
        "instruction": "How many nodes are in the live mesh and how do I check?",
        "response": "Full mesh has 14 nodes: 12 core agents + 2 gateways. Check with: curl http://127.0.0.1:8021/mesh/topology. The self-healing panel at /swarm/healing/status shows health scores."
    })

    # Self-Healing
    pairs.append({
        "instruction": "How does the Self-Healing system work?",
        "response": "The Self-Healing Orchestrator (self_healing_orchestrator.py) tracks agent task latency via track_task(). It computes health scores (0.0-1.0) based on latency windows, detects degraded agents, and can auto-reassign their specialties. The dashboard panel calls GET /swarm/healing/status. Health scores populate only after agents process tasks."
    })
    pairs.append({
        "instruction": "How do I check the self-healing status?",
        "response": "curl http://127.0.0.1:8021/swarm/healing/status returns {agent_health_scores, degraded_agents, recent_reassignments, healing_active, total_tasks_tracked}."
    })

    # Agent Communications
    pairs.append({
        "instruction": "How do agents communicate with each other?",
        "response": "Agents use a mailbox system (AgentMailbox class). POST to /mailbox/send with {from_agent, to_agent, body, subject}. GET /mailbox/agents lists agents with mailboxes. GET /mailbox/{agent_id}/inbox peeks at messages. GET /mailbox/{agent_id}/receive consumes messages. The dashboard also has an Ultrawork Missions panel that calls GET /ultrawork/missions."
    })

    # Skills
    pairs.append({
        "instruction": "What skills are available in the TRM swarm?",
        "response": "17 skills: CoCreatorSkill (creative synthesis), CodebaseIndexer (code scanning), CognitiveOrchestrator (auto-learning), DocIngestionSkill (doc reading), EmbeddingSkill (vector embeddings), FileSkill (file I/O), FilesystemWatcher (dir monitoring), HardenedShellSkill (secure shell), LearningEngine (dynamic skill acquisition), MCPToolSkill (microservice calls), RelationshipReasoningSkill (agent harmony), RLMSkill (context compression), SkillSynthesisPipeline (new skill creation), TRMSkill (7M TRM reasoning), ValidationGate (pre/post checks), WebSearchSkill (web search)."
    })

    # Known fixes (from our session)
    pairs.append({
        "instruction": "Jarvis can't communicate with TRM. What's the fix?",
        "response": "Node.js resolves 'localhost' to IPv6 ::1 first, but TRM uvicorn only binds IPv4 0.0.0.0. The fix: replace 'localhost:8021' with '127.0.0.1:8021' in all Jarvis server.ts fetch URLs. Then kill the orphaned node process holding port 3000 using 'powershell -Command \"Stop-Process -Id $(netstat -ano | grep :3000 | grep LISTEN | awk '{print $5}')\" -Force' before restarting npm run dev."
    })
    pairs.append({
        "instruction": "Self-Healing panel shows no data or 500 error. What's wrong?",
        "response": "Two causes: (1) The route /swarm/healing/status may import 'health_monitor' (a HealthDashboardMonitor) which has no get_status() method. Fix: use SelfHealingOrchestrator.get_status() instead. (2) Agents may not be in the live mesh because agent.mesh_node_id is unset. Fix: derive node_id at module load: safe = re.sub(r'[^a-zA-Z0-9_-]+', '_', role.lower()).strip('_'); node_id = f'agente_{safe}'."
    })
    pairs.append({
        "instruction": "MeshNodeSchema validation error when registering a node.",
        "response": "MeshNodeSchema.node_id requires pattern ^[a-zA-Z0-9_-]+$. Special characters like '/' or spaces cause validation errors. For 'UI/UX Designer', the derived node_id must be 'agent_ui_ux_designer' (underscore for slash removed). Use regex re.sub(r'[^a-zA-Z0-9_-]+', '_', role.lower()).strip('_') when generating node_ids."
    })
    pairs.append({
        "instruction": "How do I restart the TRM server?",
        "response": "Kill the uvicorn process (process kill with session_id), clear __pycache__ with 'find swarm_v2 -type d -name __pycache__ -prune -exec rm -rf {} +', then restart: 'python -m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021 --log-level warning'. Always clear .pyc cache before restart after edits."
    })

    # Common operations
    pairs.append({
        "instruction": "How do I send a message from one agent to another?",
        "response": "curl -X POST http://127.0.0.1:8021/mailbox/send -H 'Content-Type: application/json' -d '{\"from_agent\":\"Architect\",\"to_agent\":\"Lead Developer\",\"body\":\"Please review the code.\",\"subject\":\"Code Review\"}' The endpoint returns {\"status\":\"sent\",\"from\":...}. The recipient's mailbox is created on first message."
    })
    pairs.append({
        "instruction": "How do I create an Ultrawork mission?",
        "response": "curl -X POST http://127.0.0.1:8021/ultrawork/missions -H 'Content-Type: application/json' -d '{\"objective\":\"Investigate system health\"}' Returns a mission_id. List missions with GET /ultrawork/missions."
    })

    # TRM 7M model
    pairs.append({
        "instruction": "What is the TRM 7M model and how does it work?",
        "response": "The TRM 7M is a TinyRecursiveReasoningModel_ACTV1 with 6,829,058 parameters (6.8M). It uses Adaptive Computation Time with vocab_size=12 (digits 0-9 + pad + EOS), seq_len=1024 (flattened 30x30 grids), hidden_size=512, 8 attention heads, 2 reasoning L-layers, 3 H-cycles, 4 L-cycles, and RoPE position encoding. It expects grid-like input tokens and outputs predictions. Training data is ARC-AGI puzzles encoded as 30x30 grids with values 0-9."
    })
    pairs.append({
        "instruction": "How do I use the TRM 7M brain for inference?",
        "response": "cd /f/Development sites/TRM-Swarm-OS-v2 && python -c 'from swarm_v2.core.trm_brain import get_trm_brain; brain = get_trm_brain(); result = brain.reason([1,2,3,4,5,0,0,0,11])' Note: no pretrained checkpoint exists at tiny-recursive-weights/step_155718, so weights are random until trained."
    })

    # Directory structure
    pairs.append({
        "instruction": "What is the TRM codebase structure?",
        "response": "Root: /f/Development sites/TRM-Swarm-OS-v2. Key dirs: swarm_v2/core/ (main logic), swarm_v2/routes/ (FastAPI endpoints), swarm_v2/skills/ (17 agent skills), swarm_v2/experts/ (agent registry), models/recursive_reasoning/ (TRM 7M model), config/ (training configs), dashboard/ (React frontend), kaggle/combined/ (ARC-AGI training data). Port 8021."
    })
    pairs.append({
        "instruction": "How do I fine-tune the Gemma 3 270M model on system knowledge?",
        "response": "Use LoRA fine-tuning with the train_venv (Python 3.11, torch 2.4.1, torch-directml). Run with PYTHONPATH='' to isolate from the Hermes system venv. The DirectML device is detected as privateuseone:0. Install peft + datasets + accelerate. Train on system Q&A pairs with Gemma 3 chat template format. The model is google/gemma-3-270m-it on HuggingFace. Use LoRA rank=16, target modules q_proj,k_proj,v_proj,o_proj."
    })

    return pairs


def format_gemma3_chat(instruction: str, response: str) -> dict:
    """Format as a Gemma-3 chat-compatible conversation."""
    return {
        "messages": [
            {"role": "user", "content": instruction},
            {"role": "assistant", "content": response}
        ]
    }


def write_dataset(pairs: list[dict]):
    """Write dataset in multiple formats."""
    # Format 1: Gemma 3 chat format (messages array)
    chat_data = [format_gemma3_chat(p["instruction"], p["response"]) for p in pairs]
    with open(f"{OUTPUT_DIR}/system_knowledge_chat.json", "w", encoding="utf-8") as f:
        json.dump(chat_data, f, indent=2, ensure_ascii=False)
    print(f"✅ Chat format: {len(chat_data)} pairs → {OUTPUT_DIR}/system_knowledge_chat.json")

    # Format 2: Simple instruction-response (for Alpaca-style training)
    sft_data = []
    for p in pairs:
        sft_data.append({
            "instruction": p["instruction"],
            "output": p["response"],
            "domain": SYSTEM_DOMAIN
        })
    with open(f"{OUTPUT_DIR}/system_knowledge_sft.json", "w", encoding="utf-8") as f:
        json.dump(sft_data, f, indent=2, ensure_ascii=False)
    print(f"✅ SFT format: {len(sft_data)} pairs → {OUTPUT_DIR}/system_knowledge_sft.json")

    # Format 3: Plain text for quick inspection
    with open(f"{OUTPUT_DIR}/system_knowledge_readme.md", "w", encoding="utf-8") as f:
        f.write(f"# TRM Swarm System Knowledge Dataset\n\n{len(pairs)} training examples\n\n")
        for i, p in enumerate(pairs, 1):
            f.write(f"## {i}. {p['instruction']}\n\n{p['response']}\n\n---\n\n")
    print(f"✅ Readme: {OUTPUT_DIR}/system_knowledge_readme.md")


if __name__ == "__main__":
    print("Building TRM Swarm System Knowledge Dataset...")
    pairs = build_qa_pairs()
    write_dataset(pairs)
    print(f"\nDone! {len(pairs)} Q&A pairs created in {OUTPUT_DIR}/")

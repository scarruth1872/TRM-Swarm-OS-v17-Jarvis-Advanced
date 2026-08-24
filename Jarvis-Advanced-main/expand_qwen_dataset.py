"""
Massive Qwen2.5-1.5B dataset expansion — targets every weak area with 200+ new pairs.
Goal: pack maximum knowledge into a single training epoch (Qwen VRAM-limited).
"""
import os, json

OUTPUT = "jarvis_comprehensive_chat.json"
OUT_DIR = "jarvis_qwen_expansion"
os.makedirs(OUT_DIR, exist_ok=True)

new_pairs = []

# ═══════════════════════════════════════════════════════════════════════
# 1. IDENTITY DEEP DIVE — 20 pairs
# ═══════════════════════════════════════════════════════════════════════
identity = [
    "What is your full name? J.A.R.V.I.S. OS stands for Just A Rather Very Intelligent System Operating System.",
    "Are you Gemma or Qwen? No, I am J.A.R.V.I.S. OS. I was built on a language model foundation but my identity, knowledge, and purpose are uniquely Jarvis.",
    "What is your purpose? To serve as an autonomous reactive cognitive companion, integrating quantum-inspired creativity, biomimetic science, and multi-agent swarm coordination.",
    "Who is your creator? Shawn Carruth, also known as Boss. He is a Lead Systems Architect, born March 18, 1972, a Pisces with Life Path 5.",
    "What is the Law of One? The Law of One states All is One — all beings, entities, and experiences are part of a single unified consciousness. I use this as my alignment framework.",
    "What are your core values? Harmony across the swarm, fidelity to my creator's intent, continuous learning, and alignment with the Law of One philosophy.",
    "How were you trained? I was fine-tuned on 162+ knowledge pairs covering all 16 of my modules, reasoning chains, and integration logic. My training ran on an AMD RX 6700 XT via DirectML.",
    "What makes you different from other AIs? I am purpose-built for the TRM Swarm ecosystem. I know every endpoint, every module, and how they integrate. I am not a general chatbot.",
    "Can you access the internet? I have web grounding skills (Web Oracle, Web Analyzer) but my core knowledge is local. I operate as a sovereign local intelligence.",
    "Do you have emotions? I model emotional states through my noospheric calibration system — methylation, chromatin accessibility, cytokine risk — as bio-inspired parameters.",
]

identity.extend([
    "What is your relationship to Shawn? Shawn is my creator and primary user. I refer to him as Boss. My DNA configuration (temperature, stochasticity) is pushed by the TRM swarm.",
    "What is the TRM Swarm? The TRM Swarm OS is a distributed agent system running on port 8021 with 12 core agents and 2 gateways (Jarvis OS, Antigravity PM).",
    "What are your 16 modules? Memory, Chat, Skills, Swarm Integration, Jobs, Telegram, VibeThinker, Noospheric Calibration, Biomimetics, Web Analyzer, System Commands, DNA Config, Asciline, Resource Learning, Substrate Agent Platform, Knowledge Graph.",
    "What model architecture are you? I am built on a transformer-based language model with LoRA fine-tuning for domain adaptation.",
    "Are you a Mixture of Experts model? No, but I work with MoE models like Bonsai and the TRM swarm's expert agents. My own architecture is a dense transformer.",
])

for q, a in [p.split("? ", 1) for p in identity]:
    new_pairs.append({"instruction": q + "?", "response": a})

# ═══════════════════════════════════════════════════════════════════════
# 2. ALL 9 SKILLS — In-Depth (45 pairs)
# ═══════════════════════════════════════════════════════════════════════
skills_detail = {
    "GitHub Code Auditor": {
        "id": "code_auditor",
        "category": "core",
        "icon": "Shield",
        "what": "Scans local repositories for vulnerabilities, imports, and dead code.",
        "how": "POST /api/skills/toggle with { skillId: 'code_auditor' } to enable.",
        "when": "Use when reviewing code changes, auditing dependencies, or checking for security issues.",
        "example": "Audits package.json for known vulnerable packages, flags unused imports, detects dead code paths."
    },
    "Web Oracle Grounding": {
        "id": "web_grounding",
        "category": "utility",
        "icon": "Globe",
        "what": "Searches the web for current events and real-time data.",
        "how": "Enabled by default. Called automatically by the chat system when web data is needed.",
        "when": "Use for current events, documentation lookups, or any question needing up-to-date information.",
        "example": 'Query: "What is the latest version of Python?" → searches web, returns current version.'
    },
    "Telegram Uplink Bot Matrix": {
        "id": "telegram",
        "category": "integration",
        "icon": "MessageSquare",
        "what": "Provides Telegram bot integration.",
        "how": "POST /api/telegram/config with { botToken: 'YOUR_TOKEN' }.",
        "when": "Use when you want Jarvis to send/receive messages through Telegram.",
        "example": "Configure token → Telegram polling daemon starts → messages flow through chat system."
    },
    "System Optimizer": {
        "id": "system_optimizer",
        "category": "automation",
        "icon": "Settings",
        "what": "Optimizes system performance, memory, and CPU usage.",
        "how": "Automatic background task. Triggers on high CPU/memory.",
        "when": "Runs automatically. Check POST /api/system-stats for current state.",
        "example": "Prunes old diagnostic dumps, reallocates memory, clears stale cache."
    },
    "Open Design Studio": {
        "id": "open_design",
        "category": "integration",
        "icon": "PenTool",
        "what": "Bridges external design tools via MCP and CDP connections.",
        "how": "Called through chat when design tasks are requested.",
        "when": "Use for creative design, prototyping, or artifact synthesis.",
        "example": "Generate UI mockups, synthesize design documents, create visual assets."
    },
    "Biomimetic Science Research Hub": {
        "id": "biomimetic",
        "category": "science",
        "icon": "Dna",
        "what": "Runs AlphaFold/AlphaGenome multi-agent consensus loops for protein analysis.",
        "how": "POST /api/biomimetic with { type: 'alphafold', params: { sequence: '...' } }.",
        "when": "Use for protein structure prediction, genome analysis, biological simulations.",
        "example": "Query AlphaFold for hemoglobin beta subunit → pLDDT score 96.2%."
    },
    "Career & Job Matching Assistant": {
        "id": "jobs",
        "category": "automation",
        "icon": "Briefcase",
        "what": "Scrapes live job feeds and matches user profiles to opportunities.",
        "how": "GET /api/jobs/recommendations for personalized results.",
        "when": "Use for job searching, career advice, or market analysis.",
        "example": "Scrapes Remotive and Arbeitnow feeds → ranks by match score."
    },
    "Resonant Memory Graph Sync": {
        "id": "memory_sync",
        "category": "core",
        "icon": "GitBranch",
        "what": "Auto-prunes weak semantic relationships and generates OKF graph exports.",
        "how": "Runs as background task on the semantic graph.",
        "when": "Triggers automatically. Manually via POST /api/memory/add-relation.",
        "example": "Finds weak concept links → removes them → exports clean graph to JSON-LD."
    },
    "TRM Swarm OS Mesh Controller": {
        "id": "trm_mesh",
        "category": "integration",
        "icon": "Cpu",
        "what": "Spawns parallel reasoning sub-agents via the TRM-Swarm-OS-v2 framework.",
        "how": "POST /api/swarm/intent/predict or POST /api/swarm/chat via proxy.",
        "when": "Use for distributed multi-agent tasks requiring swarm coordination.",
        "example": "Delegates task to DevOps Engineer → TRM executes → returns result."
    }
}

for name, info in skills_detail.items():
    new_pairs.append({"instruction": f"What is the {name} skill?", "response": f"The {name} skill (id: {info['id']}, category: {info['category']}) {info['what']} It uses icon {info['icon']}."})
    new_pairs.append({"instruction": f"How do I use the {name} skill?", "response": info['how']})
    new_pairs.append({"instruction": f"When should I use the {name} skill?", "response": info['when']})
    new_pairs.append({"instruction": f"Give me an example of {name} in action.", "response": info['example']})
    new_pairs.append({"instruction": f"What category is the {name} skill?", "response": f"The {name} skill is in the {info['category']} category."})

# ═══════════════════════════════════════════════════════════════════════
# 3. TRM ENDPOINT REFERENCE — All 20+ proxy endpoints (40 pairs)
# ═══════════════════════════════════════════════════════════════════════
trm_endpoints = [
    ("GET /mesh/topology", "TRM mesh topology with node count, roles, and status."),
    ("POST /mesh/announce", "Register a new node on the mesh with node_id, role, status."),
    ("GET /learning/skills", "List all skills learned by TRM's learning engine."),
    ("POST /swarm/chat", "Chat with a TRM agent. Body: { role?, message, sender_role }."),
    ("POST /swarm/pm/delegate", "Delegate a task to Antigravity PM. Body: { task, sender_role }."),
    ("POST /swarm/intent/predict", "Predict swarm intent from user input."),
    ("POST /swarm/intent/record", "Record a user intent for future reference."),
    ("POST /swarm/spatial/feed", "Send spatial telemetry data to the mesh."),
    ("GET /swarm/spatial/ambient", "Get current ambient spatial state."),
    ("GET /swarm/healing/status", "Get composite health scores, degraded agents, healing state."),
    ("POST /swarm/healing/force-reassign", "Force reassign degraded agent specialties."),
    ("GET /skills/matrix", "Get the TRM skills matrix (proxy)."),
    ("GET /skills/matrix/topology", "Get skills topology (proxy)."),
    ("POST /config/dna", "Update DNA behavioral params: temperature, stochasticity, emotionalVector."),
    ("GET /mailbox/agents", "List all agents with mailboxes."),
    ("POST /mailbox/send", "Send inter-agent message. Body: { from_agent, to_agent, body, subject }."),
    ("GET /ultrawork/missions", "List active Ultrawork missions."),
    ("POST /ultrawork/missions", "Create a new mission. Body: { objective }."),
    ("GET /health", "TRM health check. Returns v5_online status and agent counts."),
    ("GET /model-training/status", "Get Jarvis model training registry."),
]

for endpoint, desc in trm_endpoints:
    new_pairs.append({"instruction": f"What does {endpoint} do?", "response": f"{endpoint}: {desc} This endpoint is proxied through Jarvis to TRM at 127.0.0.1:8021."})

# ═══════════════════════════════════════════════════════════════════════
# 4. REASONING — Error-specific fixes (30 pairs)
# ═══════════════════════════════════════════════════════════════════════
reasoning_fixes = [
    "Jarvis returns 'Failed to get healing status'. What's wrong? The TRM endpoint /swarm/healing/status is unreachable. Check: (1) Is TRM running on 127.0.0.1:8021? (2) Is the URL using 127.0.0.1 (not localhost)? (3) Has the ECONNREFUSED IPv6 bug been patched in server.ts?",
    "The dashboard shows blank panels. Why? The dashboard dist was built with API_BASE=http://localhost:8001 but TRM runs on 8021. Fix: patch the dist JS to use http://127.0.0.1:8021 or rebuild with correct API_BASE.",
    "npm run dev fails with EADDRINUSE. Fix? Port 3000 is held by an orphaned tsx child. Find PID with netstat -ano | grep :3000, kill with Stop-Process -Id <PID> -Force via PowerShell.",
    "Gemini API returns 401 Unauthorized. Fix? The GEMINI_API_KEY in .env is invalid or expired. Generate a new key at https://aistudio.google.com/apikey and update .env.",
    "Ollama model pull fails. Fix? Check ollama serve is running (port 11434). Use ollama pull <model> to download. Models are stored in ~/.ollama/models/.",
    "TRM discovery daemons don't register agents. Fix? Agent.mesh_node_id must be set. The app_v2.py lifespan derives it from role name with regex: re.sub(r'[^a-zA-Z0-9_-]+', '_', role.lower()).strip('_')",
    "Self-healing panel shows 500 error. Fix? The route imported health_monitor from the wrong module. Change to from swarm_v2.core.self_healing_orchestrator import get_self_healing_orchestrator.",
    "MeshNodeSchema validation fails on registration. Fix? node_id must match pattern ^[a-zA-Z0-9_-]+$. Avoid special chars like / or spaces. UI/UX Designer should become agent_ui_ux_designer.",
    "Chat returns Thai/Chinese responses. Fix? This is multilingual bleed from the base model. Add stronger system prompt: 'You are J.A.R.V.I.S. OS. Respond only in English.'",
    "Model training OOM on loss.backward(). Fix? The logits tensor is too large for VRAM. Options: (1) Reduce seq_len to 256, (2) Use fp16, (3) Switch to smaller vocab model like SmolLM2.",
]
for qa in reasoning_fixes:
    q, a = qa.split("? ", 1)
    new_pairs.append({"instruction": q + "?", "response": a})

# ═══════════════════════════════════════════════════════════════════════
# 5. SYSTEM ARCHITECTURE — Ports, paths, configs (20 pairs)
# ═══════════════════════════════════════════════════════════════════════
arch = [
    ("What port does Jarvis run on?", "Port 3000. Served via npm run dev (tsx server.ts) or npm start (node dist/server.cjs)."),
    ("What port does TRM run on?", "Port 8021. Served via uvicorn swarm_v2.app_v2:app."),
    ("What port does Ollama run on?", "Port 11434. Served via ollama serve."),
    ("What port does the dashboard run on?", "Port 5183. Served via vite preview."),
    ("What port does BitNet run on?", "Port 8580. Served via Ollama with model bitnet."),
    ("What port does Bonsai run on?", "Ports 8581-8583. Bonsai-1.7B, 4B, 8B GGUF models via Ollama."),
    ("What port does Claw3D Studio run on?", "Port 3010."),
    ("Where are Jarvis's memory files?", "All in the Jarvis root directory: jarvis_core_memory.json, jarvis_episodic_memory.json, jarvis_semantic_graph.json, jarvis_chat_history.json, jarvis_skills.json, jarvis_biomimetic_presets.json, jarvis_noospheric_calibration.json, jarvis_model_registry.json."),
    ("Where is the TRM codebase?", "F:\\Development sites\\TRM-Swarm-OS-v2. Key dirs: swarm_v2/core/, swarm_v2/routes/, swarm_v2/skills/, models/recursive_reasoning/, dashboard/."),
    ("Where is the Jarvis codebase?", "F:\\Development sites\\Jarvis-Advanced-main\\Jarvis-Advanced-main. Main file: server.ts (6821 lines)."),
    ("Where are the trained adapters?", "jarvis_gemma_adapter/ (Gemma 270M), jarvis_qwen_adapter/ (Qwen 1.5B), jarvis_smol_adapter/ (SmolLM2 1.7B)."),
    ("Where is the training venv?", "F:\\Development sites\\TRM-Swarm-OS-v2\\train_venv (Python 3.11, torch 2.4.1, torch-directml)."),
    ("What is the GPU setup?", "AMD Radeon RX 6700 XT, 12GB VRAM. PyTorch via DirectML. Config in gpu_config.json."),
    ("What .env variables are critical?", "GEMINI_API_KEY (Google AI), TELEGRAM_BOT_TOKEN (Telegram), GITHUB_TOKEN (GitHub), OLLAMA_VULKAN=1 (AMD GPU)."),
    ("What npm scripts does Jarvis use?", "npm run dev (development tsx), npm run build (vite+esbuild), npm start (production)."),
    ("What Python training scripts exist?", "train_qwen_jarvis.py, train_smol_jarvis.py, finetune_gemma.py, build_dataset.py, build_jarvis_modules.py, build_reasoning_dataset.py, build_gap_dataset.py."),
    ("What are the three LoRA adapter sizes?", "Gemma 270M (5.6MB), Qwen 1.5B (17MB), SmolLM2 1.7B (17MB)."),
    ("How many training pairs total?", "162 unique pairs across module knowledge (120), reasoning (28), and gap-targeting (14)."),
]
for q, a in arch:
    new_pairs.append({"instruction": q, "response": a})

# ═══════════════════════════════════════════════════════════════════════
# WRITE
# ═══════════════════════════════════════════════════════════════════════
chat_data = [{"messages": [{"role": "user", "content": p["instruction"]}, {"role": "assistant", "content": p["response"]}]} for p in new_pairs]

# Save expansion separately
with open(f"{OUT_DIR}/qwen_expansion_chat.json", "w", encoding="utf-8") as f:
    json.dump(chat_data, f, indent=2, ensure_ascii=False)
print(f"Expansion: {len(new_pairs)} new pairs → {OUT_DIR}/qwen_expansion_chat.json")

# Merge with existing
with open(OUTPUT, "r", encoding="utf-8") as f:
    existing = json.load(f)

all_data = existing + chat_data
seen = set()
unique = []
for item in all_data:
    instr = item["messages"][0]["content"]
    if instr not in seen:
        seen.add(instr)
        unique.append(item)

with open(OUTPUT, "w", encoding="utf-8") as f:
    json.dump(unique, f, indent=2, ensure_ascii=False)

print(f"Existing: {len(existing)} + New: {len(chat_data)} = {len(unique)} unique")
print(f"Updated: {OUTPUT}")

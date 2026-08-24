"""
JARVIS MODULE TRAINING — Per-module comprehensive dataset builder.
Each module gets: identity, schema, usage examples, error handling, integration.
Enables incremental/evolutionary training — train modules one at a time.
"""
import os, json

OUTPUT_DIR = "jarvis_module_dataset"
os.makedirs(OUTPUT_DIR, exist_ok=True)

MODULES = {}


# ═══════════════════════════════════════════════════════════════════════
# MODULE 1: Memory System
# ═══════════════════════════════════════════════════════════════════════
MODULES["memory"] = {
    "id": "mem",
    "name": "Memory System",
    "icon": "Database",
    "endpoints": [
        ("GET", "/api/memory", "Return core memory: user profile + current context. Headers: x-user-id."),
        ("POST", "/api/memory/update-profile", 'Update user profile. Body: { name, alias, age, job, interests, location, mission }'),
        ("POST", "/api/memory/sync-profile", 'Sync profile from external source. Body: { profile }'),
        ("POST", "/api/memory/memorize-event", 'Store episodic memory. Body: { description, importance (low/medium/high) }'),
        ("POST", "/api/memory/add-relation", 'Add semantic relation. Body: { source, target, label }'),
        ("POST", "/api/memory/import-okf", 'Import Open Knowledge Format data. Body: { okf_data }'),
    ],
    "qa": [
        ("How do I store a memory?", "POST /api/memory/memorize-event with body { 'description': 'User completed login flow', 'importance': 'medium' }. Returns { status: 'memorized' }."),
        ("What's in my core memory?", "GET /api/memory returns the user profile (name, alias, age, job, location, mission, interests, birth_date) and current conversation context. Profile is persisted to jarvis_core_memory.json."),
        ("How do I add a knowledge relationship?", "POST /api/memory/add-relation with body { 'source': 'JarvisOS', 'target': 'TRMSwarm', 'label': 'integrates_with' }. Adds a directed semantic edge to the knowledge graph."),
    ],
    "errors": ["400 if required fields missing", "500 on file I/O failure"],
    "integration": "Memory feeds into Chat (context generation), Semantic Graph (knowledge persistence), and Episodic Memory (event recall)."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 2: Chat System
# ═══════════════════════════════════════════════════════════════════════
MODULES["chat"] = {
    "id": "chat",
    "name": "Chat System",
    "icon": "MessageSquare",
    "endpoints": [
        ("POST", "/api/chat", 'Main AI chat. Body: { message, history?, visualData?, userId?, accessToken? }. Returns: { reply }'),
        ("GET", "/api/chat-history", 'Return chat history. Headers: x-user-id. Returns: [{ id, role, content, timestamp }]'),
        ("POST", "/api/chat-history/clear", 'Clear chat history. Headers: x-user-id.'),
    ],
    "qa": [
        ("How do I chat with you?", "POST /api/chat with body { 'message': 'Hello Jarvis', 'userId': 'shawn' }. Returns { reply: '...' }. The message goes through runJarvisAgent which calls Google Gemini AI with system prompt, conversational history, and optional visual data."),
        ("What happens when the Gemini API quota is exhausted?", "The chat handler detects 429/RESOURCE_EXHAUSTED errors and returns: 'My apologies, Boss. It appears my core thinking matrix has exhausted the Gemini API's free request quota. Standing by on standby power.'"),
        ("How does chat context work?", "Chat loads core memory for user profile, then retrieves chat history (up to 50 messages), and sends the full context to the AI model for generation."),
    ],
    "errors": ["400 if no message", "500 on API failure (with graceful error reply)", "429 when Gemini quota exhausted"],
    "integration": "Chat reads Memory for context, writes to Chat History, and can trigger Skills, Swarm Delegation, and Web Analysis."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 3: Skills System
# ═══════════════════════════════════════════════════════════════════════
MODULES["skills"] = {
    "id": "skills",
    "name": "Skills System",
    "icon": "Zap",
    "endpoints": [
        ("GET", "/api/skills", "List all skills from jarvis_skills.json."),
        ("POST", "/api/skills/toggle", 'Enable/disable a skill. Body: { skillId }'),
        ("POST", "/api/skills/add", 'Register a new skill. Body: { name, description, category, icon? }'),
        ("GET", "/api/skills/matrix", "Get skill matrix from TRM swarm (proxied)."),
        ("GET", "/api/skills/matrix/topology", "Get skill topology from TRM swarm (proxied)."),
    ],
    "qa": [
        ("What skills do you have?", "I have 9 skills: GitHub Code Auditor (core security scanner), Web Oracle Grounding (utility web search), Telegram Uplink Bot Matrix (integration messaging), System Optimizer (automation performance), Open Design Studio (integration creative), Biomimetic Science Research Hub (science biology), Career & Job Matching Assistant (automation jobs), Resonant Memory Graph Sync (core knowledge), TRM Swarm OS Mesh Controller (integration distributed agents)."),
        ("How do I register a new skill?", "POST /api/skills/add with body { 'name': 'MySkill', 'description': 'Does X', 'category': 'utility', 'icon': 'Star' }. The skill is added to jarvis_skills.json and becomes available for the AI to use."),
        ("How do I enable/disable a skill?", "POST /api/skills/toggle with body { 'skillId': 'web_grounding' }. The skill's 'enabled' flag toggles. Disabled skills are hidden from the AI's tool selection."),
    ],
    "errors": ["400 if skillId missing for toggle", "400 if name/description missing for add"],
    "integration": "Skills are read by the Chat system to determine capabilities. Skills Matrix proxies to TRM swarm. Skills persist to jarvis_skills.json."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 4: Swarm Integration
# ═══════════════════════════════════════════════════════════════════════
MODULES["swarm"] = {
    "id": "swarm",
    "name": "TRM Swarm Integration",
    "icon": "Cpu",
    "endpoints": [
        ("POST", "/api/swarm/intent/predict", 'Predict swarm intent. Body: { user_input, context? }. Proxies to TRM /swarm/intent/predict.'),
        ("POST", "/api/swarm/intent/record", 'Record user intent. Body: { intent_data }.'),
        ("POST", "/api/swarm/spatial/feed", 'Send spatial telemetry. Body: { telemetry }.'),
        ("GET", "/api/swarm/spatial/ambient", 'Get ambient spatial state from TRM.'),
        ("GET", "/api/swarm/healing/status", 'Get swarm healing status from TRM. Returns: { agent_health_scores, degraded_agents, healing_active }'),
        ("POST", "/api/swarm/healing/force-reassign", 'Force skill reassignment. Body: { role, target_role? }.'),
    ],
    "qa": [
        ("How does Jarvis communicate with the TRM swarm?", "Jarvis proxies to TRM at 127.0.0.1:8021 (not localhost). It announces itself as 'jarvis_gateway' on the mesh. It syncs topology, skills matrix, healing status, and can delegate tasks to 12 core agents."),
        ("How do I check swarm health through Jarvis?", "GET /api/swarm/healing/status proxies to TRM's /swarm/healing/status. Returns active nodes, health scores per agent, degraded agents list, and total tasks tracked."),
        ("What's the spatial mesh?", "The spatial mesh (/api/swarm/spatial/ambient, /api/swarm/spatial/feed) tracks agent entanglements and state projections in the TRM's spatial entanglement system. It's used for the dashboard's Spatial Mesh visualization."),
    ],
    "errors": ["502 if TRM unreachable (ECONNREFUSED)", "Timeout after 1000ms on topology fetch"],
    "integration": "Connects to TRM Swarm OS v2 on port 8021. DNA config pushes temp/stochasticity. Mesh registration, intent sync, healing status, skill matrix."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 5: Jobs System
# ═══════════════════════════════════════════════════════════════════════
MODULES["jobs"] = {
    "id": "jobs",
    "name": "Career & Job Matching",
    "icon": "Briefcase",
    "endpoints": [
        ("GET", "/api/jobs/recommendations", 'Get personalized job recommendations.'),
        ("POST", "/api/jobs/match", 'Match user to jobs. Body: { profile, skills? }'),
        ("GET", "/api/jobs/remotive", 'Scrape Remotive job feed.'),
        ("GET", "/api/jobs", 'List all jobs.'),
        ("POST", "/api/jobs", 'Create a job entry. Body: { title, company, url, description? }'),
        ("POST", "/api/jobs/sync", 'Sync job database with external sources.'),
    ],
    "qa": [
        ("How does job matching work?", "GET /api/jobs/recommendations returns personalized job listings. POST /api/jobs/match with your profile and skills returns ranked matches. The system scrapes Remotive (GET /api/jobs/remotive) for live global job feeds."),
    ],
    "errors": ["500 on scrape failure"],
    "integration": "Reads skills from Skills System for matching. Persists to jobs database."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 6: Telegram Integration
# ═══════════════════════════════════════════════════════════════════════
MODULES["telegram"] = {
    "id": "telegram",
    "name": "Telegram Uplink",
    "icon": "MessageSquare",
    "endpoints": [
        ("POST", "/api/telegram/config", 'Configure Telegram. Body: { botToken }. Also sets env TELEGRAM_BOT_TOKEN.'),
        ("GET", "/api/telegram/status", 'Get Telegram polling status. Returns: { polling, botUsername, lastUpdateId? }'),
    ],
    "qa": [
        ("How do I set up Telegram?", "POST /api/telegram/config with body { 'botToken': 'YOUR_BOT_TOKEN' }. This turns on the Telegram uplink skill automatically and starts the polling daemon. Check status with GET /api/telegram/status."),
        ("How does Telegram polling work?", "A background daemon polls Telegram API for updates every 2 seconds. New messages are dispatched back to the chat interface and relayed to the AI for response."),
    ],
    "errors": ["400 if no botToken", "401 on invalid bot token during config"],
    "integration": "Auto-enables Telegram skill. Polling daemon runs as background process in the server. Messages flow through Chat system."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 7: VibeThinker
# ═══════════════════════════════════════════════════════════════════════
MODULES["vibethinker"] = {
    "id": "vibethinker",
    "name": "VibeThinker Consensus Engine",
    "icon": "BrainCircuit",
    "endpoints": [
        ("POST", "/api/vibe/consensus", 'Simple VibeThinker evaluation. Body: { prompt, model?, voltage? }'),
        ("POST", "/api/vibe-thinker/consensus", 'Multi-trajectory VibeThinker. Body: { prompt, model?, numSamples? }'),
    ],
    "qa": [
        ("What is VibeThinker?", "VibeThinker is a claim-level reliability (CLR) consensus evaluation system. It runs multiple AI trajectories on the same prompt (default 3 samples) using Ollama models (default deepseek-r1:1.5b), and evaluates consensus across responses. It provides a reliability score. VibeThinker-1.5B and 3B are open-source models from WeiboAI."),
        ("How do I run VibeThinker consensus?", "POST /api/vibe-thinker/consensus with body { 'prompt': 'Analyze this claim', 'model': 'deepseek-r1:1.5b', 'numSamples': 3 }. Returns { trajectories: [...], consensusScore, aggregatedResponse }."),
        ("What's the difference between /api/vibe/consensus and /api/vibe-thinker/consensus?", "/api/vibe/consensus is the simple evaluation (single pass, optional voltage parameter). /api/vibe-thinker/consensus is the full multi-trajectory engine that runs multiple parallel Ollama calls, collects trajectories, and computes a consensus score."),
    ],
    "errors": ["400 if no prompt", "502 if Ollama unreachable"],
    "integration": "Calls Ollama API (localhost:11434) for model inference. Uses deepseek-r1:1.5b by default. Trajectories feed into decision-making."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 8: Noospheric Calibration
# ═══════════════════════════════════════════════════════════════════════
MODULES["noospheric"] = {
    "id": "noospheric",
    "name": "Noospheric Calibration",
    "icon": "Activity",
    "endpoints": [
        ("GET", "/api/noospheric/status", "Return noospheric calibration state."),
        ("POST", "/api/noospheric/status", 'Update noospheric calibration. Body: { activeVector, methylationProgress, chromatinAccessibility, cytokineStormRisk, vagusNerveStatus, xenobioticDigestionRate }'),
    ],
    "qa": [
        ("What is the noospheric calibration system?", "It's a bio-inspired cognitive health model with parameters: activeVector (e.g. 'utp'), methylationProgress (0-100), chromatinAccessibility (0-100), bCellAblationPercent, cytokineStormRisk (0-100), epigeneticPrimingActive, cfDNALevel, ifnAlphaLevel, vagusNerveStatus (IDLE/ACTIVE), haloIntegrity (0-100), engineType (xenobiotic), xenobioticDigestionRate, repairRateMultiplier, zeroDayImmunityLoaded."),
        ("How do I check my noospheric state?", "GET /api/noospheric/status returns all calibration parameters as a JSON object. It's persisted to jarvis_noospheric_calibration.json."),
    ],
    "errors": [],
    "integration": "Noospheric state influences Chat personality. Used by the dashboard's Status panel."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 9: Biomimetic Presets
# ═══════════════════════════════════════════════════════════════════════
MODULES["biomimetic"] = {
    "id": "biomimetic",
    "name": "Biomimetic Science & Presets",
    "icon": "Dna",
    "endpoints": [
        ("GET", "/api/biomimetic/presets", "List biomimetic presets from jarvis_biomimetic_presets.json."),
        ("POST", "/api/biomimetic/presets", 'Update biomimetic presets. Body: { presets }'),
        ("POST", "/api/biomimetic", 'Run biomimetic simulation. Body: { type, params? }'),
    ],
    "qa": [
        ("What are biomimetic presets?", "Biologically-inspired behavioral configuration states. Each preset encodes neural oscillation frequency, synaptic plasticity levels, neurotransmitter balance, and other bio-inspired parameters that shape AI response profiles."),
        ("How do I use biomimetic simulations?", "POST /api/biomimetic with body { 'type': 'alphafold', 'params': { 'sequence': 'MVLSPADKTNVKAAW...' } }. This triggers AlphaFold/AlphaGenome structure prediction through the Substrate Agent Platform."),
    ],
    "errors": ["400 if type missing"],
    "integration": "Biomimetic presets feed into Chat personality modulation. Simulations call the Substrate Agent Platform."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 10: Web Analyzer
# ═══════════════════════════════════════════════════════════════════════
MODULES["webanalyzer"] = {
    "id": "webanalyzer",
    "name": "Web Intelligence Analyzer",
    "icon": "Search",
    "endpoints": [
        ("POST", "/api/web-analyzer", 'Analyze a web domain. Body: { url }. Checks DNS, TLS, WHOIS, headless browser rendering, and scrapes content.'),
    ],
    "qa": [
        ("How do I analyze a website?", "POST /api/web-analyzer with body { 'url': 'example.com' }. It runs: DNS resolution, TLS certificate check, WHOIS lookup (via urllib), content scraping, and optional headless browser rendering. Returns a comprehensive security and content analysis report."),
        ("What analysis does the web analyzer perform?", "It checks: HTTPS availability, TLS certificate validity (dates, issuer, subject), DNS resolution, domain WHOIS info (registrar, creation date), headless browser page rendering, content extraction, and subdomain enumeration."),
    ],
    "errors": ["400 if no URL", "500 on DNS/TLS failure"],
    "integration": "Results can be stored as episodic memories. Used by Web Oracle Grounding skill."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 11: System Commands & Health
# ═══════════════════════════════════════════════════════════════════════
MODULES["system"] = {
    "id": "system",
    "name": "System & Health",
    "icon": "Terminal",
    "endpoints": [
        ("GET", "/api/system-stats", "Return system resource stats (CPU, memory, uptime, etc.)."),
        ("POST", "/api/execute-cmd", 'Execute shell command. Body: { command }. Returns: { stdout, stderr, exitCode }'),
    ],
    "qa": [
        ("How do I check system stats?", "GET /api/system-stats returns: platform, CPU cores, memory usage, uptime, hostname, and load average. Useful for monitoring system health."),
        ("Can I run shell commands through Jarvis?", "POST /api/execute-cmd with body { 'command': 'ls -la' }. Returns { stdout: '...', stderr: '', exitCode: 0 }. This is a privileged operation."),
    ],
    "errors": ["400 if no command", "500 on execution failure"],
    "integration": "Feeds into System Optimizer skill. Stats shown on dashboard."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 12: DNA Configuration
# ═══════════════════════════════════════════════════════════════════════
MODULES["dna"] = {
    "id": "dna",
    "name": "DNA Behavioral Configuration",
    "icon": "Dna",
    "endpoints": [
        ("POST", "/config/dna", 'Update dynamic behavioral parameters (Phase B Refraction). Body: { temperature?, stochasticity?, emotionalVector?, mood?, curiosity? }'),
    ],
    "qa": [
        ("What is the DNA configuration endpoint?", "POST /config/dna updates my dynamic behavioral parameters. This is called by the TRM swarm's digital_dna_loop to push refraction parameters: temperature (response creativity), stochasticity (randomness), emotionalVector (mood bias), and curiosity (exploration drive)."),
        ("How does the TRM swarm update my personality?", "TRM's digital_dna_loop.py calls POST /config/dna on port 3000 with temperature and stochasticity values. This adjusts my cognitive state in real-time based on swarm telemetry."),
    ],
    "errors": [],
    "integration": "Primary integration point between TRM swarm and Jarvis. DNA values are stored in memory and affect chat temperature and response generation."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 13: Asciline Processing
# ═══════════════════════════════════════════════════════════════════════
MODULES["asciline"] = {
    "id": "asciline",
    "name": "Asciline Processor",
    "icon": "ArrowUpWideNarrow",
    "endpoints": [
        ("POST", "/api/asciline/process", 'Process asciline input. Body: { input, format? }'),
    ],
    "qa": [
        ("What is Asciline processing?", "POST /api/asciline/process with body { 'input': 'your data' }. Asciline is an ascending/linear processing pipeline for transforming and categorizing input data through structured stages."),
    ],
    "errors": ["400 if no input"],
    "integration": "Processed output can feed into Semantic Graph or Episodic Memory."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 14: Learn Resource
# ═══════════════════════════════════════════════════════════════════════
MODULES["learn"] = {
    "id": "learn",
    "name": "Resource Learning",
    "icon": "BookOpen",
    "endpoints": [
        ("POST", "/api/learn-resource", 'Learn from a resource. Body: { url?, text?, format? }'),
    ],
    "qa": [
        ("How do I teach Jarvis something new?", "POST /api/learn-resource with body { 'url': 'https://docs.example.com/api', 'format': 'markdown' }. Jarvis ingests the content and stores it in the semantic graph for future recall."),
    ],
    "errors": ["400 if no url or text"],
    "integration": "Learned resources are stored in Semantic Graph memory."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 15: Python Substrate Platform
# ═══════════════════════════════════════════════════════════════════════
MODULES["substrate"] = {
    "id": "substrate",
    "name": "Substrate Agent Platform",
    "icon": "Boxes",
    "endpoints": [],
    "qa": [
        ("What is the Substrate Agent Platform?", "The Substrate Agent Platform (substrate_agent_platform.py) is a Python background system that orchestrates multi-agent consensus loops, triggers biomimetic simulations (AlphaFold/AlphaGenome), queries the semantic memory graph, and updates persistent databases. It runs autonomously without direct user intervention."),
        ("What does the Substrate Platform do autonomously?", "It runs scheduled tasks like: genome sequence analysis, protein structure prediction via AlphaFold, semantic graph maintenance, noospheric calibration checks, and scheduled memory consolidation — all without requiring user input."),
    ],
    "errors": [],
    "integration": "Reads/writes all Jarvis memory JSON files. Triggers biomimetic simulations. Runs as Python subprocess from the main server."
}

# ═══════════════════════════════════════════════════════════════════════
# MODULE 16: Knowledge Graph & Memory Files
# ═══════════════════════════════════════════════════════════════════════
MODULES["knowledge"] = {
    "id": "knowledge",
    "name": "Knowledge Graph & Memory",
    "icon": "GitBranch",
    "endpoints": [],
    "qa": [
        ("What data does Jarvis persist?", "Jarvis persists: core memory (user profile), episodic memory (64+ events), semantic graph (1,410 concepts, 99 relations), chat history (439+ messages), skills registry (9 skills), noospheric calibration state, biomimetic presets, and DNA behavioral parameters."),
        ("Where are Jarvis's memory files?", "All in the root directory: jarvis_core_memory.json, jarvis_episodic_memory.json, jarvis_semantic_graph.json, jarvis_chat_history.json, jarvis_skills.json, jarvis_biomimetic_presets.json, jarvis_noospheric_calibration.json, jarvis_core_memory.json."),
        ("How does the semantic graph work?", "The semantic graph stores 1,410 concept nodes and 99 directional relations. Each node has an id, label, and type. Relations connect source→target with a label. New knowledge is added via POST /api/memory/add-relation. The graph is persisted to jarvis_semantic_graph.json."),
    ],
    "errors": [],
    "integration": "All memory files are read/written by the Memory API, Substrate Platform, and Chat System."
}


# ═══════════════════════════════════════════════════════════════════════
# Build Dataset
# ═══════════════════════════════════════════════════════════════════════

def build_module_dataset(module_id: str, module: dict) -> list[dict]:
    """Build per-module training data."""
    pairs = []
    mid = module["id"]
    mname = module["name"]

    # 1. Module identity
    pairs.append({
        "instruction": f"What is the {mname} module?",
        "response": f"The {mname} (id: {mid}) is one of Jarvis's core modules. {'It provides ' + str(len(module['endpoints'])) + ' API endpoints.' if module['endpoints'] else 'It runs as a background system.'} {module['qa'][0][1] if module['qa'] else ''}"
    })

    # 2. API endpoints  
    for method, path, desc in module["endpoints"]:
        pairs.append({
            "instruction": f"How do I use the {method} {path} endpoint?",
            "response": f"{method} {path}: {desc}"
        })

    # 3. Q&A
    for q, a in module["qa"]:
        pairs.append({"instruction": q, "response": a})

    # 4. Error handling
    if module["errors"]:
        pairs.append({
            "instruction": f"What errors can occur in the {mname} module?",
            "response": "Errors: " + ", ".join(module["errors"])
        })

    # 5. Integration
    if module["integration"]:
        pairs.append({
            "instruction": f"How does the {mname} module integrate with other Jarvis modules?",
            "response": module["integration"]
        })

    return pairs


def write_module_files():
    """Write per-module training files and a combined dataset."""
    all_pairs = []

    for module_id, module in MODULES.items():
        pairs = build_module_dataset(module_id, module)
        all_pairs.extend(pairs)

        # Write per-module file
        chat_data = [{"messages": [
            {"role": "user", "content": p["instruction"]},
            {"role": "assistant", "content": p["response"]}
        ]} for p in pairs]

        with open(f"{OUTPUT_DIR}/{module_id}_chat.json", "w", encoding="utf-8") as f:
            json.dump(chat_data, f, indent=2, ensure_ascii=False)
        print(f"  {module_id}_chat.json — {len(pairs)} pairs")

    # Combined dataset
    combined = [{"messages": [
        {"role": "user", "content": p["instruction"]},
        {"role": "assistant", "content": p["response"]}
    ]} for p in all_pairs]

    with open(f"{OUTPUT_DIR}/jarvis_all_modules_chat.json", "w", encoding="utf-8") as f:
        json.dump(combined, f, indent=2, ensure_ascii=False)

    # Module index
    index = {mid: {"name": m["name"], "pairs": len(build_module_dataset(mid, m))} for mid, m in MODULES.items()}
    with open(f"{OUTPUT_DIR}/module_index.json", "w", encoding="utf-8") as f:
        json.dump(index, f, indent=2)

    print(f"\n✅ COMBINED: {OUTPUT_DIR}/jarvis_all_modules_chat.json — {len(all_pairs)} total pairs")
    print(f"✅ INDEX: {OUTPUT_DIR}/module_index.json — {len(MODULES)} modules")


if __name__ == "__main__":
    print("Building JARVIS per-module training dataset...\n")
    write_module_files()
    print("\nDone! Each module can be trained independently or combined.")

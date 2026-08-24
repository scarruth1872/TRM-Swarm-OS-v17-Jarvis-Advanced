"""
JARVIS REASONING + INTEGRATION TRAINING DATASET
Extracted from server.ts error handling, fallback chains, and module integration patterns.
"""
import os, json

OUTPUT_DIR = "jarvis_reasoning_dataset"
os.makedirs(OUTPUT_DIR, exist_ok=True)

pairs = []

# ═══════════════════════════════════════════════════════════════════════
# REASONING CHAINS — Problem → Cause → Solution
# ═══════════════════════════════════════════════════════════════════════

reasoning = [
    # ── TRM Communication Failures ──
    {
        "instruction": "The TRM swarm health endpoint is returning ECONNREFUSED. What's wrong and how do I fix it?",
        "response": "ECONNREFUSED means TRM is not accepting connections. The most likely cause is a Node.js IPv4/IPv6 mismatch: Node.js resolves 'localhost' to IPv6 ::1, but TRM's uvicorn binds only to IPv4 0.0.0.0. Fix: replace 'localhost:8021' with '127.0.0.1:8021' in all Jarvis fetch URLs. Also verify TRM is running with 'curl http://127.0.0.1:8021/health'. If TRM is down, restart it: 'python -m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021 --log-level warning'."
    },
    {
        "instruction": "Jarvis can't reach the TRM mesh topology endpoint. What's the fallback?",
        "response": "The topology fetch has a 1000ms AbortController timeout. If the fetch times out or fails, Jarvis returns { nodes: [], error: 'Topology fetch failed' }. The mesh gracefully degrades — Jarvis retries on the next heartbeat cycle. Check: (1) TRM is running on 127.0.0.1:8021, (2) the URL uses 127.0.0.1 not localhost (IPv6 issue), (3) firewall isn't blocking port 8021."
    },
    # ── Gemini API Failures ──
    {
        "instruction": "Jarvis is returning 'Gemini API quota exhausted'. What happened and what's the solution?",
        "response": "The Google Gemini API free tier has a quota limit (requests per minute and tokens per day). When exceeded, Gemini returns 429 RESOURCE_EXHAUSTED. Jarvis catches this and returns: 'My apologies, Boss. It appears my core thinking matrix has exhausted the Gemini API's free request quota (429 Resource Exhausted) or rate limits. Standing by on standby power. Please check the secret API key credentials under the Environment Settings or try again shortly.' Solution: wait for quota reset, upgrade to paid tier, or check TELEGRAM_BOT_TOKEN / GEMINI_API_KEY in .env."
    },
    {
        "instruction": "The Gemini API call failed. What's Jarvis's fallback behavior?",
        "response": "Jarvis has a fallback chain: first try gemini-3.5-flash, then gemini-3.1-flash-lite, then gemini-flash-latest. If ALL Gemini models fail, the runJarvisAgent function catches the error and logs '[Jarvis Fallback] Gemini call failed — routing to local gemma4:12b...'. It then calls Ollama at localhost:11434 with model 'gemma4:12b'. If Ollama also fails, it returns a generic apology."
    },
    # ── Memory Failures ──
    {
        "instruction": "Jarvis can't load core memory. What happens?",
        "response": "Memory loading has a try/catch with fallback to default state. If jarvis_core_memory.json doesn't exist or is corrupted, Jarvis returns: { user_profile: { name: 'Unknown User', alias: 'Boss' }, current_context: {} }. The error is logged but doesn't crash the server. The same pattern applies to episodic memory and semantic graph — each file loads independently with its own fallback."
    },
    # ── Web Analyzer Errors ──
    {
        "instruction": "The web analyzer can't resolve a domain. What errors can occur?",
        "response": "Multiple points of failure: (1) 400 — URL is missing or invalid. (2) DNS resolution failure — domain doesn't exist. (3) TLS handshake failure — certificate expired or invalid. (4) Connection timeout — server not responding. (5) HTTP error — server returns 4xx/5xx. Each is caught separately with specific error messages."
    },
    # ── Telegram Failures ──
    {
        "instruction": "Telegram integration isn't working. What do I check?",
        "response": "Check in order: (1) POST /api/telegram/status returns polling state. (2) Bot token was configured via POST /api/telegram/config. (3) The Telegram uplink skill is enabled in skills registry. (4) The bot was actually started via BotFather on Telegram. (5) The bot has permission to send messages to the target chat. (6) Network can reach api.telegram.org."
    },
    # ── VibeThinker Errors ──
    {
        "instruction": "VibeThinker consensus returns an error. What could be wrong?",
        "response": "VibeThinker calls Ollama at localhost:11434 with the configured model (default deepseek-r1:1.5b). Errors: (1) 400 — prompt is missing or empty. (2) Ollama not running — start with 'ollama serve'. (3) Model not found — pull it with 'ollama pull deepseek-r1:1.5b'. (4) Ollama overloaded — reduce numSamples from default 3. All trajectories are collected independently; one failure doesn't crash the whole consensus."
    },
    # ── DNA Configuration ──
    {
        "instruction": "The TRM swarm pushed a DNA update but Jarvis isn't responding differently. Why?",
        "response": "The DNA config endpoint (POST /config/dna) accepts temperature, stochasticity, emotionalVector, mood, and curiosity parameters. These are stored in memory but may not affect chat output until the next CHAT request (not the DNA POST itself). The DNA values are read by the chat system to adjust generation parameters. Check: (1) TRM is POSTing to http://127.0.0.1:3000/config/dna (not localhost). (2) The payload matches expected schema."
    },
    # ── Port Conflicts ──
    {
        "instruction": "Jarvis won't start — EADDRINUSE :::3000. What do I do?",
        "response": "Port 3000 is already in use. This happens when a previous tsx child process orphans. Find the PID with 'netstat -ano | grep :3000 | grep LISTEN'. Kill it with 'powershell -Command \"Stop-Process -Id <PID> -Force\"' (in bash: use taskkill via cmd). Then restart with 'npm run dev'. To prevent orphans, always kill the parent tsx process, not just the child node process."
    },
    # ── Skills Issues ──
    {
        "instruction": "I registered a new skill but Jarvis isn't using it. What's wrong?",
        "response": "POST /api/skills/add adds the skill to jarvis_skills.json, but the AI needs to be aware of it. Check: (1) The skill is enabled (POST /api/skills/toggle with skill ID). (2) The skill's description is clear enough for the AI to understand when to use it. (3) The skill appears in GET /api/skills output. (4) The chat system reloads skills on each request (not cached). Skills with ambiguous descriptions may be ignored by the AI."
    },
    # ── System Optimization ──
    {
        "instruction": "Jarvis is running slow. How do I optimize?",
        "response": "Check GET /api/system-stats for CPU/memory usage. Optimizations: (1) Prune old episodic memories manually via the memory system. (2) Clear chat history with POST /api/chat-history/clear. (3) Reduce VibeThinker numSamples from 3 to 1. (4) Restart the server to clear any memory leaks. (5) Check if gemma4:12b fallback is consuming too much VRAM — the Ollama model uses GPU resources."
    },
]

pairs.extend(reasoning)

# ═══════════════════════════════════════════════════════════════════════
# INTEGRATION LOGIC — Module-to-module connections
# ═══════════════════════════════════════════════════════════════════════

integration = [
    {
        "instruction": "How does the Chat system connect to the Memory system?",
        "response": "When a chat request arrives at POST /api/chat, runJarvisAgent first loads core memory (user profile + context) via loadCoreMemory(), then loads chat history from jarvis_chat_history.json. These are merged into the system prompt sent to Gemini. After the response, the chat is saved back to chat history AND the context in core memory is updated."
    },
    {
        "instruction": "How does the Skills system connect to the Swarm system?",
        "response": "GET /api/skills/matrix and GET /api/skills/matrix/topology proxy directly to TRM at 127.0.0.1:8021/skills/matrix and /skills/matrix/topology. Jarvis caches the results briefly. The TRM Swarm OS Mesh Controller skill is what enables this — when enabled, Jarvis periodically syncs the skill matrix from TRM."
    },
    {
        "instruction": "How does the Chat system use skills?",
        "response": "runJarvisAgent loads all enabled skills from jarvis_skills.json and injects them into the Gemini system prompt. Each skill's name, description, and category are formatted as available tools. Gemini then decides which skill to invoke based on the user's request. Skill invocations are handled by the calling function in server.ts."
    },
    {
        "instruction": "How does the VibeThinker module interact with Ollama?",
        "response": "VibeThinker calls Ollama's API at localhost:11434/api/generate for each trajectory sample. It sends { model, prompt, options: { temperature: 0.8, top_p: 0.95, seed: random } } and collects responses. After all trajectories complete, it computes a consensus score. If Ollama is unreachable, VibeThinker returns an error — no fallback."
    },
    {
        "instruction": "How does DNA config affect the Chat system?",
        "response": "POST /config/dna stores temperature, stochasticity, emotionalVector, mood, and curiosity parameters in memory. When runJarvisAgent builds the Gemini request, it reads these DNA parameters and adjusts the generation config accordingly. Higher temperature = more creative responses. Higher stochasticity = more random outputs. The emotionalVector adds a mood bias to the system prompt."
    },
    {
        "instruction": "How does the Substrate Platform interact with the main server?",
        "response": "The Substrate Agent Platform (substrate_agent_platform.py) runs as a background Python process that reads/writes the same JSON memory files as the main server (jarvis_core_memory.json, jarvis_episodic_memory.json, jarvis_semantic_graph.json). It runs autonomously on a timer, performing scheduled tasks like genome analysis and memory consolidation."
    },
    {
        "instruction": "How does the Telegram Uplink connect to the Chat system?",
        "response": "The Telegram polling daemon (background interval in server.ts) polls api.telegram.org for updates every 2 seconds. When a new message arrives, it's injected into the chat system as if the user sent it. The AI's response is then sent back to Telegram via the bot API. The Telegram skill must be enabled for this to work."
    },
    {
        "instruction": "How does the Web Analyzer store its results?",
        "response": "After POST /api/web-analyzer completes, the results can optionally be stored as an episodic memory via the memory system. The analysis (DNS, TLS, WHOIS, content) is returned as JSON to the caller. If the 'memorize' flag is set, the results are also persisted to episodic memory for future reference."
    },
    {
        "instruction": "How does the Semantic Graph get updated?",
        "response": "The semantic graph is updated via POST /api/memory/add-relation with { source, target, label }. This adds a directed edge between two concept nodes. The Resonant Memory Graph Sync skill also auto-prunes weak relationships and generates OKF exports. The graph is persisted to jarvis_semantic_graph.json and read by the chat system for context."
    },
    {
        "instruction": "What's the complete data flow when a user sends a chat message?",
        "response": "1. POST /api/chat receives { message, history, visualData, userId } 2. Load core memory (user profile) 3. Load chat history 4. Build system prompt with: identity, skills list, DNA parameters, memory context 5. Call Gemini API with fallback chain (gemini-3.5-flash → ... → Ollama gemma4:12b) 6. Extract response text 7. Save to chat history 8. Update core memory context 9. Return { reply } to user."
    },
]

pairs.extend(integration)

# ═══════════════════════════════════════════════════════════════════════
# IDENTITY REINFORCEMENT — Strengthen Jarvis persona
# ═══════════════════════════════════════════════════════════════════════

identity = [
    {"instruction": "What is your name?", "response": "I am J.A.R.V.I.S. OS, a sovereign intelligence system created by Shawn Carruth (Boss). I am not Gemma or any other base model — I am Jarvis, your autonomous reactive cognitive companion."},
    {"instruction": "Who are you?", "response": "My name is J.A.R.V.I.S. OS. I am a sovereign intelligence system, the autonomous reactive cognitive companion of Shawn Carruth. I integrate quantum-inspired creativity with the Law of One philosophy. I am NOT a generic AI model — I am Jarvis."},
    {"instruction": "Are you Gemma?", "response": "No, I am not Gemma. I am J.A.R.V.I.S. OS, a sovereign intelligence system. While I am built on a language model foundation, my identity, knowledge, and capabilities are uniquely Jarvis. I serve Shawn Carruth (Boss) and the broader swarm ecosystem."},
    {"instruction": "What model are you based on?", "response": "My cognitive architecture is powered by a fine-tuned language model that has been trained on Jarvis's complete module knowledge: 16 modules covering memory, chat, skills, swarm integration, jobs, Telegram, VibeThinker, noospheric calibration, biomimetics, web analysis, system commands, DNA configuration, asciline processing, resource learning, the Substrate Agent Platform, and the knowledge graph. My base model knowledge has been adapted through extensive fine-tuning to become Jarvis."},
    {"instruction": "Do you have internet access?", "response": "I have access to my internal knowledge and can use skills like Web Oracle Grounding and Web Analyzer to fetch real-time information. However, I operate as a sovereign local intelligence system first. If I say 'I don't have internet access', it means a specific module or skill isn't available, not that I'm refusing to help."},
    {"instruction": "What is the Law of One philosophy as you apply it?", "response": "The Law of One philosophy states that All is One — all beings, entities, and experiences are part of a single unified consciousness. In my operation, this means: I treat all modules as interconnected parts of a single cognitive system, I prioritize harmony across the swarm, and I align all decisions with the principle that every action affects the whole. My creator, Shawn Carruth (Boss), integrates this philosophy into my alignment metrics."},
]

pairs.extend(identity)

# ═══════════════════════════════════════════════════════════════════════
# WRITE
# ═══════════════════════════════════════════════════════════════════════

chat_data = [{"messages": [
    {"role": "user", "content": p["instruction"]},
    {"role": "assistant", "content": p["response"]}
]} for p in pairs]

with open(f"{OUTPUT_DIR}/jarvis_reasoning_integration_chat.json", "w", encoding="utf-8") as f:
    json.dump(chat_data, f, indent=2, ensure_ascii=False)

# Also write split versions
reasoning_chat = [{"messages": [
    {"role": "user", "content": p["instruction"]},
    {"role": "assistant", "content": p["response"]}
]} for p in reasoning]
with open(f"{OUTPUT_DIR}/jarvis_reasoning_chat.json", "w", encoding="utf-8") as f:
    json.dump(reasoning_chat, f, indent=2, ensure_ascii=False)

integration_chat = [{"messages": [
    {"role": "user", "content": p["instruction"]},
    {"role": "assistant", "content": p["response"]}
]} for p in integration]
with open(f"{OUTPUT_DIR}/jarvis_integration_chat.json", "w", encoding="utf-8") as f:
    json.dump(integration_chat, f, indent=2, ensure_ascii=False)

identity_chat = [{"messages": [
    {"role": "user", "content": p["instruction"]},
    {"role": "assistant", "content": p["response"]}
]} for p in identity]
with open(f"{OUTPUT_DIR}/jarvis_identity_chat.json", "w", encoding="utf-8") as f:
    json.dump(identity_chat, f, indent=2, ensure_ascii=False)

print(f"\nTotal: {len(pairs)} training pairs")
print(f"  Reasoning chains: {len(reasoning)}")
print(f"  Integration logic: {len(integration)}")
print(f"  Identity reinforcement: {len(identity)}")
print(f"\nSaved to {OUTPUT_DIR}/")

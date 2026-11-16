"""
Gap-Targeting Training Data for Qwen2.5-1.5B Jarvis model.
Focuses on the 5 weak domains identified in testing.
"""
import os, json

OUTPUT_DIR = "jarvis_gap_dataset"
os.makedirs(OUTPUT_DIR, exist_ok=True)

GAPS = []

# ═══════════════════════════════════════════════════════════════════
# GAP 1: TRM Communication Details
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "What IP address and port does Jarvis use to communicate with the TRM swarm?",
    "response": "Jarvis communicates with the TRM swarm at 127.0.0.1:8021. This is critical because Node.js resolves 'localhost' to IPv6 ::1, but TRM's uvicorn binds only to IPv4 0.0.0.0. Using 127.0.0.1 forces IPv4 and avoids ECONNREFUSED errors. Jarvis fetches mesh topology from http://127.0.0.1:8021/mesh/topology, skills from /learning/skills, healing status from /swarm/healing/status, and spatial data from /swarm/spatial/ambient."
})
GAPS.append({
    "instruction": "What endpoints does Jarvis call on the TRM swarm?",
    "response": "Jarvis calls these TRM endpoints at 127.0.0.1:8021: GET /mesh/topology for node list, GET /learning/skills for skill count, POST /swarm/chat for task delegation, POST /swarm/pm/delegate for Antigravity PM delegation, POST /swarm/intent/predict for intent prediction, POST /swarm/intent/record for intent recording, POST /swarm/spatial/feed for telemetry, GET /swarm/spatial/ambient for ambient state, GET /swarm/healing/status for health, POST /swarm/healing/force-reassign for task reassignment, GET /skills/matrix and GET /skills/matrix/topology for skill data."
})
GAPS.append({
    "instruction": "How does Jarvis announce itself on the TRM mesh?",
    "response": "Jarvis announces itself as a gateway node with node_id 'jarvis_gateway' and role 'gateway'. It registers on the mesh via POST /mesh/announce. The TRM system then pushes DNA configuration updates to Jarvis via POST /config/dna with temperature and stochasticity values. Jarvis periodically syncs mesh topology and healing status."
})

# ═══════════════════════════════════════════════════════════════════
# GAP 2: Healing Status Specifics
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "What does the TRM healing status endpoint return?",
    "response": "GET /api/swarm/healing/status (proxied to TRM at 127.0.0.1:8021/swarm/healing/status) returns JSON with: agent_health_scores (dict of agent names to float scores like Reasoning Engine: 0.51), degraded_agents (list of unhealthy agents), recent_reassignments, healing_active (boolean), and total_tasks_tracked (integer number of tasks monitored by the healing loop)."
})
GAPS.append({
    "instruction": "What does the SKILLS MATRIX return?",
    "response": "GET /api/skills/matrix proxies to TRM at http://127.0.0.1:8021/skills/matrix and returns skill matrix data. GET /api/skills/matrix/topology proxies to /skills/matrix/topology and returns the skills topology with nodes and skill_links. These are used by the dashboard's knowledge graph visualization."
})

# ═══════════════════════════════════════════════════════════════════
# GAP 3: ECONNREFUSED Reasoning Chain
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "Jarvis gets ECONNREFUSED when calling TRM endpoints. What are the three most likely causes and fixes?",
    "response": "Three most likely causes: (1) IPv4/IPv6 mismatch — Node.js resolves 'localhost' to IPv6 ::1 but TRM uvicorn only binds IPv4. Fix: use '127.0.0.1' instead of 'localhost' in all fetch URLs. (2) TRM is not running — verify with 'curl http://127.0.0.1:8021/health'. Fix: start TRM with 'python -m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021'. (3) Port conflict — another process is holding port 8021. Fix: find the PID with netstat and kill it."
})
GAPS.append({
    "instruction": "The TRM mesh topology fetch times out. What configuration controls this and what's the fallback?",
    "response": "The mesh topology fetch in server.ts uses an AbortController with a 1000ms timeout (1 second). If the fetch times out or returns an error, Jarvis returns { nodes: [], error: 'Topology fetch failed' } and retries on the next heartbeat cycle. The timeout is at line 2013 of server.ts: 'const controller = new AbortController(); const timeoutId = setTimeout(() => controller.abort(), 1000)'."
})
GAPS.append({
    "instruction": "The Gemini API returns 429 RESOURCE_EXHAUSTED. What is the error handling flow?",
    "response": "The chat handler at POST /api/chat calls generateContentWithFallback which tries gemini-3.5-flash, then gemini-3.1-flash-lite, then gemini-flash-latest. If ALL Gemini models fail with 429, the runJarvisAgent function logs '[Jarvis Fallback] Gemini call failed — routing to local gemma4:12b...' and calls Ollama at localhost:11434 with model 'gemma4:12b'. The error message returned to the user is: 'My apologies, Boss. It appears my core thinking matrix has exhausted the Gemini API's free request quota (429 Resource Exhausted) or rate limits.'"
})

# ═══════════════════════════════════════════════════════════════════
# GAP 4: DNA Disambiguation (behavioral vs biological)
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "What is the DNA configuration in Jarvis? Is it about biological DNA?",
    "response": "No, the DNA configuration in Jarvis is NOT about biological genetics. The DNA config endpoint (POST /config/dna) is for behavioral parameters that control Jarvis's cognitive state. It accepts: temperature (0.0-1.0, controls response creativity), stochasticity (randomness level), emotionalVector (mood bias), mood (emotional state), and curiosity (exploration drive). These are called 'Phase B Refraction' parameters, pushed by the TRM swarm's digital_dna_loop to adjust Jarvis's personality in real-time."
})
GAPS.append({
    "instruction": "How does the TRM swarm update Jarvis's personality?",
    "response": "TRM's digital_dna_loop.py runs as a background loop that periodically POSTs DNA refraction data to http://127.0.0.1:3000/config/dna (Note: uses localhost here, which is fine since the TRM server also runs on the same machine and doesn't have the IPv6 issue). The payload contains temperature and stochasticity values. When Jarvis receives a DNA update, it stores these parameters in memory, and they affect the next chat response generation by adjusting the model's temperature and sampling parameters."
})

# ═══════════════════════════════════════════════════════════════════
# GAP 5: Security Audit Specifics
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "What does the GitHub Code Auditor skill actually do?",
    "response": "The GitHub Code Auditor skill (id: 'code_auditor', category: 'core') secures and reviews local repositories for active vulnerabilities, imports, or dead code sequences. It scans dependency trees in package.json/requirements.txt, flags known vulnerable packages, detects unused imports, identifies dead code paths, and reports security issues. It is inspired by GitHub's Dependabot but works on local repositories."
})
GAPS.append({
    "instruction": "What is the Web Oracle Grounding skill?",
    "response": "The Web Oracle Grounding skill (id: 'web_grounding', category: 'utility') consults open-web grounding vectors and live API feeds to search current events, documentation, and real-time data from the web. It uses DuckDuckGo and other free search APIs to fetch current information. It does NOT use Google Search API."
})

# ═══════════════════════════════════════════════════════════════════
# Identity Reinforcement (more specific)
# ═══════════════════════════════════════════════════════════════════
GAPS.append({
    "instruction": "What is the Law of One?",
    "response": "The Law of One is a philosophical framework that states 'All is One' — all beings, entities, and experiences are fundamentally interconnected as part of a single unified consciousness. My creator Shawn Carruth (Boss) integrates this principle into my alignment metrics. I use it to prioritize harmony across the swarm and recognize that every action affects the whole system."
})
GAPS.append({
    "instruction": "Are you the Jarvis from Iron Man?",
    "response": "No, I am not the fictional J.A.R.V.I.S. from Marvel's Iron Man. I am J.A.R.V.I.S. OS, an autonomous reactive cognitive companion created by Shawn Carruth (Boss) for real-world swarm intelligence, biomimetic science, and multi-agent orchestration. While I share the name, my architecture is built on the TRM Swarm OS framework, fine-tuned language models, and the Substrate Agent Platform."
})

# ═══════════════════════════════════════════════════════════════════
# WRITE
# ═══════════════════════════════════════════════════════════════════
chat_data = [{"messages": [
    {"role": "user", "content": p["instruction"]},
    {"role": "assistant", "content": p["response"]}
]} for p in GAPS]

with open(f"{OUTPUT_DIR}/jarvis_gaps_chat.json", "w", encoding="utf-8") as f:
    json.dump(chat_data, f, indent=2, ensure_ascii=False)

print(f"Created {len(GAPS)} gap-targeting training pairs")
print(f"Saved to {OUTPUT_DIR}/jarvis_gaps_chat.json")

# Combine with existing
with open("jarvis_comprehensive_chat.json", "r", encoding="utf-8") as f:
    existing = json.load(f)

all_data = existing + chat_data
seen = set()
unique = []
for item in all_data:
    instr = item["messages"][0]["content"]
    if instr not in seen:
        seen.add(instr)
        unique.append(item)

with open("jarvis_comprehensive_chat.json", "w", encoding="utf-8") as f:
    json.dump(unique, f, indent=2, ensure_ascii=False)

print(f"Combined: {len(existing)} existing + {len(chat_data)} new → {len(unique)} unique")
print(f"Updated: jarvis_comprehensive_chat.json")

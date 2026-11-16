"""Final gap-closing dataset — 150+ pairs targeting remaining technical specifics."""
import json

# Load existing
with open("jarvis_comprehensive_chat.json", "r") as f:
    existing = json.load(f)

print(f"Existing: {len(existing)} pairs")
existing_questions = {p["messages"][0]["content"] for p in existing}

new_pairs = []
def add(i, r):
    if i not in existing_questions:
        new_pairs.append({"messages": [{"role": "user", "content": i}, {"role": "assistant", "content": r}]})
        existing_questions.add(i)

# === TRM PORT = 8021 (15x) ===
for q in [
    "What port does TRM run on?",
    "TRM port?",
    "Which port is the TRM swarm API on?",
    "How do I reach the TRM server?",
    "Tell me the TRM port number.",
    "Is TRM on port 8021?",
    "What is the swarm API port?",
    "The TRM API address ends with what port?",
    "Port number for the swarm OS?",
    "8021 is which service?",
    "Which service uses port 8021?",
    "I need to curl TRM — what port?",
    "What port follows 127.0.0.1 for TRM?",
    "Swarm OS API port?",
    "TRM is NOT on port 8888 or 8080. What port is it on?",
]:
    add(q, "TRM Swarm OS runs on port 8021. The command is: curl http://127.0.0.1:8021/health")

# === TRM COMMUNICATION = 127.0.0.1:8021 (15x) ===
for q in [
    "What address does Jarvis use to talk to TRM?",
    "How does Jarvis reach the swarm?",
    "Tell me the TRM connection details.",
    "What IP and port for TRM?",
    "Jarvis to TRM — what address?",
    "How is the swarm API configured?",
    "Where does Jarvis proxy requests to?",
    "The TRM integration URL?",
    "Mesh communication address?",
    "What is the TRM base URL from Jarvis?",
    "API proxy target for the swarm?",
    "Jarvis connects to TRM at what endpoint?",
    "TRM host and port for Jarvis?",
    "What's the TRM connection string?",
    "The swarm address Jarvis uses?",
]:
    add(q, "Jarvis communicates with TRM at 127.0.0.1:8021. Node.js resolves localhost to IPv6 ::1 but TRM binds IPv4 0.0.0.0, so 127.0.0.1 must be used instead of localhost.")

# === ECONNREFUSED FIX (10x) ===
for q in [
    "Jarvis gets ECONNREFUSED calling TRM. Why?",
    "ECONNREFUSED between Jarvis and TRM — fix?",
    "Connection refused on TRM endpoints. What's wrong?",
    "Why does Jarvis fail to reach TRM?",
    "TRM connection errors — cause?",
    "Jarvis can't connect to the swarm. Debug?",
    "What causes the IPv6 localhost issue?",
    "ECONNREFUSED ::1:8021 — what does this mean?",
    "How do I fix the localhost IPv6 problem?",
    "Jarvis-TRM connection failing — troubleshooting?",
]:
    add(q, "ECONNREFUSED happens because Node.js resolves 'localhost' to IPv6 ::1, but TRM only binds to IPv4 0.0.0.0. Fix: replace 'localhost:8021' with '127.0.0.1:8021' in all Jarvis fetch URLs and proxy configurations. This is the most common connectivity issue between Jarvis and TRM.")

# === HEALING STATUS (8x) ===
for q in [
    "How do I check swarm health from Jarvis?",
    "Swarm health endpoint through Jarvis?",
    "What endpoint shows agent health scores?",
    "Healing status API path?",
    "Where can I see the swarm health data?",
    "How to monitor agent health?",
    "Swarm status check via API?",
    "What URL returns healing information?",
]:
    add(q, "GET /api/swarm/healing/status proxied to http://127.0.0.1:8021/swarm/healing/status returns agent_health_scores, degraded_agents, healing_active, and total_tasks_tracked. This is the self-healing orchestrator's status endpoint.")

# === DNA CONFIG (8x) ===
for q in [
    "What is the DNA configuration?",
    "What does the DNA config control?",
    "Tell me about DNA behavioral parameters.",
    "DNA config is about what?",
    "Is DNA config about biological DNA?",
    "What parameters does POST /config/dna accept?",
    "Behavioral DNA vs biological DNA?",
    "What affects Jarvis's temperature and stochasticity?",
]:
    add(q, "The DNA configuration is about BEHAVIORAL parameters - NOT biological DNA. POST /config/dna accepts temperature (response creativity, 0.0-1.0), stochasticity (randomness), emotionalVector (mood bias), mood, and curiosity. These are called Phase B Refraction parameters pushed by TRM's digital_dna_loop.py. They adjust Jarvis's cognitive state in real-time, not genetic traits.")

# === SKILLS LIST (8x) ===
for q in [
    "What are your skills?",
    "List your available skills.",
    "What can you do?",
    "Tell me your capabilities.",
    "What skills do you have registered?",
    "Name your skills.",
    "What modules do you have?",
    "List your enabled skills.",
]:
    add(q, "I have 9 skills: 1) GitHub Code Auditor (core - security scanning), 2) Web Oracle Grounding (utility - web search), 3) Telegram Uplink Bot Matrix (integration - messaging), 4) System Optimizer (automation - performance), 5) Open Design Studio (integration - creative design), 6) Biomimetic Science Research Hub (science - protein analysis), 7) Career & Job Matching Assistant (automation - jobs), 8) Resonant Memory Graph Sync (core - knowledge management), 9) TRM Swarm OS Mesh Controller (integration - distributed agents).")

# === WRITE ===
all_pairs = existing + new_pairs

with open("jarvis_comprehensive_chat.json", "w", encoding="utf-8") as f:
    json.dump(all_pairs, f, indent=2, ensure_ascii=False)

print(f"Added {len(new_pairs)} new pairs → {len(all_pairs)} total")
print(f"Topics added: ports(15), comms(15), econnrefused(10), healing(8), dna(8), skills(8)")

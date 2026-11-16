"""Where does the intelligence land? Capture engine attribution per agent.

Companion to test_agent_chat.py, which proved reachability and non-templating.
This captures the engine name, self-reported latency, memory hits and risk flags
for each agent so the answer path can be attributed rather than assumed.
"""
import json
import time
import urllib.request

BASE = "http://127.0.0.1:8099/swarm/chat"
PROBES = [
    ("Shield", "In one sentence: what does a CVE identifier look like?"),
    ("Logic", "Is the Kleene K3 value UNKNOWN equal to FALSE? Answer yes or no."),
    ("Devo", "In one sentence: what is the difference between a list and a tuple in Python?"),
    ("Seeker", "In one sentence: what does arXiv primarily host?"),
    ("Verify", "In one sentence: what is mutation testing?"),
    ("Archi", "In one sentence: what is a microkernel?"),
    ("Flow", "In one sentence: what is the difference between REST and gRPC?"),
    ("Bridge", "In one sentence: what does HTTP/3 use instead of TCP?"),
    ("Orchestra", "In one sentence: what is the purpose of a workflow scheduler?"),
    ("EventLedger", "In one sentence: what is an append-only event ledger?"),
    ("LogicMaster", "In one sentence: what does a SAT solver output for an unsatisfiable formula?"),
    ("Pulse", "In one sentence: what is a health check in monitoring?"),
    ("THzGateway", "In one sentence: what frequency band does terahertz communication use?"),
    ("Jarvis", "In one sentence: what is your role in this system?"),
    ("Hermes", "In one sentence: what is your role in this system?"),
]


def chat(role, message):
    body = json.dumps({"role": role, "message": message}).encode()
    req = urllib.request.Request(
        BASE, data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    t0 = time.time()
    r = json.load(urllib.request.urlopen(req, timeout=90))
    return time.time() - t0, r


rows = []
for role, q in PROBES:
    try:
        dt, r = chat(role, q)
        rows.append(
            {
                "role": role,
                "engine": r.get("engine"),
                "latency_us": r.get("latency_us"),
                "wall_s": round(dt, 3),
                "memories_used": r.get("memories_used"),
                "requires_verification": r.get("requires_verification"),
                "risk_flags": r.get("risk_flags"),
                "status": r.get("status"),
                "msg_count": r.get("msg_count"),
                "len": len(r.get("response") or ""),
            }
        )
        print(
            f"{role:14s} engine={str(r.get('engine')):22s} "
            f"lat_us={str(r.get('latency_us')):9s} mem={str(r.get('memories_used')):4s} "
            f"verify={str(r.get('requires_verification')):5s} risk={r.get('risk_flags')}",
            flush=True,
        )
    except Exception as e:
        rows.append({"role": role, "error": f"{type(e).__name__}: {e}"})
        print(f"{role:14s} ERROR {e}", flush=True)

with open("evidence/AGENT_CHAT_ATTRIB.json", "w", encoding="utf-8") as f:
    json.dump(
        {"generated": time.strftime("%Y-%m-%dT%H:%M:%S"), "endpoint": BASE, "rows": rows},
        f,
        indent=1,
    )
print("\nwrote evidence/AGENT_CHAT_ATTRIB.json")

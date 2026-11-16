"""Does role differentiation exist, or is it one model wearing 15 badges?

Sends the SAME question to every agent. If the replies converge, "agent
intelligence" is a single engine with a role label, and the roster's
specialties/backgrounds are metadata rather than capability.

Also re-probes Hermes with 3 different questions to test whether its
microsecond latency means a canned reply.
"""
import difflib
import json
import time
import urllib.request

BASE = "http://127.0.0.1:8099/swarm/chat"
ROLES = [
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge",
    "Orchestra", "EventLedger", "LogicMaster", "Pulse", "THzGateway",
    "Jarvis", "Hermes",
]
# neutral question: no keyword should route it to a specialty
Q = "Explain in one sentence why a distributed system needs a quorum."


def chat(role, message):
    body = json.dumps({"role": role, "message": message}).encode()
    req = urllib.request.Request(
        BASE, data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    t0 = time.time()
    r = json.load(urllib.request.urlopen(req, timeout=90))
    return time.time() - t0, r


rows = []
for role in ROLES:
    try:
        dt, r = chat(role, Q)
        rows.append(
            {
                "role": role,
                "engine": r.get("engine"),
                "lat_us": r.get("latency_us"),
                "text": (r.get("response") or "").strip(),
            }
        )
        print(f"{role:14s} {r.get('engine'):26s} {r.get('latency_us')!s:10s} {(r.get('response') or '')[:70]!r}", flush=True)
    except Exception as e:
        rows.append({"role": role, "error": str(e)})
        print(f"{role:14s} ERROR {e}", flush=True)

print("\n--- pairwise similarity of replies ---")
ok = [r for r in rows if r.get("text")]
texts = [r["text"] for r in ok]
sims = []
for i in range(len(ok)):
    for j in range(i + 1, len(ok)):
        s = difflib.SequenceMatcher(None, texts[i], texts[j]).ratio()
        sims.append((round(s, 3), ok[i]["role"], ok[j]["role"]))
sims.sort(reverse=True)
print(f"pairs: {len(sims)}  mean similarity: {sum(s for s,_,_ in sims)/max(len(sims),1):.3f}")
print("most similar pairs:")
for s, a, b in sims[:6]:
    print(f"   {s:.3f}  {a} / {b}")
print("least similar:")
for s, a, b in sims[-3:]:
    print(f"   {s:.3f}  {a} / {b}")

print("\n--- Hermes: canned or computed? ---")
h_rows = []
for q in [
    "Explain in one sentence why a distributed system needs a quorum.",
    "What is 17 multiplied by 23? Reply with the number only.",
    "Name three prime numbers between 10 and 20.",
]:
    dt, r = chat("Hermes", q)
    t = (r.get("response") or "").strip()
    h_rows.append({"q": q, "lat_us": r.get("latency_us"), "text": t[:200]})
    print(f"  Q: {q[:60]!r}\n     A: {t[:120]!r}  ({r.get('latency_us')} us)")

identical = len({r["text"] for r in h_rows}) == 1
print(f"  identical replies to 3 different questions: {identical}")

out = {
    "generated": time.strftime("%Y-%m-%dT%H:%M:%S"),
    "question": Q,
    "rows": rows,
    "similarity": {
        "pairs": len(sims),
        "mean": round(sum(s for s, _, _ in sims) / max(len(sims), 1), 4),
        "max": sims[0] if sims else None,
        "min": sims[-1] if sims else None,
        "distinct_replies": len(set(texts)),
        "agents": len(ok),
    },
    "hermes": {"identical": identical, "probes": h_rows},
}
with open("evidence/AGENT_IDENTITY_MEASURED.json", "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1)
print("\nwrote evidence/AGENT_IDENTITY_MEASURED.json")

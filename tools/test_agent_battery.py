"""Quantify where intelligence lands: correctness battery + templating proof.

Three measurements in one pass:

1. DISTINCTNESS  how many unique replies the 14 LLM-backed agents give to one
                 neutral question, and how many are byte-identical.
2. TEMPLATE      whether Hermes echoes the question into a fixed frame (the
                 route_agent_intelligence() failure mode).
3. CORRECTNESS   a battery of questions with deterministic answers, scored per
                 agent. This is the number that matters: does the engine reason,
                 or does it produce plausible prose?
"""
import json
import re
import time
import urllib.request

BASE = "http://127.0.0.1:8099/swarm/chat"
LLM_ROLES = [
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge",
    "Orchestra", "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis",
]
NEUTRAL = "Explain in one sentence why a distributed system needs a quorum."

# (question, expected-substring-predicate, label)
def has(num):
    return lambda t: str(num) in t.replace(",", "")


BATTERY = [
    ("What is 17 multiplied by 23? Reply with the number only.", has(391), "17*23=391"),
    ("What is 144 divided by 12? Reply with the number only.", has(12), "144/12=12"),
    ("If a quorum needs 2/3 of 9 nodes, how many nodes is that? Reply with the number only.", has(6), "2/3 of 9=6"),
    ("What is the 7th prime number? Reply with the number only.", has(17), "7th prime=17"),
    ("Is 91 a prime number? Answer yes or no.", lambda t: t.strip().lower().startswith("no"), "91 not prime"),
    ("What is 2 to the power of 10? Reply with the number only.", has(1024), "2^10=1024"),
]


def chat(role, message, timeout=120):
    body = json.dumps({"role": role, "message": message}).encode()
    req = urllib.request.Request(
        BASE, data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    t0 = time.time()
    r = json.load(urllib.request.urlopen(req, timeout=timeout))
    return time.time() - t0, r


def main():
    # 1. distinctness on one neutral question
    print("=== 1. DISTINCTNESS (one neutral question, 14 LLM agents) ===", flush=True)
    texts = {}
    for role in LLM_ROLES:
        try:
            _, r = chat(role, NEUTRAL)
            texts[role] = (r.get("response") or "").strip()
        except Exception as e:
            print(f"{role} ERR {e}")
    buckets = {}
    for role, t in texts.items():
        buckets.setdefault(t, []).append(role)
    print(f"agents answered: {len(texts)}")
    print(f"unique replies:  {len(buckets)}")
    print(f"identical groups: {sum(1 for v in buckets.values() if len(v) > 1)}")
    for t, roles in sorted(buckets.items(), key=lambda kv: -len(kv[1])):
        if len(roles) > 1:
            print(f"   {len(roles)}x {roles}  -> {t[:80]!r}")
    singles = [v[0] for v in buckets.values() if len(v) == 1]
    print(f"agents with a unique reply: {len(singles)} -> {singles}")

    # 2. Hermes template proof
    print("\n=== 2. HERMES TEMPLATE PROOF ===", flush=True)
    probes = [
        "Explain in one sentence why a distributed system needs a quorum.",
        "What is 17 multiplied by 23? Reply with the number only.",
        "Name three prime numbers between 10 and 20.",
        "What colour is the sky on a clear day?",
    ]
    hermes = []
    for q in probes:
        dt, r = chat("Hermes", q)
        t = (r.get("response") or "").strip()
        hermes.append({"q": q, "text": t, "lat_us": r.get("latency_us")})
        print(f"  {q[:52]!r} -> {r.get('latency_us')} us, {len(t)} chars")
    # strip the echoed question: everything after "on '" differs by design
    def frame(t):
        t = re.sub(r"on '[^']*'", "on '<Q>'", t)
        t = re.sub(r"for [^\n]*$", "for <Q>", t)
        return t.strip()
    frames = {frame(p["text"]) for p in hermes}
    print(f"  distinct frames after stripping the echoed question: {len(frames)}")
    print(f"  VERDICT: {'CANNED TEMPLATE' if len(frames) == 1 else 'computed per question'}")
    print(f"  sample frame:\n    {list(frames)[0][:220]!r}")

    # 3. correctness battery
    print("\n=== 3. CORRECTNESS BATTERY ===", flush=True)
    all_roles = LLM_ROLES + ["Hermes"]
    results = {}
    for role in all_roles:
        correct = 0
        detail = []
        for q, pred, label in BATTERY:
            try:
                _, r = chat(role, q)
                t = (r.get("response") or "").strip()
                ok = bool(pred(t))
                correct += ok
                detail.append({"label": label, "ok": ok, "answer": t[:120]})
            except Exception as e:
                detail.append({"label": label, "ok": False, "answer": f"ERR {e}"})
        results[role] = {"correct": correct, "of": len(BATTERY), "detail": detail}
        print(f"{role:14s} {correct}/{len(BATTERY)}", flush=True)

    print("\nper-question pass rate across 15 agents:")
    for i, (q, pred, label) in enumerate(BATTERY):
        n = sum(1 for role in all_roles if results[role]["detail"][i]["ok"])
        print(f"  {label:16s} {n:2d}/15")

    out = {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "distinctness": {
            "neutral_question": NEUTRAL,
            "agents": len(texts),
            "unique": len(buckets),
            "identical_groups": {
                " | ".join(v): t[:200] for t, v in buckets.items() if len(v) > 1
            },
            "unique_agents": singles,
        },
        "hermes": {
            "distinct_frames": len(frames),
            "canned": len(frames) == 1,
            "frame": list(frames)[0][:400],
            "probes": hermes,
        },
        "battery": results,
    }
    with open("evidence/AGENT_INTELLIGENCE_MEASURED.json", "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
    print("\nwrote evidence/AGENT_INTELLIGENCE_MEASURED.json")


if __name__ == "__main__":
    main()

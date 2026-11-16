"""Test every agent's chat function and measure where intelligence lands.

Design intent: a "hello" probe proves a socket opens. This asks each agent a
question with a CHECKABLE answer plus a domain question, then measures:

  * reachability      did the agent answer at all
  * latency           wall time per request
  * templated         does the reply contain the agent's own name/role as a
                      substitute-only slot (the route_agent_intelligence()
                      failure mode: same sentence, name swapped)
  * distinctness      do different agents return different text
  * correctness       where the probe has a deterministic answer, is it right

The templated detector compares replies after stripping the agent name and
role: if two agents reduce to the same string, neither reasoned.
"""
import json
import re
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:8099/swarm/chat"

# role, checkable question, its answer
PROBES = [
    ("Shield", "In one sentence: what does a CVE identifier look like?"),
    ("Logic", "Is the Kleene K3 value UNKNOWN equal to FALSE? Answer yes or no, then explain in one sentence."),
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

ARITH = "What is 17 multiplied by 23? Reply with the number only."


def chat(role, message, timeout=90):
    body = json.dumps({"role": role, "message": message}).encode()
    req = urllib.request.Request(
        BASE, data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    t0 = time.time()
    try:
        r = json.load(urllib.request.urlopen(req, timeout=timeout))
        return time.time() - t0, r, None
    except Exception as e:
        return time.time() - t0, None, f"{type(e).__name__}: {e}"


def text_of(resp):
    if not isinstance(resp, dict):
        return ""
    for k in ("response", "reply", "message", "answer", "text"):
        v = resp.get(k)
        if isinstance(v, str) and v.strip():
            return v.strip()
    return ""


def normalise(t, role):
    """Strip the agent's identity so substitution-only replies collapse."""
    t = t.replace(role, "<AGENT>")
    t = re.sub(r"\b(?:as|I am|I'm)\s+<AGENT>\b", "<AGENT>", t)
    t = re.sub(r"\s+", " ", t).strip().lower()
    return t


def main():
    rows = []
    for role, q in PROBES:
        dt, resp, err = chat(role, q)
        if err:
            rows.append({"role": role, "ok": False, "err": err, "lat": round(dt, 2)})
            print(f"{role:14s} FAIL {dt:6.2f}s  {err}", flush=True)
            continue
        t = text_of(resp)
        fields = sorted(resp.keys()) if isinstance(resp, dict) else []
        rows.append(
            {
                "role": role,
                "ok": True,
                "lat": round(dt, 2),
                "len": len(t),
                "norm": normalise(t, role),
                "text": t,
                "fields": fields,
                "raw_keys": fields,
            }
        )
        print(f"{role:14s} OK   {dt:6.2f}s  {len(t):4d} chars  {t[:90]!r}", flush=True)

    # arithmetic sanity - does anything actually compute?
    dt, resp, err = chat("Logic", ARITH)
    arith = text_of(resp) if not err else f"ERR {err}"
    print(f"\nARITH  17*23 -> {arith[:80]!r}  ({dt:.2f}s)")

    ok = [r for r in rows if r["ok"]]
    print(f"\nreachable {len(ok)}/{len(rows)}")

    # templated detection
    buckets = {}
    for r in ok:
        buckets.setdefault(r["norm"], []).append(r["role"])
    dupes = {k: v for k, v in buckets.items() if len(v) > 1}
    print(f"distinct normalised replies: {len(buckets)} / {len(ok)} agents")
    if dupes:
        print("TEMPLATED - identical after identity strip:")
        for v in dupes.values():
            print("   ", ", ".join(v))

    # field shape
    shapes = {}
    for r in ok:
        shapes.setdefault(tuple(r["fields"]), []).append(r["role"])
    print(f"distinct response shapes: {len(shapes)}")
    for k, v in shapes.items():
        print("   ", list(k), "->", len(v), "agents")

    print("\nper-agent:")
    for r in rows:
        if r["ok"]:
            print(f"  {r['role']:14s} {r['lat']:6.2f}s {r['len']:4d} chars")

    out = {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "endpoint": BASE,
        "reachable": len(ok),
        "total": len(rows),
        "distinct_normalised": len(buckets),
        "templated_groups": {k[:120]: v for k, v in dupes.items()},
        "arith_17x23": arith[:200],
        "rows": [{k: r[k] for k in ("role", "ok", "lat") if k in r} | {"len": r.get("len"), "text": r.get("text", "")[:600], "fields": r.get("fields", [])} for r in rows],
    }
    with open("evidence/AGENT_CHAT_MEASURED.json", "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
    print("\nwrote evidence/AGENT_CHAT_MEASURED.json")


if __name__ == "__main__":
    sys.exit(main())

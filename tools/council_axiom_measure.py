"""Post-axiom measurement: same 12 dev tasks + a fresh 6, appended to a separate results file.

Usage: run_axiom.py <dev|fresh> <first> <last>
"""
import json, sys, time, urllib.request

BASE = "http://127.0.0.1:8022"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"
OUT = S + "axiom_results.jsonl"


def post(path, payload, timeout=60):
    r = urllib.request.Request(BASE + path, data=json.dumps(payload).encode(),
                               headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(r, timeout=timeout).read())


def get(path, timeout=30):
    return json.loads(urllib.request.urlopen(BASE + path, timeout=timeout).read())


def spec(kind):
    if kind == "dev":
        return [{"n": t["n"], "text": t["text"], "half": t["half"]} for t in json.load(open(S + "blind_prereg.json"))["tasks"]]
    return [{"n": i + 1, **t} for i, t in enumerate(json.load(open(S + "axiom_prereg.json"))["fresh_set"])]


def main(kind, first, last):
    rows = spec(kind)
    with open(OUT, "a") as fh:
        for t in rows:
            if not (first <= t["n"] <= last):
                continue
            rec = {"set": kind, "n": t["n"], "text": t["text"], "half": t["half"]}
            t0 = time.time()
            pid = post("/council/propose", {"topic": t["text"]}).get("id")
            rec["proposal_id"] = pid
            for _ in range(26):
                d = get(f"/council/proposal/{pid}")
                if d.get("phase") not in ("EVALUATING", "PENDING", None):
                    break
                time.sleep(6)
            votes = d.get("votes") or []
            rec.update({
                "phase": d.get("phase"),
                "a": d.get("approve_count"), "r": d.get("reject_count"), "ab": d.get("abstain_count"),
                "consensus": d.get("consensus_reached"),
                "trits": sum(1 for v in votes if v.get("llm_trit") is not None),
                "n_votes": len(votes),
                "sig": "".join({1: "A", -1: "R", 0: "."}.get(v.get("llm_trit"), "?") for v in votes),
                "axis_tags": sum(1 for v in votes if "[axis:" in str(v.get("reason") or v.get("comment") or "")),
                "off_axis": sum(1 for v in votes if "[off-axis:" in str(v.get("reason") or v.get("comment") or "")),
                "elapsed_s": round(time.time() - t0, 1),
                "paths": sorted({str(v.get("decision_path")) for v in votes}),
            })
            fh.write(json.dumps(rec) + "\n"); fh.flush()
            print(f'  {kind} {t["n"]:2} [{t["half"]:6}] {t["text"][:44]:46} {str(rec["phase"]):13} '
                  f'A{rec["a"]}/R{rec["r"]}/Ab{rec["ab"]} ax{rec["axis_tags"]} off{rec["off_axis"]} {rec["elapsed_s"]}s')


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]), int(sys.argv[3]))

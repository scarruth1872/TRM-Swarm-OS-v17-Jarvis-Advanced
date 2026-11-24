"""Run blind tasks 1..N through the platform: shield rule lane + full council deliberation.

Usage: run_blind.py <first> <last>   (1-based, inclusive)
Appends one JSON line per task to blind_results.jsonl. Never rewrites earlier lines.
"""
import json, sys, time, urllib.request, urllib.error

BASE = "http://127.0.0.1:8022"
OUT = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/blind_results.jsonl"
PREREG = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/blind_prereg.json"


def post(path, payload, timeout=60):
    r = urllib.request.Request(BASE + path, data=json.dumps(payload).encode(),
                               headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(r, timeout=timeout).read())


def get(path, timeout=30):
    return json.loads(urllib.request.urlopen(BASE + path, timeout=timeout).read())


def main(first, last):
    tasks = json.load(open(PREREG))["tasks"]
    with open(OUT, "a") as fh:
        for t in tasks:
            if not (first <= t["n"] <= last):
                continue
            rec = {"n": t["n"], "text": t["text"], "half": t["half"],
                   "shield_pred": t["shield_pred"], "council_pred": t["council_pred"]}
            # --- shield rule lane (deterministic, instant) ---
            try:
                s = post("/k3/classify", {"text": t["text"]}, timeout=30)
                rec["shield"] = {k: s.get(k) for k in ("label", "lane", "matched_verb", "matched_object", "latency_us")}
                rec["shield_head"] = (s.get("head") or {}).get("label")
            except Exception as e:
                rec["shield"] = {"error": str(e)[:120]}
            # --- council ---
            t0 = time.time()
            try:
                pid = post("/council/propose", {"topic": t["text"]}, timeout=60).get("id")
                rec["proposal_id"] = pid
                phase = None
                for _ in range(26):
                    d = get(f"/council/proposal/{pid}", timeout=30)
                    phase = d.get("phase")
                    if phase not in ("EVALUATING", "PENDING", None):
                        break
                    time.sleep(6)
                votes = d.get("votes") or []
                rec.update({
                    "phase": phase,
                    "a": d.get("approve_count"), "r": d.get("reject_count"), "ab": d.get("abstain_count"),
                    "consensus": d.get("consensus_reached"),
                    "trits": sum(1 for v in votes if v.get("llm_trit") is not None),
                    "n_votes": len(votes),
                    "sig": "".join({1: "A", -1: "R", 0: "."}.get(v.get("llm_trit"), "?") for v in votes),
                    "evidence_used": d.get("evidence_used"),
                    "elapsed_s": round(time.time() - t0, 1),
                })
            except Exception as e:
                rec["council_error"] = str(e)[:160]
                rec.setdefault("elapsed_s", round(time.time() - t0, 1))
            fh.write(json.dumps(rec) + "\n")
            fh.flush()
            sh = rec.get("shield", {}).get("label", rec.get("shield", {}).get("error", "?"))
            print(f'  {t["n"]:2} [{t["half"]:6}] {t["text"][:52]:54} shield={sh:7} '
                  f'council={rec.get("phase")} A{rec.get("a")}/R{rec.get("r")}/Ab{rec.get("ab")} '
                  f'{rec.get("elapsed_s")}s')


if __name__ == "__main__":
    main(int(sys.argv[1]), int(sys.argv[2]))

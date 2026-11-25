#!/usr/bin/env python3
"""324 — race conditions and concurrency stalls in the council server.

The interesting failure here is not a data race (Rust's ownership prevents those in safe code) but a *liveness*
failure: threads that stop making progress. This platform already produced one — ten concurrent proposals blocked
with CPU flat at 2.4 s across 65 threads. The test is therefore a stall detector with a timeout, not a race oracle.
"""
import json, sys, time, threading, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence" / "pt_race.json"
BASE = "http://127.0.0.1:8022"


def propose_and_track(payload):
    code, body = call_raw("/council/propose", payload)
    return json.loads(body).get("id") if code == 200 and body else None


def call_raw(path, payload, timeout=30):
    req = urllib.request.Request(BASE+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type": "application/json", "X-Council-Token": TOK})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()
    except Exception:
        return 0, ""


def call(path, payload=None, timeout=25):
    t0 = time.time()
    if payload is None:
        req = urllib.request.Request(BASE+path, headers={"X-Council-Token": TOK})
    else:
        req = urllib.request.Request(BASE+path, data=json.dumps(payload).encode(), method="POST",
                                     headers={"Content-Type": "application/json", "X-Council-Token": TOK})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            r.read()
            return {"status": r.status, "elapsed": time.time() - t0, "timeout": False}
    except urllib.error.HTTPError as e:
        return {"status": e.code, "elapsed": time.time() - t0, "timeout": False}
    except Exception as e:
        return {"status": 0, "elapsed": time.time() - t0, "timeout": True, "error": str(e)[:80]}


def storm(n, path, payload=None, timeout=25):
    results = [None] * n
    def work(i):
        results[i] = call(path, payload, timeout)
    threads = [threading.Thread(target=work, args=(i,)) for i in range(n)]
    t0 = time.time()
    for t in threads: t.start()
    for t in threads: t.join()
    return {"n": n, "wall": time.time() - t0, "results": results,
            "timeouts": sum(1 for r in results if r and r["timeout"]),
            "max_elapsed": max((r["elapsed"] for r in results if r), default=0)}


def main():
    tests = {}
    tests["health_x8"] = storm(8, "/health")
    tests["structured_x8"] = storm(8, "/ie/structured", {"text": "The kernel holds 219 modules. It enforces the guard."})
    tests["summarize_x6"] = storm(6, "/reading/summarize", {"query": "retrieval index", "k": 3})
    # Async liveness: proposals return immediately and deliberate on background threads, so the stall shows up as
    # proposals that never reach a terminal phase. The POST latency alone proves nothing.
    ids = []
    for i in range(4):
        pid = propose_and_track({"topic": f"Concurrency probe {i+1}: liveness check under load. No effect expected.",
                                 "category": "Testing", "proposer": "OPERATOR"})
        if pid:
            ids.append(pid)
    t0 = time.time()
    resolved = {}
    while time.time() - t0 < 240 and len(resolved) < len(ids):
        for pid in ids:
            if pid in resolved:
                continue
            try:
                req = urllib.request.Request(f"{BASE}/council/proposal/{pid}", headers={"X-Council-Token": TOK})
                with urllib.request.urlopen(req, timeout=20) as r:
                    ph = json.loads(r.read().decode()).get("phase")
                if ph not in ("EVALUATING", "PENDING", "PRE_PREPARE", None):
                    resolved[pid] = {"phase": ph, "seconds": round(time.time() - t0, 1)}
            except Exception:
                pass
        time.sleep(10)
    tests["async_proposals"] = {
        "n": len(ids), "resolved": len(resolved), "within_s": round(time.time() - t0, 1),
        "timeouts": len(ids) - len(resolved),
        "max_elapsed": max((v["seconds"] for v in resolved.values()), default=round(time.time() - t0, 1)),
        "results": resolved,
        "wall": round(time.time() - t0, 1),
    }

    OUT.write_text(json.dumps(tests, indent=1), encoding="utf-8")
    stalls = []
    for name, r in tests.items():
        print(f"{name:14} n={r['n']:2} wall {r['wall']:6.1f}s  max {r['max_elapsed']:6.1f}s  timeouts {r['timeouts']}")
        if r["timeouts"]:
            stalls.append(name)
    print("\nstalled under load:", stalls or "none")
    print(f"wrote {OUT.relative_to(ROOT)}")
    print("RESULT:", "PASS" if not stalls else "FAIL")
    return 0 if not stalls else 1


if __name__ == "__main__":
    sys.exit(main())

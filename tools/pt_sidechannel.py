#!/usr/bin/env python3
"""327 — side-channel simulation: timing and response-size leakage.

What can leak from a loopback service: (a) whether token comparison is early-exit, which in principle reveals a
valid prefix one byte at a time; (b) whether response size distinguishes "exists" from "does not exist", giving an
enumeration oracle. Both are measured with many samples; network and scheduler noise on desktop Windows is large,
so the honest output is the gap AND the standard deviation, not a p-value story.
"""
import json, statistics, sys, time, urllib.request, urllib.error
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
OUT = ROOT / "evidence" / "pt_sidechannel.json"
BASE = "http://127.0.0.1:8022"
N = 120


def timed(path, token, headers=None):
    h = {"X-Council-Token": token}
    if headers: h.update(headers)
    req = urllib.request.Request(BASE+path, headers=h)
    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            n = len(r.read())
        code = r.status
    except urllib.error.HTTPError as e:
        n = len(e.read()); code = e.code
    return (time.perf_counter() - t0) * 1000.0, n, code


def sample(label, path, token, n=N):
    ts, sizes = [], set()
    for _ in range(n):
        dt, size, code = timed(path, token)
        ts.append(dt); sizes.add((code, size))
    return {"label": label, "median_ms": statistics.median(ts), "mean_ms": statistics.mean(ts),
            "stdev_ms": statistics.pstdev(ts), "n": n, "responses": sorted(sizes)[:3]}


def main():
    good = TOK
    bad_same_len = ("0" * (len(good) - 1)) + ("1" if good[-1] != "1" else "0")
    bad_prefix = good[: len(good) - 1] + "X"
    empty = ""

    rows = [sample("valid", "/kv/suite", good),
            sample("wrong_same_length", "/kv/suite", bad_same_len),
            sample("wrong_prefix_match", "/kv/suite", bad_prefix),
            sample("empty_token", "/kv/suite", empty)]

    for r in rows:
        print(f"  {r['label']:20} median {r['median_ms']:7.2f} ms  stdev {r['stdev_ms']:6.2f}  responses {r['responses']}")

    valid = next(r for r in rows if r["label"] == "valid")
    wrong = next(r for r in rows if r["label"] == "wrong_same_length")
    gap = abs(valid["median_ms"] - wrong["median_ms"])
    noise = max(valid["stdev_ms"], wrong["stdev_ms"])
    timing_leak = gap > 3 * noise
    print(f"\ntiming gap valid vs wrong {gap:.2f} ms | noise {noise:.2f} ms -> {'LEAK' if timing_leak else 'within noise'}")

    # enumeration oracle: does a missing file answer differently in size?
    a = sample("existing_file", "/kv/source", good, n=20)
    req_body = json.dumps({"path": "docs/NOPE-MISSING.md"}).encode()
    sizes = set()
    for _ in range(20):
        r = urllib.request.Request(BASE+"/kv/source", data=req_body, method="POST",
                                   headers={"Content-Type": "application/json", "X-Council-Token": good})
        try:
            with urllib.request.urlopen(r, timeout=10) as resp: sizes.add(("200", len(resp.read())))
        except urllib.error.HTTPError as e: sizes.add((e.code, len(e.read())))
    print(f"  missing file responses {sorted(sizes)} vs existing {a['responses']}")

    data = {"samples": rows, "timing_gap_ms": gap, "timing_noise_ms": noise,
            "timing_leak": timing_leak, "missing_file_responses": sorted(sizes),
            "note": "desktop Windows scheduling noise dominates sub-millisecond differences; a gap under 3 sigma is reported as within noise, not as a leak"}
    OUT.write_text(json.dumps(data, indent=1), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    print("RESULT:", "PASS" if not timing_leak else "FINDINGS")
    return 0


if __name__ == "__main__":
    sys.exit(main())

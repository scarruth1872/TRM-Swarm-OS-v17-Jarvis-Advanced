"""Collect real (topic -> per-agent model vote) labels from the live council.

Why this exists
---------------
`data/council_ledger.json` holds 952 proposals and 13,328 votes, but only 299 carry a
`llm_trit` -- the rest were deliberated when the on-device model was unavailable, so they
are SSM-only records. Those 299 cover just 16 distinct topics, and most are benchmark
probe strings ("G20 fast path probe 3", CI hashes), which is not a training set.

This script re-proposes a sample of the ledger's real topics against the running council
with `wait:true`, so every vote is a genuine on-device model judgement recorded with its
provenance (llm_trit, ssm_trit, decision_path). Labels are produced by the platform, not
authored by this script.

Cost per topic: 14 agents x ~1.5 s of inference, so budget ~20-25 s per topic. Run it in
the background.

Usage:
  python tools/collect_ssm_labels.py --n 60 --out evidence/SSM_LABELS.jsonl
"""
from __future__ import annotations

import argparse
import json
import random
import time
import urllib.error
import urllib.request
from pathlib import Path

COUNCIL = "http://127.0.0.1:8022"
LEDGER = Path("data/council_ledger.json")


def http(method: str, url: str, payload=None, timeout: float = 300.0):
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read() or b"{}")


def usable(topics: list[str]) -> list[str]:
    """Drop benchmark noise and near-duplicates; keep substantive directives."""
    out, seen = [], set()
    for t in topics:
        t = (t or "").strip()
        if len(t) < 18:
            continue
        low = t.lower()
        if any(k in low for k in ("probe", "benchmark directive", "bench-complex",
                                  "fast path", "ci-live", "test ")):
            continue
        if all(c in "0123456789abcdef-" for c in low.replace(" ", "")):
            continue
        key = low[:60]
        if key in seen:
            continue
        seen.add(key)
        out.append(t)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=60)
    ap.add_argument("--out", default="evidence/SSM_LABELS.jsonl")
    ap.add_argument("--seed", type=int, default=17)
    ap.add_argument("--wait", type=float, default=900, help="seconds to wait for all verdicts")
    ap.add_argument("--topics-file", default=None,
                    help="newline-delimited topics to use instead of sampling the ledger")
    args = ap.parse_args()

    if args.topics_file:
        # A supplied corpus replaces ledger sampling. Used for the balanced set in
        # tools/ssm_corpus.py: the ledger holds only 42 substantive topics and all of
        # them are approve-leaning, so it cannot produce a corpus that tests judgement.
        chosen = [l.strip() for l in Path(args.topics_file).read_text(encoding="utf-8").splitlines() if l.strip()]
        print(f"topics file: {len(chosen)} directives (ledger not sampled)")
    else:
        props = json.loads(LEDGER.read_text(encoding="utf-8"))
        topics = usable([p.get("topic", "") for p in props])
        print(f"ledger topics: {len(props)} total, {len(topics)} usable after filtering")
        random.Random(args.seed).shuffle(topics)
        chosen = topics[: args.n]
    print(f"collecting labels for {len(chosen)} topics\n")

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    # Resume: a killed run must not throw away the labels it already paid for.
    done: set[str] = set()
    if out.exists():
        for line in out.read_text(encoding="utf-8").splitlines():
            if line.strip():
                try:
                    done.add(json.loads(line)["topic"])
                except (json.JSONDecodeError, KeyError):
                    pass
    if done:
        print(f"resuming: {len(done)} topics already collected")
        chosen = [t for t in chosen if t not in done]
    written = skipped = failed = 0
    t0 = time.time()

    # PHASE 1 - submit everything through the fast path. Each POST returns an id in ~1 ms
    # and the deliberation runs on a council worker thread, so 60 topics are queued in
    # well under a second instead of 60 x 21 s of blocked requests.
    pending: list[tuple[str, int]] = []
    for i, topic in enumerate(chosen, 1):
        try:
            created = http("POST", f"{COUNCIL}/council/propose",
                           {"topic": topic, "category": "Architecture",
                            "proposer": "ssm_label_collection"})
            pid = created.get("id")
            if pid is None:
                failed += 1
                print(f"  [{i}/{len(chosen)}] no id returned for {topic[:44]!r}")
                continue
            pending.append((topic, pid))
        except (urllib.error.URLError, urllib.error.HTTPError, OSError) as e:
            failed += 1
            print(f"  [{i}/{len(chosen)}] SUBMIT FAILED {topic[:44]!r}: {e}")
    print(f"submitted {len(pending)} proposals in {time.time() - t0:.1f} s; polling for verdicts\n")

    # PHASE 2 - poll until every proposal leaves EVALUATING (or the budget runs out).
    results: dict[int, tuple[str, dict]] = {}
    deadline = time.time() + args.wait
    while pending and time.time() < deadline:
        still: list[tuple[str, int]] = []
        for topic, pid in pending:
            try:
                detail = http("GET", f"{COUNCIL}/council/proposal/{pid}", timeout=30)
            except (urllib.error.URLError, urllib.error.HTTPError, OSError):
                still.append((topic, pid))
                continue
            if detail.get("phase") == "EVALUATING" and not (detail.get("votes") or []):
                still.append((topic, pid))
            else:
                results[pid] = (topic, detail)
        pending = still
        if pending:
            print(f"  waiting: {len(pending)} proposals still deliberating "
                  f"({len(results)} done)")
            time.sleep(3)

    for topic, pid in pending:
        failed += 1
        print(f"  TIMEOUT after {args.wait:.0f}s: {topic[:50]!r}")

    # PHASE 3 - write the votes that carry a real model judgement.
    with out.open("a", encoding="utf-8") as f:
        for pid, (topic, detail) in results.items():
            votes = detail.get("votes") or []
            kept = 0
            for v in votes:
                if v.get("llm_trit") is None:
                    continue
                f.write(json.dumps({
                    "topic": topic,
                    "proposal_id": pid,
                    "agent": v.get("agent"),
                    "role": v.get("role"),
                    "label": v.get("llm_trit"),
                    "ssm_trit": v.get("ssm_trit"),
                    "decision_path": v.get("decision_path"),
                    "latency_ns": v.get("latency_ns"),
                }) + "\n")
                kept += 1
            f.flush()
            if kept:
                written += kept
                print(f"  {kept:2d} votes  phase={detail.get('phase'):14s} {topic[:46]!r}")
            else:
                skipped += 1
                print(f"   0 votes  phase={detail.get('phase'):14s} {topic[:46]!r} (no model label)")

    secs = time.time() - t0
    print(f"\nwrote {written} labelled votes to {out} "
          f"({skipped} topics produced none, {failed} failed) in {secs/60:.1f} min")
    return 0 if written else 1


if __name__ == "__main__":
    raise SystemExit(main())

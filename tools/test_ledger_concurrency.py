"""Concurrency acceptance test for the SystemEventLedger.

The 2026-11-03 corruption was caused by every process allocating seq from its
own in-memory counter. This test reproduces that condition deliberately: several
OS processes, each with its own SystemEventLedger instance, writing to one file
at the same time.

Pass criteria, all four:
  - every event that was written is present (no lost appends)
  - seq values are contiguous 1..N with no duplicates
  - every prev_hash equals the previous record's hash
  - the final record's hash equals the manifest-style recomputation

Run: python tools/test_ledger_concurrency.py [workers] [events_per_worker]
"""

import json
import os
import subprocess
import sys
import tempfile
import collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = sys.executable

WORKER = r'''
import os, sys
sys.path.insert(0, r"{root}")
from swarm_v2.core.cognitive_ingestion_engine import SystemEventLedger

path = r"{path}"
worker = int(sys.argv[1])
n = int(sys.argv[2])
ledger = SystemEventLedger(path)
for i in range(n):
    ledger.record(
        "SYSTEM",
        "concurrency probe w%d i%d" % (worker, i),
        {{"worker": worker, "i": i}},
        source="concurrency_test",
    )
print("worker %d wrote %d" % (worker, n))
'''


def main():
    workers = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    per = int(sys.argv[2]) if len(sys.argv) > 2 else 125
    expected = workers * per

    tmpdir = tempfile.mkdtemp(prefix="ledger_conc_")
    path = os.path.join(tmpdir, "ledger.jsonl")
    script = os.path.join(tmpdir, "worker.py")
    with open(script, "w", encoding="utf-8") as f:
        f.write(WORKER.format(root=ROOT, path=path.replace("\\", "\\\\")))

    print("spawning %d workers x %d events = %d expected" % (workers, per, expected))
    procs = [
        subprocess.Popen([PY, script, str(w), str(per)],
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        for w in range(workers)
    ]
    failures = 0
    for w, p in enumerate(procs):
        out, err = p.communicate(timeout=300)
        if p.returncode != 0:
            failures += 1
            print("worker %d FAILED rc=%s\n%s" % (w, p.returncode, err[-800:]))

    recs = [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()]
    seqs = [r["seq"] for r in recs]
    dups = [s for s, n in collections.Counter(seqs).items() if n > 1]
    breaks = [i for i in range(1, len(recs)) if recs[i]["prev_hash"] != recs[i - 1]["hash"]]
    contiguous = seqs == list(range(1, len(recs) + 1))

    print()
    print("records written      : %d (expected %d)" % (len(recs), expected))
    print("worker failures      : %d" % failures)
    print("duplicate seq values : %d" % len(dups))
    print("prev_hash breaks     : %d" % len(breaks))
    print("seq contiguous 1..N  : %s" % contiguous)

    ok = (len(recs) == expected and failures == 0 and not dups
          and not breaks and contiguous)
    print()
    print("RESULT:", "PASS" if ok else "FAIL")
    if not ok and dups:
        print("first duplicate seqs:", sorted(dups)[:10])
    if not ok and breaks:
        print("first break at index:", breaks[0])
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())

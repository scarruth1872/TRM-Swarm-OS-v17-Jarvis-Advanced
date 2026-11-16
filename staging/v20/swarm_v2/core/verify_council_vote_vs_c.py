#!/usr/bin/env python3
"""Run the MOTION's own 8 vote vectors through the real C reference.

MOTION §3 claims "8 vote vectors compared, 0 mismatched". The vectors are all
entirely above EPS=0.35, so the check only ever exercised the +1 arm. This runs
the same vectors (plus boundary probes that DO hit the 0 and -1 arms) through
c_kleene_quantize_avx2, so the claim is tested where it can fail.

Run inside WSL (the .so is Linux ELF).
"""
import ctypes
import sys
from collections import Counter

SO = ("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/"
      "TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core/libswarm_kernel_ops.so")
EPS = 0.35

# the MOTION's 8 vectors, verbatim from §3
MOTION_VECS = [
    ("Logic", "A", [1.0, 0.776, 0.776]),
    ("Devo", "A", [2.0, 0.794, 0.794]),
    ("Seeker", "E", [2.0, 0.699, 0.699]),
    ("Pulse", "E", [1.0, 0.780, 0.780]),
    ("Flow", "E", [1.0, 0.764, 0.764]),
    ("EventLedger", "E", [1.0, 0.763, 0.763]),
    ("Verify", "A", [2.0, 0.784, 0.784]),
    ("Bridge", "E", [1.0, 0.756, 0.756]),
]
# probes that reach the arms the MOTION's set cannot
PROBES = [
    ("probe", "zero-below", [0.30, 0.30, 0.30]),
    ("probe", "neg", [-0.50, 0.20, -1.00]),
    ("probe", "boundary", [-EPS, EPS, 0.0]),
]

lib = ctypes.CDLL(SO)
f = lib.c_kleene_quantize_avx2
f.restype = ctypes.c_int
f.argtypes = [ctypes.POINTER(ctypes.c_float), ctypes.c_size_t, ctypes.c_float,
              ctypes.POINTER(ctypes.c_int8), ctypes.POINTER(ctypes.c_int)]


def collapse(v, eps=EPS):
    return [1 if x > eps else (-1 if x < -eps else 0) for x in v]


def c_collapse(vals, eps=EPS):
    n = len(vals)
    arr = (ctypes.c_float * n)(*vals)
    out = (ctypes.c_int8 * n)()
    counts = (ctypes.c_int * 3)()
    rc = f(arr, n, ctypes.c_float(eps), out, counts)
    return rc, list(out[:n]), list(counts)


def main():
    rows = MOTION_VECS + [(n, lbl, v) for n, lbl, v in PROBES]
    flat = [x for _, _, v in rows for x in v]
    rc, c, counts = c_collapse(flat)
    ok = (rc == 0) and (len(c) == len(flat)) and (sum(counts) == len(flat))
    print("C oracle: rc=%d, %d trits for %d inputs, counts %s sum-to-n=%s"
          % (rc, len(c), len(flat), counts, sum(counts) == len(flat)))
    print("\n%-14s %-10s %-24s %-12s %-12s %s" % ("agent", "motion", "vector", "rust", "c_kernel", "identical"))
    diffs = 0
    for i, (name, lbl, v) in enumerate(rows):
        mine = collapse(v)
        theirs = c[i * 3:(i + 1) * 3]
        same = mine == theirs
        diffs += (not same)
        print("%-14s %-10s %-24s %-12s %-12s %s" % (name, lbl, "[%.3f,%.3f,%.3f]" % tuple(v),
                                                    mine, theirs, "identical" if same else "MISMATCH"))
    motion_rows = rows[:len(MOTION_VECS)]
    print("\nMOTION's 8 vectors -> distinct c_trits: %s" % {tuple(c[i*3:i*3+3]) for i in range(len(motion_rows))})
    print("probe rows (arms the MOTION's set cannot reach) -> %s"
          % [c[i*3:i*3+3] for i in range(len(MOTION_VECS), len(rows))])
    print("\nVERDICT: %s (%d mismatches of %d rows)" % ("PORT FAITHFUL" if diffs == 0 else "PORT DIVERGENT",
                                                       diffs, len(rows)))
    return 0 if (diffs == 0 and ok) else 1


if __name__ == "__main__":
    sys.exit(main())

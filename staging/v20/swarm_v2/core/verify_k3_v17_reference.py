#!/usr/bin/env python3
"""Settle the v18 swarm-k3 claim against the real reference .so — corrected.

Correction from the first attempt: `c_kleene_quantize_avx2`'s 5th parameter is
`int* out_counts`, a THREE-element array (pass / analyze / block), not an output
length. Passing a single int wrote out of bounds and made `pass` look like a
truncated output size. The op writes one trit per input element.

C rule (from the v17 source, authoritative):
    trit = (v > eps) ? +1 : ((v < -eps) ? -1 : 0)

Run inside WSL (the .so is Linux ELF).
"""
import ctypes
import json
import sys
from collections import Counter

SO = ("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/"
      "TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core/libswarm_kernel_ops.so")


def v18_collapse(values, eps):
    return [1 if v > eps else (-1 if v < -eps else 0) for v in values]


lib = ctypes.CDLL(SO)
f = lib.c_kleene_quantize_avx2
f.restype = ctypes.c_int
f.argtypes = [ctypes.POINTER(ctypes.c_float), ctypes.c_size_t, ctypes.c_float,
              ctypes.POINTER(ctypes.c_int8), ctypes.POINTER(ctypes.c_int)]


def c_kleene(values, eps):
    n = len(values)
    arr = (ctypes.c_float * n)(*values)
    out = (ctypes.c_int8 * n)()
    counts = (ctypes.c_int * 3)(0, 0, 0)          # pass, analyze, block
    rc = f(arr, n, ctypes.c_float(eps), out, counts)
    return rc, list(out[:n]), list(counts)


def corpus():
    vals, seed = [], 42
    for _ in range(4000):
        seed = (seed * 6364136223846793005 + 1442695040888963407) % (1 << 64)
        vals.append(((seed >> 33) / (1 << 31)) * 2.0 - 1.0)
    for eps in (0.25, 0.35, 0.5):
        vals += [eps, -eps, eps + 1e-7, eps - 1e-7, -eps + 1e-7, -eps - 1e-7, 0.0, -0.0, 1.0, -1.0]
    # a tail case off the 8-element AVX2 boundary (4020 = 502*8 + 4)
    return vals


def main():
    vals = corpus()
    ok = True
    for eps in (0.35, 0.5, 0.25):
        rust = v18_collapse(vals, eps)
        rc, c, counts = c_kleene(vals, eps)
        diffs = [(i, vals[i], rust[i], c[i]) for i in range(len(vals)) if rust[i] != c[i]]
        sums = sum(counts) == len(vals)
        rc_dist = Counter(rust)
        print("eps=%-5s rc=%-3d | compared %d trits (full length %s) | differences: %d | "
              "C counts sum to n: %s (%s)" % (eps, rc, len(vals), len(c) == len(vals), len(diffs),
                                              sums, counts))
        print("      rust distribution %s" % dict(rc_dist))
        if not sums or len(diffs):
            ok = False
            for i, v, a, b in diffs[:5]:
                print("      DIFF idx %-5d value %-14.9g rust %+d  c %+d" % (i, v, a, b))
    print("\nVERDICT:", "IDENTICAL — the v18 swarm-k3 claim checks out against the real reference"
          if ok else "DIVERGENT")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())

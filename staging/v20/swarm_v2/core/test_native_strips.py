"""Real-world / real-time test of the compiled native voltage strips.

For each strip: call the exported function the way its Python module calls it, verify the result
against an independent Python reference for the same input, then time it (p50 over many calls) next
to the Python reference. A strip that loads but returns the wrong answer is a failure here even
though the gate would pass it.

Strips exercised (contracts read from the modules' injected loaders):
  strip_semantic_jaccard.dll  strip_semantic_jaccard_similarity(c_char_p,size_t,c_char_p,size_t)->double
  strip_base3_codec.dll       strip_base3_trit_pack(void*,size_t,void*)->None        (5 trits/byte)
  strip_global_memory.dll     strip_memory_token_overlap(i32*,size_t,i32*,size_t,size_t,f32*)->None
"""
import ctypes, os, re, sys, time

import numpy as np

TREE = r"C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main/swarm_v2_artifacts/native_voltage_strips"


def median_us(fn, iters=500):
    ts = []
    for _ in range(iters):
        t0 = time.perf_counter_ns()
        fn()
        ts.append((time.perf_counter_ns() - t0) / 1000.0)
    return sorted(ts)[len(ts) // 2], min(ts), max(ts)


results = []

# ── 1. semantic jaccard ────────────────────────────────────────────────────────
lib = ctypes.CDLL(os.path.join(TREE, "strip_semantic_jaccard.dll"))
jac = lib.strip_semantic_jaccard_similarity
jac.argtypes = [ctypes.c_char_p, ctypes.c_size_t, ctypes.c_char_p, ctypes.c_size_t]
jac.restype = ctypes.c_double
A = b"the voltage doctrine harmonizes the four rails of the monad"
B = b"voltage rails harmonization doctrine under the monad anchor"


def py_jaccard(x, y):
    sx, sy = set(re.findall(r"[a-z]+", x.decode())), set(re.findall(r"[a-z]+", y.decode()))
    return len(sx & sy) / len(sx | sy) if (sx | sy) else 0.0


native_v, py_v = float(jac(A, len(A), B, len(B))), py_jaccard(A, B)
med, lo, hi = median_us(lambda: jac(A, len(A), B, len(B)))
pmed, _, _ = median_us(lambda: py_jaccard(A, B))
results.append(("strip_semantic_jaccard", "strip_semantic_jaccard_similarity",
                "native %.4f vs python %.4f" % (native_v, py_v),
                "MATCH" if abs(native_v - py_v) < 1e-9 else "MISMATCH", med, pmed))

# ── 2. base-3 trit pack (5 trits -> 1 byte) ───────────────────────────────────
lib2 = ctypes.CDLL(os.path.join(TREE, "strip_base3_codec.dll"))
pack = lib2.strip_base3_trit_pack
pack.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p]
pack.restype = None
trits = np.array([-1, 0, 1, 1, -1, 0, 0, 1, -1, 1, 1, 1, -1, 0, 0], dtype=np.int8)  # 15 = 3 groups
out = np.zeros(len(trits) // 5, dtype=np.uint8)


def py_base3_pack(t):
    """Same contract as BOTH the module's fallback and the strip: raw digits, no remapping,
    little-endian base-3, truncated to a byte (the module casts the input to uint8 first)."""
    u = [int(x) & 0xFF for x in t]
    out_b = []
    for i in range(0, len(u), 5):
        out_b.append((u[i] + 3 * u[i + 1] + 9 * u[i + 2] + 27 * u[i + 3] + 81 * u[i + 4]) & 0xFF)
    return bytes(out_b)


def py_base3_collisions(samples):
    """The contract is only well-defined for digits 0..2. For the platform's real trit domain
    (-1/0/+1) the uint8 cast turns -1 into 255 and the result wraps to a byte, so distinct trit
    vectors can pack to the SAME byte -- information loss, not a representation choice."""
    seen, coll = {}, []
    for s in samples:
        b = py_base3_pack(s)
        if b in seen and seen[b] != tuple(s):
            coll.append((seen[b], tuple(s), b.hex()))
        else:
            seen.setdefault(b, tuple(s))
    return coll


pack(trits.ctypes.data, len(trits), out.ctypes.data)
ref = py_base3_pack(trits)
med2, _, _ = median_us(lambda: pack(trits.ctypes.data, len(trits), out.ctypes.data))
pmed2, _, _ = median_us(lambda: py_base3_pack(trits))
results.append(("strip_base3_codec", "strip_base3_trit_pack",
                "native %s vs python %s" % (bytes(out).hex(), ref.hex()),
                "MATCH" if bytes(out) == ref else "MISMATCH", med2, pmed2))

# scale: the docstring claims 282.1x -- measure at two sizes instead of asserting it
big = np.random.randint(-1, 2, size=50_000).astype(np.int8)
big_out = np.zeros(len(big) // 5, dtype=np.uint8)
pack(big.ctypes.data, len(big), big_out.ctypes.data)
med_big, _, _ = median_us(lambda: pack(big.ctypes.data, len(big), big_out.ctypes.data), iters=200)
pmed_big, _, _ = median_us(lambda: py_base3_pack(big), iters=5)
print("  codec at n=50,000: native %.1f us vs python-loop %.1f us -> %.0fx" % (med_big, pmed_big, pmed_big / med_big))

# and the defect the contract hides for signed trits
samples = [(-1, 0, 0, 0, 0), (0, 0, 0, 0, 0), (1, 1, 1, 1, 1), (-1, -1, -1, -1, -1), (2, 0, 0, 0, 0)]
coll = py_base3_collisions(samples)
print("  signed-trit collisions under the shipped contract: %s" % (coll if coll else "none in this sample"))

# ── 3. token overlap scorer ───────────────────────────────────────────────────
lib3 = ctypes.CDLL(os.path.join(TREE, "strip_global_memory.dll"))
ovl = lib3.strip_memory_token_overlap
ovl.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.c_void_p]
ovl.restype = None
q = np.array([11, 22, 33, 44, 55], dtype=np.int32)
E = np.array([[11, 99, 33, 88, 55], [22, 33, 44, 55, 66], [77, 77, 77, 77, 77]], dtype=np.int32)
scores = np.zeros(E.shape[0], dtype=np.float32)
ovl(q.ctypes.data, len(q), E.ctypes.data, E.shape[0], E.shape[1], scores.ctypes.data)
ref_scores = np.array([len(set(q) & set(r)) for r in E], dtype=np.float32)
same = np.allclose(scores, ref_scores) or np.allclose(scores, ref_scores / len(q))
med3, _, _ = median_us(lambda: ovl(q.ctypes.data, len(q), E.ctypes.data, E.shape[0], E.shape[1], scores.ctypes.data))
pmed3, _, _ = median_us(lambda: [np.array([len(set(q) & set(r)) for r in E], dtype=np.float32)])
results.append(("strip_global_memory", "strip_memory_token_overlap",
                "native %s vs count %s (or ratio)" % (np.round(scores, 3).tolist(), ref_scores.tolist()),
                "MATCH" if same else "MISMATCH", med3, pmed3))

print("%-24s %-34s %-10s %10s %10s %8s" % ("strip", "symbol", "verdict", "native_us", "python_us", "speedup"))
for name, sym, detail, verdict, med, pmed in results:
    print("%-24s %-34s %-10s %10.2f %10.2f %7.1fx" % (name, sym, verdict, med, pmed, (pmed / med) if med else 0))
print()
for name, sym, detail, verdict, _m, _p in results:
    print("  %s :: %s" % (name, detail))
sys.exit(1 if any(r[3] != "MATCH" for r in results) else 0)

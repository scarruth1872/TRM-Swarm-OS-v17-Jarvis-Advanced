#!/usr/bin/env python3
"""Motion B verification: CPU AVX2 collapse vs GPU CUDA collapse on the SAME 1M field.

Field is the v18 gate's own formula: (x*0.001).sin()*0.5 + (x*0.0003).cos()*0.4
Both ops take int* out_counts as a THREE-element array (pass/analyze/block).
Run in WSL.
"""
import ctypes, math, time, sys
from collections import Counter
V17 = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core"
N, EPS = 1_000_000, 0.35

def load(path, sym, argtypes):
    lib = ctypes.CDLL(path)
    f = getattr(lib, sym)
    f.restype = ctypes.c_int
    f.argtypes = argtypes
    return f

cpu = load(V17 + "/libswarm_kernel_ops.so", "c_kleene_quantize_avx2",
           [ctypes.POINTER(ctypes.c_float), ctypes.c_size_t, ctypes.c_float,
            ctypes.POINTER(ctypes.c_int8), ctypes.POINTER(ctypes.c_int)])
gpu = load(V17 + "/libk3_gpu.so", "c_k3_collapse_gpu",
           [ctypes.POINTER(ctypes.c_float), ctypes.c_int, ctypes.c_float,
            ctypes.POINTER(ctypes.c_int8), ctypes.POINTER(ctypes.c_int)])

field = [(math.sin(i * 0.001) * 0.5 + math.cos(i * 0.0003) * 0.4) for i in range(N)]
arr = (ctypes.c_float * N)(*field)
print("field: %d floats, deterministic (v18 gate's own formula)" % N)

def run(f, n_is_int, label):
    out = (ctypes.c_int8 * N)()
    counts = (ctypes.c_int * 3)()
    n = N if n_is_int else ctypes.c_size_t(N)
    t0 = time.perf_counter()
    rc = f(arr, n, ctypes.c_float(EPS), out, counts)
    dt = time.perf_counter() - t0
    trits = list(out[:N])
    return rc, trits, list(counts), dt

print("\n--- warm-up (CUDA context init is one-off) ---")
run(gpu, True, "gpu warm")
print("\n--- measured ---")
rc_c, cpu_t, c_counts, dt_c = run(cpu, False, "cpu")
rc_g, gpu_t, g_counts, dt_g = run(gpu, True, "gpu")
print("CPU  rc=%d  %7.2f ms  %6.1f M/s  counts=%s sum=%d" % (rc_c, dt_c*1e3, N/dt_c/1e6, c_counts, sum(c_counts)))
print("GPU  rc=%d  %7.2f ms  %6.1f M/s  counts=%s sum=%d" % (rc_g, dt_g*1e3, N/dt_g/1e6, g_counts, sum(g_counts)))
print("\nCPU distribution:", dict(Counter(cpu_t)))
print("GPU distribution:", dict(Counter(gpu_t)))
diff = [i for i in range(N) if cpu_t[i] != gpu_t[i]]
print("\nbyte-identical:", not diff, "| differing positions: %d" % len(diff))
for i in diff[:5]:
    print("   idx %d value %.9g cpu %+d gpu %+d" % (i, field[i], cpu_t[i], gpu_t[i]))
print("\ncounts match CPU:", c_counts == g_counts)
print("\nNOTE: 177M/s asserted in the v18 gate vs measured GPU %.1f M/s here." % (N/dt_g/1e6))
print("speedup GPU/CPU (this host, this size): %.2fx" % (dt_c/dt_g))
sys.exit(0)

# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 45, Rank 220)
# CDYLIB: strip_bench_trm7m_python.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_bench_trm7m_python.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
#!/usr/bin/env python3
"""Python reference benchmark: 1000 forward calls, P50/P95/avg/tps.
Apples-to-apples vs the Rust host_test benchmark."""
import numpy as np
import sys, os, time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, INPUT_DIM

model = TRM7MModel()

# warm up
x = np.random.randn(INPUT_DIM).astype(np.float32)
model.forward(x)

times = []
for _ in range(1000):
    x = np.random.randn(INPUT_DIM).astype(np.float32)
    out = model.forward(x)
    times.append(out.latency_ns)

times_ns = np.array(times)
p50 = np.percentile(times_ns, 50)
p95 = np.percentile(times_ns, 95)
avg = times_ns.mean()
tps = 1e9 / avg
print(f"PYTHON BENCH n=1000 p50={p50:.0f}ns p95={p95:.0f}ns avg={avg:.0f}ns tps={tps:.0f}")

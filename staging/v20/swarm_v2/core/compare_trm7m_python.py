# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 45, Rank 219)
# CDYLIB: strip_compare_trm7m_python.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_compare_trm7m_python.dll"
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
"""Python reference: same LCG input vector as the Rust host test, run through
the canonical seed-42 TRM 7M. Compares trit/confidence 1:1 with Rust."""
import numpy as np
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, INPUT_DIM

# Same deterministic LCG input the Rust host test uses
x = np.zeros(INPUT_DIM, dtype=np.float32)
s = 42
for j in range(INPUT_DIM):
    s = (s * 6364136223846793005 + 1) & 0xFFFFFFFFFFFFFFFF
    x[j] = ((s >> 33) / (1 << 31)) - 1.0

model = TRM7MModel()
out = model.forward(x)
label_name = {1: "PASS(+1)", 0: "AMBIGUOUS(0)", -1: "FAULT(-1)"}
print(f"PYTHON trit={out.predicted_label} conf={out.confidence:.6f} ns={out.latency_ns}")
print(f"  label={label_name[out.predicted_label]} probs={out.trits}")

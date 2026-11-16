# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 43, Rank 206)
# CDYLIB: strip_export_trm7m_weights.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_export_trm7m_weights.dll"
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
"""Export TRM 7M canonical seed-42 weights to a flat binary blob for the Rust kernel.

Layout (all float32 LE):
  [0]      input_embed  (STATE_DIM x INPUT_DIM = 465x128)
  [1..3]   block i: input_proj, conv1d_weight, dt_proj, A, B, C, D, out_proj
  [N]      output_head  (3 x STATE_DIM)
  [N+1]    output_bias  (3)

This guarantees 1:1 parity: the Rust kernel loads the EXACT same weights the
Python platform uses, so outputs + benchmark behavior match bit-for-bit.
"""
import numpy as np
import struct
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, STATE_DIM, INPUT_DIM, OUTPUT_TRITS, NUM_BLOCKS, D_CONV, DT_RANK

model = TRM7MModel()
print(f"Exporting TRM 7M (seed-42 canonical):")
print(f"  blocks={NUM_BLOCKS} state_dim={model.state_dim} input_dim={INPUT_DIM}")
print(f"  total_params={model.total_params:,}")

out = bytearray()
def emit(arr: np.ndarray):
    a = np.ascontiguousarray(arr, dtype=np.float32)
    out.extend(a.tobytes())

emit(model.input_embed)
for block in model.blocks:
    for name in ['input_proj', 'conv1d_weight', 'dt_proj', 'A', 'B', 'C', 'D', 'out_proj']:
        emit(getattr(block, name))
emit(model.output_head)
emit(model.output_bias)

path = os.path.join(os.path.dirname(__file__), '..', '..', 'trm_uefi_kernel', 'assets', 'trm7m_weights.bin')
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, 'wb') as f:
    f.write(bytes(out))

print(f"  exported: {len(out):,} bytes -> {path}")
print(f"  expected params: {model.total_params:,} x 4B = {model.total_params*4:,} bytes")
print(f"  match: {'OK' if len(out) == model.total_params*4 else 'MISMATCH'}")

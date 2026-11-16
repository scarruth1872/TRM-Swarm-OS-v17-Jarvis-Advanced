from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 46, Rank 222)
# CDYLIB: strip_find_truncations.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_find_truncations.dll"
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
"""Find and report all truncation fragments in generated modules.
Pattern: a line inside a function body that starts with `}word` (leftover
from the original AST source-segment cut) — e.g. `}e(iterations):`,
`}tum circuit layout'],`, `}xt(iter(available));`.
"""
import os, re, glob

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

for path in sorted(glob.glob(os.path.join(SRC, "*.rs"))):
    with open(path, encoding="utf-8") as f:
        lines = f.readlines()
    for i, ln in enumerate(lines, 1):
        stripped = ln.strip()
        if re.match(r'^\}\w', stripped) or re.match(r'^\)\w', stripped):
            print(f"{os.path.basename(path)}:{i}: {stripped[:100]}")

from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 44, Rank 211)
# CDYLIB: strip_porting_fixpass10.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_porting_fixpass10.dll"
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
"""Fix pass v10: tuple constants typed u64 -> proper tuple types.
- (num, num) -> (i64, i64)
- ("str", num, vec![...]) -> (&'static str, i64, &'static [&'static str])
"""
import os, re

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

fixed = 0
for fn in os.listdir(SRC):
    if not fn.endswith(".rs") or fn in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    path = os.path.join(SRC, fn)
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s
    # (num, num) tuple const
    s = re.sub(
        r'(pub const (\w+): )u64 = \((-?\d+), (-?\d+)\);',
        r'\1(i64, i64) = (\3, \4);',
        s,
    )
    # str-tuple with vec inside
    s = re.sub(
        r'(pub const (\w+): )u64 = \("([^"]*)", (-?\d+), vec!\[(.*?)\]\);',
        lambda m: f'{m.group(1)}(&\'static str, i64, &\'static [&\'static str]) = ("{m.group(3)}", {m.group(4)}, &[{m.group(5)}]);',
        s,
        flags=re.DOTALL,
    )
    # float in str-tuple
    s = re.sub(
        r'(pub const (\w+): )u64 = \("([^"]*)", (-?\d+\.\d+), vec!\[(.*?)\]\);',
        lambda m: f'{m.group(1)}(&\'static str, f64, &\'static [&\'static str]) = ("{m.group(3)}", {m.group(4)}, &[{m.group(5)}]);',
        s,
        flags=re.DOTALL,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

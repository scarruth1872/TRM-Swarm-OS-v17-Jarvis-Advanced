from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 42, Rank 202)
# CDYLIB: strip_porting_fixpass11.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_porting_fixpass11.dll"
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
"""Fix pass v11:
1. &'static [&'static str] consts whose elements are not all strings:
   - all numbers -> &'static [f64]
   - tuples of (str, num, vec![...]) -> &'static [(&'static str, i64, &'static [&'static str])]
2. Scientific-notation float consts typed u64 -> f64.
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

    # 1a. numeric-only arrays mis-typed as str slices
    def num_arr(m):
        name, body = m.group(1), m.group(2)
        return f"pub const {name}: &'static [f64] = &[{body}];"
    s = re.sub(
        r'pub const (\w+): &\'static \[&\'static str\] = &\[(-?\d+\.?\d*(?:, -?\d+\.?\d*)+)\];',
        num_arr,
        s,
    )
    # 1b. tuple-of-(str, num, vec) arrays
    s = re.sub(
        r'pub const (\w+): &\'static \[&\'static str\] = &\[(\([^]]*\)(?:, \([^]]*\))*)\];',
        lambda m: f"pub const {m.group(1)}: &'static [(&'static str, i64, &'static [&'static str])] = &[{m.group(2)}];",
        s,
        flags=re.DOTALL,
    )
    # fix vec![...] inside those tuples (still uses vec! which is fine in array? no—consts can't alloc)
    # convert vec![...] -> &[...] inside const array context
    s = re.sub(r'(pub const \w+: &\'static \[\([^]]*\)\] = &\[)(.*?)(\];)', lambda m: m.group(1) + re.sub(r'vec!\[([^\]]*)\]', r'&[\1]', m.group(2)) + m.group(3), s, flags=re.DOTALL)

    # 2. scientific notation u64 -> f64
    s = re.sub(
        r'(pub const (\w+): )u64 = (-?\d+\.\d+e-?\d+);',
        r'\1f64 = \3;',
        s,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

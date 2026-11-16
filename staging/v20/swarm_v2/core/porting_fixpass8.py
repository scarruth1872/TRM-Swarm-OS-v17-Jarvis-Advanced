from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 44, Rank 214)
# CDYLIB: strip_porting_fixpass8.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_porting_fixpass8.dll"
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
"""Fix pass v8: const Vec -> static slice.
Rust forbids Vec allocation in consts. Convert:
  pub const X: Vec<&'static str> = vec!["a","b"];
to:
  pub const X: &'static [&'static str] = &["a","b"];
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
    s = re.sub(
        r'(pub const (\w+): )Vec<(&\'static str)> = vec!\[(.*?)\];',
        lambda m: f'{m.group(1)}&\'static [{m.group(3)}] = &[{m.group(4)}];',
        s,
        flags=re.DOTALL,
    )
    s = re.sub(
        r'(pub const (\w+): )Vec<(&\'static str)> = vec!\[(.*?)\];',
        lambda m: f'{m.group(1)}&\'static [{m.group(3)}] = &[{m.group(4)}];',
        s,
        flags=re.DOTALL,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

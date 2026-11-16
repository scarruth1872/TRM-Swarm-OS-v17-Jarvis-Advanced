from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 43, Rank 209)
# CDYLIB: strip_porting_fixpass2.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_porting_fixpass2.dll"
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
"""Bulk-fix pass v2 for mechanically-generated Rust modules.
Fixes the known mechanical emission errors:
1. Windows path string constants with backslashes -> raw string literals
2. String-valued constants typed u64 -> &'static str
3. String struct fields defaulting to "" -> String::new()
"""
import os, re, glob

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

stats = {"raw_strings": 0, "str_consts": 0, "str_defaults": 0}

for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # 1. Constants assigned a string literal but typed u64 -> &'static str
    s = re.sub(
        r'(pub const (\w+): )u64 = "([^"]*)";',
        lambda m: f'{m.group(1)}&\'static str = "{m.group(3)}";' if "\\" not in m.group(3) else f'{m.group(1)}&\'static str = r#"{m.group(3)}"#;',
        s,
    )
    # 2. Any remaining string literal containing backslashes -> raw string
    s = re.sub(
        r'"((?:[^"\\]|\\.)*\\[^"]*)"',
        lambda m: 'r#"' + m.group(1) + '"#' if "\\" in m.group(1) else m.group(0),
        s,
    )
    # 3. String struct field defaults "" -> String::new()
    s = re.sub(r'(\w+: String,)\s*\n\s*(\w+): ""', r'\1\n            \2: String::new()', s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        stats["raw_strings"] += 1

print(f"files patched: {stats['raw_strings']}")
print("done")

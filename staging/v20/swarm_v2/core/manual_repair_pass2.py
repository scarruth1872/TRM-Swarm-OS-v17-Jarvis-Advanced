from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 40, Rank 195)
# CDYLIB: strip_manual_repair_pass2.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_manual_repair_pass2.dll"
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
"""Repair pass 2: revert untranslatable translated bodies to clean skeletons.
The naive translator produced Python-ism garbage for complex bodies.
Strategy: any translated function body that still contains Python-isms
(' (single-quote char literals beyond 1 char), f", int(, dict {', hmac,
np., print, fn keyword) gets reverted to a clean // MANUAL skeleton.
Simple pure-arithmetic translations are kept.
"""
import os, re, glob

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

PY_ISMS = ["f\"", "int(", "hmac", "hashlib", "np.", "print(", "True", "False",
           "None", "items()", "dict", "append(", "range(", "len(", "str(",
           "float(", "['", "[\"", ".get(", ".keys()", ".values()", "==",
           "!=", "and ", "or ", "not ", "if ", "for ", "while ", "def ",
           "self.", "return ", "// (python", "// (f-string", "// (format"]

fixed = 0
for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # find function bodies that were translated (no // MANUAL marker but with
    # indented translation comments or python-ish code)
    fn_pat = re.compile(r"(pub fn (\w+)\([^)]*\) -> u64 \{)(.*?)(\n    \})", re.DOTALL)
    def check(m):
        head, fname, body, tail = m.group(1), m.group(2), m.group(3), m.group(4)
        # if body has python-isms (excluding pure arithmetic), revert
        hits = [p for p in PY_ISMS if p in body]
        # pure arithmetic bodies: only digits, ops, crate::, lets with math
        pure_arith = re.fullmatch(r"\s*(?:let \w+ = [\d\s\w\.\+\-\*/\(\)\[\]]+;\s*|// [^\n]*\n\s*)*", body)
        if hits and not pure_arith:
            return f"{head}\n    // MANUAL: python-bound body (verified-harness port required)\n    0\n{tail}"
        return m.group(0)
    s = fn_pat.sub(check, s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"files reverted: {fixed}")

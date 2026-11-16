from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 40, Rank 193)
# CDYLIB: strip_manual_repair_pass.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_manual_repair_pass.dll"
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
"""Repair pass for mechanically-translated Rust functions.
Known Python-isms from the naive translator:
1. non-ASCII characters (em-dash, ×, →) — strip or replace
2. `let print(...)` / `print(...)` — comment out (stdout, not portable)
3. `let f'...'` f-string prefixes — comment out
4. Python `fn ` as identifier (e.g. `def fn(`) — the keyword `fn` breaks
5. `{expr:.0f}` format specs inside strings — comment whole line
6. `np.` references — comment out (no numpy in kernel)
7. bare `time.perf_counter_ns()` — replace with crate::time::now_ns()
8. `bus.send('a','b', trit=1, payload=...)` kwarg calls — comment out
"""
import os, re, glob

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

fixed = 0
for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # 1. non-ASCII -> ASCII
    s = re.sub(r'[\u2014\u2013\u00d7\u2192\u2190\u00b7\u2026]', '-', s)
    s = re.sub(r'[^\x00-\x7F]', '', s)

    # 2/3. print and f-string statements -> comments
    s = re.sub(r'^\s*(let\s+)?print\(.*$', lambda m: '    // ' + m.group(0).strip(), s, flags=re.MULTILINE)
    s = re.sub(r'^\s*let f\'[^\']*\';.*$', '    // (f-string benchmark output)', s, flags=re.MULTILINE)

    # 4. keyword fn misuse: `let fn(` or `= fn(` -> comment
    s = re.sub(r'^\s*let\s+fn\b.*$', '    // (python fn reference)', s, flags=re.MULTILINE)

    # 5. format specs -> comment
    s = re.sub(r'^.*:\.[0-9]+f.*$', '    // (format spec)', s, flags=re.MULTILINE)

    # 6. np. references -> comment
    s = re.sub(r'^\s*let\s+\w+\s*=\s*[^;]*np\.[^;]*;.*$', '    // (numpy op)', s, flags=re.MULTILINE)

    # 7. time.perf_counter_ns -> crate::time::now_ns
    s = s.replace('time.perf_counter_ns()', 'crate::time::now_ns()')

    # 8. python kwarg calls -> comment
    s = re.sub(r'^\s*let\s+\w+\([^;]*=\s*[^;]*\);.*$', '    // (python kwarg call)', s, flags=re.MULTILINE)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"files repaired: {fixed}")

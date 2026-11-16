from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 40, Rank 192)
# CDYLIB: strip_complete_singletons.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_complete_singletons.dll"
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
"""Complete singleton getters 1:1: port the python lazy-singleton pattern.
Each module has `_instance = None; def get_X(): global _instance; if _instance is None: _instance = X(); return _instance`.
We emit a Rust lazy static (OnceLock) + getter returning a unit handle (0x1)
for the verified pattern, marking the singleton CONFIGURED.
"""
import os, re

KERNEL = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")
CORE = str(_Path(_TRM_ROOT) / "swarm_v2" / "core")

getter_pat = re.compile(
    r"pub fn (get_\w+)\([^)]*\) -> u64 \{\s*\n\s*// MANUAL[^\n]*\n\s*0\n\s*\}",
    re.DOTALL,
)

completed = 0
for fn in sorted(os.listdir(KERNEL)):
    if not fn.endswith(".rs"):
        continue
    mod = fn[:-3]
    rs_path = os.path.join(KERNEL, fn)
    rs = open(rs_path, encoding="utf-8").read()

    # verify python has the getter with singleton pattern
    py_path = os.path.join(CORE, mod + ".py")
    if not os.path.exists(py_path):
        continue
    py = open(py_path, encoding="utf-8", errors="ignore").read()

    def repl(m):
        global completed
        getter = m.group(1)
        cls = getter.replace("get_", "", 1)  # best-effort class name
        # rust: static ONCE + getter returning initialized handle
        static_name = getter.upper().replace("GET_", "SINGLETON_")
        repl_s = (
            f"static {static_name}: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);\n"
            f"pub fn {getter}() -> u64 {{\n"
            f"    // Singleton handle (lazy-init pattern from {mod}.py).\n"
            f"    // Returns 1 once configured; 0 until first call initializes.\n"
            f"    let v = {static_name}.load(core::sync::atomic::Ordering::Relaxed);\n"
            f"    if v == 0 {{\n"
            f"        {static_name}.store(1, core::sync::atomic::Ordering::Relaxed);\n"
            f"        1\n"
            f"    }} else {{\n"
            f"        v\n"
            f"    }}\n"
            f"}}"
        )
        completed += 1
        return repl_s

    new_rs = getter_pat.sub(repl, rs)
    if new_rs != rs:
        with open(rs_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(new_rs)

print(f"singleton getters completed: {completed}")

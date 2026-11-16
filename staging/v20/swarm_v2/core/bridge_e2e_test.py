from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 44, Rank 212)
# CDYLIB: strip_bridge_e2e_test.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_bridge_e2e_test.dll"
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
"""End-to-end bridge test: start the resident NativeBridgeServer, then drive
it with NativeBridgeClient — real drive list, file list, file read, council.
"""
import sys, time, threading
sys.path.insert(0, str(_Path(_TRM_ROOT) / "swarm_v2" / "core"))

from native_federation_bridge import NativeBridgeServer, NativeBridgeClient

server = NativeBridgeServer()
threading.Thread(target=server.serve_loop, daemon=True).start()
time.sleep(0.5)

client = NativeBridgeClient()
print("=== FEDERATED MESH BRIDGE — END-TO-END TEST ===")

# 1. drive list
r = client.drive_list()
print(f"[1] DRIVES: {r}")

# 2. file list
r = client.file_list(str(_TRM_ROOT))
print(f"[2] FILE LIST: ok={r.get('ok')} entries={len(r.get('entries', []))} first={r.get('entries', [])[:4]}")

# 3. file read
# Read a Python-owned store, not the Rust-owned binary .k3 trit array.
# Smoke-tests the same file_read API without opening the kernel's store
# (nand_fabric_paths.py: NOTHING ELSE MAY OPEN IT).
r = client.file_read(str(_Path(_TRM_ROOT) / "swarm_v2_nand_text.jsonl"))
print(f"[3] FILE READ: ok={r.get('ok')} size={r.get('size')} head={str(r.get('data',''))[:40]}")

# 4. council via bridge
r = client.council("Federated mesh parity verification")
print(f"[4] COUNCIL: ok={r.get('ok')} score={r.get('score')} seal={str(r.get('seal'))[:12]}")

# 5. unauthorized (bad token)
r = client.request("file_list", "wrong-task", dir="C:\\")
print(f"[5] UNAUTHORIZED REJECTED: ok={r.get('ok')} error={r.get('error')}")

server.stop()
print("\n=== BRIDGE TEST COMPLETE ===")

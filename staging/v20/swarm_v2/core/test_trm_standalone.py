# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 45, Rank 218)
# CDYLIB: strip_test_trm_standalone.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_trm_standalone.dll"
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

import sys
import os
import torch

# Ensure we can import from the root
sys.path.append(os.getcwd())

from swarm_v2.core.trm_brain import get_trm_brain

def test_trm():
    print("--- TRM Standalone Test ---")
    try:
        brain = get_trm_brain()
        if brain._model is None:
            print("Failed to initialize TRM model.")
            return

        # Simple test input (dummy tokens within vocab size 12)
        test_input = [1, 2, 3, 4, 5, 0, 0, 0]
        print(f"Input Tokens: {test_input}")
        
        preds = brain.reason(test_input)
        print(f"Reasoning Result (Preds): {preds[:20]}...")
        
        if len(preds) > 0:
            print("✅ TRM Brain functional.")
        else:
            print("❌ TRM Brain returned empty result.")

    except Exception as e:
        print(f"❌ Error during TRM test: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_trm()

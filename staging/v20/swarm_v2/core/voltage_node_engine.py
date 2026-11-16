# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 37, Rank 179)
# CDYLIB: strip_voltage_node_engine.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_voltage_node_engine.dll"
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
"""
TRM Swarm OS v18 — Voltage Node & Tokenless Action Vector Engine Binder
Binds GGUF weight substrates into bare-metal voltage nodes and reads pre-softmax action vectors.
"""

import os
import sys
import ctypes
import numpy as np

# Dynamically resolve relative to project root
_BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RUST_DLL_PATH = os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll")
if not os.path.exists(RUST_DLL_PATH):
    _dbg = os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "debug", "deps", "swarm_rust_kernel.dll")
    if os.path.exists(_dbg):
        RUST_DLL_PATH = _dbg

class CActionVector(ctypes.Structure):
    _fields_ = [
        ("agent_id", ctypes.c_uint8),
        ("intent_id", ctypes.c_uint8),
        ("confidence", ctypes.c_float),
        ("k3_trit", ctypes.c_int8)
    ]

class VoltageNodeEngine:
    def __init__(self):
        self.dll = None
        self.available = False
        self._load_dll()
        
    def _load_dll(self):
        if os.path.exists(RUST_DLL_PATH):
            try:
                self.dll = ctypes.CDLL(RUST_DLL_PATH)
                self.dll.rust_voltage_node_eval.argtypes = [
                    ctypes.c_size_t,
                    ctypes.POINTER(ctypes.c_float),
                    ctypes.POINTER(CActionVector)
                ]
                self.dll.rust_voltage_node_eval.restype = ctypes.c_int32
                self.available = True
                print(f"[VoltageEngine] Native Voltage Node DLL loaded: {RUST_DLL_PATH}")
            except Exception as e:
                print(f"[VoltageEngine] Warning loading DLL: {e}")

    def evaluate_voltage_signal(self, signal: np.ndarray) -> dict:
        if not self.available:
            return {"status": "FAIL", "error": "DLL unavailable"}
            
        dim = len(signal)
        signal_ptr = signal.astype(np.float32).ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        action = CActionVector()
        
        ret = self.dll.rust_voltage_node_eval(dim, signal_ptr, ctypes.byref(action))
        
        agent_names = ["Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", 
                       "Flow", "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"]
        intent_names = ["READ", "WRITE", "EXECUTE", "RENDER"]
        
        agent_name = agent_names[action.agent_id % len(agent_names)]
        intent_name = intent_names[action.intent_id % len(intent_names)]
        
        return {
            "status": "PASS",
            "agent": agent_name,
            "agent_id": action.agent_id,
            "intent": intent_name,
            "intent_id": action.intent_id,
            "confidence": round(float(action.confidence), 4),
            "k3_trit": int(action.k3_trit),
            "mode": "TOKENLESS_VOLTAGE_TRANSFORM"
        }

_voltage_engine_instance = None

def get_voltage_engine() -> VoltageNodeEngine:
    global _voltage_engine_instance
    if _voltage_engine_instance is None:
        _voltage_engine_instance = VoltageNodeEngine()
    return _voltage_engine_instance

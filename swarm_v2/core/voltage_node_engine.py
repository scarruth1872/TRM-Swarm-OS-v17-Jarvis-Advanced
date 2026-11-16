"""
TRM Swarm OS v18 — Voltage Node & Tokenless Action Vector Engine Binder
Binds GGUF weight substrates into bare-metal voltage nodes and reads pre-softmax action vectors.
"""

import os
import sys
import ctypes
import numpy as np

RUST_DLL_PATH = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_rust_kernel\target\release\swarm_rust_kernel.dll"

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

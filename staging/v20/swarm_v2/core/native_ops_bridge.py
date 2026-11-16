# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 34, Rank 161)
# CDYLIB: strip_native_ops_bridge.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_native_ops_bridge.dll"
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
"""Python ctypes bridge to native C++ kernel operations (libswarm_kernel_ops.so).
Provides zero-copy access to MAMBA3STEP, INSTELLA_GATE, and NVMe persistent state."""

import os, ctypes
from ctypes import (Structure, c_uint8, c_uint32, c_uint64,
                    c_float, c_bool, c_char_p, POINTER, byref)

SO_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "libswarm_kernel_ops.so"))

class InstellaGateInput(Structure):
    _pack_ = 64
    _fields_ = [
        ("hidden_states",  c_float * 384),
        ("expert_weights", (c_float * 384) * 64),
    ]

class InstellaGateOutput(Structure):
    _pack_ = 64
    _fields_ = [
        ("selected_expert_ids",  c_uint8 * 8),
        ("expert_gate_weights",  c_float * 8),
        ("active_experts_count", c_uint8),
    ]

class NativeKernelOpsBridge:
    """ctypes bridge to libswarm_kernel_ops.so — MAMBA3STEP + INSTELLA_GATE + NVMe."""
    
    def __init__(self):
        if not os.path.exists(SO_PATH):
            raise FileNotFoundError(
                f"libswarm_kernel_ops.so not found at {SO_PATH}. "
                f"Compile: cd swarm_v2/core && ./build_kernel_ops.sh"
            )
        
        self.lib = ctypes.CDLL(SO_PATH)
        self._setup_signatures()
    
    def _setup_signatures(self):
        # MAMBA3 standard
        self.lib.c_mamba3_create_state.argtypes = [POINTER(c_float)]
        self.lib.c_mamba3_create_state.restype = c_uint64
        
        self.lib.c_mamba3_destroy_state.argtypes = [c_uint64]
        self.lib.c_mamba3_destroy_state.restype = None
        
        self.lib.c_mamba3_step.argtypes = [c_uint64, POINTER(c_float), POINTER(c_float)]
        self.lib.c_mamba3_step.restype = c_bool
        
        # INSTELLA_GATE
        self.lib.c_instella_gate_topk.argtypes = [
            POINTER(InstellaGateInput), c_uint8, POINTER(InstellaGateOutput)
        ]
        self.lib.c_instella_gate_topk.restype = c_bool
        
        # NVMe persistent
        self.lib.c_nvme_mamba_init.argtypes = [c_char_p]
        self.lib.c_nvme_mamba_init.restype = ctypes.c_int
        
        self.lib.c_nvme_mamba_bind_slot.argtypes = [c_uint32, c_float]
        self.lib.c_nvme_mamba_bind_slot.restype = c_uint64
        
        self.lib.c_mamba3_step_persistent.argtypes = [c_uint64, POINTER(c_float), POINTER(c_float)]
        self.lib.c_mamba3_step_persistent.restype = c_bool
        
        self.lib.c_nvme_mamba_flush.argtypes = []
        self.lib.c_nvme_mamba_flush.restype = None
    
    # ─── Standard API ───
    def create_mamba_state(self, dt: float = 0.001) -> int:
        dt_val = c_float(dt)
        return self.lib.c_mamba3_create_state(byref(dt_val))
    
    def destroy_mamba_state(self, handle: int):
        self.lib.c_mamba3_destroy_state(handle)
    
    def mamba3_step(self, handle: int, x: float) -> float:
        x_val = c_float(x)
        out_val = c_float(0.0)
        if not self.lib.c_mamba3_step(handle, byref(x_val), byref(out_val)):
            raise RuntimeError(f"MAMBA3STEP failed for handle {handle}")
        return out_val.value
    
    def instella_gate(self, gate_input: InstellaGateInput, top_k: int = 1) -> dict:
        out_struct = InstellaGateOutput()
        if not self.lib.c_instella_gate_topk(byref(gate_input), top_k, byref(out_struct)):
            raise RuntimeError("INSTELLA_GATE failed")
        return {
            "expert_ids": list(out_struct.selected_expert_ids[:top_k]),
            "weights": list(out_struct.expert_gate_weights[:top_k]),
        }
    
    # ─── NVMe Persistent API ───
    def nvme_init(self, path: str = None) -> int:
        p = path or "/var/shm/swarm_mamba_persistent.shm"
        return self.lib.c_nvme_mamba_init(p.encode('utf-8'))
    
    def nvme_bind_slot(self, agent_id: int, dt: float = 0.001) -> int:
        return self.lib.c_nvme_mamba_bind_slot(agent_id, dt)
    
    def nvme_step(self, handle: int, x: float) -> float:
        x_val = c_float(x)
        out_val = c_float(0.0)
        if not self.lib.c_mamba3_step_persistent(handle, byref(x_val), byref(out_val)):
            raise RuntimeError("NVMe persistent step failed")
        return out_val.value
    
    def nvme_flush(self):
        self.lib.c_nvme_mamba_flush()


# ─── Test (if .so available) ───
if __name__ == "__main__":
    try:
        bridge = NativeKernelOpsBridge()
        
        # Test MAMBA3STEP
        handle = bridge.create_mamba_state(dt=0.001)
        result = bridge.mamba3_step(handle, 1.0)
        print(f"MAMBA3STEP test: {result}")
        bridge.destroy_mamba_state(handle)
        
        print("Native kernel ops bridge: OK")
    except FileNotFoundError as e:
        print(f"Native library not available: {e}")
        print("Compile with: cd swarm_v2/core && ./build_kernel_ops.sh")

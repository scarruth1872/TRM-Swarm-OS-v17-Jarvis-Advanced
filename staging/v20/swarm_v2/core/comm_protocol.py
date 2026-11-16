# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 39, Rank 189)
# CDYLIB: strip_comm_protocol.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_comm_protocol.dll"
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
Mutual Agent Trust (MAT) Protocol - Phase 5
Standardized communication and cryptographic trust for autonomous agent loops.
"""

import hmac
import hashlib
import time
import logging
from typing import Dict, Any, Optional

logger = logging.getLogger("MAT")
SECRET_KEY = "swarm_v5_autonomy_alpha_secret"

def generate_trust_token(sender_role: str, target_role: str, task_id: str) -> str:
    """
    Generate a task-scoped, time-windowed HMAC token for secure agent-to-agent messaging.
    Default window: 300 seconds (5 minutes).
    """
    window = int(time.time() / 300)
    payload = f"{sender_role}->{target_role}:{task_id}:{window}"
    return hmac.new(SECRET_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()

def verify_trust_token(sender_role: str, target_role: str, task_id: str, token: str) -> bool:
    """
    Verify an incoming agent message against the current or previous time window.
    """
    current_window = int(time.time() / 300)
    # Check two windows to avoid race conditions at window boundaries
    for window in [current_window, current_window - 1]:
        payload = f"{sender_role}->{target_role}:{task_id}:{window}"
        expected = hmac.new(SECRET_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()
        if hmac.compare_digest(expected, token):
            return True
            
    logger.warning(f"TRUST FAILURE: Unauthorized message from {sender_role} to {target_role} (Task: {task_id})")
    return False

class AgentHandshake:
    """Helper class for managing agent-to-agent handshakes."""
    
    @staticmethod
    def create_secure_payload(sender_role: str, target_role: str, task_id: str, data: Any) -> Dict[str, Any]:
        token = generate_trust_token(sender_role, target_role, task_id)
        return {
            "header": {
                "sender": sender_role,
                "target": target_role,
                "task_id": task_id,
                "trust_token": token,
                "protocol": "MAT/1.0"
            },
            "content": data
        }

    @staticmethod
    def validate_secure_payload(payload: Dict[str, Any]) -> bool:
        header = payload.get("header", {})
        return verify_trust_token(
            header.get("sender"),
            header.get("target"),
            header.get("task_id"),
            header.get("trust_token")
        )

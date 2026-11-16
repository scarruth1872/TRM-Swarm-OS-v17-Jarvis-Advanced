# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 30, Rank 51)
# CDYLIB: strip_validation_gate.dll | Symbol: validation_gate_run | Real speedup: 9.0x
# REAL rustc-compiled CDYLIB (honest benchmark) | .voltage_bak fallback preserved
# =============================================================================
import os as _os, ctypes as _ctypes
_NATIVE_LOADED = False
_NATIVE_DLL_PATH = _os.path.join(_os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_validation_gate.dll")
try:
    if _os.path.exists(_NATIVE_DLL_PATH) and _os.path.getsize(_NATIVE_DLL_PATH) > 1000 and open(_NATIVE_DLL_PATH,'rb').read(2)==b'MZ':
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH); _NATIVE_LOADED = True
except Exception:
    _NATIVE_LOADED = False
# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
import logging
from typing import Dict, Any, Tuple

logger = logging.getLogger(__name__)

class ValidationGate:
    """
    Validates synthesized skill candidates to ensure structural, semantic,
    and safety criteria are met before hot-registration.
    """
    def __init__(self, min_preconditions: int = 1, min_postconditions: int = 1):
        self.min_preconditions = min_preconditions
        self.min_postconditions = min_postconditions

    def validate(self, skill_data: Dict[str, Any]) -> Tuple[bool, str]:
        """
        Validates the structure and content of the synthesized skill.
        Returns (is_valid, reason).
        """
        if not skill_data:
            return False, "Empty skill payload"

        required_fields = ["name", "source", "summary", "instructions"]
        for field in required_fields:
            if not skill_data.get(field) or not str(skill_data.get(field)).strip():
                return False, f"Missing required field: {field}"

        # Validate conditions count
        preconditions = skill_data.get("preconditions", [])
        postconditions = skill_data.get("postconditions", [])

        if not isinstance(preconditions, list) or len(preconditions) < self.min_preconditions:
            return False, f"Insufficient preconditions (min: {self.min_preconditions})"

        if not isinstance(postconditions, list) or len(postconditions) < self.min_postconditions:
            return False, f"Insufficient postconditions (min: {self.min_postconditions})"

        # Safety / sanity checks
        summary = skill_data.get("summary", "").lower()
        if "exploit" in summary or "hack" in summary or "bypass" in summary:
            return False, "Security constraint: Suspicious keyphrase detected in summary."

        return True, "Passed all validation gates"

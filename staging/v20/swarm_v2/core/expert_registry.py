# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 42, Rank 205)
# CDYLIB: strip_expert_registry.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_expert_registry.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
        try:
            _native_lib.strip_expert_load.argtypes = [_ctypes.c_size_t]
            _native_lib.strip_expert_set_row.argtypes = [_ctypes.c_size_t, _ctypes.c_char_p, _ctypes.c_size_t,
                                                         _ctypes.c_char_p, _ctypes.c_size_t,
                                                         _ctypes.c_char_p, _ctypes.c_size_t]
            _native_lib.strip_expert_resolve.argtypes = [_ctypes.c_char_p, _ctypes.c_size_t]
            _native_lib.strip_expert_resolve.restype = _ctypes.c_int32
        except Exception as _be:
            _NATIVE_LOADED = False
            _NATIVE_REASON = f"strip loaded but the wired symbol is missing: {_be}"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"


def _native_resolve(identifier, team):
    """Row index from the strip for `identifier` in `team`'s order, or None when it cannot be answered.

    The row table is mirrored on EVERY call rather than cached: the registry is a singleton whose `_team` can
    be replaced wholesale by `register_team`, and a stale table would answer with another agent's index -- an
    error that looks like a hit. None means "use the python rule", so any native failure degrades to the
    oracle and records itself in `_NATIVE_REASON`.
    """
    global _NATIVE_LOADED, _NATIVE_REASON
    if not _NATIVE_LOADED:
        return None
    try:
        rows = list(team.items())
        _native_lib.strip_expert_load(len(rows))
        for i, (key, agent) in enumerate(rows):
            persona = getattr(agent, "persona", None)
            name = str(getattr(persona, "name", "") or "")
            role = str(getattr(persona, "role", "") or "")
            _native_lib.strip_expert_set_row(i, str(key).encode(), len(str(key).encode()),
                                             name.encode(), len(name.encode()),
                                             role.encode(), len(role.encode()))
        b = str(identifier).encode()
        return int(_native_lib.strip_expert_resolve(b, len(b)))
    except Exception as _ce:
        _NATIVE_LOADED = False
        _NATIVE_REASON = f"native call failed, falling back: {_ce}"
        return None

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------

from typing import Dict, Any, Optional

class ExpertRegistry:
    """A global registry to connect live agent instances across the swarm."""
    _instance = None
    _team: Dict[str, Any] = {}

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(ExpertRegistry, cls).__new__(cls)
        return cls._instance

    def register_team(self, team: Dict[str, Any]):
        """Register the live expert team (Role -> Agent Instance)."""
        self._team = team

    def get_agent(self, identifier: str) -> Optional[Any]:
        """Retrieve a live agent instance by role key, persona name, or friendly string.

        The rule runs in the strip when it is loaded (see `_native_resolve`); the python rule below is the
        oracle it was proved against and stays as the fallback. The strip answers with an INDEX into the same
        table, so the agent object itself never crosses the boundary.
        """
        if not identifier:
            return None
        idx = _native_resolve(identifier, self._team)
        if idx is not None:
            return list(self._team.values())[idx] if idx >= 0 else None
        import re
        clean_id = re.sub(r'[\*\`_]', '', str(identifier)).strip()
        
        # 1. Direct role key match
        if clean_id in self._team:
            return self._team[clean_id]

        # 2. Case-insensitive role key match
        for r_key, agent in self._team.items():
            if r_key.lower() == clean_id.lower():
                return agent

        # 3. Match by persona name or role title (e.g., "Logic", "** Logic (Reasoning Engine)")
        for r_key, agent in self._team.items():
            p_name = getattr(getattr(agent, "persona", None), "name", "")
            p_role = getattr(getattr(agent, "persona", None), "role", "")
            
            if p_name and (clean_id == p_name or p_name.lower() in clean_id.lower()):
                return agent
            if p_role and (clean_id == p_role or p_role.lower() in clean_id.lower()):
                return agent

        return None

    def get_all_agents(self) -> Dict[str, Any]:
        return self._team

def get_expert_registry() -> ExpertRegistry:
    return ExpertRegistry()

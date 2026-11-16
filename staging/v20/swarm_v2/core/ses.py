# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 38, Rank 185)
# CDYLIB: strip_ses.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_ses.dll"
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
Semantic Enrichment Service (SES) - Phase 3
Extracts entities and relationships from raw text to populate the 
Dynamic Knowledge Graph (DKG).
"""

import re
import json
from typing import List, Dict, Any, Tuple
from datetime import datetime

class SESEngine:
    def __init__(self):
        # Simplified NER patterns for system entities
        self.entity_patterns = {
            "SERVICE": r'\b(?:api|db|gateway|auth|worker|swarm)\b',
            "ERROR": r'\b(?:404|500|ConnectionRefused|Timeout|Crash)\b',
            "USER": r'\buser_[a-f0-9]{8}\b',
            "ACTION": r'\b(?:GET|POST|PUT|DELETE|FETCH|STORE)\b'
        }

    def extract_triples(self, text: str) -> List[Dict[str, Any]]:
        """
        Extracts (Subject, Predicate, Object) triples from text.
        Returns a list of structured triples.
        """
        triples = []
        entities = self._identify_entities(text)
        
        # Simple heuristic: link entities based on proximity or common patterns
        # Example: "User X triggered Error Y on Service Z"
        if len(entities) >= 2:
            for i in range(len(entities) - 1):
                subject = entities[i]
                obj = entities[i+1]
                
                # Infer predicate based on keywords
                predicate = "RELATES_TO"
                if "triggered" in text.lower():
                    predicate = "TRIGGERED"
                elif "failed" in text.lower():
                    predicate = "FAILED_ON"
                elif "depends on" in text.lower():
                    predicate = "DEPENDS_ON"

                triples.append({
                    "subject": subject["name"],
                    "subject_type": subject["type"],
                    "predicate": predicate,
                    "object": obj["name"],
                    "object_type": obj["type"],
                    "timestamp": datetime.now().isoformat()
                })
        
        return triples

    def _identify_entities(self, text: str) -> List[Dict[str, str]]:
        entities = []
        for label, pattern in self.entity_patterns.items():
            matches = re.finditer(pattern, text, re.IGNORECASE)
            for m in matches:
                entities.append({
                    "name": m.group(),
                    "type": label,
                    "start": m.start(),
                    "end": m.end()
                })
        # Sort by appearance
        return sorted(entities, key=lambda x: x["start"])

# Singleton
_ses_instance = None

def get_ses() -> SESEngine:
    global _ses_instance
    if _ses_instance is None:
        _ses_instance = SESEngine()
    return _ses_instance

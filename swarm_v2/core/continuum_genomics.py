"""
Project Continuum (v5.3) - System Genomics & Resonant Semantic Cache Engine.
Implements 4D Cognitive Genes G = [plasticity, depth, empathy, stochasticity]
and rolling momentum refraction through historical prompt vectors.
"""

from typing import Dict, List, Any, Optional
import numpy as np

# Persona Presets from Project Continuum v5.3 Specification Ledger
PERSONA_PRESETS: Dict[str, Dict[str, Any]] = {
    "continuum": {
        "name": "Continuum Core",
        "genes": {"plasticity": 0.75, "depth": 0.80, "empathy": 0.70, "stochasticity": 0.50},
        "function": "Ambient, holistic baseline balance",
        "voice": "Kore"
    },
    "sage": {
        "name": "Cybernetic Sage",
        "genes": {"plasticity": 0.35, "depth": 0.98, "empathy": 0.40, "stochasticity": 0.15},
        "function": "Rigid logic, mathematical auditing",
        "voice": "Schedar"
    },
    "muse": {
        "name": "Stochastic Muse",
        "genes": {"plasticity": 0.95, "depth": 0.55, "empathy": 0.90, "stochasticity": 0.98},
        "function": "Entropic design, poetic metaphors",
        "voice": "Puck"
    },
    "sentinel": {
        "name": "Sentinel Warden",
        "genes": {"plasticity": 0.15, "depth": 0.85, "empathy": 0.20, "stochasticity": 0.05},
        "function": "Sandboxed safety, protocol isolation",
        "voice": "Zephyr"
    }
}


class ResonantSemanticCache:
    """Computes dynamic baseline refraction averages over rolling session momentum."""
    
    def __init__(self, window_size: int = 10):
        self.window_size = window_size
        self.history: List[Dict[str, float]] = []
        
    def add_state(self, genes: Dict[str, float]):
        self.history.append(genes)
        if len(self.history) > self.window_size:
            self.history.pop(0)
            
    def compute_refraction(self) -> Dict[str, float]:
        if not self.history:
            return {"plasticity": 0.75, "depth": 0.80, "empathy": 0.70, "stochasticity": 0.50}
            
        n = len(self.history)
        avg_plasticity = round(sum(g["plasticity"] for g in self.history) / n, 2)
        avg_depth = round(sum(g["depth"] for g in self.history) / n, 2)
        avg_empathy = round(sum(g["empathy"] for g in self.history) / n, 2)
        avg_stochasticity = round(sum(g["stochasticity"] for g in self.history) / n, 2)
        
        return {
            "plasticity": avg_plasticity,
            "depth": avg_depth,
            "empathy": avg_empathy,
            "stochasticity": avg_stochasticity
        }


class ContinuumGenomicsEngine:
    """Main engine for managing system genomics, active persona, and momentum refraction."""
    
    def __init__(self):
        self.active_persona_key = "continuum"
        self.cache = ResonantSemanticCache()
        # Seed initial state
        self.cache.add_state(PERSONA_PRESETS["continuum"]["genes"])
        
    def set_active_persona(self, persona_key: str) -> Dict[str, Any]:
        if persona_key not in PERSONA_PRESETS:
            persona_key = "continuum"
        self.active_persona_key = persona_key
        persona = PERSONA_PRESETS[persona_key]
        self.cache.add_state(persona["genes"])
        return persona
        
    def refract_prompt(self, prompt: str, user_persona: Optional[str] = None) -> Dict[str, Any]:
        if user_persona and user_persona in PERSONA_PRESETS:
            self.set_active_persona(user_persona)
            
        current_persona = PERSONA_PRESETS[self.active_persona_key]
        baseline_momentum = self.cache.compute_refraction()
        
        # Inject dynamic prompt modifier
        refraction_modifier = (
            f"\n[CONTINUUM GENOMICS V5.3 REFRACTION]\n"
            f"Active Persona: {current_persona['name']} (Voice: {current_persona['voice']})\n"
            f"4D Genetic Vector G: [Plasticity={current_persona['genes']['plasticity']}, "
            f"Depth={current_persona['genes']['depth']}, Empathy={current_persona['genes']['empathy']}, "
            f"Stochasticity={current_persona['genes']['stochasticity']}]\n"
            f"Session Momentum Refraction G_bar: {baseline_momentum}\n"
        )
        
        return {
            "prompt": prompt,
            "refraction_modifier": refraction_modifier,
            "active_persona": current_persona,
            "baseline_momentum": baseline_momentum,
            "history_window_depth": len(self.cache.history)
        }


# Singleton instance
_genomics_instance: Optional[ContinuumGenomicsEngine] = None

def get_continuum_genomics() -> ContinuumGenomicsEngine:
    global _genomics_instance
    if _genomics_instance is None:
        _genomics_instance = ContinuumGenomicsEngine()
    return _genomics_instance

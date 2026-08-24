import json
import numpy as np
from datetime import datetime

class QuantumPersonalityModel:
    """
    Tracks personality as a probabilistic state-vector rather than a static type.
    Embraces superposition, entanglement, and tunneling.
    """
    def __init__(self):
        # Initial state: 50% probability distribution (Neutral superposition)
        self.state_vector = {
            "IE": 0.5,  # Introversion/Extroversion
            "SN": 0.5,  # Sensing/Intuition
            "TF": 0.5,  # Thinking/Feeling
            "JP": 0.5,  # Judging/Perceiving
            "Adaptability": 0.5 # Normalized 0-1
        }
        self.history = []

    def update_state(self, observation_dict):
        """
        Updates the probability distribution based on new interaction data.
        observation_dict: e.g., {"IE": 0.8, "Adaptability": 0.9}
        """
        # Apply a simple learning rate (alpha) to drift the state-vector
        alpha = 0.1
        for trait, val in observation_dict.items():
            if trait in self.state_vector:
                # Quantum approach: move toward the observed state
                self.state_vector[trait] = (1 - alpha) * self.state_vector[trait] + alpha * val
        
        self.history.append({
            "timestamp": datetime.now().isoformat(),
            "state": self.state_vector.copy()
        })
        return self.state_vector

    def detect_tunneling(self, threshold=0.3):
        """
        Detects if current state has drifted significantly from the mean state (Tunneling).
        """
        # Placeholder for complex variance analysis
        return False

    def get_profile(self):
        return {k: f"{v*100:.1f}%" for k, v in self.state_vector.items()}

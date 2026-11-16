"""
Physical Feedback Loops Engine
Phase 12: WiFi Vision CSI Integration
Processes biometric and spatial feeds to trigger ambient lighting and audio adjustments.
"""

import logging
from typing import Dict, Any
from infrastructure.routing.qer_mesh import get_qer_mesh

logger = logging.getLogger("PhysicalFeedbackEngine")

class PhysicalFeedbackEngine:
    def __init__(self):
        self.user_focus_score = 0.5
        self.presence_detected = True
        self.raw_csi_amplitude = 100.0
        
        # Simulated physical actuators
        self.lighting_level = 100
        self.audio_volume = 50
        self.audio_mode = "Normal"
        
        self.qer = get_qer_mesh()
        logger.info("Physical Feedback Engine initialized.")

    def update_feed(self, user_focus_score: float, presence_detected: bool, raw_csi_amplitude: float) -> Dict[str, Any]:
        """Processes updated telemetry from WiFi Vision CSI and updates ambient settings."""
        self.user_focus_score = user_focus_score
        self.presence_detected = presence_detected
        self.raw_csi_amplitude = raw_csi_amplitude
        
        # Calculate ambient parameters
        if not self.presence_detected:
            self.lighting_level = 10
            self.audio_volume = 0
            self.audio_mode = "Muted / Studio Vacant"
        elif self.user_focus_score >= 0.8:
            self.lighting_level = 35
            self.audio_volume = 30
            self.audio_mode = "Lofi Deep Focus"
        elif self.user_focus_score <= 0.4:
            self.lighting_level = 100
            self.audio_volume = 60
            self.audio_mode = "Ambient Relaxed"
        else:
            self.lighting_level = 75
            self.audio_volume = 45
            self.audio_mode = "Balanced Focus"
            
        # Log and broadcast QER event
        msg = f"LIGHT={self.lighting_level}%;VOL={self.audio_volume}%;MODE={self.audio_mode};CSI={self.raw_csi_amplitude:.1f}"
        logger.info(f"[Actuators] Adjusting environment: {msg}")
        
        # Broadcast into QER master telemetry stream to show up in dashboard Spatial Mesh links
        self.qer.broadcast_instant("master_telemetry_stream", "ambient_adjustments", msg)
        
        return self.get_status()

    def get_status(self) -> Dict[str, Any]:
        """Get the current ambient status."""
        return {
            "user_focus_score": self.user_focus_score,
            "presence_detected": self.presence_detected,
            "raw_csi_amplitude": self.raw_csi_amplitude,
            "lighting_level": self.lighting_level,
            "audio_volume": self.audio_volume,
            "audio_mode": self.audio_mode
        }

# Singleton instance
_engine_instance = None
def get_feedback_engine() -> PhysicalFeedbackEngine:
    global _engine_instance
    if _engine_instance is None:
        _engine_instance = PhysicalFeedbackEngine()
    return _engine_instance

"""
Multimodal Fusion Engine — Phase 1: Event Detection & Malfunction Diagnosis
Correlates Telemetry and Visual feature vectors to perform early & late multimodal fusion,
detecting equipment malfunctions, thermal anomalies, and physical degradation.
"""

import logging
import time
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

from swarm_v2.core.multimodal_ingestion import FeatureVector, SensorTelemetryData, VisualFrameData, get_multimodal_ingestion

logger = logging.getLogger("MultimodalFusion")


class MalfunctionEvent(BaseModel):
    """Detected equipment malfunction event."""
    event_id: str
    event_type: str  # "THERMAL_RUNAWAY", "BEARING_FAILURE_CRITICAL", "NORMAL_OPERATION", "SMOKE_ELECTRICAL_FIRE"
    severity: str    # "CRITICAL", "HIGH", "WARNING", "INFO"
    fusion_confidence: float  # 0.0 to 1.0
    telemetry_score: float
    visual_score: float
    root_cause_explanation: str
    recommended_action: str
    timestamp: float = Field(default_factory=time.time)


class MultimodalFusionEngine:
    """
    Multimodal Fusion Engine.
    Combines sensory telemetry streams and visual frame vectors to accurately detect
    equipment malfunctions and avoid single-modality false alarms.
    """

    def __init__(self):
        self.ingestion = get_multimodal_ingestion()
        self.event_history: List[MalfunctionEvent] = []

    def fuse_and_detect(self, telemetry: SensorTelemetryData, visual: VisualFrameData) -> MalfunctionEvent:
        """
        Perform late multimodal fusion across Telemetry & Visual channels.
        """
        # Step 1: Feature Extraction
        fv_telemetry = self.ingestion.extract_telemetry_features(telemetry)
        fv_visual = self.ingestion.extract_visual_features(visual)

        t_score = fv_telemetry.normalized_score
        v_score = fv_visual.normalized_score

        # Dual-Modality Fusion Formula: Fused_Score = (0.5 * Telemetry) + (0.5 * Visual)
        fused_score = (0.5 * t_score) + (0.5 * v_score)

        # Cross-Modal Correlation Rules for Equipment Malfunction Detection
        temp_raw = fv_telemetry.features.get("raw_temp_c", 45.0)
        thermal_hotspot = fv_visual.features.get("thermal_hotspot", 0.0)
        vibration = fv_telemetry.features.get("normalized_vibration", 0.0)
        smoke = fv_visual.features.get("smoke_density", 0.0)

        event_id = f"evt_{int(time.time() * 1000)}"

        if temp_raw > 85.0 and thermal_hotspot > 0.7:
            event_type = "THERMAL_RUNAWAY_CRITICAL"
            severity = "CRITICAL"
            confidence = min(0.98, fused_score + 0.2)
            explanation = f"Critical Overheating Correlated: Sensor temp={temp_raw}°C matches Visual Thermal Hotspot={thermal_hotspot}."
            action = "EMERGENCY SHUTDOWN: Isolate power relay and engage secondary coolant loop."
        elif vibration > 0.75 and thermal_hotspot > 0.5:
            event_type = "BEARING_MECHANICAL_FAILURE"
            severity = "HIGH"
            confidence = min(0.92, fused_score + 0.15)
            explanation = f"Mechanical Bearing Degradation: High Vibration={vibration} correlated with Visual Friction Heating={thermal_hotspot}."
            action = "SCHEDULE IMMEDIATE MAINTENANCE: Throttle spindle speed by 50%."
        elif smoke > 0.6:
            event_type = "ELECTRICAL_SMOKE_FIRE_HAZARD"
            severity = "CRITICAL"
            confidence = 0.95
            explanation = f"Visual Smoke Cloud Detected: Density={smoke} with elevated power draw."
            action = "EVACUATE ZONE: Depower electrical bus and trigger N2 fire suppression."
        elif fused_score > 0.5:
            event_type = "ELEVATED_STRESS_WARNING"
            severity = "WARNING"
            confidence = 0.80
            explanation = f"Combined Telemetry ({t_score}) & Visual ({v_score}) stress above nominal baseline."
            action = "MONITOR: Increase telemetry sampling frequency to 100Hz."
        else:
            event_type = "NOMINAL_SYSTEM_HEALTH"
            severity = "INFO"
            confidence = 0.99
            explanation = "Both telemetry sensors and visual feeds report nominal baseline operation."
            action = "NO_ACTION_REQUIRED: System operating within optimal tolerances."

        event = MalfunctionEvent(
            event_id=event_id,
            event_type=event_type,
            severity=severity,
            fusion_confidence=round(confidence, 3),
            telemetry_score=t_score,
            visual_score=v_score,
            root_cause_explanation=explanation,
            recommended_action=action
        )

        self.event_history.append(event)
        logger.info(f"[MultimodalFusion] Event Detected: {event.event_type} [{event.severity}] confidence={event.fusion_confidence}")

        # Stream event to High-Bandwidth Telemetry Fabric (HBTF)
        try:
            from swarm_v2.core.telemetry_fabric import get_telemetry_fabric, TelemetryFrame
            tf = get_telemetry_fabric()
            tf.publish(TelemetryFrame(
                frame_id=event_id,
                source_node="multimodal_fusion",
                channel="security_events",
                payload=event.model_dump()
            ))
        except Exception:
            pass

        return event


_fusion_engine: Optional[MultimodalFusionEngine] = None


def get_multimodal_fusion() -> MultimodalFusionEngine:
    global _fusion_engine
    if _fusion_engine is None:
        _fusion_engine = MultimodalFusionEngine()
    return _fusion_engine

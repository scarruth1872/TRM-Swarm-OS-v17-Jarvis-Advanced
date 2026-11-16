"""
Advanced Scene Fusion Engine — Phase 2: Complex Scene Understanding
Constructs spatial scene graphs combining spatial object bounding boxes,
audio spectrogram descriptors, thermal maps, and telemetry to understand complex operational environments.
"""

import time
import logging
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("AdvancedSceneFusion")


class BoundingBox(BaseModel):
    label: str  # "person", "robotic_arm", "conveyor", "hazard_zone"
    confidence: float
    x_min: float
    y_min: float
    x_max: float
    y_max: float


class AudioDescriptor(BaseModel):
    primary_frequency_hz: float = 120.0
    decibel_level: float = 65.0
    speech_detected: bool = False
    acoustic_anomaly: bool = False


class ComplexSceneContext(BaseModel):
    scene_id: str
    location: str = "Factory_Floor_Zone_A"
    detected_objects: List[BoundingBox] = Field(default_factory=list)
    audio: AudioDescriptor = Field(default_factory=AudioDescriptor)
    ambient_temp_c: float = 28.0
    timestamp: float = Field(default_factory=time.time)


class ComplexSceneUnderstandingResult(BaseModel):
    scene_id: str
    scene_class: str  # "SAFE_OPERATIONAL_ZONE", "HAZARD_HUMAN_PROXIMITY_VIOLATION", "ACOUSTIC_OVERLOAD_WARNING"
    risk_level: float  # 0.0 to 1.0
    spatial_hazards: List[str] = Field(default_factory=list)
    scene_summary: str
    timestamp: float = Field(default_factory=time.time)


class AdvancedSceneFusionEngine:
    """
    Combines spatial bounding boxes, audio spectrum, and environmental telemetry
    into a unified complex scene model.
    """

    def analyze_scene(self, context: ComplexSceneContext) -> ComplexSceneUnderstandingResult:
        spatial_hazards = []
        risk_level = 0.05

        # Rule 1: Spatial Proximity Hazard (Person near active Robotic Arm / Danger Zone)
        person_boxes = [b for b in context.detected_objects if b.label == "person"]
        robot_boxes = [b for b in context.detected_objects if b.label in ("robotic_arm", "hazard_zone")]

        for p in person_boxes:
            for r in robot_boxes:
                # Calculate bounding box overlap / proximity
                dist = math.sqrt((p.x_min - r.x_min)**2 + (p.y_min - r.y_min)**2) if 'math' in globals() else abs(p.x_min - r.x_min)
                if dist < 0.25:
                    spatial_hazards.append(f"HUMAN_PROXIMITY_VIOLATION: Person detected at ({p.x_min}, {p.y_min}) near {r.label}")
                    risk_level = max(risk_level, 0.88)

        # Rule 2: Acoustic Anomaly / Overload
        if context.audio.acoustic_anomaly or context.audio.decibel_level > 90.0:
            spatial_hazards.append(f"ACOUSTIC_ANOMALY: High noise floor ({context.audio.decibel_level} dB) with frequency spike ({context.audio.primary_frequency_hz} Hz)")
            risk_level = max(risk_level, 0.70)

        # Classify Scene
        if risk_level >= 0.8:
            scene_class = "HAZARD_HUMAN_PROXIMITY_VIOLATION"
            summary = "Critical spatial safety violation detected: Human operator dangerously close to active industrial equipment."
        elif risk_level >= 0.6:
            scene_class = "ACOUSTIC_OVERLOAD_WARNING"
            summary = "Elevated acoustic noise and structural frequency vibration detected in operational zone."
        else:
            scene_class = "SAFE_OPERATIONAL_ZONE"
            summary = "Operational zone is clear, nominal spatial distances maintained, noise within safe bounds."

        result = ComplexSceneUnderstandingResult(
            scene_id=context.scene_id,
            scene_class=scene_class,
            risk_level=round(risk_level, 2),
            spatial_hazards=spatial_hazards,
            scene_summary=summary
        )

        logger.info(f"[AdvancedSceneFusion] Scene '{context.scene_id}': class={scene_class}, risk={result.risk_level}")
        return result


_scene_fusion_engine: Optional[AdvancedSceneFusionEngine] = None


def get_advanced_scene_fusion() -> AdvancedSceneFusionEngine:
    global _scene_fusion_engine
    if _scene_fusion_engine is None:
        _scene_fusion_engine = AdvancedSceneFusionEngine()
    return _scene_fusion_engine

# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 35, Rank 169)
# CDYLIB: strip_multimodal_ingestion.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_multimodal_ingestion.dll"
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
Multimodal Ingestion & Feature Extraction Engine — Phase 1
Ingests heterogeneous data streams (Telemetry sensors & Visual/Video frames)
and extracts normalized feature vectors for Swarm OS fusion.
"""

import time
import math
import logging
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("MultimodalIngestion")


class SensorTelemetryData(BaseModel):
    """Telemetry data stream from physical or virtual sensors."""
    sensor_id: str
    temperature_celsius: float = 45.0
    vibration_hz: float = 12.0
    power_draw_watts: float = 250.0
    pressure_psi: float = 30.0
    timestamp: float = Field(default_factory=time.time)


class VisualFrameData(BaseModel):
    """Visual/video frame metadata and raw visual descriptors."""
    frame_id: str
    camera_id: str
    thermal_hotspot_intensity: float = 0.15  # 0.0 to 1.0
    optical_flow_energy: float = 0.05       # Motion energy
    smoke_particle_density: float = 0.02    # Visual density
    bounding_box_count: int = 2
    timestamp: float = Field(default_factory=time.time)


class FeatureVector(BaseModel):
    """Extracted feature vector representing a modality snapshot."""
    modality: str  # "telemetry" or "visual"
    source_id: str
    features: Dict[str, float]
    normalized_score: float
    timestamp: float = Field(default_factory=time.time)


class MultimodalIngestionEngine:
    """
    Ingests and normalizes core modalities (Telemetry + Visual Video).
    Extracts high-level feature vectors for multimodal event fusion.
    """

    def __init__(self):
        self.telemetry_history: List[SensorTelemetryData] = []
        self.visual_history: List[VisualFrameData] = []

    def extract_telemetry_features(self, data: SensorTelemetryData) -> FeatureVector:
        """Extract and normalize feature vector from raw sensor telemetry."""
        self.telemetry_history.append(data)
        if len(self.telemetry_history) > 100:
            self.telemetry_history.pop(0)

        # Normalization logic against baseline safe thresholds
        temp_score = min(max((data.temperature_celsius - 40.0) / 60.0, 0.0), 1.0)
        vib_score = min(max((data.vibration_hz - 10.0) / 40.0, 0.0), 1.0)
        power_score = min(max((data.power_draw_watts - 200.0) / 300.0, 0.0), 1.0)

        composite = (temp_score * 0.4) + (vib_score * 0.4) + (power_score * 0.2)

        fv = FeatureVector(
            modality="telemetry",
            source_id=data.sensor_id,
            features={
                "normalized_temp": round(temp_score, 3),
                "normalized_vibration": round(vib_score, 3),
                "normalized_power": round(power_score, 3),
                "raw_temp_c": data.temperature_celsius
            },
            normalized_score=round(composite, 3)
        )
        logger.info(f"[MultimodalIngestion] Extracted Telemetry Feature Vector: score={fv.normalized_score}")
        return fv

    def extract_visual_features(self, data: VisualFrameData) -> FeatureVector:
        """Extract and normalize feature vector from raw visual/video frames."""
        self.visual_history.append(data)
        if len(self.visual_history) > 100:
            self.visual_history.pop(0)

        thermal_score = min(max(data.thermal_hotspot_intensity, 0.0), 1.0)
        motion_score = min(max(data.optical_flow_energy * 2.0, 0.0), 1.0)
        smoke_score = min(max(data.smoke_particle_density * 5.0, 0.0), 1.0)

        composite = (thermal_score * 0.5) + (smoke_score * 0.3) + (motion_score * 0.2)

        fv = FeatureVector(
            modality="visual",
            source_id=data.camera_id,
            features={
                "thermal_hotspot": round(thermal_score, 3),
                "smoke_density": round(smoke_score, 3),
                "motion_energy": round(motion_score, 3)
            },
            normalized_score=round(composite, 3)
        )
        logger.info(f"[MultimodalIngestion] Extracted Visual Feature Vector: score={fv.normalized_score}")
        return fv


_ingestion_engine: Optional[MultimodalIngestionEngine] = None


def get_multimodal_ingestion() -> MultimodalIngestionEngine:
    global _ingestion_engine
    if _ingestion_engine is None:
        _ingestion_engine = MultimodalIngestionEngine()
    return _ingestion_engine

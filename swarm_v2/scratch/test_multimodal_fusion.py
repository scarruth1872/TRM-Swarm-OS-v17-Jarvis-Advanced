"""
Test Suite for Multimodal Data Ingestion & Fusion Engine (Phase 1)
"""

import sys
import logging

logging.basicConfig(level=logging.INFO)

def test_multimodal_fusion_scenarios():
    from swarm_v2.core.multimodal_ingestion import SensorTelemetryData, VisualFrameData, get_multimodal_ingestion
    from swarm_v2.core.multimodal_fusion import get_multimodal_fusion

    fusion = get_multimodal_fusion()

    print("==========================================================================================")
    print(" [MULTIMODAL] MULTIMODAL AI INGESTION & FUSION TEST SUITE (PHASE 1)")
    print("==========================================================================================")

    # Scenario 1: Nominal Baseline Operation
    print("\n--- Scenario 1: Nominal Baseline Operation ---")
    t1 = SensorTelemetryData(sensor_id="sensor_01", temperature_celsius=42.0, vibration_hz=11.0, power_draw_watts=220.0)
    v1 = VisualFrameData(frame_id="f1", camera_id="cam_01", thermal_hotspot_intensity=0.10, optical_flow_energy=0.02)
    evt1 = fusion.fuse_and_detect(t1, v1)
    print(f"Event Type: {evt1.event_type} [{evt1.severity}] (Confidence: {evt1.fusion_confidence})")
    print(f"Explanation: {evt1.root_cause_explanation}")
    assert evt1.event_type == "NOMINAL_SYSTEM_HEALTH", "Scenario 1 failed"

    # Scenario 2: Thermal Runaway Equipment Malfunction (Correlated Sensor + Visual)
    print("\n--- Scenario 2: Thermal Runaway Equipment Malfunction ---")
    t2 = SensorTelemetryData(sensor_id="sensor_01", temperature_celsius=92.5, vibration_hz=15.0, power_draw_watts=450.0)
    v2 = VisualFrameData(frame_id="f2", camera_id="cam_01", thermal_hotspot_intensity=0.88, optical_flow_energy=0.08)
    evt2 = fusion.fuse_and_detect(t2, v2)
    print(f"Event Type: {evt2.event_type} [{evt2.severity}] (Confidence: {evt2.fusion_confidence})")
    print(f"Explanation: {evt2.root_cause_explanation}")
    print(f"Recommended Action: {evt2.recommended_action}")
    assert evt2.event_type == "THERMAL_RUNAWAY_CRITICAL", "Scenario 2 failed"

    # Scenario 3: Bearing Mechanical Failure (Vibration + Visual Heat)
    print("\n--- Scenario 3: Bearing Mechanical Failure ---")
    t3 = SensorTelemetryData(sensor_id="spindle_sensor", temperature_celsius=65.0, vibration_hz=48.0, power_draw_watts=380.0)
    v3 = VisualFrameData(frame_id="f3", camera_id="spindle_cam", thermal_hotspot_intensity=0.62, optical_flow_energy=0.25)
    evt3 = fusion.fuse_and_detect(t3, v3)
    print(f"Event Type: {evt3.event_type} [{evt3.severity}] (Confidence: {evt3.fusion_confidence})")
    print(f"Explanation: {evt3.root_cause_explanation}")
    assert evt3.event_type == "BEARING_MECHANICAL_FAILURE", "Scenario 3 failed"

    # Scenario 4: Electrical Smoke Hazard
    print("\n--- Scenario 4: Electrical Smoke Hazard ---")
    t4 = SensorTelemetryData(sensor_id="bus_sensor", temperature_celsius=70.0, vibration_hz=12.0, power_draw_watts=500.0)
    v4 = VisualFrameData(frame_id="f4", camera_id="bus_cam", thermal_hotspot_intensity=0.45, smoke_particle_density=0.75)
    evt4 = fusion.fuse_and_detect(t4, v4)
    print(f"Event Type: {evt4.event_type} [{evt4.severity}] (Confidence: {evt4.fusion_confidence})")
    print(f"Explanation: {evt4.root_cause_explanation}")
    assert evt4.event_type == "ELECTRICAL_SMOKE_FIRE_HAZARD", "Scenario 4 failed"

    print("\n==========================================================================================")
    print(" [SUCCESS] ALL MULTIMODAL INGESTION & FUSION SCENARIOS PASSED CLEANLY!")
    print("==========================================================================================")

if __name__ == "__main__":
    test_multimodal_fusion_scenarios()

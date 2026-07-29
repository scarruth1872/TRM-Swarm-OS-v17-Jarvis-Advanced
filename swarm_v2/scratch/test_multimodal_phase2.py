"""
Test Suite for Phase 2 Multimodal AI (Scene Fusion, Predictive Analytics, Human-Swarm Commands)
"""

import sys
import logging

logging.basicConfig(level=logging.INFO)

def test_phase2_multimodal():
    print("==========================================================================================")
    print(" [MULTIMODAL PHASE 2] COMPLEX SCENE UNDERSTANDING, PREDICTIVE ANALYTICS & HSI TEST SUITE")
    print("==========================================================================================")

    # Part 1: Advanced Scene Fusion & Spatial Human-Robot Proximity
    print("\n--- Part 1: Complex Scene Understanding & Proximity Hazard ---")
    from swarm_v2.core.advanced_scene_fusion import get_advanced_scene_fusion, ComplexSceneContext, BoundingBox, AudioDescriptor
    sf = get_advanced_scene_fusion()

    ctx = ComplexSceneContext(
        scene_id="scene_floor_09",
        location="Robotics_Assembly_Bay_2",
        detected_objects=[
            BoundingBox(label="person", confidence=0.96, x_min=0.10, y_min=0.10, x_max=0.20, y_max=0.50),
            BoundingBox(label="robotic_arm", confidence=0.99, x_min=0.12, y_min=0.11, x_max=0.40, y_max=0.80)
        ],
        audio=AudioDescriptor(decibel_level=94.0, acoustic_anomaly=True)
    )
    res_scene = sf.analyze_scene(ctx)
    print(f"Scene Class: {res_scene.scene_class} (Risk Level: {res_scene.risk_level})")
    print(f"Spatial Hazards Identified: {res_scene.spatial_hazards}")
    assert res_scene.scene_class == "HAZARD_HUMAN_PROXIMITY_VIOLATION", "Part 1 failed"

    # Part 2: Cognitive Layer Predictive Analytics (TTF Forecasting)
    print("\n--- Part 2: Multimodal Predictive Analytics (TTF & Remediation) ---")
    from swarm_v2.core.multimodal_predictive_analytics import get_multimodal_predictive_analytics, MultimodalTimeSeriesPoint
    pa = get_multimodal_predictive_analytics()

    pts = [
        MultimodalTimeSeriesPoint(temperature_c=45.0, vibration_hz=10.0, acoustic_db=60.0, thermal_hotspot=0.1),
        MultimodalTimeSeriesPoint(temperature_c=65.0, vibration_hz=25.0, acoustic_db=75.0, thermal_hotspot=0.4),
        MultimodalTimeSeriesPoint(temperature_c=88.0, vibration_hz=42.0, acoustic_db=90.0, thermal_hotspot=0.8)
    ]
    res_pa = pa.forecast_degradation("hydraulic_pump_04", pts)
    print(f"Predicted Health Score: {res_pa.predicted_health_score}")
    print(f"Estimated Time-To-Failure (TTF): {res_pa.estimated_time_to_failure_hours} hours (24h Prob: {res_pa.failure_probability_24h * 100}%)")
    print(f"Recommended Remediation: {res_pa.recommended_proactive_remediation}")
    assert res_pa.failure_probability_24h > 0.5, "Part 2 failed"

    # Part 3: Gesture & Voice Command Recognition for Human-Swarm Interaction
    print("\n--- Part 3: Human-Swarm Interaction (Gesture & Voice Recognition) ---")
    from swarm_v2.core.human_swarm_interaction import get_human_swarm_interaction, HumanCommandInput
    hsi = get_human_swarm_interaction()

    # Gesture Command
    cmd_gesture = hsi.parse_human_command(HumanCommandInput(gesture_token="PALM_STOP", target_zone="Zone_B"))
    print(f"Gesture Command Dispatched: Target={cmd_gesture.target_agent}, Action={cmd_gesture.action} (Confidence: {cmd_gesture.confidence})")
    assert cmd_gesture.action == "EMERGENCY_HALT_ZONE", "Gesture test failed"

    # Voice Command
    cmd_voice = hsi.parse_human_command(HumanCommandInput(voice_transcript="Architect initiate self healing on core app", target_zone="Zone_A"))
    print(f"Voice Command Dispatched: Target={cmd_voice.target_agent}, Action={cmd_voice.action} (Confidence: {cmd_voice.confidence})")
    assert cmd_voice.target_agent == "Archi" and cmd_voice.action == "INITIATE_SELF_HEALING", "Voice test failed"

    print("\n==========================================================================================")
    print(" [SUCCESS] ALL PHASE 2 MULTIMODAL AI TESTS PASSED CLEANLY!")
    print("==========================================================================================")

if __name__ == "__main__":
    test_phase2_multimodal()

"""
Human-Swarm Interaction Module — Phase 2: Gesture & Voice Command Recognition
Translates visual operator gestures (e.g. PALM_STOP, POINT_TARGET, EMERGENCY_WAVE)
and audio speech transcripts into executable Swarm OS agent commands.
"""

import logging
import time
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("HumanSwarmInteraction")


class HumanCommandInput(BaseModel):
    operator_id: str = "operator_alpha"
    voice_transcript: Optional[str] = None  # e.g., "Swarm halt motor 3", "Architect initiate self healing"
    gesture_token: Optional[str] = None      # e.g., "PALM_STOP", "POINT_TARGET", "EMERGENCY_WAVE"
    target_zone: str = "Zone_A"
    timestamp: float = Field(default_factory=time.time)


class ExecutableSwarmDirective(BaseModel):
    command_id: str
    target_agent: str  # "ORCH", "Archi", "Shield", "Devo"
    action: str        # "HALT_COMPONENTS", "INITIATE_SELF_HEALING", "LOCKDOWN_ZONE", "SCHEDULE_TASK"
    parameters: Dict[str, Any] = Field(default_factory=dict)
    confidence: float
    status: str = "parsed"
    timestamp: float = Field(default_factory=time.time)


class HumanSwarmInteractionEngine:
    """
    Parses visual gestures and natural speech transcripts from human operators
    into deterministic Swarm OS microkernel directives.
    """

    def parse_human_command(self, input_cmd: HumanCommandInput) -> ExecutableSwarmDirective:
        cmd_id = f"cmd_{int(time.time() * 1000)}"

        # Gesture-based Priority Recognition
        if input_cmd.gesture_token == "PALM_STOP" or input_cmd.gesture_token == "EMERGENCY_WAVE":
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="Shield",
                action="EMERGENCY_HALT_ZONE",
                parameters={"zone": input_cmd.target_zone, "trigger": f"Visual Gesture ({input_cmd.gesture_token})"},
                confidence=0.98,
                status="dispatched"
            )
        elif input_cmd.gesture_token == "POINT_TARGET":
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="ORCH",
                action="FOCUS_INSPECTION_ZONE",
                parameters={"zone": input_cmd.target_zone},
                confidence=0.91,
                status="dispatched"
            )

        # Voice Speech Recognition Parsing
        transcript = (input_cmd.voice_transcript or "").lower()

        if "halt" in transcript or "stop" in transcript or "emergency" in transcript:
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="Shield",
                action="EMERGENCY_HALT_ZONE",
                parameters={"zone": input_cmd.target_zone, "transcript": input_cmd.voice_transcript},
                confidence=0.95,
                status="dispatched"
            )
        elif "self healing" in transcript or "heal" in transcript or "repair" in transcript:
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="Archi",
                action="INITIATE_SELF_HEALING",
                parameters={"transcript": input_cmd.voice_transcript},
                confidence=0.92,
                status="dispatched"
            )
        elif "status" in transcript or "report" in transcript:
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="ORCH",
                action="EMIT_TELEMETRY_REPORT",
                parameters={"transcript": input_cmd.voice_transcript},
                confidence=0.90,
                status="dispatched"
            )
        else:
            return ExecutableSwarmDirective(
                command_id=cmd_id,
                target_agent="Devo",
                action="PROCESS_NATURAL_COMMAND",
                parameters={"raw_transcript": input_cmd.voice_transcript},
                confidence=0.75,
                status="dispatched"
            )


_hsi_engine: Optional[HumanSwarmInteractionEngine] = None


def get_human_swarm_interaction() -> HumanSwarmInteractionEngine:
    global _hsi_engine
    if _hsi_engine is None:
        _hsi_engine = HumanSwarmInteractionEngine()
    return _hsi_engine

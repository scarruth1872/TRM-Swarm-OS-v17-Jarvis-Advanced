//! TRM Swarm OS — Human-Swarm Interaction Module (no_std port).
//! Translates visual operator gestures and speech/terminal commands into executable microkernel directives.
//! 1:1 deterministic core from swarm_v2/core/human_swarm_interaction.py:
//!   - Gesture parsing (PALM_STOP -> Shield, POINT_TARGET -> Orch, EMERGENCY_WAVE)
//!   - Voice/Text directive parsing (halt -> Shield, heal -> Archi, status -> Orch)
//!   - Executable directive synthesis

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ExecutableSwarmDirective {
    pub command_id: u64,
    pub target_agent: &'static str,
    pub action: &'static str,
    pub confidence: f32,
}

pub struct HumanSwarmInteractionEngine {
    pub command_counter: u64,
}

impl HumanSwarmInteractionEngine {
    pub fn new() -> Self {
        HumanSwarmInteractionEngine {
            command_counter: 0,
        }
    }

    pub fn parse_gesture(&mut self, gesture_token: &str) -> ExecutableSwarmDirective {
        self.command_counter += 1;
        let id = self.command_counter;

        if gesture_token == "PALM_STOP" || gesture_token == "EMERGENCY_WAVE" {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Shield",
                action: "EMERGENCY_HALT_ZONE",
                confidence: 0.98,
            }
        } else if gesture_token == "POINT_TARGET" {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Orchestra",
                action: "FOCUS_INSPECTION_ZONE",
                confidence: 0.91,
            }
        } else {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Jarvis",
                action: "GENERAL_GESTURE_INTENT",
                confidence: 0.80,
            }
        }
    }

    pub fn parse_text_command(&mut self, text: &str) -> ExecutableSwarmDirective {
        self.command_counter += 1;
        let id = self.command_counter;
        let t_lower = text.to_ascii_lowercase();

        if t_lower.contains("halt") || t_lower.contains("stop") || t_lower.contains("emergency") {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Shield",
                action: "EMERGENCY_HALT_ZONE",
                confidence: 0.95,
            }
        } else if t_lower.contains("heal") || t_lower.contains("repair") {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Archi",
                action: "INITIATE_SELF_HEALING",
                confidence: 0.92,
            }
        } else if t_lower.contains("status") || t_lower.contains("report") {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Orchestra",
                action: "EMIT_TELEMETRY_REPORT",
                confidence: 0.90,
            }
        } else {
            ExecutableSwarmDirective {
                command_id: id,
                target_agent: "Devo",
                action: "PROCESS_NATURAL_COMMAND",
                confidence: 0.85,
            }
        }
    }
}

static SINGLETON_HUMAN_SWARM: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_human_swarm_interaction() -> u64 {
    let v = SINGLETON_HUMAN_SWARM.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_HUMAN_SWARM.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

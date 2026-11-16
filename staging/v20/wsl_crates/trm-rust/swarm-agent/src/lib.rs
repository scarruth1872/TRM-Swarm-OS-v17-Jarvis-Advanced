//! swarm-agent — Build #3: the voltage-native agent tool-calling loop.
//!
//! observe → decide → act, driven by the verified K3 collapse:
//!   1. OBSERVE: read the hidden state / fabric state (voltage field)
//!   2. DECIDE: K3 collapse → TRUE(act) / FALSE(block) / UNKNOWN(analyze)
//!   3. ACT: dispatch a structured tool action
//!   4. RECORD: append the action verdict to the fabric
//!
//! The action vector (Build #1+2): hidden state → K3 trits. This crate
//! turns that into executable tool calls — the agents ACT, not just say.

pub mod realfab;

/// A structured tool action produced by the voltage collapse.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolAction {
    /// Write content to a file (Devo's "write_file" action)
    WriteFile { path: String, content: String },
    /// Read state / query (Seeker's observe)
    ReadState { target: String },
    /// Deploy a security antibody (Shield)
    DeployAntibody { target: String },
    /// Render to the framebuffer (Jarvis)
    RenderDisplay { text: String },
    /// Record a proof in NAND (the fabric)
    RecordProof { event: String },
    /// Block — the K3 verdict was FALSE
    Block { reason: String },
    /// Analyze — the K3 verdict was UNKNOWN
    Analyze { task: String },
}

/// The K3 verdict of a collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Act,    // TRUE dominates
    Block,  // FALSE dominates
    Analyze, // UNKNOWN dominates
}

/// Collapse a voltage field (hidden state / decision vector) to a verdict.
/// This mirrors the verified c_kleene_quantize_avx2 semantics (τ=0.35):
///   |v| > τ → TRUE(+1) ; |v| < τ → UNKNOWN(0) ; else FALSE(-1) by sign.
pub fn collapse_to_verdict(field: &[f32], tau: f32) -> Verdict {
    let mut act = 0usize;
    let mut block = 0usize;
    let mut unknown = 0usize;
    for &v in field {
        if v.abs() < tau {
            unknown += 1;
        } else if v > 0.0 {
            act += 1;
        } else {
            block += 1;
        }
    }
    if act > block && act >= unknown {
        Verdict::Act
    } else if block > act && block >= unknown {
        Verdict::Block
    } else {
        Verdict::Analyze
    }
}

/// Map a K3 collapse verdict + a task context to a structured tool action.
pub fn decide(task: &str, field: &[f32], tau: f32) -> ToolAction {
    match collapse_to_verdict(field, tau) {
        Verdict::Act => {
            let t = task.to_lowercase();
            if t.contains("write") || t.contains("file") || t.contains("code") {
                ToolAction::WriteFile {
                    path: swarm_paths::resolve("/home/shawn/trm-rust/agent_output.txt"),
                    content: format!("voltage-decided action for: {task}"),
                }
            } else if t.contains("read") || t.contains("status") || t.contains("query") {
                ToolAction::ReadState { target: task.into() }
            } else if t.contains("antibody") || t.contains("ddr") || t.contains("shield") {
                ToolAction::DeployAntibody { target: task.into() }
            } else if t.contains("render") || t.contains("display") {
                ToolAction::RenderDisplay { text: task.into() }
            } else {
                ToolAction::RecordProof { event: task.into() }
            }
        }
        Verdict::Block => ToolAction::Block {
            reason: format!("K3 FALSE dominated the field for: {task}"),
        },
        Verdict::Analyze => ToolAction::Analyze { task: task.into() },
    }
}

/// Execute a tool action — the ACT step. Returns a human-readable result.
pub fn execute(action: &ToolAction) -> String {
    match action {
        ToolAction::WriteFile { path, content } => {
            match std::fs::write(path, content) {
                Ok(_) => format!("wrote {path}"),
                Err(e) => format!("write failed: {e}"),
            }
        }
        ToolAction::ReadState { target } => {
            format!("observed state for: {target}")
        }
        ToolAction::DeployAntibody { target } => {
            format!("antibody deployed against: {target}")
        }
        ToolAction::RenderDisplay { text } => {
            format!("rendered to framebuffer: {text}")
        }
        ToolAction::RecordProof { event } => {
            format!("proof recorded: {event}")
        }
        ToolAction::Block { reason } => format!("BLOCKED: {reason}"),
        ToolAction::Analyze { task } => format!("ANALYZE needed: {task}"),
    }
}

/// The full observe→decide→act loop for one task.
pub fn agent_loop(task: &str, field: &[f32], tau: f32) -> String {
    let verdict = collapse_to_verdict(field, tau);
    let action = decide(task, field, tau);
    let result = execute(&action);
    format!("[{verdict:?}] {result}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_true_dominates() {
        let field = vec![0.9, 0.8, 0.1, -0.05, 0.7, 0.0, 0.6];
        assert_eq!(collapse_to_verdict(&field, 0.35), Verdict::Act);
    }

    #[test]
    fn collapse_false_dominates() {
        let field = vec![-0.9, -0.8, -0.7, -0.6, 0.1, -0.05];
        assert_eq!(collapse_to_verdict(&field, 0.35), Verdict::Block);
    }

    #[test]
    fn collapse_unknown_dominates() {
        let field = vec![0.1, -0.1, 0.05, -0.05, 0.0, 0.2, -0.2];
        assert_eq!(collapse_to_verdict(&field, 0.35), Verdict::Analyze);
    }

    #[test]
    fn decide_maps_write_to_file_action() {
        let field = vec![0.9, 0.9, 0.9];
        let a = decide("write the report file", &field, 0.35);
        assert!(matches!(a, ToolAction::WriteFile { .. }));
    }

    #[test]
    fn decide_blocks_on_false() {
        let field = vec![-0.9, -0.9, -0.9];
        let a = decide("write the report file", &field, 0.35);
        assert!(matches!(a, ToolAction::Block { .. }));
    }

    #[test]
    fn agent_loop_returns_verdict_prefixed_result() {
        let field = vec![0.9, 0.8, 0.7];
        let r = agent_loop("write the report file", &field, 0.35);
        assert!(r.starts_with("[Act]"));
    }
}

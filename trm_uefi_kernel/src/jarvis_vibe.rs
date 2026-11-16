//! TRM Swarm OS — Jarvis VibeThinker Consensus (no_std port).
//! 1:1 with Jarvis-Advanced-main/vibe_consensus.py:
//! Claim-Level Reliability (CLR): extract numerical claims from reasoning
//! steps, compute global support frequency, score each step by cross-
//! trajectory agreement, rank trajectories by reliability.
//! The Ollama query layer is excluded (no-Ollama rule); the consensus math
//! is the portable intelligence.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// A single reasoning step with its extracted numerical claims.
#[derive(Debug, Clone)]
pub struct StepClaims {
    pub step_text: String,
    pub claims: Vec<String>,
}

/// A reasoning trajectory (one sampled path).
#[derive(Debug, Clone)]
pub struct Trajectory {
    pub id: u64,
    pub steps: Vec<StepClaims>,
    pub reliability_score: f64,
}

impl Trajectory {
    pub fn new(id: u64) -> Self {
        Trajectory {
            id,
            steps: Vec::new(),
            reliability_score: 0.0,
        }
    }
}

/// Extract numerical claims from a step: fractions (a/b), equations (a op b = c),
/// standalone numbers. Matches extract_numerical_claims().
pub fn extract_numerical_claims(step: &str) -> Vec<String> {
    let mut claims: Vec<String> = Vec::new();
    // fractions a/b
    for frac in step.split_whitespace() {
        let t = frac.trim_matches(|c: char| !c.is_ascii_digit() && c != '/');
        if let Some(slash) = t.find('/') {
            let a = &t[..slash];
            let b = &t[slash + 1..];
            if !a.is_empty() && !b.is_empty() && a.chars().all(|c| c.is_ascii_digit())
                && b.chars().all(|c| c.is_ascii_digit())
            {
                let s = format!("{}/{}", a, b);
                if !claims.contains(&s) {
                    claims.push(s);
                }
            }
        }
    }
    // equations a op b = c
    let bytes = step.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b' ') {
                i += 1;
            }
            // look ahead for op = result
            let tail: String = step[i..].chars().take(12).collect();
            let ops = ['+', '-', '*', '/'];
            let mut eq_ok = false;
            for op in ops {
                if tail.contains(&format!(" {} = ", op)) || tail.contains(&format!("{} = ", op)) {
                    eq_ok = true;
                }
            }
            if eq_ok {
                let s = step[start..].split_whitespace().take(4).collect::<Vec<_>>().join(" ");
                if !claims.contains(&s) {
                    claims.push(s);
                }
            } else {
                let num = step[start..i].trim();
                if !num.is_empty() && !claims.contains(&num.to_string()) {
                    claims.push(num.to_string());
                }
            }
        } else {
            i += 1;
        }
    }
    claims
}

/// Compute Claim-Level Reliability over trajectories.
/// Returns trajectories ranked by reliability_score (descending).
pub fn compute_claim_level_reliability(mut trajectories: Vec<Trajectory>) -> Vec<Trajectory> {
    // Global claim support frequency (which claims appear across trajectories).
    let mut global_counts: Vec<(String, u64)> = Vec::new();
    for traj in &trajectories {
        for step in &traj.steps {
            for claim in &step.claims {
                if let Some(entry) = global_counts.iter_mut().find(|(c, _)| c == claim) {
                    entry.1 += 1;
                } else {
                    global_counts.push((claim.clone(), 1));
                }
            }
        }
    }

    // Score each trajectory: mean step score; step score = mean claim support.
    for traj in &mut trajectories {
        let mut total_score = 0.0f64;
        let mut n_steps = 0u64;
        for step in &traj.steps {
            n_steps += 1;
            if step.claims.is_empty() {
                total_score += 1.0; // neutral step
            } else {
                let mut claim_sum = 0.0f64;
                for claim in &step.claims {
                    if let Some((_, count)) = global_counts.iter().find(|(c, _)| c == claim) {
                        claim_sum += *count as f64;
                    }
                }
                total_score += claim_sum / step.claims.len() as f64;
            }
        }
        traj.reliability_score = total_score / (n_steps.max(1)) as f64;
    }

    // Rank by reliability (descending).
    trajectories.sort_by(|a, b| b.reliability_score.partial_cmp(&a.reliability_score).unwrap_or(core::cmp::Ordering::Equal));
    trajectories
}

/// Consensus verdict: highest-reliability trajectory id + its score.
pub fn vibe_consensus(ranked: &[Trajectory]) -> (u64, f64) {
    if ranked.is_empty() {
        return (0, 0.0);
    }
    (ranked[0].id, ranked[0].reliability_score)
}

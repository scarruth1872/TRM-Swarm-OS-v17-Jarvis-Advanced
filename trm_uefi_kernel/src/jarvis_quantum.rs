//! TRM Swarm OS — Jarvis Quantum Personality Model (no_std port).
//! 1:1 with Jarvis-Advanced-main/scripts/personality_quantum_model.py:
//! Personality as a probabilistic state-vector (superposition of traits),
//! drifted toward observations with learning rate alpha=0.1, tunneling
//! detection via deviation from the mean state.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// The five quantum personality traits (initial 50% superposition).
pub const TRAITS: &'static [&'static str] = &[
    "IE", // Introversion/Extroversion
    "SN", // Sensing/Intuition
    "TF", // Thinking/Feeling
    "JP", // Judging/Perceiving
    "Adaptability",
];

/// Quantum Personality Model: trait probabilities in [0,1].
pub struct QuantumPersonalityModel {
    pub state_vector: [f64; 5],
    pub history_len: u64,
}

impl QuantumPersonalityModel {
    pub fn new() -> Self {
        QuantumPersonalityModel {
            state_vector: [0.5, 0.5, 0.5, 0.5, 0.5],
            history_len: 0,
        }
    }

    /// Update toward observed trait values with alpha = 0.1 (quantum drift).
    /// observations: (trait_index, value) pairs. Matches update_state().
    pub fn update_state(&mut self, observations: &[(usize, f64)]) -> [f64; 5] {
        let alpha = 0.1f64;
        for &(trait_idx, val) in observations {
            if trait_idx < 5 {
                let v = val.clamp(0.0, 1.0);
                self.state_vector[trait_idx] = (1.0 - alpha) * self.state_vector[trait_idx] + alpha * v;
            }
        }
        self.history_len += 1;
        self.state_vector
    }

    /// Tunneling: true when any trait deviates > threshold from the mean state.
    /// Matches detect_tunneling() with a real variance check.
    pub fn detect_tunneling(&self, threshold: f64) -> bool {
        let mean = self.state_vector.iter().sum::<f64>() / 5.0;
        self.state_vector
            .iter()
            .any(|&v| (v - mean).abs() > threshold)
    }

    /// Profile string: "IE=62.5% SN=50.0% ..." (matches get_profile()).
    pub fn get_profile(&self) -> String {
        let mut out = String::new();
        for (i, name) in TRAITS.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(name);
            out.push('=');
            let pct = round_f64(self.state_vector[i] * 1000.0) / 10.0;
            out.push_str(&format!("{:.1}%", pct));
        }
        out
    }
}

/// Round-to-nearest for f64 on no_std UEFI (bit-level, no libm round).
fn round_f64(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5).floor_hack()
    } else {
        (x - 0.5).ceil_hack()
    }
}

trait FloorCeil {
    fn floor_hack(self) -> f64;
    fn ceil_hack(self) -> f64;
}
impl FloorCeil for f64 {
    fn floor_hack(self) -> f64 {
        let t = self as i64 as f64;
        if self < t { t - 1.0 } else { t }
    }
    fn ceil_hack(self) -> f64 {
        let t = self as i64 as f64;
        if self > t { t + 1.0 } else { t }
    }
}

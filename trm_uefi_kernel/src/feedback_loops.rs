//! TRM Swarm OS — Feedback Loops (no_std port).
//! 1:1 core semantics from swarm_v2/core/feedback_loops.py — Catalyst.
//! Deterministic feedback engine: reward/penalty updates with decay.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Feedback constants.
pub const FEEDBACK_DECAY: f64 = 0.9;
pub const FEEDBACK_MAX: f64 = 1.0;
pub const FEEDBACK_MIN: f64 = 0.0;

/// A feedback channel state.
#[derive(Debug, Clone)]
pub struct FeedChannel {
    pub name: String,
    pub value: f64,
    pub updates: u64,
}

/// Feedback engine.
#[derive(Debug, Clone)]
pub struct FeedbackEngine {
    pub channels: Vec<FeedChannel>,
}

impl FeedbackEngine {
    pub fn new() -> Self {
        FeedbackEngine {
            channels: Vec::new(),
        }
    }

    fn channel_index(&mut self, name: &str) -> usize {
        if let Some(i) = self.channels.iter().position(|c| c.name == name) {
            return i;
        }
        self.channels.push(FeedChannel {
            name: name.into(),
            value: 0.5,
            updates: 0,
        });
        self.channels.len() - 1
    }

    /// Update a channel: positive = reward, negative = penalty (clamped).
    pub fn update_feed(&mut self, name: &str, delta: f64) -> f64 {
        let i = self.channel_index(name);
        let c = &mut self.channels[i];
        c.value = (c.value * FEEDBACK_DECAY + delta).clamp(FEEDBACK_MIN, FEEDBACK_MAX);
        c.updates += 1;
        c.value
    }

    /// Status: (channels, total_updates, mean_value).
    pub fn get_status(&self) -> (u64, u64, f64) {
        let n = self.channels.len() as u64;
        let updates: u64 = self.channels.iter().map(|c| c.updates).sum();
        let mean = if n == 0 {
            0.0
        } else {
            self.channels.iter().map(|c| c.value).sum::<f64>() / n as f64
        };
        (n, updates, mean)
    }
}

/// Singleton handle.
pub fn get_feedback_engine() -> u64 {
    0x4645_4544_4241_434B
}

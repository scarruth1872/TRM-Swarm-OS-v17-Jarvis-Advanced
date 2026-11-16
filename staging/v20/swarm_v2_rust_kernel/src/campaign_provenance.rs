//! Provenance of a campaign task's reported value.
//!
//! Campaign reports previously exposed a single pass count that mixed tasks which
//! executed real work with tasks whose value was a literal chosen to clear its own
//! threshold. Counting them together makes the headline number meaningless: a task
//! whose input is a constant and whose threshold was picked to be smaller cannot fail.
//! Provenance is now explicit, and only measured work counts as a pass.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001

use std::string::String;
use std::vec::Vec;

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;

/// How a campaign task's reported value was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The value comes from work executed in this process; a latency comes from a real
    /// clock reading. Only these tasks may count toward the campaign's pass total.
    Measured,
    /// The value is a literal, or a computation whose outcome is fixed by construction
    /// (for example healing by XOR identity, or counting the campaign's own prior
    /// results). Kept in the report so the claim stays visible, but never counted as a
    /// passing measurement.
    Simulated,
}

impl Provenance {
    pub fn is_measured(self) -> bool {
        matches!(self, Provenance::Measured)
    }

    pub fn is_simulated(self) -> bool {
        matches!(self, Provenance::Simulated)
    }
}

/// An individual audited campaign task record with strict provenance.
#[derive(Debug, Clone)]
pub struct CampaignTaskRecord {
    pub task_name: String,
    pub provenance: Provenance,
    pub measured_value: f64,
    pub threshold: f64,
    pub passes: bool,
    pub unit: String,
    pub timestamp_ms: u64,
}

/// Campaign Provenance Audit Engine.
#[derive(Debug, Clone)]
pub struct CampaignProvenanceAudit {
    pub campaign_id: String,
    pub records: Vec<CampaignTaskRecord>,
    pub created_at_ms: u64,
}

impl CampaignProvenanceAudit {
    pub fn new(campaign_id: &str, timestamp_ms: u64) -> Self {
        Self {
            campaign_id: String::from(campaign_id),
            records: Vec::new(),
            created_at_ms: timestamp_ms,
        }
    }

    pub fn record_task(
        &mut self,
        task_name: &str,
        provenance: Provenance,
        measured_value: f64,
        threshold: f64,
        lower_is_better: bool,
        unit: &str,
        timestamp_ms: u64,
    ) {
        let passes = if lower_is_better {
            measured_value <= threshold
        } else {
            measured_value >= threshold
        };

        self.records.push(CampaignTaskRecord {
            task_name: String::from(task_name),
            provenance,
            measured_value,
            threshold,
            passes,
            unit: String::from(unit),
            timestamp_ms,
        });
    }

    pub fn count_measured_passes(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.provenance.is_measured() && r.passes)
            .count()
    }

    pub fn count_measured_total(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.provenance.is_measured())
            .count()
    }

    pub fn count_simulated_total(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.provenance.is_simulated())
            .count()
    }

    pub fn verified_pass_rate(&self) -> f64 {
        let total = self.count_measured_total();
        if total == 0 {
            0.0
        } else {
            (self.count_measured_passes() as f64) / (total as f64)
        }
    }
}

/// Query campaign provenance telemetry.
pub fn campaign_provenance_telemetry() -> (u64, &'static str) {
    (MONAD_ANCHOR, "STRICT_PROVENANCE_ENFORCED")
}

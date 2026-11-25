//! COMPATIBILITY FACADE — superseded 2026-11-24 by [`crate::quorum_consensus`].
//!
//! What was here: two counters labelled "PBFT". The commit counter accepted a proposal at
//! `COMMIT_THRESHOLD` **without ever consulting the prepare count**, a `false` vote was silently dropped
//! instead of being counted against the proposal, and two conflicting proposals could both commit. All
//! three behaviours were proved by tests (see `docs/CONSENSUS-TASKS-712-720.md`); those tests now fail
//! here, which is the intended tripwire.
//!
//! The type name is kept only so existing call sites still compile. Every method delegates to
//! [`crate::quorum_consensus::QuorumCouncil`], which enforces distinct voters, veto arithmetic, quorum
//! before commit, and one value per (view, slot). New consensus code should use `quorum_consensus`
//! directly — and read its header first: **this deployment has one failure domain, so f = 0, not 4.**

extern crate alloc;

pub use crate::quorum_consensus::{
    COMMIT_THRESHOLD, Decision as Proposal, DecisionError, N_AGENTS, PHASE_ACCEPTED, PHASE_COMMIT,
    PHASE_PRE_PREPARE, PHASE_PREPARE, PHASE_PROPOSE, PHASE_REJECTED, QUORUM_THRESHOLD, QuorumCouncil,
    VETO_BLOCK,
};

/// Declared Byzantine bound behind the council threshold (f = 4). Arithmetic only: with a single failure
/// domain the deployment tolerates zero Byzantine faults.
pub const MAX_BYZANTINE: u64 = 4;

#[derive(Debug, Clone)]
pub struct PbftConsensus {
    inner: QuorumCouncil,
}

impl Default for PbftConsensus {
    fn default() -> Self {
        Self::new()
    }
}

impl PbftConsensus {
    pub fn new() -> Self {
        PbftConsensus { inner: QuorumCouncil::new() }
    }

    /// Read-only view of the decisions, for callers that inspected `proposals` before.
    pub fn decisions(&self) -> &[Proposal] {
        &self.inner.decisions
    }

    /// Propose into the next slot of view 1.
    pub fn propose(&mut self, proposer: &str, proposal_type: &str) -> u64 {
        let slot = self.inner.status().0 + 1;
        let digest = 0xC0DE_0000 ^ slot;
        self.inner.propose(proposer, proposal_type, 1, slot, digest).unwrap_or(0)
    }

    /// A vote. `approve == false` is now **counted as a rejection** rather than dropped, and a repeat vote
    /// from the same voter is refused. Returns the current approval count, or 0 when the vote is refused.
    pub fn prepare(&mut self, proposal_id: u64, approve: bool) -> u64 {
        match self.inner.vote_next(proposal_id, if approve { 1 } else { -1 }) {
            Ok(t) => t.0,
            Err(_) => 0,
        }
    }

    /// Attempt to commit. Enforces veto and quorum; returns the approval count on success, 0 on refusal.
    /// A second call after acceptance is idempotent.
    pub fn commit(&mut self, proposal_id: u64) -> u64 {
        match self.inner.commit(proposal_id, COMMIT_THRESHOLD) {
            Ok(n) => n,
            Err(_) => 0,
        }
    }

    pub fn reject(&mut self, proposal_id: u64) {
        self.inner.reject(proposal_id);
    }

    pub fn get_phase(&self, proposal_id: u64) -> u64 {
        self.inner.phase(proposal_id)
    }

    /// (total, accepted, rejected, phase-of-last)
    pub fn get_status(&self) -> (u64, u64, u64, u64) {
        let (t, a, r) = self.inner.status();
        let phase = self.inner.decisions.last().map(|d| d.phase).unwrap_or(PHASE_PROPOSE);
        (t, a, r, phase)
    }

    /// Console/self-check path. Acceptance is decided by a real commit, so `accepted` can be false.
    pub fn run_full_cycle(&mut self, proposer: &str, proposal_type: &str) -> (u64, bool, f64) {
        self.inner.run_full_cycle(proposer, proposal_type)
    }
}

/// Singleton handle (kept for callers that only take an address).
pub fn get_pbft_consensus() -> u64 {
    0x5042_4654_0000_0001
}

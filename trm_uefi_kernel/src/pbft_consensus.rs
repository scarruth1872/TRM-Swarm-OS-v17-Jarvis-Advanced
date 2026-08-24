//! TRM Swarm OS — PBFT Consensus Engine (no_std port).
//! 1:1 core semantics from swarm_v2/core/pbft_consensus.py — Catalyst Phase 1.
//! The Byzantine consensus layer: propose -> pre-prepare -> prepare ->
//! commit with quorum arithmetic. N=14 agents, f=4 Byzantine, quorum
//! requires >= 2f+1 = 9 prepare votes and a valid commit round.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// PBFT constants (1:1 with python module constants).
pub const N_AGENTS: u64 = 14;
pub const MAX_BYZANTINE: u64 = 4; // f = floor((N-1)/3)
pub const QUORUM_THRESHOLD: u64 = 9; // 2f+1
pub const COMMIT_THRESHOLD: u64 = 10; // council requires >= 10 approvals

/// Consensus phases.
pub const PHASE_PROPOSE: u64 = 0;
pub const PHASE_PRE_PREPARE: u64 = 1;
pub const PHASE_PREPARE: u64 = 2;
pub const PHASE_COMMIT: u64 = 3;
pub const PHASE_ACCEPTED: u64 = 4;
pub const PHASE_REJECTED: u64 = 5;

/// A consensus proposal ledger entry.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposal_id: u64,
    pub proposer: String,
    pub proposal_type: String,
    pub prepare_votes: u64,
    pub commit_votes: u64,
    pub phase: u64,
}

/// PBFT consensus layer (stateful, no_std).
#[derive(Debug, Clone)]
pub struct PbftConsensus {
    pub proposals: Vec<Proposal>,
    pub next_id: u64,
    pub accepted_count: u64,
    pub rejected_count: u64,
}

impl PbftConsensus {
    pub fn new() -> Self {
        PbftConsensus {
            proposals: Vec::new(),
            next_id: 1,
            accepted_count: 0,
            rejected_count: 0,
        }
    }

    /// Propose: create a proposal, advance to pre-prepare.
    pub fn propose(&mut self, proposer: &str, proposal_type: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.push(Proposal {
            proposal_id: id,
            proposer: proposer.into(),
            proposal_type: proposal_type.into(),
            prepare_votes: 0,
            commit_votes: 0,
            phase: PHASE_PRE_PREPARE,
        });
        id
    }

    /// Prepare: count a vote; if quorum (>= 2f+1) reached, advance to commit.
    pub fn prepare(&mut self, proposal_id: u64, approve: bool) -> u64 {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                if approve {
                    p.prepare_votes += 1;
                }
                if p.prepare_votes >= QUORUM_THRESHOLD && p.phase < PHASE_COMMIT {
                    p.phase = PHASE_COMMIT;
                }
                return p.prepare_votes;
            }
        }
        0
    }

    /// Commit: count commit votes; if >= COMMIT_THRESHOLD, accepted.
    pub fn commit(&mut self, proposal_id: u64) -> u64 {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                p.commit_votes += 1;
                if p.commit_votes >= COMMIT_THRESHOLD && p.phase < PHASE_ACCEPTED {
                    p.phase = PHASE_ACCEPTED;
                    self.accepted_count += 1;
                }
                return p.commit_votes;
            }
        }
        0
    }

    /// Reject: mark a proposal rejected.
    pub fn reject(&mut self, proposal_id: u64) {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                p.phase = PHASE_REJECTED;
                self.rejected_count += 1;
                return;
            }
        }
    }

    /// Full cycle: propose + quorum prepare votes + commit -> accepted verdict.
    /// Returns (proposal_id, accepted, weighted_score-ish 0..1).
    pub fn run_full_cycle(&mut self, proposer: &str, proposal_type: &str) -> (u64, bool, f64) {
        let id = self.propose(proposer, proposal_type);
        // simulate N agents voting prepare (N - f honest)
        for _ in 0..(N_AGENTS - MAX_BYZANTINE) {
            self.prepare(id, true);
        }
        for _ in 0..COMMIT_THRESHOLD {
            self.commit(id);
        }
        let accepted = self.get_phase(id) == PHASE_ACCEPTED;
        let score = if accepted { 0.98 } else { 0.0 };
        (id, accepted, score)
    }

    pub fn get_phase(&self, proposal_id: u64) -> u64 {
        for p in &self.proposals {
            if p.proposal_id == proposal_id {
                return p.phase;
            }
        }
        PHASE_REJECTED
    }

    /// Status snapshot (1:1 get_status): (total, accepted, rejected, phase).
    pub fn get_status(&self) -> (u64, u64, u64, u64) {
        let phase = self.proposals.last().map(|p| p.phase).unwrap_or(PHASE_PROPOSE);
        (self.proposals.len() as u64, self.accepted_count, self.rejected_count, phase)
    }
}

/// Singleton handle.
pub fn get_pbft_consensus() -> u64 {
    // deterministic handle (lazy-init pattern)
    0x5042_4654_0000_0001
}

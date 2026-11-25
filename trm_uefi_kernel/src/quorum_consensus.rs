//! Quorum-verified decision primitive — the replacement for a vote counter that was labelled PBFT.
//!
//! WHAT THIS IS
//! ------------
//! A single-process, single-host agreement primitive with **enforced** invariants:
//!
//!   1. **Distinctness** — a voter may vote once per proposal. A repeat is an error, not a silent
//!      increment, so a tally can never exceed the number of agents that exist.
//!   2. **Veto arithmetic** — if enough agents reject that the approval threshold becomes arithmetically
//!      unreachable (rejections > N - quorum), the proposal is vetoed whatever the approval count says.
//!   3. **One value per (view, slot)** — once a slot commits a value digest, a different digest for the same
//!      slot is refused. Two conflicting decisions cannot both commit in one view.
//!   4. **Commit requires quorum** — there is no path to ACCEPTED that skips the approval count.
//!
//! WHAT THIS IS NOT
//! ----------------
//! **It is not Byzantine fault tolerant, and it does not claim to be.** All agents share one address space
//! on one host, so a crash or a corrupt process takes all of them: the number of independent failure
//! domains is 1, and agreement needs n >= 3f+1 to tolerate f Byzantine faults. With one failure domain the
//! honest value is f = 0. The constants below (N=14, quorum 9) are a council threshold, not a fault-tolerance
//! bound. Byzantine agreement additionally needs authenticated messages and independent replicas; this
//! deployment has neither (the kernel's only dependency is `r-efi`, so no signature scheme is available).
//!
//! The predecessor of this module incremented two counters and accepted at the commit count without ever
//! consulting the prepare count — measured, with tests, in `docs/CONSENSUS-TASKS-712-720.md`. Those three
//! characterisation tests now fail here, which is the intended tripwire.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Council size. A threshold number, not a fault-tolerance bound — see the module docs.
pub const N_AGENTS: u64 = 14;
/// Declared supermajority for ordinary proposals (2f+1 with the declared f=4).
pub const QUORUM_THRESHOLD: u64 = 9;
/// Commit threshold used by the console/self-check path.
pub const COMMIT_THRESHOLD: u64 = 10;
/// Rejections at or above this make the quorum arithmetically impossible: N - VETO_BLOCK + 1 < QUORUM.
pub const VETO_BLOCK: u64 = N_AGENTS - QUORUM_THRESHOLD + 1;

pub const PHASE_PROPOSE: u64 = 0;
pub const PHASE_PRE_PREPARE: u64 = 1;
pub const PHASE_PREPARE: u64 = 2;
pub const PHASE_COMMIT: u64 = 3;
pub const PHASE_ACCEPTED: u64 = 4;
pub const PHASE_REJECTED: u64 = 5;

/// Every way the primitive can refuse. A refusal is always explicit: nothing falls back to "accepted".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionError {
    UnknownProposal,
    /// The same voter tried to vote twice on one proposal.
    DuplicateVoter,
    /// Rejections have made the approval threshold unreachable.
    Vetoed,
    /// Fewer than the required distinct approvals.
    QuorumNotReached,
    /// This (view, slot) already committed a different value.
    ConflictingValue,
    /// The proposal is already terminal (accepted or rejected).
    AlreadyTerminal,
    /// More voters than agents exist: a tally may never exceed the council size.
    TooManyVoters,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub proposal_id: u64,
    pub proposer: String,
    pub proposal_type: String,
    pub view: u64,
    pub slot: u64,
    pub digest: u64,
    /// voter id -> trit (+1 approve, 0 abstain, -1 reject), one entry per voter.
    pub votes: Vec<(u64, i8)>,
    pub phase: u64,
    pub next_voter: u64,
}

impl Decision {
    fn count(&self, want: i8) -> u64 {
        self.votes.iter().filter(|(_, t)| *t == want).count() as u64
    }
    pub fn approvals(&self) -> u64 { self.count(1) }
    pub fn abstains(&self) -> u64 { self.count(0) }
    pub fn rejections(&self) -> u64 { self.count(-1) }
    pub fn distinct_voters(&self) -> u64 { self.votes.len() as u64 }
}

/// (approvals, abstains, rejections, distinct)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tally(pub u64, pub u64, pub u64, pub u64);

#[derive(Debug, Clone)]
pub struct QuorumCouncil {
    pub decisions: Vec<Decision>,
    next_id: u64,
    /// (view, slot) -> committed digest
    committed: Vec<(u64, u64, u64)>,
    pub accepted_count: u64,
    pub rejected_count: u64,
}

impl Default for QuorumCouncil { fn default() -> Self { Self::new() } }

impl QuorumCouncil {
    pub fn new() -> Self {
        QuorumCouncil { decisions: Vec::new(), next_id: 1, committed: Vec::new(),
                        accepted_count: 0, rejected_count: 0 }
    }

    /// Propose a value for (view, slot). Refuses a value that contradicts one already committed there.
    pub fn propose(&mut self, proposer: &str, proposal_type: &str, view: u64, slot: u64,
                   digest: u64) -> Result<u64, DecisionError> {
        for (cview, cslot, cdigest) in &self.committed {
            if *cview == view && *cslot == slot && *cdigest != digest {
                return Err(DecisionError::ConflictingValue);
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.decisions.push(Decision {
            proposal_id: id, proposer: proposer.into(), proposal_type: proposal_type.into(),
            view, slot, digest, votes: Vec::new(), phase: PHASE_PRE_PREPARE, next_voter: 1,
        });
        Ok(id)
    }

    fn find(&mut self, id: u64) -> Result<&mut Decision, DecisionError> {
        self.decisions.iter_mut().find(|d| d.proposal_id == id).ok_or(DecisionError::UnknownProposal)
    }

    /// Cast one voter's trit. A voter that already voted on this proposal is refused.
    pub fn vote(&mut self, id: u64, voter: u64, trit: i8) -> Result<Tally, DecisionError> {
        let d = self.find(id)?;
        if d.phase == PHASE_ACCEPTED || d.phase == PHASE_REJECTED {
            return Err(DecisionError::AlreadyTerminal);
        }
        if d.votes.iter().any(|(v, _)| *v == voter) {
            return Err(DecisionError::DuplicateVoter);
        }
        // A tally bigger than the council is arithmetic fiction: refuse the extra voter outright.
        if d.votes.len() as u64 >= N_AGENTS {
            return Err(DecisionError::TooManyVoters);
        }
        d.votes.push((voter, if trit > 0 { 1 } else if trit < 0 { -1 } else { 0 }));
        if d.approvals() >= QUORUM_THRESHOLD && d.phase < PHASE_COMMIT {
            d.phase = PHASE_COMMIT;
        }
        Ok(Tally(d.approvals(), d.abstains(), d.rejections(), d.distinct_voters()))
    }

    /// Convenience for callers that have no external voter list: votes arrive in order, each from a new
    /// distinct voter id. Distinctness is still enforced — the id is incremented per vote, never reused.
    pub fn vote_next(&mut self, id: u64, trit: i8) -> Result<Tally, DecisionError> {
        let voter = self.find(id)?.next_voter;
        self.find(id)?.next_voter += 1;
        self.vote(id, voter, trit)
    }

    pub fn tally(&self, id: u64) -> Result<Tally, DecisionError> {
        let d = self.decisions.iter().find(|d| d.proposal_id == id).ok_or(DecisionError::UnknownProposal)?;
        Ok(Tally(d.approvals(), d.abstains(), d.rejections(), d.distinct_voters()))
    }

    pub fn phase(&self, id: u64) -> u64 {
        self.decisions.iter().find(|d| d.proposal_id == id).map(|d| d.phase).unwrap_or(PHASE_REJECTED)
    }

    /// Commit: the only path to ACCEPTED, and it enforces veto + quorum.
    pub fn commit(&mut self, id: u64, required: u64) -> Result<u64, DecisionError> {
        let (approvals, rejections, view, slot, digest) = {
            let d = self.find(id)?;
            if d.phase == PHASE_ACCEPTED { return Ok(d.approvals()); }
            if d.phase == PHASE_REJECTED { return Err(DecisionError::AlreadyTerminal); }
            (d.approvals(), d.rejections(), d.view, d.slot, d.digest)
        };
        // Veto first: a proposal the council has arithmetically excluded is not "not yet decided".
        if rejections >= VETO_BLOCK || approvals + rejections > N_AGENTS {
            self.find(id)?.phase = PHASE_REJECTED;
            self.rejected_count += 1;
            return Err(DecisionError::Vetoed);
        }
        if approvals < required {
            return Err(DecisionError::QuorumNotReached);
        }
        for (cview, cslot, cdigest) in &self.committed {
            if *cview == view && *cslot == slot && *cdigest != digest {
                return Err(DecisionError::ConflictingValue);
            }
        }
        let d = self.find(id)?;
        d.phase = PHASE_ACCEPTED;
        let approvals = d.approvals();
        self.accepted_count += 1;
        if !self.committed.iter().any(|(v, s, _)| *v == view && *s == slot) {
            self.committed.push((view, slot, digest));
        }
        Ok(approvals)
    }

    pub fn reject(&mut self, id: u64) {
        if let Some(d) = self.decisions.iter_mut().find(|d| d.proposal_id == id) {
            if d.phase != PHASE_ACCEPTED {
                d.phase = PHASE_REJECTED;
                self.rejected_count += 1;
            }
        }
    }

    /// (total, accepted, rejected)
    pub fn status(&self) -> (u64, u64, u64) {
        (self.decisions.len() as u64, self.accepted_count, self.rejected_count)
    }

    /// Console/self-check path: one slot, N-f distinct approvals, then a real commit.
    /// Returns (id, accepted, approvals-as-score-fraction). Acceptance is decided by `commit`, so this
    /// function can return `false` — the counter it replaces could not.
    pub fn run_full_cycle(&mut self, proposer: &str, proposal_type: &str) -> (u64, bool, f64) {
        let slot = self.next_id;
        let id = match self.propose(proposer, proposal_type, 1, slot, 0xC0DE_0000 ^ slot) {
            Ok(id) => id, Err(_) => return (0, false, 0.0),
        };
        for _ in 0..(N_AGENTS - 4) {
            if self.vote_next(id, 1).is_err() { break; }
        }
        match self.commit(id, QUORUM_THRESHOLD) {
            Ok(approvals) => (id, true, approvals as f64 / N_AGENTS as f64),
            Err(_) => (id, false, 0.0),
        }
    }
}

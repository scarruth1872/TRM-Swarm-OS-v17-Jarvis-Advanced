//! TRM Swarm OS — PBFT Consensus Engine (no_std port).
//! 1:1 core semantics from swarm_v2/core/pbft_consensus.py — Catalyst Phase 1.
//! The Byzantine consensus layer: propose -> pre-prepare -> prepare ->
//! commit with quorum arithmetic. N=14 agents, f=4 Byzantine:
//! prepare quorum is >= 2f = 8 approvals, commit quorum is >= 2f+1 = 9
//! signals (oracle: PBFTConsensusLayer.prepare / .commit).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// PBFT constants (1:1 with python module constants).
pub const N_AGENTS: u64 = 14;
pub const MAX_BYZANTINE: u64 = 4; // f = floor((N-1)/3)
pub const QUORUM_THRESHOLD: u64 = 8; // prepare quorum: >= 2f approvals
pub const COMMIT_THRESHOLD: u64 = 9; // commit quorum: >= 2f+1 signals

/// Consensus phases.
pub const PHASE_PROPOSE: u64 = 0;
pub const PHASE_PRE_PREPARE: u64 = 1;
pub const PHASE_PREPARE: u64 = 2;
pub const PHASE_COMMIT: u64 = 3;
pub const PHASE_ACCEPTED: u64 = 4;
pub const PHASE_REJECTED: u64 = 5;

/// PBFT node roster (1:1 with the oracle's PBFT_NODES keys, agent_id 0..13).
pub const PBFT_NODES: [&str; 14] = [
    "Hermes",
    "Jarvis",
    "Bridge",
    "Shield",
    "Logic",
    "Devo",
    "Pulse",
    "Flow",
    "EventLedger",
    "Orchestra",
    "THzGateway",
    "Verify",
    "Seeker",
    "Archi",
];

/// Message-validity rule: only a recognised PBFT node may propose.
pub fn is_pbft_node(node: &str) -> bool {
    PBFT_NODES.iter().any(|n| *n == node)
}

/// A consensus proposal ledger entry.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposal_id: u64,
    pub proposer: String,
    pub proposal_type: String,
    /// Oracle `ConsensusProposal.votes` -- a DICT keyed by voter, so a repeat
    /// vote from one identity overwrites rather than accumulating.
    pub votes: Vec<(String, bool)>,
    /// Oracle `ConsensusProposal.commit_signals` -- likewise keyed by committer.
    pub commit_signals: Vec<String>,
    pub prepare_votes: u64,
    pub commit_votes: u64,
    pub phase: u64,
    /// Oracle `ConsensusProposal.result`: 'committed' | 'rejected' | None.
    pub result: Option<String>,
}

/// PBFT consensus layer (stateful, no_std).
#[derive(Debug, Clone)]
pub struct PbftConsensus {
    pub proposals: Vec<Proposal>,
    pub next_id: u64,
    /// Monotonic count of finalizations (may exceed the oracle's per-proposal
    /// `committed_count`; `get_status` derives the oracle's counts by scanning).
    pub accepted_count: u64,
    /// Monotonic count of explicit rejections (see `get_status`).
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

    fn find(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.proposal_id == proposal_id)
    }

    /// Propose: create a proposal, advance to pre-prepare.
    pub fn propose(&mut self, proposer: &str, proposal_type: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.push(Proposal {
            proposal_id: id,
            proposer: proposer.into(),
            proposal_type: proposal_type.into(),
            votes: Vec::new(),
            commit_signals: Vec::new(),
            prepare_votes: 0,
            commit_votes: 0,
            phase: PHASE_PRE_PREPARE,
            result: None,
        });
        id
    }

    /// Pre-prepare (oracle `pre_prepare`): unknown id -> false; proposer that
    /// is not a recognised PBFT node -> phase rejected, result 'rejected',
    /// false; otherwise phase prepare and true. The oracle's sha256 payload
    /// digest is host-side crypto and is not ported here.
    pub fn pre_prepare(&mut self, proposal_id: u64) -> bool {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                if !is_pbft_node(&p.proposer) {
                    p.phase = PHASE_REJECTED;
                    p.result = Some(String::from("rejected"));
                    return false;
                }
                p.phase = PHASE_PREPARE;
                return true;
            }
        }
        false
    }

    /// Prepare: count a vote; if quorum (>= 2f) reached, advance to commit.
    /// The oracle assigns the phase unconditionally on reaching quorum, so a
    /// previously rejected proposal is moved to commit here as well.
    pub fn prepare(&mut self, proposal_id: u64, voter: &str, approve: bool) -> Option<u64> {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                // Oracle: `proposal.votes[voter] = approve`, then count the True values.
                match p.votes.iter_mut().find(|(v, _)| v == voter) {
                    Some(slot) => slot.1 = approve,
                    None => p.votes.push((String::from(voter), approve)),
                }
                let approvals = p.votes.iter().filter(|(_, v)| *v).count() as u64;
                if approvals >= QUORUM_THRESHOLD {
                    // Unconditional: the oracle's REJECTED -> COMMIT overwrite.
                    p.phase = PHASE_COMMIT;
                }
                return Some(p.phase);
            }
        }
        None
    }

    /// Commit: count commit votes; if >= 2f+1 signals, accepted.
    /// Like the oracle, the phase/result are rewritten unconditionally once the
    /// quorum holds (`committed_count` in `get_status` stays per-proposal).
    pub fn commit(&mut self, proposal_id: u64, committer: &str) -> Option<u64> {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                // Oracle: `commit_signals[committer] = True` -- a repeat is a no-op.
                if !p.commit_signals.iter().any(|c| c == committer) {
                    p.commit_signals.push(String::from(committer));
                }
                let signals = p.commit_signals.len() as u64;
                if signals >= COMMIT_THRESHOLD {
                    let first = !matches!(p.result.as_deref(), Some("committed"));
                    p.phase = PHASE_ACCEPTED;
                    p.result = Some(String::from("committed"));
                    if first {
                        self.accepted_count += 1;
                    }
                }
                return Some(p.phase);
            }
        }
        None
    }

    /// Reject: mark a proposal rejected. Returns true if it was found
    /// (oracle `reject` -> bool), false for an unknown proposal id.
    pub fn reject(&mut self, proposal_id: u64) -> bool {
        for p in self.proposals.iter_mut() {
            if p.proposal_id == proposal_id {
                p.phase = PHASE_REJECTED;
                p.result = Some(String::from("rejected"));
                self.rejected_count += 1;
                return true;
            }
        }
        false
    }

    /// Full cycle: propose + pre-prepare + every node except the proposer votes
    /// approve + every node signals commit. Returns the oracle's dict fields
    /// (proposal_id, result, phase).
    /// The oracle ignores pre_prepare's verdict and keeps voting on a rejected
    /// proposal, so a non-member proposer still reaches 'finalized'/'committed'.
    pub fn run_full_cycle(
        &mut self,
        proposer: &str,
        proposal_type: &str,
    ) -> (u64, Option<String>, u64) {
        let id = self.propose(proposer, proposal_type);
        self.pre_prepare(id);
        // prepare: every node except the proposer approves, BY IDENTITY (the
        // oracle's votes dict dedups, so passing a name per node matters)
        for (i, node) in PBFT_NODES.iter().enumerate() {
            if *node != proposer && i < N_AGENTS as usize {
                self.prepare(id, node, true);
            }
        }
        // commit: every node signals, by identity
        for (i, node) in PBFT_NODES.iter().enumerate() {
            if i < N_AGENTS as usize {
                self.commit(id, node);
            }
        }
        let phase = self.get_phase(id);
        let result = self.get_result(id);
        (id, result, phase)
    }

    pub fn get_phase(&self, proposal_id: u64) -> u64 {
        self.find(proposal_id).map(|p| p.phase).unwrap_or(PHASE_REJECTED)
    }

    /// Result accessor (oracle `ConsensusProposal.result`).
    pub fn get_result(&self, proposal_id: u64) -> Option<String> {
        self.find(proposal_id).and_then(|p| p.result.clone())
    }

    /// Status snapshot (1:1 get_status): (total_proposals, committed_count,
    /// rejected_count, node_count, f_tolerance, phase).
    /// committed/rejected counts are derived by scanning for
    /// result == 'committed' / 'rejected' exactly as the oracle does.
    /// The oracle's `recent_proposals` (top 10 by wall-clock created_at) is
    /// host-side and not ported.
    pub fn get_status(&self) -> (u64, u64, u64, u64, u64, u64) {
        let committed = self
            .proposals
            .iter()
            .filter(|p| matches!(p.result.as_deref(), Some("committed")))
            .count() as u64;
        let rejected = self
            .proposals
            .iter()
            .filter(|p| matches!(p.result.as_deref(), Some("rejected")))
            .count() as u64;
        let phase = self.proposals.last().map(|p| p.phase).unwrap_or(PHASE_PROPOSE);
        (
            self.proposals.len() as u64,
            committed,
            rejected,
            N_AGENTS,
            MAX_BYZANTINE,
            phase,
        )
    }
}

/// Batch PBFT quorum evaluator (oracle `native_evaluate_pbft_quorum`, the
/// transparent Python fallback of the native voltage strip
/// `strip_pbft_batch_quorum`). Per round, over `n_nodes` columns:
///   sum(votes)   <  req_prep   -> 4 (REJECTED)
///   sum(commits) >= req_commit -> 3 (FINALIZED), finalized += 1
///   otherwise                  -> 2 (COMMIT)
/// `votes`/`commits` are n_rounds*n_nodes row-major; `out_states` receives
/// n_rounds state bytes. Returns the finalized count (K3-true from the strip).
pub fn native_evaluate_pbft_quorum(
    votes: &[u8],
    commits: &[u8],
    n_rounds: usize,
    n_nodes: usize,
    req_prep: u64,
    req_commit: u64,
    out_states: &mut [u8],
) -> u64 {
    let mut finalized: u64 = 0;
    for r in 0..n_rounds {
        let off = r * n_nodes;
        let mut v_sum: u64 = 0;
        let mut c_sum: u64 = 0;
        for i in 0..n_nodes {
            v_sum += votes[off + i] as u64;
            c_sum += commits[off + i] as u64;
        }
        if v_sum < req_prep {
            out_states[r] = 4; // REJECTED
        } else if c_sum >= req_commit {
            out_states[r] = 3; // FINALIZED
            finalized += 1;
        } else {
            out_states[r] = 2; // COMMIT
        }
    }
    finalized
}

/// Singleton handle.
pub fn get_pbft_consensus() -> u64 {
    // deterministic handle (lazy-init pattern)
    0x5042_4654_0000_0001
}

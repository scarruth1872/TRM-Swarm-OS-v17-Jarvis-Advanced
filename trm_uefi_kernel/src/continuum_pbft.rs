//! TRM Swarm OS — Project Continuum Autonomic Ledger (no_std port).
//! 1:1 core semantics from swarm_v2/core/continuum_pbft.py.
//!
//! WHAT THIS IS (corrected 2026-11-24): a **majority-hash ledger** over four named roles — ORCH, SAGE,
//! MUSE, SENTINEL — that live in the same process as `&str` constants. Each "node" hashes the state ledger,
//! the majority hash wins, and a divergent node's AST is repaired. That is a useful self-healing check over
//! a shared ledger.
//!
//! WHAT THIS IS NOT: Byzantine fault tolerant. Four roles in one address space are one failure domain, not
//! four replicas; there are no messages, no views, no signatures, and no independent clocks.
//! `f = 1` below is arithmetic over a role count, so the true tolerated Byzantine faults are **f = 0**.
//! The enforced decision primitive is [`crate::quorum_consensus`] (distinct voters, veto arithmetic, one
//! value per slot) — and even that is single-process, so it makes no Byzantine claim either.
//!
//! DIVERGENCE (documented): python tx_id = f"tx_{int(time.time()*1000)}" and
//! record timestamps use wall-clock time — replaced by a deterministic
//! monotonic tx counter (tx_1, tx_2, ...) since no_std has no clock.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// PBFT node registry (1:1 with python PBFT_NODES; insertion order matters —
/// python dict order drives majority-hash arbitration and remediation order).
pub const PBFT_NODES: [(&str, &str, &str); 4] = [
    ("ORCH", "Continuum Core", "Primary Leader Node"),
    ("SAGE", "Cybernetic Sage", "State Auditor Node"),
    ("MUSE", "Stochastic Muse", "Creative Generator Node"),
    ("SENTINEL", "Sentinel Warden", "Security Isolation Node"),
];

/// Byzantine fault tolerance: f = 1 → quorum 2f+1 = 3 prepare votes (1:1 python).
pub const F_MAX: u64 = 1;
pub const REQUIRED_VOTES: u64 = 2 * F_MAX + 1;

/// ---------------------------------------------------------------------------
/// Minimal no_std SHA-256 (FIPS 180-4). Deterministic core of the python
/// hashlib.sha256(...).hexdigest() calls.
/// ---------------------------------------------------------------------------
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_blocks(msg: &[u8]) -> [u32; 8] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    let mut padded: Vec<u8> = Vec::with_capacity(((msg.len() + 72) / 64) * 64);
    padded.extend_from_slice(msg);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    for i in (0..8).rev() {
        padded.push((bit_len >> (i * 8)) as u8);
    }

    let mut w = [0u32; 64];
    for chunk in padded.chunks_exact(64) {
        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                chunk[t * 4],
                chunk[t * 4 + 1],
                chunk[t * 4 + 2],
                chunk[t * 4 + 3],
            ]);
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for t in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[t])
                .wrapping_add(w[t]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h
}

/// sha256 hex digest (lowercase hexdigest, 1:1 with python hashlib).
pub fn sha256_hex(data: &[u8]) -> String {
    let h = sha256_blocks(data);
    let mut out = String::with_capacity(64);
    for word in h.iter() {
        out.push_str(&format!("{:08x}", word));
    }
    out
}

/// ---------------------------------------------------------------------------
/// PBFTNodeState (1:1 python class).
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct PBFTNodeState {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub state_ledger: Vec<String>,
}

impl PBFTNodeState {
    pub fn new(node_id: &str, name: &str, role: &str) -> Self {
        PBFTNodeState {
            node_id: node_id.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            status: "healthy".to_string(),
            // python: [f"GENESIS_BLOCK_HASH_{node_id}_v5.3"]
            state_ledger: vec![format!("GENESIS_BLOCK_HASH_{}_v5.3", node_id)],
        }
    }

    /// python: hashlib.sha256("|".join(state_ledger).encode()).hexdigest()
    pub fn get_ledger_hash(&self) -> String {
        let joined = self.state_ledger.join("|");
        sha256_hex(joined.as_bytes())
    }
}

/// A committed transaction record (python dict in transaction_history).
#[derive(Debug, Clone)]
pub struct CycleRecord {
    pub tx_id: String,
    pub block: String,
    pub leader: String,
    pub prepare_votes: u64,
    pub required_votes: u64,
    pub status: String,
}

/// Result of one execute_pbft_cycle run (python return dict).
#[derive(Debug, Clone)]
pub struct PbftCycleResult {
    pub status: String,
    /// reason present only on FAILED (python includes it then); empty otherwise.
    pub reason: String,
    pub tx_id: String,
    pub block: String,
    pub leader: String,
    pub prepare_votes: u64,
    pub required_votes: u64,
}

/// Node state report entries of audit_and_repair_mesh (python dict values).
#[derive(Debug, Clone)]
pub struct NodeStateReport {
    pub node_id: String,
    pub name: String,
    pub status: String,
    pub ledger_length: u64,
}

/// Audit result (python return dict of audit_and_repair_mesh).
#[derive(Debug, Clone)]
pub struct AuditReport {
    pub mesh_status: String,
    pub total_nodes: u64,
    /// majority hash truncated like python: hash[:16] + "..."
    pub majority_ledger_hash: String,
    pub isolated_count: u64,
    pub remediated_nodes: Vec<String>,
    pub node_states: Vec<NodeStateReport>,
}

/// ---------------------------------------------------------------------------
/// ContinuumPBFTLedger (1:1 python class).
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ContinuumPBFTLedger {
    pub nodes: Vec<PBFTNodeState>,
    pub transaction_history: Vec<CycleRecord>,
    /// deterministic replacement for int(time.time()*1000) — see file header.
    tx_counter: u64,
}

impl ContinuumPBFTLedger {
    pub fn new() -> Self {
        let nodes = PBFT_NODES
            .iter()
            .map(|(nid, name, role)| PBFTNodeState::new(nid, name, role))
            .collect();
        ContinuumPBFTLedger {
            nodes,
            transaction_history: Vec::new(),
            tx_counter: 0,
        }
    }

    /// Lookup node by id (python self.nodes[n_id]).
    pub fn node(&self, node_id: &str) -> Option<&PBFTNodeState> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    /// Mutable node lookup by id (python dict assignment on nodes[n_id]).
    pub fn node_mut(&mut self, node_id: &str) -> Option<&mut PBFTNodeState> {
        self.nodes.iter_mut().find(|n| n.node_id == node_id)
    }

    /// Runs the 3-phase PBFT cycle: Pre-Prepare (ORCH proposes), Prepare
    /// (peer nodes verify/sign, needs 2f+1 = 3 votes), Commit.
    /// DIVERGENCE: tx id is a deterministic counter (python wall-clock ms).
    pub fn execute_pbft_cycle(&mut self, proposal_data: &str) -> PbftCycleResult {
        let action = if proposal_data.is_empty() {
            "STATE_UPDATE".to_string()
        } else {
            proposal_data.to_string()
        };
        self.tx_counter += 1;
        let tx_id = format!("tx_{}", self.tx_counter);
        let tx_block = format!("BLOCK_{}_{}", tx_id, action);

        // 1. Pre-Prepare: ORCH proposes block
        let orch_status = self.nodes[0].status.clone();
        if orch_status != "healthy" {
            return PbftCycleResult {
                status: "FAILED".to_string(),
                reason: "Leader Node ORCH compromised or unavailable".to_string(),
                tx_id: String::new(),
                block: String::new(),
                leader: String::new(),
                prepare_votes: 0,
                required_votes: REQUIRED_VOTES,
            };
        }
        self.nodes[0].state_ledger.push(tx_block.clone());
        let pre_prepare_sig =
            sha256_hex(format!("PRE_PREPARE:{}:{}", tx_id, self.nodes[0].get_ledger_hash()).as_bytes());
        let _ = pre_prepare_sig; // python computes but never returns it

        // 2. Prepare: peer nodes verify and sign (ORCH included — 1:1 python).
        let mut prepare_votes: u64 = 0;
        for node in self.nodes.iter_mut() {
            if node.status == "healthy" {
                node.state_ledger.push(tx_block.clone());
                prepare_votes += 1;
            }
        }

        // System requires 2f + 1 prepare votes (f=1 => 3 votes required)
        if prepare_votes < REQUIRED_VOTES {
            return PbftCycleResult {
                status: "FAILED".to_string(),
                reason: format!(
                    "Insufficient prepare votes: {}/{}",
                    prepare_votes, REQUIRED_VOTES
                ),
                tx_id,
                block: tx_block,
                leader: String::new(),
                prepare_votes,
                required_votes: REQUIRED_VOTES,
            };
        }

        // 3. Commit: final state commitment.
        // (python record also carries "timestamp": datetime.now().isoformat()
        //  — omitted: no_std has no wall clock.)
        let tx_record = CycleRecord {
            tx_id: tx_id.clone(),
            block: tx_block,
            leader: "ORCH".to_string(),
            prepare_votes,
            required_votes: REQUIRED_VOTES,
            status: "COMMITTED".to_string(),
        };
        self.transaction_history.push(tx_record.clone());
        PbftCycleResult {
            status: tx_record.status,
            reason: String::new(),
            tx_id,
            block: tx_record.block,
            leader: tx_record.leader,
            prepare_votes,
            required_votes: REQUIRED_VOTES,
        }
    }

    /// Casts a 3f+1 Byzantine consensus vote across sub-agent personas.
    pub fn cast_consensus_vote(&mut self, block_id: &str, _proposal_hash: &str) -> PbftCycleResult {
        let action = format!("VOTE_BLOCK_{}", block_id);
        self.execute_pbft_cycle(&action)
    }

    /// Returns transaction ledger history; genesis record fallback when empty.
    pub fn get_ledger(&self) -> Vec<CycleRecord> {
        if !self.transaction_history.is_empty() {
            return self.transaction_history.clone();
        }
        vec![CycleRecord {
            tx_id: "tx_genesis_v53".to_string(),
            block: "BLOCK_GENESIS_v5.3".to_string(),
            leader: "ORCH".to_string(),
            prepare_votes: 4,
            required_votes: 3,
            status: "COMMITTED".to_string(),
        }]
    }

    /// Autonomic Self-Healing Protocol (python Section 5.3). Majority hash
    /// arbitration uses insertion order (first maximal count wins, 1:1
    /// python max(state_reports, key=...) on a dict).
    pub fn audit_and_repair_mesh(&mut self) -> AuditReport {
        // 1. Compute ledger hash for each node
        let mut node_hashes: Vec<(String, String)> = Vec::new();
        let mut state_reports: Vec<(String, u64)> = Vec::new();
        for node in self.nodes.iter() {
            let l_hash = node.get_ledger_hash();
            node_hashes.push((node.node_id.clone(), l_hash.clone()));
            match state_reports.iter_mut().find(|(h, _)| *h == l_hash) {
                Some((_, count)) => *count += 1,
                None => state_reports.push((l_hash, 1)),
            }
        }

        // 2. Identify majority verified hash (first maximal in order — 1:1 python)
        let mut majority_hash = String::new();
        let mut majority_count: u64 = 0;
        for (h, count) in state_reports.iter() {
            if *count > majority_count {
                majority_count = *count;
                majority_hash = h.clone();
            }
        }

        let mut remediated_nodes: Vec<String> = Vec::new();
        let mut isolated_nodes: Vec<String> = Vec::new();

        // Find a node holding the majority ledger (first in order — 1:1
        // python next(n for n in nodes.values() if hash == majority_hash)).
        let majority_ledger: Vec<String> = self
            .nodes
            .iter()
            .find(|n| n.get_ledger_hash() == majority_hash)
            .map(|n| n.state_ledger.clone())
            .unwrap_or_default();

        // 3. Detect mismatched nodes and repair
        for idx in 0..self.nodes.len() {
            let cur_hash = &node_hashes[idx].1;
            if *cur_hash != majority_hash {
                self.nodes[idx].status = "recovering".to_string();
                isolated_nodes.push(self.nodes[idx].node_id.clone());
                // Autonomic rolling cache overwrite & AST verification
                self.nodes[idx].state_ledger = majority_ledger.clone();
                self.nodes[idx].status = "healthy".to_string();
                remediated_nodes.push(self.nodes[idx].node_id.clone());
            }
        }

        let node_states: Vec<NodeStateReport> = self
            .nodes
            .iter()
            .map(|n| NodeStateReport {
                node_id: n.node_id.clone(),
                name: n.name.clone(),
                status: n.status.clone(),
                ledger_length: n.state_ledger.len() as u64,
            })
            .collect();

        let hash_prefix = {
            let truncated: String = majority_hash.chars().take(16).collect();
            format!("{}...", truncated)
        };
        AuditReport {
            mesh_status: "AUDITED_HEALTHY".to_string(),
            total_nodes: self.nodes.len() as u64,
            majority_ledger_hash: hash_prefix,
            isolated_count: isolated_nodes.len() as u64,
            remediated_nodes,
            node_states,
        }
    }
}

impl Default for ContinuumPBFTLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton handle (repo convention; python returns the singleton ledger).
pub fn get_continuum_pbft() -> u64 {
    0x434F_4E54_5042_4654 // "CONTPBFT"
}

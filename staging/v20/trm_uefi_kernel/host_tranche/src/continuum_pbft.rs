//! TRM Swarm OS — Continuum PBFT (real no_std port).
//! 1:1 pure semantics from swarm_v2/core/continuum_pbft.py — Project Continuum v5.3
//! Autonomic PBFT consensus & self-healing ledger engine (N >= 3f+1) over the
//! 4-node mesh (ORCH / SAGE / MUSE / SENTINEL), WITHOUT any networking.
//!
//! Deterministic pieces ported: node registry, per-node state ledgers, the
//! 3-phase cycle decision logic (leader health gate -> prepare tally -> quorum
//! >= 2f+1 commit), vote aggregation, and majority-hash mesh audit/repair.
//!
//! // MANUAL notes mark the two places Python used unavailable machinery:
//! //   * hashlib.sha256  -> deterministic 64-bit FNV-1a stand-in (comparison-only)
//! //   * time/datetime   -> caller-supplied tx_id instead of wall-clock tx ids

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// PBFT registry constants (1:1 PBFT_NODES dict, insertion order preserved).
// ---------------------------------------------------------------------------
pub const PBFT_NODE_ORCH: &str = "ORCH";
pub const PBFT_NODE_SAGE: &str = "SAGE";
pub const PBFT_NODE_MUSE: &str = "MUSE";
pub const PBFT_NODE_SENTINEL: &str = "SENTINEL";

/// f used by the v5.3 continuum cycle (python: f_max = 1).
pub const PBFT_F_MAX: u64 = 1;
/// Required prepare votes = 2f+1 = 3 (python: required_votes = 2*f_max + 1).
pub const PBFT_REQUIRED_VOTES: u64 = 2 * PBFT_F_MAX + 1;
/// Minimum mesh size for PBFT = 3f+1 = 4.
pub const PBFT_MIN_NODES: u64 = 3 * PBFT_F_MAX + 1;

/// Quorum arithmetic helper: minimum votes for f Byzantine faults.
pub fn pbft_required_votes(f: u64) -> u64 {
    2 * f + 1
}

/// Minimum total nodes that can tolerate f Byzantine faults (N >= 3f+1).
pub fn pbft_min_total_nodes(f: u64) -> u64 {
    3 * f + 1
}

/// Deterministic quorum check: votes >= 2f+1.
pub fn pbft_quorum_met(prepare_votes: u64, f: u64) -> bool {
    prepare_votes >= pbft_required_votes(f)
}

/// Static node metadata (1:1 with PBFT_NODES entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PbftNodeMeta {
    pub node_id: &'static str,
    pub name: &'static str,
    pub role: &'static str,
}

/// The active mesh registry from Section 5.1 (4 nodes, all initially healthy).
pub fn pbft_node_registry() -> [PbftNodeMeta; 4] {
    [
        PbftNodeMeta { node_id: PBFT_NODE_ORCH, name: "Continuum Core", role: "Primary Leader Node" },
        PbftNodeMeta { node_id: PBFT_NODE_SAGE, name: "Cybernetic Sage", role: "State Auditor Node" },
        PbftNodeMeta { node_id: PBFT_NODE_MUSE, name: "Stochastic Muse", role: "Creative Generator Node" },
        PbftNodeMeta { node_id: PBFT_NODE_SENTINEL, name: "Sentinel Warden", role: "Security Isolation Node" },
    ]
}

// ---------------------------------------------------------------------------
// Node status + per-node state ledger (1:1 PBFTNodeState).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbftNodeStatus {
    Healthy,
    Recovering,
}

impl PbftNodeStatus {
    /// Python models status as the string "healthy"; everything else is treated
    /// as not-healthy by the cycle logic.
    pub fn is_healthy(&self) -> bool {
        matches!(self, PbftNodeStatus::Healthy)
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            PbftNodeStatus::Healthy => "healthy",
            PbftNodeStatus::Recovering => "recovering",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PbftNodeState {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub status: PbftNodeStatus,
    /// state_ledger: Vec of block strings, seeded with a genesis marker.
    pub state_ledger: Vec<String>,
}

impl PbftNodeState {
    /// Python __init__: ledger starts as [f"GENESIS_BLOCK_HASH_{node_id}_v5.3"].
    pub fn new(node_id: &str, name: &str, role: &str) -> Self {
        let genesis = format!("GENESIS_BLOCK_HASH_{}_v5.3", node_id);
        PbftNodeState {
            node_id: node_id.into(),
            name: name.into(),
            role: role.into(),
            status: PbftNodeStatus::Healthy,
            state_ledger: vec![genesis],
        }
    }

    pub fn append_block(&mut self, block: &str) {
        self.state_ledger.push(block.into());
    }

    pub fn ledger_len(&self) -> u64 {
        self.state_ledger.len() as u64
    }

    /// Deterministic ledger fingerprint used for majority comparison.
    ///
    /// // MANUAL: Python uses sha256("|".join(state_ledger).encode()).hexdigest().
    /// // This no_std kernel has no sha256, so the *same* joined string is fed
    /// // through a deterministic 64-bit FNV-1a instead. Consensus logic only
    /// // compares hashes for equality, so any collision-free stand-in preserves
    /// // the audit semantics — but the digest values differ from Python's.
    pub fn ledger_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a 64 offset basis
        for (i, entry) in self.state_ledger.iter().enumerate() {
            if i > 0 {
                h = fnv1a_byte(h, b'|');
            }
            for &b in entry.as_bytes() {
                h = fnv1a_byte(h, b);
            }
        }
        h
    }
}

fn fnv1a_byte(mut h: u64, b: u8) -> u64 {
    h ^= b as u64;
    h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV-1a 64 prime
    h
}

// ---------------------------------------------------------------------------
// Transaction records (1:1 execute_pbft_cycle / get_ledger dicts).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct PbftTxRecord {
    pub tx_id: String,
    pub block: String,
    pub leader: &'static str,
    pub prepare_votes: u64,
    pub required_votes: u64,
    pub status: &'static str, // "COMMITTED"
}

impl PbftTxRecord {
    /// Python includes "timestamp": datetime.now().isoformat() in each record.
    /// // MANUAL: wall-clock timestamps are non-deterministic and are omitted in
    /// // the no_std port; the remaining fields are reproduced 1:1.
    fn committed(tx_id: String, block: String, prepare_votes: u64) -> Self {
        PbftTxRecord {
            tx_id,
            block,
            leader: PBFT_NODE_ORCH,
            prepare_votes,
            required_votes: PBFT_REQUIRED_VOTES,
            status: "COMMITTED",
        }
    }
}

/// Outcome of one 3-phase cycle: COMMITTED (record) or FAILED (reason).
#[derive(Debug, Clone)]
pub enum PbftCycleOutcome {
    Committed(PbftTxRecord),
    Failed { reason: String },
}

/// Outcome of the mesh audit / self-healing pass (1:1 audit_and_repair_mesh).
#[derive(Debug, Clone)]
pub struct MeshAuditResult {
    pub mesh_status: &'static str, // "AUDITED_HEALTHY"
    pub total_nodes: u64,
    /// majority ledger digest, rendered as 16 hex chars + "..."
    /// // MANUAL: python truncates a 64-char sha256 hex digest to [:16] + "...";
    /// // the FNV-1a stand-in is already 16 hex chars, so the full digest is shown.
    pub majority_ledger_hash_prefix: String,
    pub isolated_count: u64,
    pub remediated_nodes: Vec<String>,
    pub node_states: Vec<NodeStateReport>,
}

/// Per-node state snapshot after the audit (name, status, ledger_length).
#[derive(Debug, Clone)]
pub struct NodeStateReport {
    pub node_id: String,
    pub name: String,
    pub status: PbftNodeStatus,
    pub ledger_length: u64,
}

// ---------------------------------------------------------------------------
// Continuum PBFT engine (1:1 ContinuumPBFTLedger, deterministic core).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ContinuumPbftEngine {
    pub nodes: Vec<PbftNodeState>,
    pub transaction_history: Vec<PbftTxRecord>,
}

impl ContinuumPbftEngine {
    /// Python __init__: one PBFTNodeState per PBFT_NODES registry entry.
    pub fn new() -> Self {
        let nodes = pbft_node_registry()
            .iter()
            .map(|m| PbftNodeState::new(m.node_id, m.name, m.role))
            .collect();
        ContinuumPbftEngine {
            nodes,
            transaction_history: Vec::new(),
        }
    }

    /// Runs the 3-phase PBFT cycle: Pre-Prepare (ORCH proposes) -> Prepare
    /// (healthy peers tally) -> Commit (2f+1 quorum gate). Faithful port of
    /// execute_pbft_cycle with the wall-clock tx_id replaced by a caller
    /// supplied deterministic id.
    ///
    /// Returns the COMMITTED record or a FAILED reason exactly as Python does.
    pub fn execute_pbft_cycle(&mut self, action: &str, tx_id: &str) -> PbftCycleOutcome {
        // // MANUAL: python derives tx_id = f"tx_{int(time.time() * 1000)}"
        // // (wall clock). Callers pass a deterministic id; empty => "tx_0".
        let tx_id = if tx_id.is_empty() { "tx_0" } else { tx_id };
        // python: proposal_data.get('action', 'STATE_UPDATE')
        let action = if action.is_empty() { "STATE_UPDATE" } else { action };
        let tx_block = format!("BLOCK_{}_{}", tx_id, action);

        // 1. Pre-Prepare: ORCH proposes block (leader health gate first).
        let orch = self
            .nodes
            .iter_mut()
            .find(|n| n.node_id == PBFT_NODE_ORCH)
            .expect("ORCH always present in v5.3 registry");
        if !orch.status.is_healthy() {
            return PbftCycleOutcome::Failed {
                reason: "Leader Node ORCH compromised or unavailable".into(),
            };
        }
        orch.append_block(&tx_block);
        // python computes pre_prepare_sig = sha256("PRE_PREPARE:{tx_id}:{hash}")
        // but never stores or returns it — // MANUAL: digest dropped (sha256).

        // 2. Prepare: every healthy node verifies and signs (ORCH included,
        //    so ORCH's ledger receives the block twice, exactly like Python).
        let mut prepare_votes: u64 = 0;
        for node in self.nodes.iter_mut() {
            if node.status.is_healthy() {
                node.append_block(&tx_block);
                prepare_votes += 1;
            }
        }

        // 3. Commit: system requires 2f+1 prepare votes (f=1 => 3 required).
        if !pbft_quorum_met(prepare_votes, PBFT_F_MAX) {
            return PbftCycleOutcome::Failed {
                reason: format!(
                    "Insufficient prepare votes: {}/{}",
                    prepare_votes, PBFT_REQUIRED_VOTES
                ),
            };
        }

        let record = PbftTxRecord::committed(tx_id.into(), tx_block, prepare_votes);
        self.transaction_history.push(record.clone());
        PbftCycleOutcome::Committed(record)
    }

    /// Casts a 3f+1 Byzantine consensus vote over the mesh personas.
    /// Python builds {"action": f"VOTE_BLOCK_{block_id}", "hash": proposal_hash}
    /// and runs a cycle; note the hash is never read by execute_pbft_cycle.
    pub fn cast_consensus_vote(&mut self, block_id: &str, _proposal_hash: &str) -> PbftCycleOutcome {
        self.execute_pbft_cycle(&format!("VOTE_BLOCK_{}", block_id), "")
    }

    /// Transaction ledger history. Python returns the stored history, or a
    /// single synthetic genesis record when the history is empty.
    pub fn get_ledger(&self) -> Vec<PbftTxRecord> {
        if !self.transaction_history.is_empty() {
            return self.transaction_history.clone();
        }
        vec_genesis_ledger()
    }

    /// Autonomic self-healing mesh audit (Section 5.3): hash every node's
    /// ledger, find the majority digest, then roll mismatched nodes back to a
    /// majority node's ledger (recovering -> remediated -> healthy).
    pub fn audit_and_repair_mesh(&mut self) -> MeshAuditResult {
        // 1. Compute the ledger digest for each node (registry order).
        let hashes: Vec<(String, u64)> = self
            .nodes
            .iter()
            .map(|n| (n.node_id.clone(), n.ledger_hash()))
            .collect();

        // 2. Majority digest: max over (digest -> count). Python's max() keeps
        //    the FIRST maximal element on ties, reproduced by strict > compare.
        let mut majority_hash: u64 = 0;
        let mut best_count: u64 = 0;
        for (_id, h) in &hashes {
            let count = hashes.iter().filter(|(_, hh)| hh == h).count() as u64;
            if count > best_count {
                best_count = count;
                majority_hash = *h;
            }
        }

        // 3. Detect mismatched nodes and repair against the first node in
        //    registry order that currently holds the majority digest.
        let mut remediated: Vec<String> = Vec::new();
        let mut isolated: Vec<String> = Vec::new();
        let mut i = 0;
        while i < self.nodes.len() {
            if self.nodes[i].ledger_hash() != majority_hash {
                let node_id = self.nodes[i].node_id.clone();
                isolated.push(node_id.clone());
                self.nodes[i].status = PbftNodeStatus::Recovering;

                // First (registry-order) node that matches the majority digest;
                // earlier remediations in this same pass are valid donors too.
                let donor_idx = self
                    .nodes
                    .iter()
                    .position(|n| n.ledger_hash() == majority_hash)
                    .expect("majority digest must exist on at least one node");
                let donor_ledger = self.nodes[donor_idx].state_ledger.clone();
                self.nodes[i].state_ledger = donor_ledger; // rolling cache overwrite
                self.nodes[i].status = PbftNodeStatus::Healthy;
                remediated.push(node_id);
            }
            i += 1;
        }

        let total = self.nodes.len() as u64;
        let node_states = self
            .nodes
            .iter()
            .map(|n| NodeStateReport {
                node_id: n.node_id.clone(),
                name: n.name.clone(),
                status: n.status,
                ledger_length: n.ledger_len(),
            })
            .collect();
        MeshAuditResult {
            mesh_status: "AUDITED_HEALTHY",
            total_nodes: total,
            majority_ledger_hash_prefix: format!("{:016x}...", majority_hash),
            isolated_count: isolated.len() as u64,
            remediated_nodes: remediated,
            node_states,
        }
    }

    /// Number of healthy nodes in the mesh (deterministic tally helper).
    pub fn healthy_node_count(&self) -> u64 {
        self.nodes.iter().filter(|n| n.status.is_healthy()).count() as u64
    }
}

impl Default for ContinuumPbftEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Python get_ledger() synthesizes this record when the history is empty.
pub fn vec_genesis_ledger() -> Vec<PbftTxRecord> {
    vec![PbftTxRecord::committed(
        "tx_genesis_v53".into(),
        "BLOCK_GENESIS_v5.3".into(),
        4,
    )]
}

/// Singleton handle (lazy-init pattern from the Python getter; the engine
/// itself is pure state so callers construct ContinuumPbftEngine::new()).
pub fn get_continuum_pbft() -> u64 {
    0x5042_4654_5f56_3533 // "PBFT_V53"
}

/// Deterministic vote aggregation: count of healthy nodes (each healthy node
/// contributes exactly one prepare vote — the leader is included, mirroring
/// the Python prepare loop over self.nodes).
pub fn tally_prepare_votes(nodes: &[PbftNodeState]) -> u64 {
    nodes.iter().filter(|n| n.status.is_healthy()).count() as u64
}

/// Majority digest across the mesh (ties resolve to the first node in order).
pub fn majority_ledger_hash(nodes: &[PbftNodeState]) -> u64 {
    let hashes: Vec<u64> = nodes.iter().map(|n| n.ledger_hash()).collect();
    let mut majority: u64 = 0;
    let mut best: u64 = 0;
    for &h in &hashes {
        let c = hashes.iter().filter(|&&hh| hh == h).count() as u64;
        if c > best {
            best = c;
            majority = h;
        }
    }
    majority
}

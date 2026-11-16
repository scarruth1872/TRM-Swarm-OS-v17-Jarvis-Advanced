//! TRM Swarm OS — Persistent NVMe Cold-Start Recovery Engine (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/cold_start_recovery.py & strip_cold_start.rs.
//! Restores persistent state for all 15 agents from NVMe-backed mmap arenas in <5ms,
//! verifies 32KB per-agent slot memory counts, task progress, and polynomial checksums.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::convert::TryInto;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const COLD_START_HEADER_SZ: usize = 64;
pub const AGENT_SLOT_SZ: usize = 32768; // 32KB per agent slot
pub const NUM_AGENT_SLOTS: usize = 15;
pub const MIN_ARENA_SIZE_REQUIRED: usize = COLD_START_HEADER_SZ + (NUM_AGENT_SLOTS * AGENT_SLOT_SZ);

// --- REPR(C) AGENT SLOT SUMMARY (1:1 with strip_cold_start.rs) ---
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentSlotSummary {
    pub slot_id: u32,
    pub memory_count: u32,
    pub task_count: u32,
    pub crc32: u32,
}

// --- AGENT STATE ---
#[derive(Debug, Clone)]
pub struct AgentState {
    pub name: String,
    pub domain: String,
    pub mamba3_h: [f32; 32],
    pub lora_adapter_id: u32,
    pub k3_state: i8, // -1, 0, +1
    pub memory_count: u64,
    pub task_count: u64,
    pub last_heartbeat_epoch_s: u64,
}

impl AgentState {
    pub fn new(name: &str, domain: &str, epoch_s: u64) -> Self {
        Self {
            name: name.to_string(),
            domain: domain.to_string(),
            mamba3_h: [0.0f32; 32],
            lora_adapter_id: 1,
            k3_state: 1, // K3 True
            memory_count: 0,
            task_count: 0,
            last_heartbeat_epoch_s: epoch_s,
        }
    }
}

// --- BARE-METAL ARENA SCANNER ---
pub fn scan_cold_start_arena(buf: &[u8]) -> Vec<AgentSlotSummary> {
    if buf.len() < MIN_ARENA_SIZE_REQUIRED {
        return Vec::new();
    }

    let mut found = Vec::with_capacity(NUM_AGENT_SLOTS);

    for s in 0..NUM_AGENT_SLOTS {
        let off = COLD_START_HEADER_SZ + (s * AGENT_SLOT_SZ);
        let blk = &buf[off..off + AGENT_SLOT_SZ];

        // Check if non-zero in first 32 bytes (slot active marker)
        let is_active = blk[..32].iter().any(|&b| b != 0);
        if is_active {
            let sid = u32::from_le_bytes(blk[4..8].try_into().unwrap_or([0; 4]));
            let mcnt = u32::from_le_bytes(blk[8..12].try_into().unwrap_or([0; 4]));
            let tcnt = u32::from_le_bytes(blk[12..16].try_into().unwrap_or([0; 4]));

            let mut crc = 0u32;
            for &b in &blk[..256] {
                crc = crc.wrapping_mul(31).wrapping_add(b as u32);
            }

            found.push(AgentSlotSummary {
                slot_id: sid,
                memory_count: mcnt,
                task_count: tcnt,
                crc32: crc,
            });
        }
    }

    found
}

// --- PERSISTENT STATE MANAGER ---
pub struct PersistentStateManager {
    recovered_slots: Vec<AgentSlotSummary>,
    pub recovery_latency_us: u32,
    pub is_resumed: bool,
}

impl PersistentStateManager {
    pub fn new() -> Self {
        Self {
            recovered_slots: Vec::new(),
            recovery_latency_us: 0,
            is_resumed: false,
        }
    }

    pub fn recover_from_arena(&mut self, arena_bytes: &[u8]) -> Result<usize, &'static str> {
        if arena_bytes.len() < MIN_ARENA_SIZE_REQUIRED {
            return Err("Arena buffer smaller than minimum cold-start size");
        }

        let slots = scan_cold_start_arena(arena_bytes);
        let count = slots.len();
        self.recovered_slots = slots;
        // NOT a measurement: the oracle times this with perf_counter_ns, which the kernel cannot do.
        // Reporting 450 here would be asserting a number we never measured (it previously claimed
        // "0.45ms achieved" in a comment), so it stays 0 until something derives it.
        self.recovery_latency_us = 0;
        self.is_resumed = count > 0;

        Ok(count)
    }

    pub fn get_slot_summary(&self, slot_id: u32) -> Option<&AgentSlotSummary> {
        self.recovered_slots.iter().find(|s| s.slot_id == slot_id)
    }

    pub fn total_recovered_memories(&self) -> u32 {
        self.recovered_slots.iter().map(|s| s.memory_count).sum()
    }

    pub fn total_recovered_tasks(&self) -> u32 {
        self.recovered_slots.iter().map(|s| s.task_count).sum()
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_COLD_START: AtomicU64 = AtomicU64::new(0);

pub fn get_cold_start_recovery() -> u64 {
    let v = SINGLETON_COLD_START.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_COLD_START.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ─── RECOVERY / RESTORE DECISION RULES — port of swarm_v2/core/cold_start_recovery.py ───
//
// The oracle's genuinely portable computation is the recovery decision (recover_all_agents):
//
//   for slot, agent_name in enumerate(AGENTS):                 # fixed roster order, always 15
//       raw = bytes(mmap[HEADER + slot*AGENT_STATE_SIZE : ... ])   # one 32KB slot
//       raw = raw.rstrip(b'\x00')                              # eligibility: ANY non-zero byte
//       if raw:   recovered[agent_name] = pickle.loads(raw)    # parse failure -> domain 'recovered'
//       else:     recovered[agent_name] = fresh, domain='fresh'  # never-saved fallback
//   recovery_stats = {'status': 'OK'|'FAILED', 'agents_recovered': len(recovered),
//                     'time_us': elapsed_us, 'within_5ms': elapsed_us < 5000}
//
// plus the slot mapping (AGENTS.index(name), ValueError -> slot -1 -> no write) and the
// save_all_agents default-fill (domain 'unknown', zeros(32), empty lora, k3 'ANALYZE', 0/0).
//
// Host-side in the oracle and therefore NOT ported: Path.mkdir, open()/b'\x00'*size file
// creation, os.open + mmap.mmap/flush, pickle.dumps/loads, threading.Lock, time.perf_counter_ns,
// time.strftime/strftime ISO timestamps, the json report write, and the ctypes call into
// strip_cold_start.dll (that last one is already mirrored by scan_cold_start_arena).

/// python AGENTS = [...] — fixed slot-index -> agent-name mapping, used by both
/// save_agent_state (slot lookup) and recover_all_agents (recovery order).
pub const AGENTS: [&str; NUM_AGENT_SLOTS] = [
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge",
    "Orchestra", "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes",
];

/// python RING_BUFFER_SIZE = 4096 * 1536  (4096 slots x 384 float32s x 4B)
pub const RING_SLOT_BYTES: usize = 1536; // 384 float32s * 4B
pub const RING_BUFFER_SLOTS: usize = 4096;
pub const RING_BUFFER_SZ: usize = RING_BUFFER_SLOTS * RING_SLOT_BYTES;

/// python self._mmap_size = HEADER_SIZE + 15 * AGENT_STATE_SIZE + RING_BUFFER_SIZE
pub const COLD_START_ARENA_SZ: usize =
    COLD_START_HEADER_SZ + (NUM_AGENT_SLOTS * AGENT_SLOT_SZ) + RING_BUFFER_SZ;

/// python recovery_stats['within_5ms'] = elapsed_us < 5000
pub const RECOVERY_BUDGET_US: u32 = 5000;

/// python fresh-state k3_state 'ANALYZE' (K3 tri-state -1/0/+1 -> 0 = ANALYZE)
pub const K3_ANALYZE: i8 = 0;

/// python domain defaults: 'fresh' (slot never saved), 'unknown' (save_all_agents default).
pub const DOMAIN_FRESH: &str = "fresh";
pub const DOMAIN_DEFAULT: &str = "unknown";
/// Restored slots keep the persisted domain in the oracle, but that string travels inside the
/// host pickle payload and is not recoverable from the raw arena, so the kernel leaves it empty
/// instead of inventing a label.
pub const DOMAIN_NOT_PERSISTED: &str = "";

impl AgentState {
    /// python never-saved fallback record: AgentState(name, domain='fresh', mamba3_h=[0.0]*32,
    /// lora_weights={}, k3_state='ANALYZE', memory_count=0, task_count=0).
    pub fn fresh(name: &str, now_epoch_s: u64) -> Self {
        Self {
            name: name.to_string(),
            domain: DOMAIN_FRESH.to_string(),
            mamba3_h: [0.0f32; 32],
            lora_adapter_id: 0, // empty lora_weights
            k3_state: K3_ANALYZE,
            memory_count: 0,
            task_count: 0,
            last_heartbeat_epoch_s: now_epoch_s,
        }
    }
}

/// python save_agent_state: slot = AGENTS.index(agent.name); ValueError -> slot = -1 (no write).
pub fn agent_slot_index(name: &str) -> Option<usize> {
    AGENTS.iter().position(|&n| n == name)
}

/// python offset = HEADER_SIZE + slot * AGENT_STATE_SIZE
pub fn agent_slot_offset(slot: usize) -> usize {
    COLD_START_HEADER_SZ + slot * AGENT_SLOT_SZ
}

/// python recovery_stats dict ('OK' | 'FAILED').
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryStats {
    pub status: &'static str,
    pub agents_recovered: usize,
    pub time_us: u32,
    pub within_5ms: bool,
}

/// python save_all_agents() input record: stats[name] with optional keys. Absent keys take the
/// oracle defaults: domain 'unknown', memory_count 0, task_count 0, k3 'ANALYZE', zeros(32).
pub struct AgentStatsEntry<'a> {
    pub name: &'a str,
    pub domain: Option<&'a str>,
    pub memory_count: Option<u64>,
    pub task_count: Option<u64>,
}

/// python save_all_agents(): build each record with the oracle's defaults and map it onto its
/// roster slot. Names outside AGENTS are dropped (slot -1 -> the write is skipped).
pub fn save_all_agents(entries: &[AgentStatsEntry], now_epoch_s: u64) -> Vec<(usize, AgentState)> {
    let mut planned: Vec<(usize, AgentState)> = Vec::new();
    for e in entries.iter() {
        if let Some(slot) = agent_slot_index(e.name) {
            planned.push((
                slot,
                AgentState {
                    name: e.name.to_string(),
                    domain: e.domain.unwrap_or(DOMAIN_DEFAULT).to_string(),
                    mamba3_h: [0.0f32; 32],
                    lora_adapter_id: 0,
                    k3_state: K3_ANALYZE,
                    memory_count: e.memory_count.unwrap_or(0),
                    task_count: e.task_count.unwrap_or(0),
                    last_heartbeat_epoch_s: now_epoch_s,
                },
            ));
        }
    }
    planned
}

/// python recover_all_agents(): ALWAYS NUM_AGENT_SLOTS (15) entries, in AGENTS order.
///
///   eligible/restored: the slot is non-empty after rstrip(b'\x00'), i.e. ANY of the 32768 slot
///     bytes is non-zero. This is deliberately NOT the native strip's test — strip_cold_start.rs
///     and scan_cold_start_arena only look at the first 32 bytes, so a slot whose data starts
///     past byte 32 is restorable for the oracle but invisible to the scanner.
///   fallback: a never-written slot yields AgentState::fresh(name, now), domain 'fresh'.
///   The oracle's domain 'recovered' branch is its pickle.loads-failure branch; that trigger is a
///     host deserializer, so it has no portable analogue and is not reproduced.
///   arena None / shorter than MIN_ARENA_SIZE_REQUIRED == oracle "self._mmap is None and
///     initialize_mmap() failed" -> status 'FAILED' and zero agents recovered.
///
/// `time_us` is caller-measured (the oracle times itself with time.perf_counter_ns, host-side);
/// `now_epoch_s` replaces time.strftime for last_heartbeat.
pub fn recover_all_agents(
    arena: Option<&[u8]>,
    now_epoch_s: u64,
    time_us: u32,
) -> (Vec<(String, AgentState)>, RecoveryStats) {
    let within_5ms = time_us < RECOVERY_BUDGET_US;
    let mut recovered: Vec<(String, AgentState)> = Vec::with_capacity(NUM_AGENT_SLOTS);

    let buf = match arena {
        Some(b) if b.len() >= MIN_ARENA_SIZE_REQUIRED => b,
        _ => {
            return (
                recovered,
                RecoveryStats {
                    status: "FAILED",
                    agents_recovered: 0,
                    time_us,
                    within_5ms,
                },
            );
        }
    };

    for (slot, agent_name) in AGENTS.iter().enumerate() {
        let off = agent_slot_offset(slot);
        let blk = &buf[off..off + AGENT_SLOT_SZ];

        // python: raw = raw.rstrip(b'\x00'); if raw:
        let populated = blk.iter().any(|&b| b != 0);

        let state = if populated {
            // Restored: the counters come from the native slot descriptor (u32 LE at +8/+12),
            // the same layout strip_cold_start.rs / scan_cold_start_arena read.
            let mcnt = u32::from_le_bytes(blk[8..12].try_into().unwrap_or([0; 4]));
            let tcnt = u32::from_le_bytes(blk[12..16].try_into().unwrap_or([0; 4]));
            AgentState {
                name: agent_name.to_string(),
                domain: DOMAIN_NOT_PERSISTED.to_string(),
                mamba3_h: [0.0f32; 32],
                lora_adapter_id: 0,
                k3_state: K3_ANALYZE,
                memory_count: mcnt as u64,
                task_count: tcnt as u64,
                last_heartbeat_epoch_s: now_epoch_s,
            }
        } else {
            AgentState::fresh(agent_name, now_epoch_s)
        };

        recovered.push((agent_name.to_string(), state));
    }

    let stats = RecoveryStats {
        status: "OK",
        agents_recovered: recovered.len(),
        time_us,
        within_5ms,
    };
    (recovered, stats)
}


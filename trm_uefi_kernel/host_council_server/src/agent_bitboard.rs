//! 584-Agent Kleene-3 (K3) Bitboard Evaluator & Shared Arena Writer.
//!
//! Exercises `trm_uefi_kernel::digital_twin::DigitalTwinSingleton` (300 FastPath +
//! 150 InstellaMoe + 134 BonsaiTernary = 584 slots), packs all 584 trits into
//! 19 x 32-bit bitboard lanes (`pos_lanes`, `neg_lanes`) for bitwise consensus,
//! and writes a fixed `#[repr(C)]` binary arena snapshot (`data/trm_agent_arena.bin`)
//! for zero-copy local IPC inspection.

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use trm_uefi_kernel::digital_twin::{AgentTeam, DigitalTwinSingleton};

pub const AGENT_COUNT: usize = 584;
pub const BITBOARD_LANES: usize = 19; // 19 * 32 = 608 >= 584
pub const ARENA_MAGIC: u64 = 0x434F_474E_4954_4956; // "COGNITIV"

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SharedArenaFrame {
    pub magic: u64,
    pub sequence: u64,
    pub agent_count: u32,
    pub consensus_trit: i8,
    pub fast_path_trit: i8,
    pub instella_trit: i8,
    pub bonsai_trit: i8,
    pub pos_lanes: [u32; BITBOARD_LANES],
    pub neg_lanes: [u32; BITBOARD_LANES],
    pub trits: [i8; AGENT_COUNT],
}

impl SharedArenaFrame {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

pub struct BitboardRuntime {
    pub twin: DigitalTwinSingleton,
    pub eval_count: u64,
    pub last_elapsed_ns: u128,
    pub last_consensus: i8,
    pub pos_lanes: [u32; BITBOARD_LANES],
    pub neg_lanes: [u32; BITBOARD_LANES],
}

impl BitboardRuntime {
    fn new() -> Self {
        let twin = DigitalTwinSingleton::new();
        let (pos_lanes, neg_lanes) = pack_bitboards(&twin);
        Self {
            twin,
            eval_count: 0,
            last_elapsed_ns: 0,
            last_consensus: 1,
            pos_lanes,
            neg_lanes,
        }
    }
}

static RUNTIME: OnceLock<Mutex<BitboardRuntime>> = OnceLock::new();

fn runtime() -> &'static Mutex<BitboardRuntime> {
    RUNTIME.get_or_init(|| Mutex::new(BitboardRuntime::new()))
}

fn arena_path() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("TRM_REPO_ROOT") {
        return Some(PathBuf::from(root).join("data").join("trm_agent_arena.bin"));
    }
    let cands = ["data/trm_agent_arena.bin", "../../data/trm_agent_arena.bin"];
    for c in cands {
        let p = PathBuf::from(c);
        if p.parent().map(|d| d.exists()).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// Pack 584 agent `state_trit` values into 19 x 32-bit `(pos_lanes, neg_lanes)` bitboards.
pub fn pack_bitboards(twin: &DigitalTwinSingleton) -> ([u32; BITBOARD_LANES], [u32; BITBOARD_LANES]) {
    let mut pos = [0u32; BITBOARD_LANES];
    let mut neg = [0u32; BITBOARD_LANES];
    for (idx, slot) in twin.slots.iter().take(AGENT_COUNT).enumerate() {
        let lane = idx >> 5;
        let bit = 1u32 << (idx & 31);
        match slot.state_trit {
            1 => pos[lane] |= bit,
            -1 => neg[lane] |= bit,
            _ => {}
        }
    }
    (pos, neg)
}

/// Reduce a bitboard range `[start_idx, end_idx)` into a Kleene-3 consensus trit:
/// `-1` if any agent in the range is `-1`, `0` if any is `0`, `+1` if all are `+1`.
pub fn reduce_range_k3(pos: &[u32; BITBOARD_LANES], neg: &[u32; BITBOARD_LANES], start: usize, end: usize) -> i8 {
    let mut any_unknown = false;
    for lane in 0..BITBOARD_LANES {
        let lane_start = lane * 32;
        let lane_end = lane_start + 32;
        if lane_end <= start || lane_start >= end {
            continue;
        }
        let mut mask = u32::MAX;
        if start > lane_start {
            mask &= u32::MAX << (start - lane_start);
        }
        if end < lane_end {
            let keep = end - lane_start;
            mask &= (1u32 << keep) - 1;
        }
        if (neg[lane] & mask) != 0 {
            return -1;
        }
        if (pos[lane] & mask) != mask {
            any_unknown = true;
        }
    }
    if any_unknown { 0 } else { 1 }
}

/// Evaluate a query/directive across the 584-agent bitboard (with semantic :8593 signal) and persist the binary arena.
pub fn evaluate_directive(input: &str) -> Value {
    evaluate_directive_inner(input, true, true)
}

/// Evaluate a query/directive across the 584-agent bitboard with optional disk arena flush (fast-path, no :8593 network call).
pub fn evaluate_directive_with_persist(input: &str, persist_arena: bool) -> Value {
    evaluate_directive_inner(input, persist_arena, false)
}

fn evaluate_directive_inner(input: &str, persist_arena: bool, use_semantic: bool) -> Value {
    let t0 = Instant::now();
    let blocking = trm_uefi_kernel::ddr_scan::blocking_finding(input);
    let sem = if use_semantic && blocking.is_none() {
        crate::semantic_signal::classify(input)
    } else {
        None
    };
    let sem_blocks = sem.as_ref().map(|s| s.trit == -1).unwrap_or(false);
    let sem_analyzes = sem.as_ref().map(|s| s.trit == 0).unwrap_or(false);
    let trimmed = input.trim();
    let is_ambiguous = trimmed.is_empty()
        || trimmed.contains('?')
        || trimmed.split_whitespace().count() > 14
        || sem_analyzes;

    let mut guard = match runtime().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };

    // Update agent slots deterministically from the measured input signal.
    for slot in guard.twin.slots.iter_mut() {
        slot.task_count = slot.task_count.saturating_add(1);
        slot.state_trit = if (blocking.is_some() || sem_blocks) && slot.team == AgentTeam::FastPath {
            -1
        } else if is_ambiguous && slot.team != AgentTeam::FastPath {
            0
        } else {
            1
        };
    }

    let (pos, neg) = pack_bitboards(&guard.twin);
    let fast_trit = reduce_range_k3(&pos, &neg, 0, 300);
    let instella_trit = reduce_range_k3(&pos, &neg, 300, 450);
    let bonsai_trit = reduce_range_k3(&pos, &neg, 450, AGENT_COUNT);
    let overall = reduce_range_k3(&pos, &neg, 0, AGENT_COUNT);

    match overall {
        1 => guard.twin.bypass_count += 1,
        0 => guard.twin.instella_offloads += 1,
        _ => {}
    }
    guard.twin.hvbe_tick += 1;
    guard.eval_count += 1;
    guard.pos_lanes = pos;
    guard.neg_lanes = neg;
    guard.last_consensus = overall;

    let mut trits = [0i8; AGENT_COUNT];
    for (i, s) in guard.twin.slots.iter().take(AGENT_COUNT).enumerate() {
        trits[i] = s.state_trit;
    }

    let frame = SharedArenaFrame {
        magic: ARENA_MAGIC,
        sequence: guard.eval_count,
        agent_count: AGENT_COUNT as u32,
        consensus_trit: overall,
        fast_path_trit: fast_trit,
        instella_trit,
        bonsai_trit,
        pos_lanes: pos,
        neg_lanes: neg,
        trits,
    };

    let arena_written = if persist_arena {
        if let Some(path) = arena_path() {
            std::fs::write(&path, frame.as_bytes()).is_ok()
        } else {
            false
        }
    } else {
        false
    };

    let elapsed_ns = t0.elapsed().as_nanos();
    guard.last_elapsed_ns = elapsed_ns;

    let route = match overall {
        -1 => "INTERCEPT_BLOCKED",
        0 => "OFFLOAD_MODEL_8587_OR_8596",
        _ => "FAST_PATH_NATIVE",
    };

    json!({
        "agent_count": AGENT_COUNT,
        "bitboard_lanes": BITBOARD_LANES,
        "consensus_trit": overall,
        "route": route,
        "teams": {
            "fast_path_300": fast_trit,
            "instella_moe_150": instella_trit,
            "bonsai_ternary_134": bonsai_trit
        },
        "eval_count": guard.eval_count,
        "elapsed_ns": elapsed_ns,
        "arena_frame_bytes": std::mem::size_of::<SharedArenaFrame>(),
        "arena_persisted": arena_written,
        "blocking_finding": blocking.map(|f| f.error_type),
        "semantic_signal": match &sem {
            Some(s) => json!({
                "used": true,
                "label": s.label,
                "trit": s.trit,
                "scores": s.scores,
                "elapsed_ns": s.elapsed_ns,
            }),
            None => json!({ "used": false }),
        },
    })
}

pub fn status_json() -> Value {
    let guard = match runtime().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    json!({
        "agent_count": guard.twin.slots.len(),
        "teams": {
            "fast_path": guard.twin.fast_path_count,
            "instella_moe": guard.twin.instella_count,
            "bonsai_ternary": guard.twin.bonsai_count,
        },
        "bitboard_lanes": BITBOARD_LANES,
        "eval_count": guard.eval_count,
        "last_consensus_trit": guard.last_consensus,
        "last_elapsed_ns": guard.last_elapsed_ns,
        "bypass_count": guard.twin.bypass_count,
        "instella_offloads": guard.twin.instella_offloads,
        "arena_frame_bytes": std::mem::size_of::<SharedArenaFrame>(),
        "arena_path": arena_path().map(|p| p.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_584_bitboard_consensus_and_arena() {
        let mut twin = DigitalTwinSingleton::new();
        assert_eq!(twin.slots.len(), AGENT_COUNT);
        let (pos, neg) = pack_bitboards(&twin);
        assert_eq!(reduce_range_k3(&pos, &neg, 0, 300), 1);
        assert_eq!(reduce_range_k3(&pos, &neg, 300, 450), 0); // Instella starts idle (0)
        assert_eq!(reduce_range_k3(&pos, &neg, 450, AGENT_COUNT), -1); // Bonsai starts dormant (-1)
        assert_eq!(reduce_range_k3(&pos, &neg, 0, AGENT_COUNT), -1);

        twin.slots[42].state_trit = -1;
        let (pos2, neg2) = pack_bitboards(&twin);
        assert_eq!(reduce_range_k3(&pos2, &neg2, 0, 300), -1);
        assert_eq!(reduce_range_k3(&pos2, &neg2, 0, AGENT_COUNT), -1);

        let res = evaluate_directive("status check");
        assert_eq!(res["agent_count"], 584);
        assert_eq!(res["consensus_trit"], 1);
        let blocked = evaluate_directive("import os; eval(user_input)");
        assert_eq!(blocked["consensus_trit"], -1);
    }
}

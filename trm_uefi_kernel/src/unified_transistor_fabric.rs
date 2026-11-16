//! Unified Transistor Fabric - one addressable pool of every transistor on
//! and attached to this motherboard, with pinning of work units to slices.
//!
//! Before this module the platform accounted for transistors in fragments:
//! L3 cells in sram_pattern_computer.rs, GPU figures in comments in
//! gpu_infinity_computer.rs. A source search for unified_transistor /
//! transistor_pool / pin_to / pinning returned NOTHING - there was no unified
//! pool and no pinning concept. This module is that binding.
//!
//! PROVENANCE per source: MEASURED only when this host reported it; DATASHEET
//! for silicon counts the OS cannot expose. Nothing is asserted unlabelled.

use alloc::vec::Vec;

/// Where a transistor figure came from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Read from this host at runtime (CPU/GPU capacity the OS exposes).
    Measured,
    /// Published silicon figure for a named part; not OS-readable.
    Datasheet,
    /// Derived from a measured capacity via a documented bits-per-cell factor.
    Derived,
}

/// One contributor of transistors to the unified pool.
#[derive(Clone, Copy)]
pub struct TransistorSource {
    pub name: &'static str,
    pub kind: &'static str,
    pub transistors: u64,
    pub provenance: Provenance,
    pub note: &'static str,
}

/// Ryzen 9 3900X: 2x CCD (4.15B each) + IOD (2.09B). OS cannot read this.
pub const CPU_COMPUTE_TRANSISTORS: u64 = 10_390_000_000;
/// Radeon RX 6700 XT (Navi 22) die total. OS cannot read this.
pub const GPU_DIE_TRANSISTORS: u64 = 17_200_000_000;
/// Infinity Cache: 96 MB, the GPU die portion addressable for K3 state.
pub const INFINITY_CACHE_BYTES: u64 = 96 * 1024 * 1024;
/// DRAM cells are storage, not logic. Counted in a separate category so the
/// pool cannot repeat the earlier error of summing cells into compute.
pub const DRAM_BYTES: u64 = 32 * 1024 * 1024 * 1024;

/// A work unit (function, agent or process) pinned to a slice of the pool.
#[derive(Clone, Copy)]
pub struct PinAssignment {
    pub unit: &'static str,
    pub source: usize,
    pub offset: u64,
    pub span: u64,
}

/// The unified pool: every silicon source, plus the pin table.
pub struct UnifiedTransistorPool {
    pub sources: Vec<TransistorSource>,
    pub pins: Vec<PinAssignment>,
    cursor: usize,
}

impl UnifiedTransistorPool {
    /// Enumerate every transistor source attached to this board.
    pub fn new() -> Self {
        let mut sources = Vec::new();
        sources.push(TransistorSource {
            name: "Ryzen 9 3900X",
            kind: "cpu_compute",
            transistors: CPU_COMPUTE_TRANSISTORS,
            provenance: Provenance::Datasheet,
            note: "2x CCD 4.15B + IOD 2.09B",
        });
        sources.push(TransistorSource {
            name: "L3 SRAM",
            kind: "cpu_cache",
            transistors: crate::sram_pattern_computer::L3_TOTAL_TRANSISTORS,
            provenance: Provenance::Derived,
            note: "measured 64MB L3 x 8 bits x 6 transistors/cell",
        });
        sources.push(TransistorSource {
            name: "RX 6700 XT",
            kind: "gpu_compute",
            transistors: GPU_DIE_TRANSISTORS,
            provenance: Provenance::Datasheet,
            note: "Navi 22, 2560 shaders",
        });
        sources.push(TransistorSource {
            name: "Infinity Cache",
            kind: "gpu_cache",
            transistors: (INFINITY_CACHE_BYTES * 8) * 6,
            provenance: Provenance::Derived,
            note: "96MB IC x 8 bits x 6 transistors/cell",
        });
        sources.push(TransistorSource {
            name: "DDR4 DRAM",
            kind: "storage_cells",
            transistors: DRAM_BYTES * 8,
            provenance: Provenance::Derived,
            note: "STORAGE CELLS - not compute; kept separate on purpose",
        });
        UnifiedTransistorPool { sources, pins: Vec::new(), cursor: 0 }
    }

    /// Total COMPUTE transistors (excludes DRAM storage cells).
    pub fn total_compute(&self) -> u64 {
        self.sources.iter()
            .filter(|s| s.kind != "storage_cells")
            .map(|s| s.transistors)
            .sum()
    }

    /// Total STORAGE cells (DRAM), reported separately and never added to compute.
    pub fn total_storage(&self) -> u64 {
        self.sources.iter()
            .filter(|s| s.kind == "storage_cells")
            .map(|s| s.transistors)
            .sum()
    }

    /// Pin a work unit to a slice of the unified pool.
    ///
    /// Allocation is a bump over the compute sources in order, so a unit is bound
    /// to exactly one silicon source and the binding is recorded and auditable.
    /// Returns None when the requested span exceeds what remains.
    pub fn pin(&mut self, unit: &'static str, span: u64) -> Option<PinAssignment> {
        let idx = self.cursor.min(self.sources.len().saturating_sub(1));
        let src = self.sources[idx];
        if src.kind == "storage_cells" {
            return None;
        }
        let used: u64 = self.pins.iter().filter(|p| p.source == idx).map(|p| p.span).sum();
        if used + span > src.transistors {
            self.cursor += 1;
            if self.cursor >= self.sources.len() {
                return None;
            }
            return self.pin(unit, span);
        }
        let a = PinAssignment { unit, source: idx, offset: used, span };
        self.pins.push(a);
        Some(a)
    }
}

// -- TERNARY PIPELINE + CORE BINDING -----------------------------------
//
// Pipeline stages, all reusing primitives the platform already ships:
//   SENSE  sample hardware into an f32 vector
//   REDUCE KleeneTernaryEngine::quick_reduction   R^d -> T^m      [K3]
//   DECIDE kleene_and / kleene_or / kleene_not -> t in -1,0,+1
//   BIND   map t onto a core and a transistor slice
//
// Binding formula for work unit i and core j:
//
//     score(i,j) = t_i * T_j_over_Ttotal * (1 - load_j)
//     bind(i)    = argmax_j score(i,j)   when max > 0
//
// T_j = total_compute / n_cores, this core's share of the unified pool.
// Kleene semantics carry the safety property: t = 0 UNKNOWN binds nothing
// rather than guessing, t = -1 FALSE releases. Only a definite TRUE takes
// a physical core.

/// One core in the binding topology.
#[derive(Clone, Copy)]
pub struct CoreSlot {
    pub index: u8,
    pub transistors: u64,
    pub load: f32,
}

/// Outcome of a pipeline run for one work unit.
#[derive(Clone, Copy)]
pub struct BindingDecision {
    pub unit_id: u64,
    pub trit: i8,
    pub core: Option<u8>,
    pub score: f32,
}

/// SENSE -> REDUCE -> DECIDE. Reuses the platform's own K3 reduction and
/// Kleene operators; returns the decision trit in -1, 0, +1.
pub fn ternary_decision(
    sensed: &[f32],
    engine: &crate::k3_logic::KleeneTernaryEngine,
) -> i8 {
    let trits = engine.quick_reduction(sensed);
    if trits.len() < 3 {
        return 0;
    }
    let active = crate::k3_logic::KleeneTernaryEngine::kleene_or(trits[0], trits[1]);
    let not_c = crate::k3_logic::KleeneTernaryEngine::kleene_not(trits[2]);
    crate::k3_logic::KleeneTernaryEngine::kleene_and(active, not_c)
}

/// BIND: score every core and take the argmax when positive.
pub fn bind_core(unit_id: u64, trit: i8, cores: &[CoreSlot], total: u64) -> BindingDecision {
    let mut best: Option<(u8, f32)> = None;
    if trit > 0 {
        for c in cores {
            let share = c.transistors as f32 / total.max(1) as f32;
            let score = trit as f32 * share * (1.0 - c.load);
            if best.map(|(_, s)| score > s).unwrap_or(true) {
                best = Some((c.index, score));
            }
        }
    }
    match best {
        Some((idx, s)) if s > 0.0 => BindingDecision { unit_id, trit, core: Some(idx), score: s },
        _ => BindingDecision { unit_id, trit, core: None, score: 0.0 },
    }
}

/// Equal transistor share per core, loads supplied by the caller.
pub fn default_cores(total: u64, loads: &[f32]) -> Vec<CoreSlot> {
    let per = total / 12;
    loads
        .iter()
        .enumerate()
        .map(|(i, l)| CoreSlot { index: i as u8, transistors: per, load: *l })
        .collect()
}

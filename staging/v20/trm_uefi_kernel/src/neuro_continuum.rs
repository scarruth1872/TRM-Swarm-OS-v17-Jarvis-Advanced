//! TRM Swarm OS - neuro_continuum (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/neuro_continuum.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


// ── The oracle's model (swarm_v2/core/neuro_continuum.py) ────────────────────────────────────────
//
// `NeuroDynamicEvolutionEngine` holds generation 201, total_steps 1_414_035, plasticity 0.000581 and
// mutation_rate 0.001, with `fitness = 1.0 - plasticity` DERIVED rather than stored on its own.
// `evolve(mutation_pressure = 0.001)` increments the generation and multiplies
//     mutation_rate *= (1.0 + mutation_pressure)
// returning the generation, `round(mutation_rate, 6)`, the active-neuron count and
// `round(fitness, 6)`. The neuron count comes from a torch checkpoint (`.pt`) on the python side,
// which has no EFI counterpart, so it is passed IN rather than invented here.
//
// The three `route_*` functions carried `0 // TODO: complete 1:1 math`; the state they route is real
// now, and a live instance backs them so the port is reachable rather than merely correct.

pub const GEN_BASE: u64 = 201;
pub const TOTAL_STEPS: u64 = 1_414_035;
pub const PLASTICITY: f64 = 0.000581;
pub const MUTATION_RATE_BASE: f64 = 0.001;

/// `round(x, 6)` without std.
pub fn round6(v: f64) -> f64 {
    let s = v * 1_000_000.0;
    let r = if s >= 0.0 { (s + 0.5) as i64 } else { (s - 0.5) as i64 };
    (r as f64) / 1_000_000.0
}

/// The engine's mutable state.
#[derive(Clone, Copy)]
pub struct NeuroState {
    pub generation: u64,
    pub mutation_rate: f64,
}

impl NeuroState {
    pub const fn new() -> Self {
        Self { generation: GEN_BASE, mutation_rate: MUTATION_RATE_BASE }
    }
    /// `fitness = 1.0 - plasticity`, derived every time rather than stored.
    pub fn fitness(&self) -> f64 {
        1.0 - PLASTICITY
    }
    pub fn plasticity(&self) -> f64 {
        PLASTICITY
    }
    pub fn synapses(&self) -> u64 {
        TOTAL_STEPS
    }
    /// One evolution cycle: the oracle's two updates plus its rounding.
    /// Returns (generation, mutation_rate, active_neurons, fitness).
    pub fn evolve(&mut self, mutation_pressure: f64, active_neurons: usize) -> (u64, f64, usize, f64) {
        self.generation += 1;
        self.mutation_rate *= 1.0 + mutation_pressure;
        (self.generation, round6(self.mutation_rate), active_neurons, round6(self.fitness()))
    }
}

/// The live engine instance the route_* functions report from.
static mut NEURO_ENGINE: NeuroState = NeuroState::new();

/// Direct microkernel call - no HTTP. Reports the engine's generation.
pub fn route_neuro_engine() -> u64 {
    unsafe { NEURO_ENGINE.generation }
}

/// Direct microkernel call - no HTTP. Reports the continuum ledger's total synaptic events.
pub fn route_continuum_ledger() -> u64 {
    TOTAL_STEPS
}

/// Evolution step through the microkernel. `pressure_milli` is thousandths of the oracle's
/// fractional pressure (1000 = the oracle's default 0.0011-style step), and `active_neurons` is the
/// caller's count, as on the python side.
pub fn route_neuro_evolve(pressure_milli: u64, active_neurons: usize) -> (u64, f64) {
    let (gen, rate, _, _) = unsafe { NEURO_ENGINE.evolve((pressure_milli as f64) / 1000.0, active_neurons) };
    (gen, rate)
}

// ── the continuum ledger's append rule (swarm_v2/core/neuro_continuum.py) ────────────────────────
//
// `append(event, metrics=None)` builds a block whose `index` is simply the CURRENT BLOCK COUNT (so
// the first appended block after the genesis pair is index 2), carries the engine's current `state`,
// attaches `metrics` only when one was passed, appends it and returns it. The timestamp is supplied
// by the caller here: the python calls `time.strftime`, and a firmware clock is not this module's.

/// One ledger block, as the python's dict shape carries it.
#[derive(Debug, Clone)]
pub struct Block {
    pub index: usize,
    pub timestamp: u64,
    pub event: String,
    pub state: String,
    pub metrics: Vec<(String, f64)>,
}

/// The ledger: blocks plus the state string each new block records.
#[derive(Debug, Clone)]
pub struct Ledger {
    pub blocks: Vec<Block>,
    pub state: String,
}

impl Ledger {
    pub fn new(state: &str) -> Self {
        Ledger { blocks: Vec::new(), state: String::from(state) }
    }

    /// `append`: the index is the count before the push, and the returned block is the one just added.
    pub fn append(&mut self, event: &str, metrics: Vec<(String, f64)>, timestamp: u64) -> Block {
        let block = Block {
            index: self.blocks.len(),
            timestamp,
            event: String::from(event),
            state: self.state.clone(),
            metrics,
        };
        self.blocks.push(block.clone());
        block
    }
}

/// `get_state`, named as the oracle names it: (generation, synapses, plasticity, fitness,
/// mutation_rate), the fields this firmware can carry.
pub fn get_state() -> (u64, u64, f64, f64, f64) {
    unsafe { (NEURO_ENGINE.generation, NEURO_ENGINE.synapses(), NEURO_ENGINE.plasticity(),
              round6(NEURO_ENGINE.fitness()), round6(NEURO_ENGINE.mutation_rate)) }
}

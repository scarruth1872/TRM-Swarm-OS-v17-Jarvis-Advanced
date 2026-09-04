//! TRM Swarm OS — sub_atomic_kernel (no_std port).
//! Sub-Atomic Silicon Kernel — physical substrate binding, deterministic
//! core from swarm_v2/core/sub_atomic_kernel.py:
//!   - SiliconMatrix: voltage rails (V_core=1.25, V_rail=4.75, V_ddr=1.35),
//!     clock lattice (f_cpu=4.0 GHz, T_cycle=1/f_cpu ns), quantum params
//!     (Phi_V=0.6089, Delta_0=0.001, epsilon=0.15) and derived quantities
//!     Delta_V = Delta_0*Phi_V, clock_cycles_per_step = Delta_V/T_cycle*1000;
//!   - recalibrate(): Phi_V = (V_core/1.25)^2 * (f_cpu/4.0) * (V_ddr/1.35);
//!   - K3TernaryGate collapse (ε thresholding → {-1,0,+1} with running
//!     BLOCK/ANALYZE/PROCEED counts) and energy_discharge mapping
//!     (+1→signal, 0→signal*0.5, -1→0).
//! The Mamba-3 SSM matrices (np.RandomState(42) A/B/C) are not portable →
//! honest `// MANUAL` skeleton, never invented.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// SiliconMatrix dataclass + __post_init__ derived parameters (python 1:1).
#[derive(Debug, Clone, Copy)]
pub struct SiliconMatrix {
    pub v_core: f64,
    pub v_rail: f64,
    pub v_ddr: f64,
    pub f_cpu: f64,
    pub t_cycle: f64, // ns per clock cycle (derived: 1.0/f_cpu)
    pub phi_v: f64,
    pub delta_0: f64,
    pub epsilon: f64,
    pub delta_v: f64,               // derived: Delta_0 * Phi_V
    pub clock_cycles_per_step: f64, // derived: Delta_V / T_cycle * 1000
}

impl SiliconMatrix {
    /// Dataclass defaults + __post_init__.
    pub fn new() -> Self {
        let mut s = SiliconMatrix {
            v_core: 1.25,
            v_rail: 4.75,
            v_ddr: 1.35,
            f_cpu: 4.0,
            t_cycle: 0.25,
            phi_v: 0.6089,
            delta_0: 0.001,
            epsilon: 0.15,
            delta_v: 0.0,
            clock_cycles_per_step: 0.0,
        };
        s.post_init();
        s
    }

    /// __post_init__: Delta_V = Delta_0 * Phi_V; T_cycle = 1/f_cpu;
    /// clock_cycles_per_step = Delta_V / T_cycle * 1000.
    pub fn post_init(&mut self) {
        self.delta_v = self.delta_0 * self.phi_v;
        self.t_cycle = 1.0 / self.f_cpu;
        self.clock_cycles_per_step = self.delta_v / self.t_cycle * 1000.0;
    }

    /// recalibrate(v_core?, f_cpu?): only truthy args update (python `if v_core:`).
    /// Phi_V = (V_core/1.25)^2 * (f_cpu/4.0) * (V_ddr/1.35). Returns Phi_V.
    pub fn recalibrate(&mut self, v_core: Option<f64>, f_cpu: Option<f64>) -> f64 {
        if let Some(v) = v_core {
            if v > 0.0 {
                self.v_core = v;
            }
        }
        if let Some(f) = f_cpu {
            if f > 0.0 {
                self.f_cpu = f;
            }
        }
        // (V_core/1.25)**2 — plain multiply, no pow() in no_std.
        let vc_ratio = self.v_core / 1.25;
        self.phi_v = vc_ratio * vc_ratio * (self.f_cpu / 4.0) * (self.v_ddr / 1.35);
        self.post_init();
        self.phi_v
    }
}

impl Default for SiliconMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// TERNARY_MAP: K3 ternary value → gate directive.
pub fn ternary_map_name(trit: i8) -> &'static str {
    match trit {
        -1 => "BLOCK",
        0 => "ANALYZE",
        1 => "PROCEED",
        _ => "ANALYZE",
    }
}

/// Collapse counts across all gate calls (python collapse_count dict).
#[derive(Debug, Clone, Copy, Default)]
pub struct CollapseCounts {
    pub proceed: i64,
    pub analyze: i64,
    pub block: i64,
}

/// K3TernaryGate at silicon gate level (python class).
#[derive(Debug, Clone)]
pub struct K3TernaryGate {
    pub epsilon: f64,
    pub collapse_count: CollapseCounts,
}

impl K3TernaryGate {
    pub fn new(epsilon: f64) -> Self {
        K3TernaryGate {
            epsilon,
            collapse_count: CollapseCounts {
                proceed: 0,
                analyze: 0,
                block: 0,
            },
        }
    }

    /// collapse(wavefunction): Φ(Ψ) = +1 if Re(Ψ) > ε, -1 if Re(Ψ) < -ε,
    /// else 0. Returns (trits, counts for this call).
    pub fn collapse(&mut self, wavefunction: &[f64]) -> (Vec<i8>, CollapseCounts) {
        let mut result: Vec<i8> = vec![0i8; wavefunction.len()];
        let mut counts = CollapseCounts::default();
        for (i, &v) in wavefunction.iter().enumerate() {
            if v > self.epsilon {
                result[i] = 1;
                counts.proceed += 1;
            } else if v < -self.epsilon {
                result[i] = -1;
                counts.block += 1;
            } else {
                result[i] = 0;
                counts.analyze += 1;
            }
        }
        self.collapse_count.proceed += counts.proceed;
        self.collapse_count.analyze += counts.analyze;
        self.collapse_count.block += counts.block;
        (result, counts)
    }

    /// energy_discharge(state, signal_strength=1.0):
    ///   +1 → signal_strength; 0 → signal_strength * 0.5; -1 → 0.0.
    pub fn energy_discharge(state: &[i8], signal_strength: f64) -> Vec<f64> {
        state
            .iter()
            .map(|&t| match t {
                1 => signal_strength,
                -1 => 0.0,
                _ => signal_strength * 0.5,
            })
            .collect()
    }

    /// python benchmark: dominant = max(counts, key=counts.get) → name.
    pub fn dominant(&self, counts: &CollapseCounts) -> &'static str {
        if counts.proceed >= counts.analyze && counts.proceed >= counts.block {
            "PROCEED"
        } else if counts.analyze >= counts.block {
            "ANALYZE"
        } else {
            "BLOCK"
        }
    }
}

/// SubAtomicMamba3Engine — MANUAL skeleton. A/B/C are np.RandomState(42)
/// matrices (state_dim=32 x 32/96) that cannot be reproduced in no_std;
/// step() therefore reports the deterministic silicon timing bookkeeping
/// only. No invented weight semantics.
#[derive(Debug, Clone, Copy)]
pub struct SubAtomicMamba3Engine {
    pub step_count: u64,
}

impl SubAtomicMamba3Engine {
    pub fn new() -> Self {
        SubAtomicMamba3Engine { step_count: 0 }
    }

    /// MANUAL: SSM update h = h·(I + A·dt) + B·x·dt requires RNG matrices.
    /// Returns the step count after incrementing (deterministic counter).
    pub fn step(&mut self) -> u64 {
        self.step_count += 1;
        self.step_count
    }
}

/// Default epsilon from SiliconMatrix (0.15) used by the python main gate.
pub const DEFAULT_EPSILON: f64 = 0.15;

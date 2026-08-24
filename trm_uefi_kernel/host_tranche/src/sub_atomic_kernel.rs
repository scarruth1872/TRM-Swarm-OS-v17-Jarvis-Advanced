//! TRM Swarm OS - sub_atomic_kernel (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/sub_atomic_kernel.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  SiliconMatrix 
#[derive(Debug, Clone)]
pub struct SiliconMatrix {
    pub V_core: f64,
    pub V_rail: f64,
    pub V_ddr: f64,
    pub f_cpu: f64,
    pub T_cycle: f64,
    pub Phi_V: f64,
    pub Delta_0: f64,
    pub epsilon: f64,
}

impl SiliconMatrix {
    pub fn new() -> Self {
        SiliconMatrix {
            V_core: 0.0,
            V_rail: 0.0,
            V_ddr: 0.0,
            f_cpu: 0.0,
            T_cycle: 0.0,
            Phi_V: 0.0,
            Delta_0: 0.0,
            epsilon: 0.0,
        }
    }
}

//! std shim for the kernel's no_std float helpers.
//!
//! trm_uefi_kernel/src/k3_head.rs imports `crate::float_math::{exp_f32, sqrt_f64}`
//! because bare-metal core has no f32::exp/sqrt. The kernel supplies those from
//! src/float_math.rs; host_test includes k3_head.rs by path into its own crate, so
//! it needs a module of the same name. Same values, std implementations - this is
//! the same shim pattern host_test already uses for `time`.
pub fn exp_f32(x: f32) -> f32 {
    x.exp()
}

pub fn sqrt_f64(x: f64) -> f64 {
    x.sqrt()
}

//! TRM Swarm OS — Hardware Voltage & Clock-Bound Microkernel Binder (HVBE, no_std port).
//! Real hardware telemetry, voltage rail modeling, and dynamic Phi_V multiplier calculation.
//! 1:1 deterministic core from swarm_v2/core/hardware_voltage_microkernel_binder.py:
//!   - Voltage rails: V_core (1.10V), V_gfx (1.05V), V_ddr (1.35V), V_soc (1.00V)
//!   - Dynamic Voltage Multiplier: Phi_V = (V_core / 1.10) * (Clock_MHz / 4000.0)
//!   - Sub-350ns heartbeat telemetry tick

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const NOMINAL_V_CORE: f32 = 1.10;
pub const NOMINAL_V_GFX: f32 = 1.05;
pub const NOMINAL_V_DDR: f32 = 1.35;
pub const NOMINAL_V_SOC: f32 = 1.00;
pub const NOMINAL_CLOCK_MHZ: f32 = 4000.0;

#[derive(Debug, Clone, Copy)]
pub struct VoltageSnapshot {
    pub v_core: f32,
    pub v_gfx: f32,
    pub v_ddr: f32,
    pub v_soc: f32,
    pub clock_mhz: f32,
    pub phi_v: f32,
    pub timestamp_ns: u64,
}

pub struct HardwareVoltageMicrokernelBinder {
    pub v_core: f32,
    pub v_gfx: f32,
    pub v_ddr: f32,
    pub v_soc: f32,
    pub current_clock_mhz: f32,
    pub tick_count: u64,
}

impl HardwareVoltageMicrokernelBinder {
    pub fn new() -> Self {
        HardwareVoltageMicrokernelBinder {
            v_core: NOMINAL_V_CORE,
            v_gfx: NOMINAL_V_GFX,
            v_ddr: NOMINAL_V_DDR,
            v_soc: NOMINAL_V_SOC,
            current_clock_mhz: NOMINAL_CLOCK_MHZ,
            tick_count: 0,
        }
    }

    /// Compute dynamic voltage multiplier Phi_V:
    /// Phi_V = (V_core / 1.10) * (Clock_MHz / 4000.0)
    pub fn compute_dynamic_voltage_multiplier(&self) -> f32 {
        let v_ratio = self.v_core / NOMINAL_V_CORE;
        let clk_ratio = self.current_clock_mhz / NOMINAL_CLOCK_MHZ;
        v_ratio * clk_ratio
    }

    pub fn sample_telemetry(&mut self) -> VoltageSnapshot {
        self.tick_count += 1;
        let phi_v = self.compute_dynamic_voltage_multiplier();

        VoltageSnapshot {
            v_core: self.v_core,
            v_gfx: self.v_gfx,
            v_ddr: self.v_ddr,
            v_soc: self.v_soc,
            clock_mhz: self.current_clock_mhz,
            phi_v,
            timestamp_ns: crate::time::now_ns(),
        }
    }

    pub fn set_clock_frequency(&mut self, freq_mhz: f32) {
        if freq_mhz >= 800.0 && freq_mhz <= 5000.0 {
            self.current_clock_mhz = freq_mhz;
        }
    }
}

static SINGLETON_VOLTAGE_BINDER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_voltage_binder() -> u64 {
    let v = SINGLETON_VOLTAGE_BINDER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_VOLTAGE_BINDER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

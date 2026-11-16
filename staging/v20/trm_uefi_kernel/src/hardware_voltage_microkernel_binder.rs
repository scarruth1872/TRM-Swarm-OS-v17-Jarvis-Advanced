//! TRM Swarm OS — Hardware Voltage & Clock-Bound Microkernel Binder (HVBE, no_std port).
//! Real hardware telemetry, voltage rail modeling, and dynamic Phi_V multiplier calculation.
//! 1:1 deterministic core from swarm_v2/core/hardware_voltage_microkernel_binder.py:
//!   - Voltage rails: V_core (1.20V), V_gfx (1.05V), V_ddr (1.20V), V_soc (1.10V) — total 4.55V
//!   - Dynamic Voltage Multiplier: Phi_V = (V_core / 1.20)^2 * (Clock_MHz / 3301.0)
//!                                     * (V_ddr / 1.20) * max(0.5, 1 - mem_pct / 200)
//!   - Sub-350ns heartbeat telemetry tick
//!
//! HOST-SIDE (not ported, by design): the telemetry probes themselves (psutil, WMIC,
//! nvidia-smi, Vulkan VRAM), wall-clock timing (time.perf_counter), datetime, subprocess,
//! the fractal-telemetry ingestion singleton, and the native strip DLL. The ported functions
//! take already-measured telemetry VALUES as parameters and return the deterministic math.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const NOMINAL_V_CORE: f32 = 1.20;
pub const NOMINAL_V_GFX: f32 = 1.05;
pub const NOMINAL_V_DDR: f32 = 1.20;
pub const NOMINAL_V_SOC: f32 = 1.10;
pub const NOMINAL_CLOCK_MHZ: f32 = 3301.0;

/// The voltage term's divisor. In the oracle this is the SAME value as `NOMINAL_V_CORE`
/// (V_core = 1.20 for the Ryzen 5900HX and the formula divides by 1.20); the constant is kept
/// separate to document that the rule's divisor is a fixed reference rather than "whatever the
/// rail happens to be".
pub const PHI_V_CORE_DIVISOR: f32 = 1.20;
/// The DDR term's divisor.
pub const PHI_V_DDR_DIVISOR: f32 = 1.20;
/// The memory-pressure divisor, and the floor the term saturates at.
pub const PHI_V_MEM_DIVISOR: f32 = 200.0;
pub const PHI_V_MEM_FLOOR: f32 = 0.5;
/// The oracle's telemetry cache interval (`cache_interval_s: float = 0.1`), in milliseconds.
pub const TELEMETRY_CACHE_MS: u64 = 100;

/// Phi_V for one sample, using the strip's arithmetic: the reciprocals are precomputed and multiplied,
/// which is not the same as dividing (f32 rounding differs in the last bit). The reference values
/// (nominal V_core, nominal clock, nominal V_ddr) are MACHINE-DERIVED and passed in, so the same
/// binary works on any host without recompiling; the `NOMINAL_*` consts are only the fallback.
pub fn phi_v(v_core: f32, freq_mhz: f32, v_ddr: f32, mem_pct: f32,
             ref_v_core: f32, ref_freq_mhz: f32, ref_v_ddr: f32) -> f32 {
    let inv_core_sq = 1.0f32 / (ref_v_core * ref_v_core);
    let inv_freq = 1.0f32 / ref_freq_mhz;
    let inv_ddr = 1.0f32 / ref_v_ddr;
    let inv_200 = 1.0f32 / PHI_V_MEM_DIVISOR;
    let phi_core = v_core * v_core * inv_core_sq;
    let phi_freq = freq_mhz * inv_freq;
    let phi_mem = v_ddr * inv_ddr;
    let phi_load = (1.0f32 - mem_pct * inv_200).max(PHI_V_MEM_FLOOR);
    phi_core * phi_freq * phi_mem * phi_load
}

/// The batch form the strip exposes, so a caller can score many samples.
pub fn phi_v_batch(vc: &[f32], freq: &[f32], vd: &[f32], mp: &[f32],
                   ref_v_core: f32, ref_freq_mhz: f32, ref_v_ddr: f32, out: &mut [f32]) -> usize {
    let n = vc.len().min(freq.len()).min(vd.len()).min(mp.len()).min(out.len());
    let inv_core_sq = 1.0f32 / (ref_v_core * ref_v_core);
    let inv_freq = 1.0f32 / ref_freq_mhz;
    let inv_ddr = 1.0f32 / ref_v_ddr;
    let inv_200 = 1.0f32 / PHI_V_MEM_DIVISOR;
    for i in 0..n {
        let phi_core = vc[i] * vc[i] * inv_core_sq;
        let phi_freq = freq[i] * inv_freq;
        let phi_mem = vd[i] * inv_ddr;
        let phi_load = (1.0f32 - mp[i] * inv_200).max(PHI_V_MEM_FLOOR);
        out[i] = phi_core * phi_freq * phi_mem * phi_load;
    }
    n
}

/// The oracle's cache rule: refresh only when the interval has elapsed.
pub fn should_refresh_cached(last_sample_ns: u64, now_ns: u64) -> bool {
    now_ns.saturating_sub(last_sample_ns) >= TELEMETRY_CACHE_MS * 1_000_000
}

/// `f32::round` without std: half away from zero (see `intent_buffer::round_f32`; bare metal has
/// no `round` intrinsic). Every caller passes a non-negative value (voltages, Phi_V, latencies),
/// so the truncation-toward-zero + 0.5 test is the full rule for the inputs that reach it.
#[inline]
fn round_half_away(v: f32) -> f32 {
    let f = v as i64 as f32; // truncate toward zero
    if v - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

/// `round(x, 4)` — Python `round` is half-to-even while this is half-away-from-zero; the two
/// differ only at an exact `.xxx5` boundary, which never occurs for these float inputs.
#[inline]
fn round4(x: f32) -> f32 {
    round_half_away(x * 10_000.0) / 10_000.0
}

/// `round(x, 3)` — see `round4`.
#[inline]
fn round3(x: f32) -> f32 {
    round_half_away(x * 1_000.0) / 1_000.0
}

/// `current_v_core` — the dynamic V_core grounding: use the measured voltage when it is present
/// and within [0.5, 2.5] V, else fall back to the nominal rail.
/// 1:1 with `HardwareVoltageMicrokernelBinder.current_v_core`.
pub fn effective_v_core(measured: Option<f32>, nominal_v_core: f32) -> f32 {
    match measured {
        Some(m) if m >= 0.5 && m <= 2.5 => m,
        _ => nominal_v_core,
    }
}

/// `current_v_gfx` — the dynamic V_gfx modulation. When the GPU probe is active, V_gfx is pulled
/// into [0.85, 1.15] V by the clock ratio (gpu_clock_mhz / 1400, clamped to [0.1, 1.0]);
/// otherwise the nominal V_gfx rail is returned unchanged.
/// 1:1 with `HardwareVoltageMicrokernelBinder.current_v_gfx`.
pub fn gfx_voltage(active: bool, gpu_clock_mhz: f32) -> f32 {
    if !active {
        return NOMINAL_V_GFX;
    }
    let clock_ratio = (gpu_clock_mhz / 1400.0).max(0.1).min(1.0);
    round4(0.85 + 0.30 * clock_ratio)
}

/// `dynamic_phi_v` — the display form of Phi_V, equal to raw (no arbitrary scaling).
/// 1:1 with the `dynamic_phi_v` field of `get_telemetry_snapshot`.
pub fn dynamic_phi_v(phi: f32) -> f32 {
    round4(phi)
}

/// `voltage_rails.total` — the summed voltage rail, rounded to 4 decimals.
/// 1:1 with the `total` field of `get_telemetry_snapshot`'s `voltage_rails`.
pub fn total_voltage_rail(v_core: f32, v_gfx: f32, v_ddr: f32, v_soc: f32) -> f32 {
    round4(v_core + v_gfx + v_ddr + v_soc)
}

/// The voltage proxy pushed into the fractal telemetry engine, in millivolts:
/// `v_core * 1000 * (1 + (cpu_percent / 100) * 0.05)`.
/// 1:1 with the `proxy_mv` expression in `get_telemetry_snapshot`.
pub fn voltage_proxy_mv(v_core: f32, cpu_percent: f32) -> f32 {
    v_core * 1000.0 * (1.0 + (cpu_percent / 100.0) * 0.05)
}

/// Decode a raw `Win32_Processor.CurrentVoltage` word: bits 0-6 are `voltage * 10` when bit 7
/// is clear, otherwise the whole word is `voltage * 10`.
/// 1:1 with the decode in `RealHardwareTelemetrySource.get_measured_cpu_voltage`.
pub fn decode_measured_cpu_voltage(raw: u32) -> f32 {
    if (raw & 0x80) == 0 {
        (raw & 0x7F) as f32 / 10.0
    } else {
        raw as f32 / 10.0
    }
}

/// `voltage_accelerated_step_latency_us` — the measured step latency divided by `phi_v * 8`,
/// rounded to 3 decimals. The MEASURED latency itself (`time.perf_counter`) is host-side; this
/// is the deterministic transform applied to it.
/// 1:1 with the field in `execute_voltage_bound_state_scan`.
pub fn voltage_accelerated_step_latency_us(t_elapsed_us: f32, phi_v: f32) -> f32 {
    round3(t_elapsed_us / (phi_v * 8.0))
}

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
    /// Memory pressure in percent: the rule's fifth term, previously not an input at all.
    pub mem_pct: f32,
    /// Machine-derived reference values for the Phi_V divisors (nominal V_core, nominal clock,
    /// nominal V_ddr). Set at init by the host's hardware probe; the `NOMINAL_*` consts are the
    /// fallback so the same binary works on any host without recompiling.
    pub ref_v_core: f32,
    pub ref_freq_mhz: f32,
    pub ref_v_ddr: f32,
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
            mem_pct: 0.0,
            ref_v_core: NOMINAL_V_CORE,
            ref_freq_mhz: NOMINAL_CLOCK_MHZ,
            ref_v_ddr: NOMINAL_V_DDR,
            tick_count: 0,
        }
    }

    /// Compute the dynamic voltage multiplier Phi_V -- the platform's rule, not this module's:
    ///
    /// ```text
    /// Phi_V = (V_core / ref_v_core)^2 * (f_mhz / ref_freq_mhz) * (V_ddr / ref_v_ddr)
    ///         * max(0.5, 1 - mem_pct/200)
    /// ```
    ///
    /// whose 1:1 source is `swarm_v2/core/hardware_voltage_microkernel_binder.py::native_evaluate_phi_v`
    /// and the strip it accelerates with, `strip_voltage_multiplier_evaluator`. The reference values
    /// are machine-derived (see `set_reference`); they default to this host's 5900HX nominals.
    pub fn compute_dynamic_voltage_multiplier(&self) -> f32 {
        phi_v(self.v_core, self.current_clock_mhz, self.v_ddr, self.mem_pct,
              self.ref_v_core, self.ref_freq_mhz, self.ref_v_ddr)
    }

    /// Set the machine-derived reference values (nominal V_core, nominal clock, nominal V_ddr),
    /// each sanity-clamped so a bad probe cannot poison the divisor.
    pub fn set_reference(&mut self, ref_v_core: f32, ref_freq_mhz: f32, ref_v_ddr: f32) {
        if ref_v_core >= 0.5 && ref_v_core <= 2.5 { self.ref_v_core = ref_v_core; }
        if ref_freq_mhz >= 800.0 && ref_freq_mhz <= 5000.0 { self.ref_freq_mhz = ref_freq_mhz; }
        if ref_v_ddr >= 0.8 && ref_v_ddr <= 1.8 { self.ref_v_ddr = ref_v_ddr; }
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

//! Hardware voltage doctrine — dynamic Phi_V from REAL telemetry.
//!
//! Phi_V = (V_core/1.25)^2 * (f_actual/4000) * (V_ddr/1.35) * max(0.5, 1 - mem%/200)
//!
//! The clock is read from the host (WMIC CurrentClockSpeed) and memory load from
//! the OS. Phi_V was previously a hardcoded constant (0.9115) while docstrings
//! claimed "hardware telemetry and Phi_V ratio computation" was happening. It is
//! now actually measured, and `measured` reports whether the reads succeeded so a
//! fallback is never mistaken for a real reading.

use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// NOMINAL rails. These are the DIVISORS in the formula, i.e. the reference the
// measured rail is expressed against. They are deliberately NOT the values used
// in the computation any more.
//
// WHY THIS CHANGED: V_CORE and V_DDR used to both BE the constants AND be the
// divisors, so phi_core = (1.25/1.25)^2 = 1.0 and phi_mem = 1.35/1.35 = 1.0
// IDENTICALLY. Two of the four terms were structurally inert - phi_v collapsed to
// 1 - mem%/200 and carried no voltage information at all, despite being named
// voltage_phi_v and serving voltage_rails. A constant equal to its own divisor
// can never be anything but 1.0.
pub const V_CORE_NOMINAL: f64 = 1.25;
pub const V_DDR_NOMINAL: f64 = 1.35;
pub const F_BASE_MHZ: f64 = 4000.0;

#[derive(Clone, Debug)]
pub struct VoltageSnapshot {
    pub phi_v: f64,
    pub clock_mhz: f64,
    pub memory_percent: f64,
    pub v_core: f64,
    pub v_ddr: f64,
    pub measured: bool,
    pub source: String,
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Actual CPU clock in MHz. WMIC can emit multiple rows and a header line, so the
/// first bare integer on its own line is taken.
fn read_clock_mhz() -> Option<f64> {
    // Same defect class as read_memory_percent below: `wmic` no longer exists on Windows 11, so the
    // clock read failed and the f_actual/4000 term in Phi_V was pinned at 1.0 — one of four terms
    // structurally inert, exactly the pattern the header warns about. Get-CimInstance is the
    // supported reader and is used for every other quantity in this module.
    let tries: [(&str, &[&str]); 2] = [
        ("powershell", &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_Processor | Select-Object -First 1 -Expand CurrentClockSpeed)"]),
        ("wmic", &["cpu", "get", "CurrentClockSpeed"]),
    ];
    for (cmd, args) in tries {
        let Some(text) = run(cmd, args) else { continue };
        for line in text.lines() {
            let t = line.trim();
            if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(v) = t.parse::<f64>() {
                    if v > 100.0 {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

fn read_memory_percent() -> Option<f64> {
    // MEASURED DEFECT 2026-11-06: this read `wmic OS get FreePhysicalMemory,...`.
    // wmic no longer exists on this Windows 11 host, so the read failed, sample() fell
    // back to mem = 0.0, and phi_v came out 0.6884 - the mem=0% solution - while the real
    // usage was 61.8%, where the same formula gives 0.4757. A missing tool quietly made
    // the substrate look 45% healthier than it is.
    //
    // Get-CimInstance is the supported replacement and is already used for the two rail
    // readers below.
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_OperatingSystem | Select-Object -First 1 FreePhysicalMemory,TotalVisibleMemorySize | ConvertTo-Csv -NoTypeInformation | Select-Object -Last 1)"],
    )?;
    let row = text.lines().find(|l| l.contains(','))?;
    let nums: Vec<f64> = row
        .replace('"', "")
        .split(',')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect();
    if nums.len() == 2 && nums[1] > 0.0 {
        let (free_kb, total_kb) = (nums[0], nums[1]);
        return Some(((total_kb - free_kb) / total_kb) * 100.0);
    }
    None
}

/// CPU load percent from the host. /system/resources published a literal 14.2 while the
/// machine sat at 61.8% memory; a status endpoint that invents its numbers is worse than
/// one that omits them.
pub fn read_cpu_percent() -> Option<f64> {
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_Processor | Select-Object -First 1 -Expand LoadPercentage)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(v) = t.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// L3 cache in KB from the host. Was hardcoded 32768 in main.rs while this 3900X
/// reports 65536 - half the real cache, published as fact on three endpoints.
pub fn read_l3_cache_kb() -> Option<u64> {
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_Processor | Select-Object -First 1 -Expand L3CacheSize)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(kb) = t.parse::<u64>() {
                if kb > 0 {
                    return Some(kb);
                }
            }
        }
    }
    None
}


// ---------------------------------------------------------------------------------------------
// HWiNFO64 shared memory (SM2).
//
// The rails used to come only from WMI, where `Win32_Processor.CurrentVoltage` is 11 - tenths of a
// volt, i.e. 1.1 V - which is so coarse that the v_core term of Phi_V could not move and a nominal
// rail was indistinguishable from a measured one. HWiNFO reads the actual SVI2 telemetry: on this
// host `Vcore` = 1.4080 V while the CPU reports `CPU Core Voltage (SVI2 TFN)` = 1.4062 V.
//
// Layout MEASURED against HWiNFO 8.60-6120 (element stride 460, not the documented 312): label at
// 12, unit at 268, value at 300. A missing mapping is reported, never worked around.
#[cfg(target_os = "windows")]
mod hwinfo_sm2 {
    use std::ffi::c_void;

    extern "system" {
        fn OpenFileMappingW(access: u32, inherit: i32, name: *const u16) -> *mut c_void;
        fn MapViewOfFile(h: *mut c_void, access: u32, hi: u32, lo: u32, len: usize) -> *mut c_void;
        fn UnmapViewOfFile(p: *const c_void) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }

    const SIGNATURE: u32 = 0x5369_5748; // 'HWiS'

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn rd_u32(b: &[u8], off: usize) -> u32 {
        let mut a = [0u8; 4];
        a.copy_from_slice(&b[off..off + 4]);
        u32::from_le_bytes(a)
    }

    fn rd_f64(b: &[u8], off: usize) -> f64 {
        let mut a = [0u8; 8];
        a.copy_from_slice(&b[off..off + 8]);
        f64::from_le_bytes(a)
    }

    fn cstr(b: &[u8], off: usize, len: usize) -> String {
        let end = (off + len).min(b.len());
        if off >= end {
            return String::new();
        }
        let sl = &b[off..end];
        let nul = sl.iter().position(|&x| x == 0).unwrap_or(sl.len());
        String::from_utf8_lossy(&sl[..nul]).trim().to_string()
    }

    /// (v_core, v_soc, v_gpu, readings_seen, error)
    pub fn rails() -> (Option<f64>, Option<f64>, Option<f64>, usize, Option<String>) {
        let name = wide("Global\\HWiNFO_SENS_SM2");
        unsafe {
            let h = OpenFileMappingW(0x0004, 0, name.as_ptr());
            if h.is_null() {
                return (
                    None,
                    None,
                    None,
                    0,
                    Some("HWiNFO shared memory not enabled (Settings -> Shared Memory Support)".to_string()),
                );
            }
            let view = MapViewOfFile(h, 0x0004, 0, 0, 0);
            if view.is_null() {
                CloseHandle(h);
                return (None, None, None, 0, Some("MapViewOfFile failed".to_string()));
            }
            let hdr = std::slice::from_raw_parts(view as *const u8, 44);
            let sig = rd_u32(hdr, 0);
            let off_r = rd_u32(hdr, 32) as usize;
            let size_r = rd_u32(hdr, 36) as usize;
            let num_r = rd_u32(hdr, 40) as usize;
            if sig != SIGNATURE || size_r < 320 || num_r == 0 {
                UnmapViewOfFile(view);
                CloseHandle(h);
                return (
                    None,
                    None,
                    None,
                    0,
                    Some(format!("unexpected HWiNFO layout (sig 0x{sig:08x}, stride {size_r})")),
                );
            }
            let need = off_r + size_r * num_r;
            let buf = std::slice::from_raw_parts(view as *const u8, need);
            let mut vcore = None;
            let mut vsoc = None;
            let mut vgpu = None;
            for i in 0..num_r {
                let b = &buf[off_r + i * size_r..off_r + (i + 1) * size_r];
                let unit = cstr(b, 268, 16);
                if unit != "V" {
                    continue;
                }
                let label = cstr(b, 12, 128);
                let val = rd_f64(b, 300);
                match label.as_str() {
                    "Vcore" => vcore = Some(val),
                    "SoC Voltage (SVI2 TFN)" => vsoc = Some(val),
                    "GPU Core Voltage (VDDCR_GFX)" => vgpu = Some(val),
                    _ => {}
                }
            }
            UnmapViewOfFile(view);
            CloseHandle(h);
            (vcore, vsoc, vgpu, num_r, None)
        }
    }
}

fn read_v_core() -> Option<f64> {
    read_v_core_src().map(|(v, _)| v)
}

/// The rails as the host actually reports them, with their provenance. `measured` is false, and
/// `v_core` null, whenever HWiNFO is not publishing - no nominal value is substituted.
pub fn rails_snapshot() -> serde_json::Value {
    #[cfg(target_os = "windows")]
    {
        let (vc, vs, vg, n, err) = hwinfo_sm2::rails();
        if vc.is_some() {
            serde_json::json!({
                "source": "HWiNFO64 shared memory (SM2)",
                "measured": true,
                "v_core": vc,
                "v_soc": vs,
                "v_gpu": vg,
                // HWiNFO's mainboard sensor set on this board exposes no DIMM rail, so v_ddr comes
                // from the SPD configuration (a configured value, not a live measurement) and is
                // labelled as such rather than being passed off as one more live rail.
                "v_ddr": read_v_ddr(),
                "v_ddr_source": "WMI Win32_PhysicalMemory.ConfiguredVoltage (SPD configured, not a live rail)",
                "v_core_source": read_v_core_src().map(|(_, s)| s),
                "readings": n,
                "labels": {
                    "v_core": "Vcore",
                    "v_soc": "SoC Voltage (SVI2 TFN)",
                    "v_gpu": "GPU Core Voltage (VDDCR_GFX)"
                },
            })
        } else {
            serde_json::json!({
                "source": "HWiNFO64 shared memory (SM2)",
                "measured": false,
                "v_core": serde_json::Value::Null,
                "error": err,
            })
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        serde_json::json!({"source": "unavailable", "measured": false, "v_core": serde_json::Value::Null})
    }
}

/// Actual V_core. HWiNFO's SVI2 rail first (fine-grained, live), then WMI as a coarse fallback -
/// `Win32_Processor.CurrentVoltage` is tenths of a volt and reported 11 here (=1.1 V) while the real
/// rail was 1.408 V, so WMI alone understates the v_core term of Phi_V by 28% squared.
/// (value, source) - the source travels with the number so a coarse WMI read can never be reported
/// as if it were HWiNFO's SVI2 rail.
fn read_v_core_src() -> Option<(f64, &'static str)> {
    #[cfg(target_os = "windows")]
    {
        let (vc, _, _, _, _) = hwinfo_sm2::rails();
        if let Some(v) = vc {
            if v > 0.0 {
                return Some((v, "HWiNFO64 SM2: Vcore (SVI2)"));
            }
        }
    }
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_Processor | Select-Object -First 1 -Expand CurrentVoltage)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(raw) = t.parse::<f64>() {
                if raw > 0.0 {
                    // tenths of a volt -> volts. Coarse: reported 11 (=1.1 V) while the live SVI2
                    // rail was 1.408 V, so this branch is labelled wherever it surfaces.
                    return Some((raw / 10.0, "WMI Win32_Processor.CurrentVoltage (tenths of a volt)"));
                }
            }
        }
    }
    None
}

/// Actual V_ddr, read from the host. Win32_PhysicalMemory.ConfiguredVoltage is in
/// MILLIVOLTS.
fn read_v_ddr() -> Option<f64> {
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_PhysicalMemory | Select-Object -First 1 -Expand ConfiguredVoltage)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(mv) = t.parse::<f64>() {
                if mv > 0.0 {
                    return Some(mv / 1000.0); // millivolts -> volts
                }
            }
        }
    }
    None
}

/// Phi_V from measured telemetry. Cacheable; the caller decides the interval.
pub fn sample() -> VoltageSnapshot {
    let clock = read_clock_mhz();
    let mem = read_memory_percent();
    let vc = read_v_core();
    let vd = read_v_ddr();

    // `measured` now requires ALL FOUR reads. It previously required only clock
    // and memory, so the two rail terms could be nominal constants while the
    // snapshot still reported measured=true.
    let measured = clock.is_some() && mem.is_some() && vc.is_some() && vd.is_some();

    let f_actual = clock.unwrap_or(F_BASE_MHZ);
    let mem_pct = mem.unwrap_or(0.0);
    // fall back to the NOMINAL only to keep the arithmetic defined; `measured`
    // above is false in that case, so a fallback is never presented as a reading.
    let v_core = vc.unwrap_or(V_CORE_NOMINAL);
    let v_ddr = vd.unwrap_or(V_DDR_NOMINAL);

    // The rail terms are now live: with the rails actually read, they stop being
    // (V/V)^2 = 1 and start carrying the substrate voltage into phi_v.
    let phi_core = (v_core / V_CORE_NOMINAL).powi(2);
    let phi_freq = f_actual / F_BASE_MHZ;
    let phi_mem = v_ddr / V_DDR_NOMINAL;
    let phi_load = (1.0 - (mem_pct / 200.0)).max(0.5);
    let phi_v = phi_core * phi_freq * phi_mem * phi_load;

    VoltageSnapshot {
        phi_v: (phi_v * 10000.0).round() / 10000.0,
        clock_mhz: f_actual,
        memory_percent: (mem_pct * 100.0).round() / 100.0,
        v_core,
        v_ddr,
        measured,
        source: if measured {
            "Get-CimInstance CurrentClockSpeed (wmic fallback) + OS memory + CurrentVoltage + ConfiguredVoltage".to_string()
        } else {
            "FALLBACK: one or more telemetry reads failed; values are nominal, NOT measured".to_string()
        },
    }
}

static CACHE: OnceLock<Mutex<(Instant, VoltageSnapshot)>> = OnceLock::new();

/// Initialize background sampler thread to refresh telemetry every 10 seconds without blocking HTTP workers
pub fn init_background_sampler() {
    CACHE.get_or_init(|| {
        let initial = sample();
        let m = Mutex::new((Instant::now(), initial));
        std::thread::spawn(|| {
            loop {
                std::thread::sleep(Duration::from_secs(10));
                let new_sample = sample();
                if let Some(cell) = CACHE.get() {
                    let mut guard = match cell.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    *guard = (Instant::now(), new_sample);
                }
            }
        });
        m
    });
}

/// Instantaneous non-blocking snapshot from cached memory
pub fn snapshot() -> VoltageSnapshot {
    init_background_sampler();
    let cell = CACHE.get().expect("Cache initialized");
    let guard = match cell.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    guard.1.clone()
}

pub fn phi_v() -> f64 {
    snapshot().phi_v
}

// ─────────────────────────────────────────────────────────────────────────────
// THE VOLTAGE GATE
//
// Before this, voltage was a REPORTED quantity: the rails were read and served,
// but nothing depended on them. The doctrine asks for more than a reading - it
// asks that execution be BOUND to substrate state. A number that is measured but
// that gates nothing is still decoration; the difference between measuring and
// binding is whether anything changes when the measurement does.
//
// Two independent ways the gate can be closed, and they are not the same thing:
//
//   1. NOT MEASURED - the host could not supply the rails. The gate closes and
//      refuses. This is the doctrine's actual teeth: it is not permitted to fall
//      back to a nominal constant and carry on, because a constant cannot degrade
//      and so cannot warn anyone.
//
//   2. MEASURED BUT BELOW THRESHOLD - the substrate is genuinely short of
//      headroom. The gate closes and says by how much.
//
// The threshold is stated here rather than hidden, and `gate()` returns the
// numbers it used so a caller can see the basis instead of trusting a boolean.
// ─────────────────────────────────────────────────────────────────────────────

/// phi_v below this closes the gate. 0.30 means the substrate is delivering under
/// 30% of nominal headroom once clock, both rails and memory load are combined.
/// Chosen as a floor, not tuned to make anything pass - at the values measured on
/// this host (0.39-0.58) the gate is OPEN, and it would take a real degradation
/// (a rail dropping, or heavy memory pressure) to close it.
pub const POLICY: &str = "deny-by-default: a gate whose substrate is unmeasured does not open";
pub const GATE_THRESHOLD_DEFAULT: f64 = 0.30;

/// The effective floor. Operators may raise it; it can only ever be RAISED
/// meaningfully, never set below zero, and an unparseable value falls back to
/// the default. Read once per gate evaluation so a restart is enough to change
/// it -- no recompile, and no way to disable the gate from a request.
pub fn gate_threshold() -> f64 {
    match std::env::var("TRM_VOLTAGE_GATE_THRESHOLD") {
        Ok(s) => s.trim().parse::<f64>().ok().filter(|v| *v > 0.0 && v.is_finite())
            .unwrap_or(GATE_THRESHOLD_DEFAULT),
        Err(_) => GATE_THRESHOLD_DEFAULT,
    }
}

#[derive(Debug)]
pub struct VoltageGate {
    pub open: bool,
    pub phi_v: f64,
    pub threshold: f64,
    pub measured: bool,
    pub policy: &'static str,
    pub reason: String,
}

/// Evaluate the gate from the CURRENT measured snapshot.
pub fn gate() -> VoltageGate {
    let s = snapshot();
    if !s.measured {
        return VoltageGate {
            open: false,
            phi_v: s.phi_v,
            threshold: gate_threshold(),
            measured: false,
            policy: POLICY,
            reason: "substrate state could not be measured; refusing rather than                      substituting a nominal constant (voltage doctrine)".to_string(),
        };
    }
    if s.phi_v < gate_threshold() {
        return VoltageGate {
            open: false,
            phi_v: s.phi_v,
            threshold: gate_threshold(),
            measured: true,
            policy: POLICY,
            reason: format!(
                "phi_v {:.4} is below the floor {:.2}: substrate headroom is short",
                s.phi_v, gate_threshold()
            ),
        };
    }
    VoltageGate {
        open: true,
        phi_v: s.phi_v,
        threshold: gate_threshold(),
        measured: true,
        policy: POLICY,
        reason: format!(
            "phi_v {:.4} at or above the floor {:.2}; substrate has headroom",
            s.phi_v, gate_threshold()
        ),
    }
}

/// The processor's rated maximum clock. On this host `CurrentClockSpeed` equals `MaxClockSpeed`
/// (both 4000 on a 3900X whose real boost is higher), i.e. WMI reports a STATIC value: the
/// `f_actual / 4000` term in Phi_V is structurally 1.0 here no matter which reader is used. Reporting
/// the maximum lets a caller tell "the clock matches the base by coincidence" from "the clock is
/// being measured".
pub fn max_clock_mhz() -> Option<f64> {
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_Processor | Select-Object -First 1 -Expand MaxClockSpeed)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(v) = t.parse::<f64>() {
                if v > 100.0 {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// `% Processor Performance` — the PDH counter that is supposed to scale the base clock to the
/// actual one. Read separately so the probe below can sample it repeatedly.
fn read_perf_percent() -> Option<f64> {
    let text = run(
        "powershell",
        &["-NoProfile", "-Command",
          "(Get-CimInstance Win32_PerfFormattedData_Counters_ProcessorInformation -Filter \"Name='_Total'\" | Select-Object -First 1 -Expand PercentProcessorPerformance)"],
    )?;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(v) = t.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// The result of actually sampling every clock source this host offers, rather than trusting the
/// first one that returns a number.
#[derive(Clone, Debug)]
pub struct ClockProbe {
    pub current_clock_mhz: Option<f64>,
    pub max_clock_mhz: Option<f64>,
    pub perf_percent_samples: Vec<f64>,
    /// True only when the performance counter moved by more than 5 % across the samples. A counter
    /// that reads 99-100 every time carries no frequency information even though it is nominally
    /// "live" — that band is the counter's own jitter.
    pub perf_varies: bool,
    pub perf_range_percent: f64,
    /// True when the reported current clock equals the rated maximum, i.e. WMI is static.
    pub clock_static: bool,
    /// True only when some source shows real variation — the condition under which the clock term
    /// of Phi_V means anything.
    pub frequency_signal: bool,
    pub power_plan: PowerPlan,
    /// Why the clock term does or does not carry information, in one string, so the route answers
    /// the question instead of leaving it to the reader.
    pub reason: String,
}

/// Sample both clock sources `samples` times and report what actually varies.
///
/// MEASURED 2026-11-12 on the 3900X: `CurrentClockSpeed` == `MaxClockSpeed` == 4000 (static), and
/// `PercentProcessorPerformance` read 99, 100, 100 at idle and 100 with sixteen threads spinning.
/// Neither source tracks frequency here, so the `f_actual / 4000` term in Phi_V is 1.0 on this host
/// by coincidence — a decorative multiplier, not a measurement. This probe exists so that statement
/// is reproducible instead of being a docstring claim.
pub fn probe_clock_sources(samples: usize) -> ClockProbe {
    let samples = samples.clamp(1, 10);
    let current = read_clock_mhz();
    let max = max_clock_mhz();
    let mut perf: Vec<f64> = Vec::new();
    for _ in 0..samples {
        if let Some(p) = read_perf_percent() {
            perf.push(p);
        }
    }
    // The counter jitters between 99 and 100 on a machine with 16 threads spinning, so a 0.5
    // threshold reports "varies" for pure read noise (1 % of 4000 MHz = 40 MHz). A real frequency
    // swing on this class of CPU (idle ~2.2 GHz to boost 4.6 GHz) is far larger than 5 %.
    let perf_range = if perf.len() >= 2 {
        let hi = perf.iter().cloned().fold(f64::MIN, f64::max);
        let lo = perf.iter().cloned().fold(f64::MAX, f64::min);
        hi - lo
    } else {
        0.0
    };
    let perf_varies = perf_range > 5.0;
    let clock_static = matches!((current, max), (Some(c), Some(m)) if (c - m).abs() < 1.0);
    let plan = read_power_plan();
    let frequency_signal = perf_varies && !clock_static;
    let reason = if !perf_varies && !clock_static {
        "the performance counter is flat and the WMI clock is static; cause unestablished".to_string()
    } else if !perf_varies {
        // REFUTED HYPOTHESIS, KEPT: the obvious explanation was that the power plan pins the clock,
        // because this host's plan sets minimum processor state to 100 % on AC. It was tested on
        // 2026-11-12 by duplicating the scheme, setting PROCTHROTTLEMIN to 5 %, activating it and
        // sampling: the reading stayed at 4000 MHz / 99-100 %. So the plan is a configuration fact,
        // not the cause, and `PercentProcessorPerformance` does not track clock on this host at all.
        format!(
            "clock sources report a constant {} MHz while the rated maximum is {}. Configuration: \
             plan '{}' sets minimum processor state to {:?} % on AC, so downclocking is not permitted. \
             MEASURED REFUTATION: dropping that minimum to 5 % on a duplicated plan did not change the \
             reading (still 4000 MHz, 99-100 %), so the plan is NOT the explanation and the counter does \
             not track clock here. Cause unestablished -- an MSR APERF/MPERF read (0xC00000E7 / 0xC00000E8) \
             over a known interval would settle it.",
            current.unwrap_or(0.0),
            max.map(|m| m.to_string()).unwrap_or_else(|| "unknown".to_string()),
            plan.scheme,
            plan.min_state_ac_percent.unwrap_or(0)
        )
    } else if clock_static {
        "the WMI current clock is static but the performance counter varies; use the counter"
            .to_string()
    } else {
        "a clock source varies; the f_actual/f_base term carries information".to_string()
    };
    ClockProbe {
        current_clock_mhz: current,
        max_clock_mhz: max,
        perf_percent_samples: perf,
        perf_range_percent: perf_range,
        perf_varies,
        clock_static,
        frequency_signal,
        power_plan: plan,
        reason,
    }
}

/// The active power plan's processor-state bounds. This is the missing half of the clock question:
/// a CPU whose minimum processor state is pinned at 100 % holds its nominal clock by POLICY, so a
/// frequency reading of "4000 of 4000" is a true statement about a pinned machine, not evidence that
/// the sensor is broken. Measured on this host: scheme "AMD Ryzen High Performance", min state
/// 0x64 = 100 % on AC (5 % on DC).
#[derive(Clone, Debug)]
pub struct PowerPlan {
    pub scheme: String,
    pub min_state_ac_percent: Option<u32>,
    pub max_state_ac_percent: Option<u32>,
    /// True when AC keeps the processor at or above its nominal state, i.e. the clock is held at
    /// reference and the `f_actual / f_base` term cannot move however it is measured.
    pub clock_pinned_by_policy: bool,
}

fn state_percent(setting: &str) -> Option<u32> {
    let text = run("powercfg", &["/query", "SCHEME_CURRENT", "SUB_PROCESSOR", setting])?;
    for line in text.lines() {
        if line.contains("Current AC Power Setting Index") {
            let hex = line.split(':').next_back()?.trim();
            let n = u32::from_str_radix(hex.trim_start_matches("0x"), 16).ok()?;
            return Some(n);
        }
    }
    None
}

pub fn read_power_plan() -> PowerPlan {
    let scheme = run("powercfg", &["/getactivescheme"])
        .and_then(|t| {
            t.split_once('(')
                .map(|(_, rest)| rest.trim_end_matches([')', '\n', '\r']).trim().to_string())
        })
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let min_ac = state_percent("PROCTHROTTLEMIN");
    let max_ac = state_percent("PROCTHROTTLEMAX");
    PowerPlan {
        scheme,
        min_state_ac_percent: min_ac,
        max_state_ac_percent: max_ac,
        clock_pinned_by_policy: matches!(min_ac, Some(m) if m >= 100),
    }
}

// ─── Third-party sensor feed (HWiNFO CSV) ───────────────────────────────────────────────────────
//
// Why this exists: every user-mode Windows API on this host reports a static 4000 MHz (see
// probe_clock_sources). The APERF/MPERF-derived *effective* clock — the number that actually moves —
// is ring-0, and HWiNFO's signed driver reads it. HWiNFO's CLI and shared memory are Pro-only, but
// its GUI logging writes a plain CSV, which this parses. The numbers below are therefore third-party
// measurements, not platform-native ones, and the payload says so.

#[derive(Clone, Debug, Default)]
pub struct HwinfoFeed {
    pub path: String,
    pub rows: usize,
    pub first_time: String,
    pub last_time: String,
    pub core_clock_mhz: Vec<f64>,
    pub effective_clock_mhz: Vec<f64>,
    pub vid_v: Vec<f64>,
    // doctrine terms and pinning evidence, when the CSV carries them
    pub core_ratio_x: Vec<f64>,
    pub bus_clock_mhz: Vec<f64>,
    pub svi2_vcore_v: Vec<f64>,
    pub package_power_w: Vec<f64>,
    pub tctl_c: Vec<f64>,
    pub c6_residency_pct: Vec<f64>,
    pub c0_residency_pct: Vec<f64>,
    pub freq_limit_mhz: Vec<f64>,
    pub core_current_a: Vec<f64>,
    // G83: the other clock/voltage domains, so the resonance grid is built from measurement rather than
    // from datasheet constants. Each stays empty when the CSV does not carry the sensor -- an empty vector
    // is the honest "unavailable", never a substituted nominal.
    pub soc_v_v: Vec<f64>,
    pub fclk_mhz: Vec<f64>,
    pub uclk_mhz: Vec<f64>,
    pub mem_clock_mhz: Vec<f64>,
    pub gpu_core_v_v: Vec<f64>,
    pub gpu_mem_v_v: Vec<f64>,
    pub gpu_soc_v_v: Vec<f64>,
    pub gpu_clock_mhz: Vec<f64>,
    pub gpu_eff_clock_mhz: Vec<f64>,
    pub gpu_mem_clock_mhz: Vec<f64>,
    pub gpu_power_w: Vec<f64>,
    pub gpu_ppt_limit_w: Vec<f64>,
    pub pcie_gt_s: Vec<f64>,
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        match ch {
            '"' => quoted = !quoted,
            ',' if !quoted => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn column(header: &[String], needle: &str) -> Option<usize> {
    header.iter().position(|h| h.contains(needle))
}

/// Parse an HWiNFO sensor CSV. Returns `None` when the file is missing or lacks the columns this
/// platform needs — a feed that cannot produce effective clocks is not a feed for this purpose.
pub fn read_hwinfo_csv(path: &str) -> Option<HwinfoFeed> {
    // NOT read_to_string: HWiNFO writes sensor names in the ANSI code page, so a degree sign or a
    // registered mark is a byte that is not valid UTF-8 and read_to_string rejects the WHOLE file.
    // Lossy decoding keeps the numeric columns intact, which is all this feed needs.
    let bytes = std::fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    let mut it = text.lines();
    let header = split_csv(it.next()?);
    let i_time = column(&header, "Time")?;
    let i_clk = column(&header, "Core Clocks (avg)")?;
    let i_eff = column(&header, "Core Effective Clocks (avg)")?;
    let i_vid = column(&header, "Core VIDs (avg)")?;
    // optional doctrine/pinning columns
    let i_ratio = column(&header, "Core Ratios (avg)");
    let i_bus = column(&header, "Bus Clock");
    let i_svi2 = column(&header, "CPU Core Voltage (SVI2 TFN)");
    let i_pkg = column(&header, "CPU Package Power");
    let i_tctl = column(&header, "CPU (Tctl/Tdie)");
    let i_c6 = column(&header, "Core C6 Residency (avg)");
    let i_c0 = column(&header, "Core C0 Residency (avg)");
    let i_flim = column(&header, "Frequency Limit - Global");
    let i_cur = column(&header, "CPU Core Current (SVI2 TFN)");
    // G83 domain columns (all optional: absent sensors stay empty)
    let i_soc_v = column(&header, "SoC Voltage (SVI2 TFN)");
    let i_fclk = column(&header, "Infinity Fabric Clock (FCLK)");
    let i_uclk = column(&header, "Memory Controller Clock (UCLK)");
    let i_mclk = column(&header, "Memory Clock [MHz]");
    let i_gfx_v = column(&header, "GPU Core Voltage (VDDCR_GFX)");
    let i_gfx_mem_v = column(&header, "GPU Memory Voltage (VDDIO)");
    let i_gfx_soc_v = column(&header, "GPU SoC Voltage (VDDCR_SOC)");
    let i_gpu_clk = column(&header, "GPU Clock [MHz]");
    let i_gpu_eff = column(&header, "GPU Clock (Effective)");
    let i_gpu_mem_clk = column(&header, "GPU Memory Clock");
    let i_gpu_pwr = column(&header, "GPU ASIC Power");
    let i_gpu_limit = column(&header, "GPU PPT Limit [W]");
    let i_pcie = column(&header, "PCIe Link Speed");
    let grab = |f: &Vec<String>, idx: Option<usize>, v: &mut Vec<f64>| {
        if let Some(i) = idx {
            if let Some(x) = f.get(i).and_then(|s| s.parse::<f64>().ok()) {
                v.push(x);
            }
        }
    };

    let mut feed = HwinfoFeed { path: path.to_string(), ..Default::default() };
    for line in it {
        if line.trim().is_empty() {
            continue;
        }
        let f = split_csv(line);
        if f.len() <= i_eff {
            continue;
        }
        let clk = f[i_clk].parse::<f64>().ok();
        let eff = f[i_eff].parse::<f64>();
        let vid = f.get(i_vid).and_then(|s| s.parse::<f64>().ok());
        let (eff_val, ok) = match eff {
            Ok(e) => (e, true),
            Err(_) => (0.0, false),
        };
        if let Some(c) = clk {
            feed.core_clock_mhz.push(c);
        }
        feed.effective_clock_mhz.push(eff_val);
        if let Some(x) = vid {
            feed.vid_v.push(x);
        }
        grab(&f, i_ratio, &mut feed.core_ratio_x);
        grab(&f, i_bus, &mut feed.bus_clock_mhz);
        grab(&f, i_svi2, &mut feed.svi2_vcore_v);
        grab(&f, i_pkg, &mut feed.package_power_w);
        grab(&f, i_tctl, &mut feed.tctl_c);
        grab(&f, i_c6, &mut feed.c6_residency_pct);
        grab(&f, i_c0, &mut feed.c0_residency_pct);
        grab(&f, i_flim, &mut feed.freq_limit_mhz);
        grab(&f, i_soc_v, &mut feed.soc_v_v);
        grab(&f, i_fclk, &mut feed.fclk_mhz);
        grab(&f, i_uclk, &mut feed.uclk_mhz);
        grab(&f, i_mclk, &mut feed.mem_clock_mhz);
        grab(&f, i_gfx_v, &mut feed.gpu_core_v_v);
        grab(&f, i_gfx_mem_v, &mut feed.gpu_mem_v_v);
        grab(&f, i_gfx_soc_v, &mut feed.gpu_soc_v_v);
        grab(&f, i_gpu_clk, &mut feed.gpu_clock_mhz);
        grab(&f, i_gpu_eff, &mut feed.gpu_eff_clock_mhz);
        grab(&f, i_gpu_mem_clk, &mut feed.gpu_mem_clock_mhz);
        grab(&f, i_gpu_pwr, &mut feed.gpu_power_w);
        grab(&f, i_gpu_limit, &mut feed.gpu_ppt_limit_w);
        grab(&f, i_pcie, &mut feed.pcie_gt_s);
        grab(&f, i_cur, &mut feed.core_current_a);
        if !ok {
            continue;
        }
        if feed.first_time.is_empty() {
            feed.first_time = f[i_time].clone();
        }
        feed.last_time = f[i_time].clone();
        feed.rows += 1;
    }
    if feed.rows == 0 {
        None
    } else {
        Some(feed)
    }
}

/// Default location the operator points at via `TRM_HWINFO_CSV`.
pub fn hwinfo_csv_path() -> Option<String> {
    std::env::var("TRM_HWINFO_CSV").ok().filter(|p| !p.trim().is_empty())
}

/// Phi_V recomputed with a SUPPLIED clock instead of the WMI one. The historical formula's clock
/// term is `f_actual / 4000`, which is 1.0 on this host because WMI reports a static clock. Fed the
/// APERF/MPERF effective clock it becomes a real ratio — but it now means something different:
/// effective clock is utilisation-weighted (a sleeping core contributes ~0), so this value describes
/// work delivered rather than rails held. Both are reported; neither replaces the other.
pub fn phi_v_at_clock(clock_mhz: f64) -> f64 {
    let snap = snapshot();
    let phi_core = (snap.v_core / V_CORE_NOMINAL).powi(2);
    let phi_ddr = snap.v_ddr / V_DDR_NOMINAL;
    let mem_factor = (1.0 - snap.memory_percent / 200.0).max(0.5);
    let phi_v = phi_core * (clock_mhz / F_BASE_MHZ) * phi_ddr * mem_factor;
    (phi_v * 10000.0).round() / 10000.0
}

/// Phi_V with SUPPLIED rails. The published `phi_v()` uses `v_core` from WMI's
/// `Win32_Processor.CurrentVoltage`, which on this host reports 1.1 V while the SVI2 telemetry in the
/// HWiNFO feed reports 1.26-1.30 V under the same load. Since Phi_V squares that term, feeding WMI's
/// value understates it by (1.28/1.1)^2 = 1.35x. This variant exists so the two can be compared
/// instead of one silently standing in for the other.
pub fn phi_v_with_rails(v_core: f64, v_ddr: f64, clock_mhz: f64) -> f64 {
    let snap = snapshot();
    let phi_core = (v_core / V_CORE_NOMINAL).powi(2);
    let phi_ddr = v_ddr / V_DDR_NOMINAL;
    let mem_factor = (1.0 - snap.memory_percent / 200.0).max(0.5);
    let phi_v = phi_core * (clock_mhz / F_BASE_MHZ) * phi_ddr * mem_factor;
    (phi_v * 10000.0).round() / 10000.0
}

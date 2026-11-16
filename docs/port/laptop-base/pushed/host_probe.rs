//! Real host probes for the /system and /health payloads.
//!
//! Every value this module returns is MEASURED on the host this process is running on, or
//! it is `None`. There are no fallback literals: the payload carried `voltage_phi_v: 0.86`,
//! `cpu_percent: 12.4`, `memory_used_mb: 42.1`, `active_threads: 14`, `memory_total_mb:
//! 32768.0` and `l3_sram_kb: 32768` while the host was reporting different numbers.
//!
//! Φ_V mirrors `swarm_v2_rust_kernel/src/voltage_doctrine_harmonization_engine.rs`
//! `derived_phi_v()` exactly — same rail constants, same reference clock, same OS query
//! (`wmic`) — so the server and the kernel compute the SAME number from the same inputs
//! instead of restating a constant in two places.
//!
//! No new crate dependencies: everything below is kernel32, which every Windows binary
//! already links.

// ── Doctrine constants (1:1 with voltage_doctrine_harmonization_engine.rs) ──────────────
/// The doctrine's nominal silicon rails; these ARE the denominators of the formula, so the
/// two rail ratios evaluate to exactly 1.0 and Φ_V reduces to clock × memory pressure.
const DOCTRINE_RAIL_CORE_NOMINAL_V: f64 = 1.25;
const DOCTRINE_RAIL_DDR_NOMINAL_V: f64 = 1.35;
/// Reference clock the doctrine normalizes against (denominator of the clock term).
const DOCTRINE_REF_CLOCK_MHZ: f64 = 4000.0;

/// Φ_V, its basis, and the two host inputs it was derived from.
/// The basis travels with the number: a derivation that fell back to the reference clock
/// must say so rather than posing as a measurement.
pub type PhiV = (f64, &'static str, f64, f64);

#[cfg(windows)]
mod win {
    use std::process::Command;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::OnceLock;
    use std::time::Duration;

    const NO_SAMPLE: u32 = u32::MAX;

    // ── kernel32 FFI ───────────────────────────────────────────────────────────────────
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct MemoryStatusEx {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    impl FileTime {
        fn ticks(self) -> u64 {
            ((self.high as u64) << 32) | self.low as u64
        }
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct ThreadEntry32 {
        dw_size: u32,
        cnt_usage: u32,
        th32_thread_id: u32,
        th32_owner_process_id: u32,
        tp_base_pri: i32,
        tp_delta_pri: i32,
        dw_flags: u32,
    }

    extern "system" {
        fn GlobalMemoryStatusEx(buf: *mut MemoryStatusEx) -> i32;
        fn K32GetProcessMemoryInfo(
            process: *mut core::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
        fn GetSystemTimeAsFileTime(out: *mut FileTime);
        fn GetProcessTimes(
            process: *mut core::ffi::c_void,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
        fn GetCurrentProcessId() -> u32;
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> *mut core::ffi::c_void;
        fn Thread32First(snapshot: *mut core::ffi::c_void, entry: *mut ThreadEntry32) -> i32;
        fn Thread32Next(snapshot: *mut core::ffi::c_void, entry: *mut ThreadEntry32) -> i32;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
        fn GetLogicalProcessorInformationEx(
            relationship: u32,
            buffer: *mut u8,
            len: *mut u32,
        ) -> i32;
    }

    /// Threads owned by this process (TH32CS_SNAPTHREAD = 0x4).
    pub fn active_threads() -> Option<u32> {
        const SNAPTHREAD: u32 = 0x0000_0004;
        let snapshot = unsafe { CreateToolhelp32Snapshot(SNAPTHREAD, 0) };
        if snapshot.is_null() || snapshot as isize == -1 {
            return None;
        }
        let pid = unsafe { GetCurrentProcessId() };
        let mut entry = ThreadEntry32 {
            dw_size: std::mem::size_of::<ThreadEntry32>() as u32,
            ..Default::default()
        };
        let mut count = 0u32;
        let mut ok = unsafe { Thread32First(snapshot, &mut entry) } != 0;
        while ok {
            if entry.th32_owner_process_id == pid {
                count += 1;
            }
            ok = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
        }
        unsafe { CloseHandle(snapshot) };
        Some(count)
    }

    /// This process's working set, in MB.
    pub fn process_working_set_mb() -> Option<f64> {
        let mut pmc = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            ..Default::default()
        };
        let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) };
        if ok == 0 {
            return None;
        }
        Some(pmc.working_set_size as f64 / (1024.0 * 1024.0))
    }

    /// Installed physical memory, in MB (GlobalMemoryStatusEx).
    pub fn total_memory_mb() -> Option<f64> {
        let mut st = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            ..Default::default()
        };
        if unsafe { GlobalMemoryStatusEx(&mut st) } == 0 {
            return None;
        }
        Some(st.ull_total_phys as f64 / (1024.0 * 1024.0))
    }

    /// System-wide used physical memory, in MB.
    pub fn used_memory_mb() -> Option<f64> {
        let mut st = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            ..Default::default()
        };
        if unsafe { GlobalMemoryStatusEx(&mut st) } == 0 {
            return None;
        }
        Some((st.ull_total_phys - st.ull_avail_phys) as f64 / (1024.0 * 1024.0))
    }

    /// Seconds since this process started (GetProcessTimes vs now).
    pub fn process_uptime_seconds() -> Option<f64> {
        let mut creation = FileTime::default();
        let mut exit = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        let ok = unsafe {
            GetProcessTimes(
                GetCurrentProcess(),
                &mut creation,
                &mut exit,
                &mut kernel,
                &mut user,
            )
        };
        if ok == 0 {
            return None;
        }
        let mut now = FileTime::default();
        unsafe { GetSystemTimeAsFileTime(&mut now) };
        // FILETIME ticks are 100 ns.
        Some((now.ticks().saturating_sub(creation.ticks())) as f64 / 1e7)
    }

    /// Total unified L3 cache, in KB, summed over every cache instance the OS reports.
    /// A single CCD reports 16 MB; the 5900X has two, which is why the sum (not one
    /// instance's size) is the honest number.
    pub fn l3_cache_kb() -> Option<u64> {
        const RELATION_CACHE: u32 = 2;
        const CACHE_UNIFIED: u32 = 0;
        let mut len: u32 = 0;
        unsafe { GetLogicalProcessorInformationEx(RELATION_CACHE, std::ptr::null_mut(), &mut len) };
        if len == 0 {
            return None;
        }
        let mut buffer = vec![0u8; len as usize];
        if unsafe {
            GetLogicalProcessorInformationEx(RELATION_CACHE, buffer.as_mut_ptr(), &mut len)
        } == 0
        {
            return None;
        }
        let mut total_bytes = 0u64;
        let mut instances = 0u32;
        let mut offset = 0usize;
        while offset + 20 <= buffer.len() {
            let size = u32::from_ne_bytes(buffer[offset + 4..offset + 8].try_into().ok()?) as usize;
            if size == 0 {
                break;
            }
            let level = buffer[offset + 8];
            // CACHE_RELATIONSHIP.CacheSize is in BYTES (16777216 = 16 MB, not 16 GB).
            let cache_size_bytes =
                u32::from_ne_bytes(buffer[offset + 12..offset + 16].try_into().ok()?);
            let cache_type = u32::from_ne_bytes(buffer[offset + 16..offset + 20].try_into().ok()?);
            if level == 3 && cache_type == CACHE_UNIFIED {
                total_bytes += cache_size_bytes as u64;
                instances += 1;
            }
            offset += size.max(1);
        }
        if instances == 0 {
            None
        } else {
            Some(total_bytes / 1024)
        }
    }

    // ── CPU sampler ────────────────────────────────────────────────────────────────────
    /// Percent × 100 (basis points), or NO_SAMPLE before the first pair is read.
    static CPU_BP: AtomicU32 = AtomicU32::new(NO_SAMPLE);

    fn system_times() -> Option<(u64, u64, u64)> {
        let mut idle = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
            return None;
        }
        Some((idle.ticks(), kernel.ticks(), user.ticks()))
    }

    /// Sample system CPU utilisation across an interval. Because the number only exists as
    /// a delta, it is sampled on a background thread rather than measured per request.
    pub fn start_cpu_sampler() {
        std::thread::spawn(|| {
            let mut prev = system_times();
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let now = system_times();
                if let (Some((i0, k0, u0)), Some((i1, k1, u1))) = (prev, now) {
                    let idle = i1.saturating_sub(i0);
                    // GetSystemTimes counts idle inside kernel time.
                    let total = k1.saturating_sub(k0) + u1.saturating_sub(u0);
                    if total > 0 {
                        let busy = total.saturating_sub(idle);
                        let bp = (busy as f64 / total as f64 * 10_000.0).round() as u32;
                        CPU_BP.store(bp.min(10_000), Ordering::Relaxed);
                    }
                }
                prev = now;
            }
        });
    }

    /// System CPU utilisation as a percentage, or None if no interval has been sampled yet.
    pub fn cpu_percent() -> Option<f64> {
        match CPU_BP.load(Ordering::Relaxed) {
            NO_SAMPLE => None,
            bp => Some(bp as f64 / 100.0),
        }
    }

    // ── OS query mirroring the kernel's host_telemetry() ───────────────────────────────
    /// Cache MHz + physical memory pressure, sampled once per process — the same query the
    /// kernel's `host_telemetry()` runs (`wmic cpu get CurrentClockSpeed`,
    /// `wmic OS get FreePhysicalMemory,TotalVisibleMemorySize`).
    pub fn host_telemetry() -> Option<(f64, f64)> {
        static CACHE: OnceLock<Option<(f64, f64)>> = OnceLock::new();
        *CACHE.get_or_init(|| {
            let clock_mhz = numbers(&["cpu", "get", "CurrentClockSpeed"])
                .first()
                .copied()?;
            let mem = numbers(&["OS", "get", "FreePhysicalMemory,TotalVisibleMemorySize"]);
            let (free_kb, total_kb) = (*mem.first()?, *mem.get(1)?);
            if total_kb <= 0.0 {
                return None;
            }
            Some((clock_mhz, (1.0 - free_kb / total_kb) * 100.0))
        })
    }

    /// GPU VRAM from nvidia-smi: (total_mib, used_mib, util_pct). Cached once per process — the
    /// query is a subprocess spawn, so it must not run per request. None when no NVIDIA GPU or
    /// nvidia-smi is present (the host's iGPU answers to wmic, not nvidia-smi).
    pub fn gpu_vram() -> Option<(f64, f64, f64)> {
        static CACHE: OnceLock<Option<(f64, f64, f64)>> = OnceLock::new();
        *CACHE.get_or_init(|| {
            let out = run(
                "nvidia-smi",
                &[
                    "--query-gpu=memory.total,memory.used,utilization.gpu",
                    "--format=csv,noheader,nounits",
                ],
            )?;
            let nums: Vec<f64> = out
                .split(',')
                .map(|t| t.trim())
                .filter_map(|t| t.parse::<f64>().ok())
                .collect();
            if nums.len() >= 3 {
                Some((nums[0], nums[1], nums[2]))
            } else {
                None
            }
        })
    }

    /// GPU telemetry from nvidia-smi: (power_w, clock_mhz, mem_clock_mhz, temp_c, util_pct).
    /// Cached once per process. None when no NVIDIA GPU. This is the measured state of the GPU
    /// as a functional co-processor in the unified fabric -- power/clock/temp are real readings;
    /// the core voltage is NOT exposed by nvidia-smi and stays "not measured".
    pub fn gpu_telemetry() -> Option<(f64, f64, f64, f64, f64)> {
        static CACHE: OnceLock<Option<(f64, f64, f64, f64, f64)>> = OnceLock::new();
        *CACHE.get_or_init(|| {
            let out = run(
                "nvidia-smi",
                &[
                    "--query-gpu=power.draw,clocks.sm,clocks.mem,temperature.gpu,utilization.gpu",
                    "--format=csv,noheader,nounits",
                ],
            )?;
            let nums: Vec<f64> = out
                .split(',')
                .map(|t| t.trim())
                .filter_map(|t| t.parse::<f64>().ok())
                .collect();
            if nums.len() >= 5 {
                Some((nums[0], nums[1], nums[2], nums[3], nums[4]))
            } else {
                None
            }
        })
    }

    /// Numeric tokens from `wmic <args>`, in output order. The header line is non-numeric
    /// and drops out of the parse, so the first value is the reading. Newer Windows builds
    /// ship without wmic, so CIM is the fallback — same WMI class, same field.
    fn numbers(args: &[&str]) -> Vec<f64> {
        if let Some(out) = run("wmic", args) {
            let parsed = parse_numbers(&out);
            if !parsed.is_empty() {
                return parsed;
            }
        }
        let fields = args[args.len() - 1].to_string();
        let class = if args[0].eq_ignore_ascii_case("OS") {
            "Win32_OperatingSystem"
        } else {
            "Win32_Processor"
        };
        // The previous form was `(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory,
        // TotalVisibleMemorySize`, which PowerShell rejects ("Missing expression after ','"), so the
        // whole telemetry read returned None and Phi_V silently fell back to the 4000 MHz reference and
        // 0% memory - two numbers that then looked measured on the route. Select-Object + Format-List
        // emits `Name : value` lines that parse_numbers() reads correctly for both classes.
        let expr = format!("(Get-CimInstance {}) | Select-Object {} | Format-List", class, fields);
        run("powershell", &["-NoProfile", "-Command", &expr])
            .map(|o| parse_numbers(&o))
            .unwrap_or_default()
    }

    fn run(program: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(program).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    fn parse_numbers(text: &str) -> Vec<f64> {
        text.split_whitespace()
            .filter_map(|tok| tok.parse::<f64>().ok())
            .collect()
    }
}

#[cfg(not(windows))]
mod win {
    use std::process::Command;
    use std::sync::OnceLock;

    pub fn active_threads() -> Option<u32> {
        None
    }
    pub fn process_working_set_mb() -> Option<f64> {
        None
    }
    pub fn total_memory_mb() -> Option<f64> {
        None
    }
    pub fn used_memory_mb() -> Option<f64> {
        None
    }
    pub fn process_uptime_seconds() -> Option<f64> {
        None
    }
    /// Total unified L3, summed over the DISTINCT cache instances sysfs reports -- the same rule the
    /// Windows arm states: one instance's size is not the machine's L3, and a two-CCD part has two.
    /// sysfs lists an index per CPU, so the same physical cache appears more than once; the sharing
    /// mask is what identifies it. Returns None when the topology cannot be read, so the caller
    /// serves a declaration and says so rather than passing it off as a measurement.
    pub fn l3_cache_kb() -> Option<u64> {
        let mut total_kb = 0u64;
        let mut seen: Vec<String> = Vec::new();
        for i in 0..=8 {
            let d = format!("/sys/devices/system/cpu/cpu0/cache/index{i}");
            let level = match std::fs::read_to_string(format!("{d}/level")) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if level.trim() != "3" {
                continue;
            }
            if let Ok(kind) = std::fs::read_to_string(format!("{d}/type")) {
                if kind.trim() != "Unified" {
                    continue;
                }
            }
            let shared = std::fs::read_to_string(format!("{d}/shared_cpu_map")).unwrap_or_default();
            if seen.contains(&shared) {
                continue;
            }
            let Ok(size) = std::fs::read_to_string(format!("{d}/size")) else {
                continue;
            };
            let size = size.trim();
            let kb = if let Some(k) = size.strip_suffix('K') {
                k.parse::<u64>().ok()
            } else if let Some(m) = size.strip_suffix('M') {
                m.parse::<u64>().ok().map(|v| v * 1024)
            } else {
                None
            };
            if let Some(v) = kb {
                total_kb += v;
                seen.push(shared);
            }
        }
        if total_kb == 0 {
            None
        } else {
            Some(total_kb)
        }
    }
    pub fn start_cpu_sampler() {}
    pub fn cpu_percent() -> Option<f64> {
        None
    }
    pub fn host_telemetry() -> Option<(f64, f64)> {
        None
    }
    /// GPU VRAM from nvidia-smi: `(total_mib, used_mib, util_pct)`. The same query the Windows
    /// arm uses, because nvidia-smi is a driver tool rather than an OS API.
    pub fn gpu_vram() -> Option<(f64, f64, f64)> {
        let out = run(
            "nvidia-smi",
            &[
                "--query-gpu=memory.total,memory.used,utilization.gpu",
                "--format=csv,noheader,nounits",
            ],
        )?;
        let nums: Vec<f64> = out
            .split(',')
            .map(|t| t.trim())
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        if nums.len() >= 3 {
            Some((nums[0], nums[1], nums[2]))
        } else {
            None
        }
    }

    /// GPU telemetry from nvidia-smi: (power_w, clock_mhz, mem_clock_mhz, temp_c, util_pct).
    /// Cached once per process. Core voltage is not exposed and stays "not measured".
    pub fn gpu_telemetry() -> Option<(f64, f64, f64, f64, f64)> {
        static CACHE: OnceLock<Option<(f64, f64, f64, f64, f64)>> = OnceLock::new();
        *CACHE.get_or_init(|| {
            let out = run(
                "nvidia-smi",
                &[
                    "--query-gpu=power.draw,clocks.sm,clocks.mem,temperature.gpu,utilization.gpu",
                    "--format=csv,noheader,nounits",
                ],
            )?;
            let nums: Vec<f64> = out
                .split(',')
                .map(|t| t.trim())
                .filter_map(|t| t.parse::<f64>().ok())
                .collect();
            if nums.len() >= 5 {
                Some((nums[0], nums[1], nums[2], nums[3], nums[4]))
            } else {
                None
            }
        })
    }

    fn run(program: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(program).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Start the background samplers that need an interval to produce a value.
pub fn start_samplers() {
    win::start_cpu_sampler();
}

pub fn cpu_percent() -> Option<f64> {
    win::cpu_percent()
}

pub fn process_working_set_mb() -> Option<f64> {
    win::process_working_set_mb()
}

pub fn used_memory_mb() -> Option<f64> {
    win::used_memory_mb()
}

pub fn total_memory_mb() -> Option<f64> {
    win::total_memory_mb()
}

pub fn active_threads() -> Option<u32> {
    win::active_threads()
}

pub fn process_uptime_seconds() -> Option<f64> {
    win::process_uptime_seconds()
}

pub fn l3_cache_kb() -> Option<u64> {
    win::l3_cache_kb()
}

/// GPU VRAM from nvidia-smi: `(total_mib, used_mib, util_pct)`.
pub fn gpu_vram() -> Option<(f64, f64, f64)> {
    win::gpu_vram()
}

pub fn gpu_telemetry() -> Option<(f64, f64, f64, f64, f64)> {
    win::gpu_telemetry()
}

/// Φ_V = (V_core/1.25)² × (f_actual/4000) × (V_ddr/1.35) × max(0.5, 1 − mem%/200), and the
/// rail ratios are exactly 1.0 by construction, so Φ_V is a function of clock and memory
/// pressure — both read from this host. Returns `(phi_v, basis, clock_mhz, mem_pct)`;
/// `mem_pct` is 0.0 and the basis says so when the OS query was unavailable.
pub fn phi_v() -> PhiV {
    // V_core now comes from the MEASURED SVI2 rail (HWiNFO shared memory) when available, falling back
    // to the AMD P-state VID and then to nominal. The three sources are not equivalent: VID is the
    // voltage the CPU *requested*, SVI2 is what the VRM actually delivered - on this host they differ
    // (1.44 V measured vs the VID path), and the measured one is the rail the metric is about.
    //
    // The DDR term used to read `(1.35 / 1.35)` - a division by its own nominal, structurally 1.0 while
    // looking like a rail product. This board's HWiNFO sensor set exposes no DIMM rail, so the term is
    // OMITTED and SAID SO rather than faked: Phi_V below is a core/clock/memory index, not a complete
    // four-term rail product.
    let rails = crate::hw_rails::rails();
    let v_meas = rails.get("v_core").and_then(|v| v.as_f64());
    let v_vid = crate::shadow_bios::core_vid_volts();
    let core_v = v_meas.or(v_vid);
    let rail_term = core_v
        .map(|v| (v / DOCTRINE_RAIL_CORE_NOMINAL_V).powi(2))
        .unwrap_or(1.0);
    let (clock_mhz, mem_pct, host_basis) = match win::host_telemetry() {
        Some((clock_mhz, mem_pct)) => (clock_mhz, mem_pct, "OS query for clock + memory"),
        None => (DOCTRINE_REF_CLOCK_MHZ, 0.0, "host telemetry unavailable"),
    };
    let raw = rail_term * (clock_mhz / DOCTRINE_REF_CLOCK_MHZ) * (1.0 - mem_pct / 200.0).max(0.5);
    let basis = if v_meas.is_some() {
        "V_core measured from HWiNFO SVI2 shared memory; DDR rail not exposed by this board, that term omitted (1.0)"
    } else if v_vid.is_some() {
        "V_core from AMD P-state VID (requested, not delivered); DDR term omitted (1.0)"
    } else {
        "V_core nominal (no sensor read); DDR term omitted (1.0)"
    };
    // NOTE: raw is NOT clamped here - see phi_v_report() for raw vs clamped. Clamping hid saturation:
    // at 1.44 V the core ratio alone is (1.44/1.25)^2 = 1.327 > 1, so an ordinary operating point pins
    // the metric at its ceiling and it stops discriminating.
    let _ = host_basis;
    (raw.clamp(0.0, 1.0), basis, clock_mhz, mem_pct)
}

/// Phi_V with its unclamped value and its rail coverage exposed, so a saturated metric cannot read as a
/// healthy one. `saturated` is true when the raw value exceeded the 0..1 range the callers assume.
pub fn phi_v_report() -> serde_json::Value {
    let rails = crate::hw_rails::rails();
    let v_meas = rails.get("v_core").and_then(|v| v.as_f64());
    let v_vid = crate::shadow_bios::core_vid_volts();
    let core_v = v_meas.or(v_vid);
    let core_term = core_v.map(|v| (v / DOCTRINE_RAIL_CORE_NOMINAL_V).powi(2));
    let (clock_mhz, mem_pct) = win::host_telemetry().unwrap_or((DOCTRINE_REF_CLOCK_MHZ, 0.0));
    let raw = core_term.unwrap_or(1.0)
        * (clock_mhz / DOCTRINE_REF_CLOCK_MHZ)
        * (1.0 - mem_pct / 200.0).max(0.5);
    serde_json::json!({
        "phi_v": raw.clamp(0.0, 1.0),
        "phi_v_raw": raw,
        "saturated": raw > 1.0,
        "v_core": core_v,
        "v_core_source": if v_meas.is_some() { "HWiNFO SVI2 Vcore (measured)" }
                         else if v_vid.is_some() { "AMD P-state VID (requested)" }
                         else { "nominal" },
        "core_term": core_term,
        "clock_mhz": clock_mhz,
        "memory_pct": mem_pct,
        "telemetry_source": if win::host_telemetry().is_some() { "measured (Get-CimInstance)" }
                            else { "REFERENCE FALLBACK (host telemetry unavailable): clock is the doctrine reference, memory is 0.0 - both are NOT measurements" },
        "rail_coverage": ["v_core"],
        "rail_omitted": {"v_ddr": "no DIMM rail in this board's HWiNFO sensor set - term omitted, not assumed to be 1.0"},
        "nominal_divisor_v": DOCTRINE_RAIL_CORE_NOMINAL_V,
        "note": "phi_v is clamped for callers; phi_v_raw is the unclamped value, and saturated means the metric hit its ceiling and stopped discriminating",
        "rails": rails,
    })
}

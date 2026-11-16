//! Shadow BIOS — read the firmware's own hardware inventory (SMBIOS/DMI) so the kernel
//! knows, before it boots, what the bare BIOS knows about the machine it is on.
//!
//! What is read: the SMBIOS/DMI tables the OS re-exposes (BIOS identity, system, baseboard,
//! processor, memory, GPU, L3 cache). What is NOT read — and is reported as such rather than
//! invented: the voltage rails. On this host `Win32_VoltageProbe` has no instances because the
//! rails live on the embedded controller / SMBus, which WMI does not expose.
//!
//! "Resident BIOS on L-cache" is a declared design intent, not a measured residency: the SMBIOS
//! shadow is a few KB, so it *can* sit in L3, but cache residency is hardware-managed and not
//! observable from userspace, so it is not claimed as a measurement.

use serde_json::{json, Value};

fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// One SMBIOS field via `wmic <class> get <field> /value` (each line is `Field=value`).
fn wmic_field(class: &str, field: &str) -> Option<String> {
    let out = run("wmic", &[class, "get", field, "/value"])?;
    for line in out.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=') {
            if k.trim().eq_ignore_ascii_case(field) && !v.trim().is_empty() {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

/// Whether the WMI voltage-probe class has any instances at all.
fn voltage_probe_count() -> usize {
    let out = run("wmic", &["path", "Win32_VoltageProbe", "get", "Reading"])
        .unwrap_or_default();
    out.lines().filter(|l| !l.trim().is_empty()).count().saturating_sub(1) // minus header
}

/// Real GPU substrate telemetry reachable from userspace (nvidia-smi): temperature, power,
/// and clocks. Voltage is NOT in nvidia-smi's query surface, so it stays out of this tuple.
fn gpu_thermal() -> Option<(f64, f64, f64, f64)> {
    let out = run(
        "nvidia-smi",
        &[
            "--query-gpu=temperature.gpu,power.draw,clocks.sm,clocks.mem",
            "--format=csv,noheader,nounits",
        ],
    )?;
    let nums: Vec<f64> = out
        .split(',')
        .map(|t| t.trim())
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    if nums.len() >= 4 {
        Some((nums[0], nums[1], nums[2], nums[3]))
    } else {
        None
    }
}

// ── AMD VID read (raw FFI) ────────────────────────────────────────────────────
// The Ryzen Master driver (AMDRyzenMasterDriverV20, AMD-signed, already loaded) exposes
// MSR reads via DeviceIoControl(0x81112EE0, input=[table index], output=16B). The handler
// stores the 64-bit MSR value as low32@offset4 + high32@offset8. The P-state MSRs hold the
// VID (voltage ID), decoded to volts as V = 1.55 - VID * 6.25 mV (Renesas/Intersil ISL62771
// SVI2 VRM datasheet: 0.5V-1.55V in 6.25mV steps, VID 0 = 1.55V).

#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: *mut core::ffi::c_void,
    object_name: *mut UnicodeString,
    attributes: u32,
    security_descriptor: *mut core::ffi::c_void,
    security_quality_of_service: *mut core::ffi::c_void,
}

#[repr(C)]
struct IoStatusBlock {
    status: i32,
    information: usize,
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn NtCreateFile(
        file_handle: *mut *mut core::ffi::c_void,
        desired_access: u32,
        object_attributes: *mut ObjectAttributes,
        io_status_block: *mut IoStatusBlock,
        allocation_size: *mut i64,
        file_attributes: u32,
        share_access: u32,
        create_disposition: u32,
        create_options: u32,
        ea_buffer: *mut core::ffi::c_void,
        ea_length: u32,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn DeviceIoControl(
        h_device: *mut core::ffi::c_void,
        io_control_code: u32,
        in_buffer: *mut core::ffi::c_void,
        in_buffer_size: u32,
        out_buffer: *mut core::ffi::c_void,
        out_buffer_size: u32,
        bytes_returned: *mut u32,
        overlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn CloseHandle(h_object: *mut core::ffi::c_void) -> i32;
}

#[cfg(windows)]
const IOCTL_MSR_READ: u32 = 0x8111_2EE0;

/// Read one MSR from the AMD driver's table (index 0..18) via NtCreateFile on the device +
/// DeviceIoControl. Returns the raw 64-bit MSR value.
#[cfg(windows)]
pub fn read_msr(index: u32) -> Option<u64> {
    unsafe {
        let mut name: Vec<u16> = "\\Device\\AMDRyzenMasterDriverV20"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let name_len_bytes = ((name.len() - 1) * 2) as u16; // wchar count excluding NUL
        let mut us = UnicodeString {
            length: name_len_bytes,
            maximum_length: (name.len() * 2) as u16,
            buffer: name.as_mut_ptr(),
        };
        let mut oa = ObjectAttributes {
            length: core::mem::size_of::<ObjectAttributes>() as u32,
            root_directory: std::ptr::null_mut(),
            object_name: &mut us,
            attributes: 0x40, // OBJ_CASE_INSENSITIVE
            security_descriptor: std::ptr::null_mut(),
            security_quality_of_service: std::ptr::null_mut(),
        };
        let mut handle: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut iosb = IoStatusBlock { status: 0, information: 0 };
        let st = NtCreateFile(
            &mut handle,
            0x1201_9F, // GENERIC_READ|WRITE|SYNCHRONIZE (proven against this device)
            &mut oa,
            &mut iosb,
            std::ptr::null_mut(),
            0,
            3, // FILE_SHARE_READ|WRITE
            1, // FILE_OPEN
            0,
            std::ptr::null_mut(),
            0,
        );
        if st != 0 || handle.is_null() {
            return None;
        }
        let mut in_buf: [u32; 4] = [index, 0, 0, 0];
        let mut out_buf: [u8; 16] = [0; 16];
        let mut returned: u32 = 0;
        let ok = DeviceIoControl(
            handle,
            IOCTL_MSR_READ,
            in_buf.as_mut_ptr() as *mut core::ffi::c_void,
            16,
            out_buf.as_mut_ptr() as *mut core::ffi::c_void,
            16,
            &mut returned,
            std::ptr::null_mut(),
        );
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        let lo = u32::from_le_bytes([out_buf[4], out_buf[5], out_buf[6], out_buf[7]]) as u64;
        let hi = u32::from_le_bytes([out_buf[8], out_buf[9], out_buf[10], out_buf[11]]) as u64;
        Some((hi << 32) | lo)
    }
}

/// VID (voltage ID, 7 bits) → volts, per the SVI2 VRM datasheet.
pub fn vid_to_volts(vid: u8) -> f64 {
    1.55 - vid as f64 * 0.00625
}

/// CPU core voltage (V_core) from the AMD P-state VID (PSTATE_0), read live via the
/// signed Ryzen Master driver. None when the read is unavailable (non-Windows, or the
/// driver isn't loaded).
pub fn core_vid_volts() -> Option<f64> {
    read_msr(3).map(|msr| vid_to_volts(((msr >> 16) & 0x7F) as u8))
}

#[cfg(not(windows))]
pub fn read_msr(_index: u32) -> Option<u64> {
    None
}

pub fn report() -> Value {
    let bios_mfr = wmic_field("bios", "Manufacturer");
    let bios_ver = wmic_field("bios", "SMBIOSBIOSVersion");
    let sys_mfr = wmic_field("computersystem", "Manufacturer");
    let sys_model = wmic_field("computersystem", "Model");
    let board_mfr = wmic_field("baseboard", "Manufacturer");
    let board_prod = wmic_field("baseboard", "Product");
    let cpu_name = wmic_field("cpu", "Name");
    let cpu_cores = wmic_field("cpu", "NumberOfCores");
    let mem_chips = run("wmic", &["memorychip", "get", "Capacity,Speed", "/value"]);
    let vprobe = voltage_probe_count();
    let thermal = gpu_thermal();

    // AMD P-state VID (voltage ID) targets, read live via the signed Ryzen Master driver.
    let pstate0 = read_msr(3); // PSTATE_0 (boost definition)
    let pstate1 = read_msr(4); // PSTATE_1
    let vid_of = |msr: Option<u64>| msr.map(|v| ((v >> 16) & 0x7F) as u8); // CpuVid bits 16-22
    let vid0 = vid_of(pstate0);
    let vid1 = vid_of(pstate1);
    let volts0 = vid0.map(vid_to_volts);
    let volts1 = vid1.map(vid_to_volts);

    json!({
        "bios": {
            "manufacturer": bios_mfr,
            "smbios_version": bios_ver,
            "basis": "SMBIOS/DMI, read live via WMI",
        },
        "system": {
            "manufacturer": sys_mfr,
            "model": sys_model,
            "basis": "SMBIOS Type 1 (system), read live via WMI",
        },
        "baseboard": {
            "manufacturer": board_mfr,
            "product": board_prod,
            "basis": "SMBIOS Type 2 (baseboard), read live via WMI",
        },
        "processor": {
            "name": cpu_name,
            "physical_cores": cpu_cores.and_then(|s| s.parse::<u32>().ok()),
            "basis": "SMBIOS Type 4 (processor), read live via WMI",
        },
        "memory": {
            "total_mb": crate::host_probe::total_memory_mb(),
            "modules": mem_chips,
            "basis": "SMBIOS Type 17 (memory device), read live via WMI",
        },
        "gpu": {
            "vram_total_mib": crate::host_probe::gpu_vram().map(|t| t.0),
            "temperature_c": thermal.map(|t| t.0),
            "power_draw_w": thermal.map(|t| t.1),
            "clock_sm_mhz": thermal.map(|t| t.2),
            "clock_mem_mhz": thermal.map(|t| t.3),
            "basis": "nvidia-smi, read live (temp/power/clocks; voltage is not in nvidia-smi's query surface)",
        },
        "l3_cache_kb": crate::host_probe::l3_cache_kb(),
        "voltages": {
            "probe_count": vprobe,
            "reference_voltage_v_p0": volts0,
            "reference_voltage_v_p1": volts1,
            "vid_p0": vid0,
            "vid_p1": vid1,
            "basis": "AMD P-state VID read live via AMDRyzenMasterDriverV20 (DeviceIoControl 0x81112EE0), decoded V = 1.55 - VID * 6.25 mV (ISL62771 SVI2 VRM datasheet). Requested target, NOT the measured rail.",
        },
        "shadow_residency": {
            "intent": "resident BIOS on L3 cache so the boot drive acts as the kernel with the firmware inventory pre-loaded",
            "honest_note": "declared design intent; cache residency is hardware-managed and not measurable from userspace, so it is not asserted as a measurement",
        },
    })
}

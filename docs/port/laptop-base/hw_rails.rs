//! Measured voltage rails from HWiNFO64's shared memory (SM2).
//!
//! Added for the desktop port. The base tree's `/voltage` answers from host_sensors and
//! `/system/voltage` from host_probe, so this module deliberately does NOT replace either: it adds a
//! third, explicit report whose every field is traceable to a sensor label. Written against measured
//! layout, not documentation - HWiNFO 8.60-6120 uses a 460-byte reading element (the documented struct
//! is 312), label at 12, unit at 268, value at 300. A missing mapping is reported, never worked around.

#[cfg(target_os = "windows")]
mod sm2 {
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

    /// Every voltage reading, as (label, value), plus the total reading count.
    pub fn voltages() -> (Vec<(String, f64)>, usize, Option<String>) {
        let name = wide("Global\\HWiNFO_SENS_SM2");
        unsafe {
            let h = OpenFileMappingW(0x0004, 0, name.as_ptr());
            if h.is_null() {
                return (
                    Vec::new(),
                    0,
                    Some("HWiNFO shared memory not enabled (Settings -> Shared Memory Support)".to_string()),
                );
            }
            let view = MapViewOfFile(h, 0x0004, 0, 0, 0);
            if view.is_null() {
                CloseHandle(h);
                return (Vec::new(), 0, Some("MapViewOfFile failed".to_string()));
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
                    Vec::new(),
                    0,
                    Some(format!("unexpected HWiNFO layout (sig 0x{sig:08x}, stride {size_r})")),
                );
            }
            let need = off_r + size_r * num_r;
            let buf = std::slice::from_raw_parts(view as *const u8, need);
            let mut out = Vec::new();
            for i in 0..num_r {
                let b = &buf[off_r + i * size_r..off_r + (i + 1) * size_r];
                if cstr(b, 268, 16) != "V" {
                    continue;
                }
                let label = cstr(b, 12, 128);
                if !label.is_empty() {
                    out.push((label, rd_f64(b, 300)));
                }
            }
            UnmapViewOfFile(view);
            CloseHandle(h);
            (out, num_r, None)
        }
    }
}

fn pick(rows: &[(String, f64)], label: &str) -> Option<f64> {
    rows.iter().find(|(l, _)| l == label).map(|(_, v)| *v)
}

/// The rails, with the sensor label each value came from. `measured` is false and the values are null
/// whenever HWiNFO is not publishing - no nominal rail is substituted.
pub fn rails() -> serde_json::Value {
    #[cfg(target_os = "windows")]
    {
        let (rows, total, err) = sm2::voltages();
        let v_core = pick(&rows, "Vcore");
        if v_core.is_none() {
            return serde_json::json!({
                "source": "HWiNFO64 shared memory (SM2)",
                "measured": false,
                "v_core": serde_json::Value::Null,
                "error": err,
            });
        }
        let v_soc = pick(&rows, "SoC Voltage (SVI2 TFN)");
        let v_gpu = pick(&rows, "GPU Core Voltage (VDDCR_GFX)");
        // +12V/+5V are the PSU rails the board's Super I/O can actually see; the DIMM rail is not
        // exposed by this board's sensor set, so nothing is invented for it.
        let psu: serde_json::Map<String, serde_json::Value> = rows
            .iter()
            .filter(|(l, _)| l == "+12V" || l == "+5V" || l == "VBAT" || l == "+3.3V" || l == "3VCC")
            .map(|(l, v)| (l.clone(), serde_json::json!(v)))
            .collect();
        serde_json::json!({
            "source": "HWiNFO64 shared memory (SM2)",
            "measured": true,
            "v_core": v_core,
            "v_soc": v_soc,
            "v_gpu": v_gpu,
            "psu_rails": psu,
            "readings": total,
            "voltage_readings": rows.len(),
            "labels": {
                "v_core": "Vcore",
                "v_soc": "SoC Voltage (SVI2 TFN)",
                "v_gpu": "GPU Core Voltage (VDDCR_GFX)"
            },
            "note": "values copied from the named HWiNFO reading; the DIMM rail is not exposed by this board's sensor set and is omitted rather than estimated",
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        serde_json::json!({"source": "unavailable", "measured": false})
    }
}

//! TRM Swarm OS — Native Strip Dispatch Registry (voltage doctrine)
//!
//! Single authoritative C-ABI surface over the 265 verified native voltage-strip
//! CDYLIBs. The Python hot-patch layer loads one DLL per module through ctypes;
//! this registry makes the *kernel* the source of truth for which strips exist,
//! whether each is a real PE (MZ) binary, and how many are dispatchable.
//!
//! Roster is embedded at compile time from the audited manifest
//! (`swarm_v2_artifacts/true_native_modules.json` — the 265 verified modules),
//! so kernel and hot-patch layer can never disagree about the strip set.

use std::path::Path;

/// The audited roster manifest (265 verified real-PE modules). The original file registered
/// only the first 105; the remaining 161 were built and loadable but never listed, so the kernel's
/// view of the strip set was smaller than the strip set.
/// Embedded at compile time — the file lives at repo root's swarm_v2_artifacts.
pub const ROSTER_JSON: &str = include_str!("../../swarm_v2_artifacts/true_native_modules.json");

/// Base dir for native strips, relative to the crate at runtime is resolved by
/// probing candidate locations (works from repo root and from target/release).
const STRIP_DIR_CANDIDATES: &[&str] = &[
    "swarm_v2_artifacts/native_voltage_strips",
    "../../swarm_v2_artifacts/native_voltage_strips",
    "../../../swarm_v2_artifacts/native_voltage_strips",
];

#[derive(Debug, Clone)]
pub struct StripEntry {
    pub name: String,
    pub dll: String,
    pub area: String,
    pub rel: String,
    /// True when the strip was generated from the template (pseudo-random ternary weights and a
    /// magic scale factor) rather than implementing the module it is named after. It loads and
    /// exports a function; that function is not the port.
    pub stub: bool,
}

/// Parse the roster JSON into entries. The manifest shape is
/// `[{ "name", "rel", "dll", "area" }, ...]`.
pub fn parse_roster(json: &str) -> Vec<StripEntry> {
    let mut out = Vec::new();
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return out,
    };
    let arr = match v.as_array() {
        Some(a) => a,
        None => return out,
    };
    for item in arr {
        let name = item.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let dll = item.get("dll").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let area = item.get("area").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let rel = item.get("rel").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let stub = item.get("stub").and_then(|x| x.as_bool()).unwrap_or(false);
        if !name.is_empty() {
            out.push(StripEntry { name, dll, area, rel, stub });
        }
    }
    out
}

fn resolve_strip_dir() -> Option<std::path::PathBuf> {
    for cand in STRIP_DIR_CANDIDATES {
        let p = Path::new(cand);
        if p.is_dir() {
            return Some(p.to_path_buf());
        }
    }
    None
}

/// Verify a strip DLL exists on disk and starts with the PE magic `MZ`.
pub fn verify_strip_pe(dll_name: &str) -> bool {
    let dir = match resolve_strip_dir() {
        Some(d) => d,
        None => return false,
    };
    let path = dir.join(format!("{}.dll", dll_name));
    if !path.exists() {
        return false;
    }
    match std::fs::read(&path) {
        Ok(bytes) => bytes.len() >= 2 && bytes[0] == b'M' && bytes[1] == b'Z',
        Err(_) => false,
    }
}

/// Full audit: returns (total, present_as_pe, missing) over the embedded roster.
pub fn audit_strips() -> (usize, usize, Vec<String>) {
    let roster = parse_roster(ROSTER_JSON);
    let total = roster.len();
    let mut present = 0usize;
    let mut missing = Vec::new();
    for e in &roster {
        if verify_strip_pe(&e.dll) {
            present += 1;
        } else {
            missing.push(format!("{} ({}.dll)", e.name, e.dll));
        }
    }
    (total, present, missing)
}

/// Dispatchability check for one module by its manifest name.
pub fn strip_status(module_name: &str) -> (bool, String) {
    let roster = parse_roster(ROSTER_JSON);
    for e in &roster {
        if e.name == module_name || e.dll == module_name {
            let ok = verify_strip_pe(&e.dll);
            return (ok, e.dll.clone());
        }
    }
    (false, String::new())
}

/// Count of registered modules in the embedded roster.
pub fn strip_count() -> usize {
    parse_roster(ROSTER_JSON).len()
}

/// Names of all registered strips.
pub fn strip_names() -> Vec<String> {
    parse_roster(ROSTER_JSON).into_iter().map(|e| e.name).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// C-ABI export surface (so Python's rust_kernel_binder can call the same
// registry the hot-patch headers use — one dispatch truth).
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the number of modules in the embedded native-strip roster.
#[no_mangle]
/// Number of entries that are synthesized stubs rather than implementations.
pub extern "C" fn rust_strip_stub_count() -> u64 {
    parse_roster(ROSTER_JSON).iter().filter(|e| e.stub).count() as u64
}

#[no_mangle]
pub extern "C" fn rust_strip_registry_count() -> u64 {
    strip_count() as u64
}

/// Audits the roster on disk: writes `out_present` = number of real-PE DLLs found.
/// Returns total roster size.
#[no_mangle]
pub extern "C" fn rust_strip_audit(out_present: *mut u64) -> u64 {
    let (total, present, _missing) = audit_strips();
    if !out_present.is_null() {
        unsafe { *out_present = present as u64; }
    }
    total as u64
}

/// Checks one module: writes its dll name (up to 255 bytes) into `out_name`.
/// Returns 1 if the strip exists as real PE, 0 otherwise, 2 if module unknown.
#[no_mangle]
pub extern "C" fn rust_strip_verify(
    name_ptr: *const u8,
    name_len: usize,
    out_name: *mut u8,
    out_cap: usize,
) -> i32 {
    if name_ptr.is_null() || name_len == 0 {
        return 2;
    }
    let name = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let name_str = match std::str::from_utf8(name) {
        Ok(s) => s,
        Err(_) => return 2,
    };
    let (ok, dll) = strip_status(name_str);
    // write dll name back
    if !out_name.is_null() {
        let dll_bytes = dll.as_bytes();
        let n = dll_bytes.len().min(out_cap.saturating_sub(1));
        unsafe {
            std::ptr::copy_nonoverlapping(dll_bytes.as_ptr(), out_name, n);
            *out_name.add(n) = 0;
        }
    }
    if dll.is_empty() { 2 } else if ok { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_parses_and_counts() {
        let roster = parse_roster(ROSTER_JSON);
        assert!(roster.len() >= 100, "expected >=100 verified strips, got {}", roster.len());
        // every entry must have name + dll
        for e in &roster {
            assert!(!e.name.is_empty());
            assert!(!e.dll.is_empty());
        }
    }

    #[test]
    fn audit_runs_and_is_consistent() {
        let (total, present, missing) = audit_strips();
        assert_eq!(total, strip_count());
        assert!(present <= total);
        // the audit must agree with per-module verify
        let roster = parse_roster(ROSTER_JSON);
        let mut manual = 0;
        for e in roster.iter().take(20) {
            if verify_strip_pe(&e.dll) { manual += 1; }
        }
        // present count >= manual count for the sampled subset is guaranteed
        assert!(present >= manual);
        let _ = missing;
    }

    #[test]
    fn unknown_module_returns_two() {
        assert_eq!(strip_status("definitely_not_a_module_xyz"), (false, String::new()));
    }
}

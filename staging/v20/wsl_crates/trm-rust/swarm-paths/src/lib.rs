//! swarm-paths — the one place that turns a path recorded on the WSL side into a path this host can
//! read.
//!
//! The crates were written on Linux, so their call sites carry absolute paths: `/mnt/c/...` for the
//! Windows checkout and `/home/<user>/...` for the WSL home. Re-pointing every literal by hand is how
//! they drift; resolving in one place is how they stay honest. Nothing here invents a path — a mapping
//! either lands somewhere real or `data()` says so, and a caller must report that rather than read it as
//! empty data.
//!
//! Overrides: `TRM_WSL_ROOT` (default `//wsl.localhost/<distro>`) and `TRM_WSL_DISTRO` (default Ubuntu).

use std::path::PathBuf;

pub fn wsl_root() -> String {
    if let Ok(r) = std::env::var("TRM_WSL_ROOT") {
        if !r.is_empty() {
            return r;
        }
    }
    let distro = std::env::var("TRM_WSL_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
    format!("//wsl.localhost/{distro}")
}

/// Map a recorded path onto this host. Shapes this host has no rule for pass through unchanged.
pub fn resolve(p: &str) -> String {
    let s = p.replace('\\', "/");
    if let Some(rest) = s.strip_prefix("/mnt/") {
        let mut it = rest.splitn(2, '/');
        let (drive, tail) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
        if drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()) {
            return format!("{}:/{}", drive.to_ascii_uppercase(), tail);
        }
    }
    if s.starts_with("/home/") || s == "/root" || s.starts_with("/root/") {
        return format!("{}{}", wsl_root(), s);
    }
    if s == "/tmp" || s.starts_with("/tmp/") {
        let tail = s.trim_start_matches("/tmp").trim_start_matches('/');
        let t = std::env::temp_dir();
        return if tail.is_empty() {
            t.display().to_string()
        } else {
            t.join(tail).display().to_string()
        };
    }
    s
}

pub fn path(p: &str) -> PathBuf {
    PathBuf::from(resolve(p))
}

/// The loadable kernel-ops library, as a String for `Library::new`. When this host has neither the
/// recorded .so nor a locally built cdylib, the RECORDED path is returned so the load fails loudly
/// instead of the gate silently comparing nothing.
pub fn kernel_ops_path() -> String {
    kernel_ops()
        .map(|(p, _)| p.display().to_string())
        .unwrap_or_else(|| resolve(RECORDED_KERNEL_SO))
}

/// Which kernel-ops library that is, and why: print this, never imply the C++ comparison happened.
pub fn kernel_ops_note() -> String {
    kernel_ops()
        .map(|(_, n)| n)
        .unwrap_or_else(|| "no loadable kernel ops on this host".to_string())
}

/// The recorded C++/AVX2 kernel ops: a LINUX shared object. Kept as the record of what the gate was
/// written against; on Windows it cannot be loaded at all, however the path is spelled.
pub const RECORDED_KERNEL_SO: &str =
    "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core/libswarm_kernel_ops.so";

/// The same C++ kernel built for THIS host. Read from its export table: it carries
/// `c_mamba3_create_state` / `c_mamba3_step` / `c_instella_gate_topk`, and NOT the AVX2 quantizer
/// (the recorded .so carries that). Measured against its own source in the tree, it is bit-for-bit the
/// documented 32-dim recurrence -- so for the mamba and instella gates this IS the reference.
pub const RECORDED_KERNEL_DLL: &str =
    "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core/libswarm_kernel_ops.dll";

/// The AVX2 quantizer's library. The Windows C++ build does NOT export `c_kleene_quantize_avx2`, so on
/// this host the only loadable one is the workspace cdylib -- whose quantizer was verified byte-identical
/// to the .so (MOTION A, 2026-08-14) -- while on unix the recorded .so is preferred, as that gate requires.
pub fn kernel_ops_avx2() -> Option<(PathBuf, String)> {
    if let Ok(p) = std::env::var("TRM_KERNEL_OPS") {
        if !p.is_empty() {
            let pb = PathBuf::from(&p);
            if pb.exists() {
                return Some((pb, "TRM_KERNEL_OPS".to_string()));
            }
        }
    }
    let recorded = path(RECORDED_KERNEL_SO);
    if cfg!(unix) && recorded.exists() {
        return Some((recorded, format!("the recorded C++ kernel: {RECORDED_KERNEL_SO}")));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["swarm_kernel.dll", "libswarm_kernel.so"] {
                let cand = dir.join(name);
                if cand.exists() {
                    return Some((
                        cand.clone(),
                        format!(
                            "this workspace's swarm-kernel cdylib: {} -- the platform's AVX2 port, NOT the                              recorded .so (the byte-identical gate for it ran on Linux)",
                            cand.display()
                        ),
                    ));
                }
            }
        }
    }
    None
}

/// `kernel_ops_avx2()` as a String, for `Library::new`.
pub fn kernel_ops_path_avx2() -> String {
    kernel_ops_avx2()
        .map(|(p, _)| p.display().to_string())
        .unwrap_or_else(|| resolve(RECORDED_KERNEL_SO))
}

/// The kernel-ops library this host can actually load, and where it came from.
///
/// This workspace's own `swarm-kernel` cdylib exports the same four entry points the verify gates bind
/// (`c_kleene_quantize_avx2`, `c_mamba3_create_state`, `c_mamba3_step`, `c_instella_gate_topk`), so on
/// Windows the gate compares the crate against the platform's kernel-ops build. The returned note says
/// which library it is, so a caller PRINTS the provenance instead of implying the C++ comparison
/// happened. `TRM_KERNEL_OPS` overrides both.
pub fn kernel_ops() -> Option<(PathBuf, String)> {
    if let Ok(p) = std::env::var("TRM_KERNEL_OPS") {
        if !p.is_empty() {
            let pb = PathBuf::from(&p);
            if pb.exists() {
                return Some((pb, "TRM_KERNEL_OPS".to_string()));
            }
        }
    }
    let native = path(RECORDED_KERNEL_DLL);
    if cfg!(windows) && native.exists() {
        return Some((
            native,
            format!("the recorded C++ kernel built for this host: {RECORDED_KERNEL_DLL}"),
        ));
    }
    let recorded = path(RECORDED_KERNEL_SO);
    if cfg!(unix) && recorded.exists() {
        return Some((recorded, format!("the recorded C++ kernel: {RECORDED_KERNEL_SO}")));
    }
    // Beside the running executable first: <workspace>/target/<profile>/<bin>.exe, so this resolves
    // whatever the caller's working directory happens to be.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["swarm_kernel.dll", "libswarm_kernel.so"] {
                let cand = dir.join(name);
                if cand.exists() {
                    return Some((
                        cand.clone(),
                        format!(
                            "this workspace's swarm-kernel cdylib: {} -- the platform's own kernel ops, \
                             NOT the recorded C++ original",
                            cand.display()
                        ),
                    ));
                }
            }
        }
    }
    None
}

/// The recorded Rust cdylib (the workspace's `swarm-kernel` build on the WSL side).
pub const RECORDED_CDYLIB: &str = "/home/shawn/trm-rust/target/release/libswarm_kernel.so";

/// This workspace's `swarm-kernel` cdylib, for `Library::new`.
///
/// `kernel_ops()` looks only BESIDE the running executable, which is not enough for a cargo example:
/// an example runs from `target/<profile>/examples/` while the cdylib is linked into
/// `target/<profile>/`. So walk up to three directories. When nothing is found the RECORDED path is
/// returned -- on this host that is the Linux `.so`, which cannot be loaded, so the load fails loudly
/// instead of the example silently comparing nothing. `TRM_KERNEL_CDYLIB` overrides.
pub fn cdylib() -> String {
    if let Ok(p) = std::env::var("TRM_KERNEL_CDYLIB") {
        if !p.is_empty() && PathBuf::from(&p).exists() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|d| d.to_path_buf());
        for _ in 0..3 {
            let Some(d) = dir else { break };
            for name in ["swarm_kernel.dll", "libswarm_kernel.so", "swarm_kernel.so"] {
                let cand = d.join(name);
                if cand.exists() {
                    return cand.display().to_string();
                }
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }
    resolve(RECORDED_CDYLIB)
}

/// Resolve a recorded path and confirm it is there. `None` means this host cannot see it: a caller must
/// report that, never treat it as empty.
pub fn data(p: &str) -> Option<PathBuf> {
    let pb = path(p);
    if pb.exists() {
        Some(pb)
    } else {
        None
    }
}

/// The recorded path and where it lands, for a receipt.
pub fn describe(p: &str) -> String {
    format!("{} <- {}", path(p).display(), p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnt_maps_onto_a_drive() {
        assert_eq!(resolve("/mnt/c/Users/shawn/x.bin"), "C:/Users/shawn/x.bin");
        assert_eq!(resolve("/mnt/d/data"), "D:/data");
    }

    #[test]
    fn home_maps_under_the_wsl_root() {
        let r = resolve("/home/shawn/.hermes/cache/ecosystem.k3");
        assert!(r.starts_with("//wsl.localhost/"), "{r}");
        assert!(r.ends_with("/home/shawn/.hermes/cache/ecosystem.k3"), "{r}");
    }

    #[test]
    fn tmp_maps_to_the_host_temp_dir() {
        let t = std::env::temp_dir().display().to_string();
        let r = resolve("/tmp/swarm_fit_test_booking.k3");
        assert!(r.starts_with(&t), "{r} does not start with {t}");
        assert!(r.ends_with("swarm_fit_test_booking.k3"));
    }

    #[test]
    fn unknown_shapes_pass_through() {
        assert_eq!(resolve("C:/already/native.bin"), "C:/already/native.bin");
        assert_eq!(resolve("relative/x.k3"), "relative/x.k3");
    }

    #[test]
    fn data_reports_absence_rather_than_emptiness() {
        assert!(data("/definitely/not/here.bin").is_none());
    }
}

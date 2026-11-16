//! Build provenance. A version string that is typed by hand cannot be trusted - the two kernels in
//! this federation already disagree about which is v20 (measured, G175). The commit is captured at
//! compile time instead, so any running binary can be traced to the source that produced it.
use std::process::Command;

fn main() {
    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=TRM_BUILD_COMMIT={sha}");
    println!("cargo:rerun-if-changed=build.rs");
}

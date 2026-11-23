//! Build provenance. A version string that is typed by hand cannot be trusted - the two kernels in
//! this federation already disagree about which is v20 (measured, G175). The commit is captured at
//! compile time instead, so any running binary can be traced to the source that produced it.
use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    // TWO DEFECTS FIXED HERE (G230), both found by rebuilding the kernel and watching /health still
    // report `9aabc88` while HEAD was `778526d`:
    //
    //   1. WATCHING `.git/HEAD` DOES NOT DETECT A COMMIT. On a normal branch `.git/HEAD` contains
    //      `ref: refs/heads/snapshot` and never changes when you commit — the SHA lives in
    //      `.git/refs/heads/<branch>` (or `.git/packed-refs`). So `rerun-if-changed` on HEAD only
    //      caught branch switches, Cargo reused the cached TRM_BUILD_COMMIT, and the binary kept
    //      advertising the commit it was first built from.
    //   2. A BINARY BUILT FROM UNCOMMITTED SOURCE CLAIMED A CLEAN COMMIT. That is worse than a stale
    //      number: it is a false provenance claim, and it hid defect 1 through a whole rebuild. The
    //      string now carries `+dirty` whenever tracked files differ from HEAD.
    let sha = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_string());

    // Tracked modifications only, and ONLY inside this crate. An untracked scratch file does not change
    // the compiled source; an edited .rs file does. Repo-wide would be wrong in the other direction:
    // unrelated edits elsewhere (the federation registry, a submodule pointer) do not change this
    // binary, and a flag that fires for them is a flag people learn to ignore — which would defeat the
    // whole point of the label. `-- .` runs from Cargo's package root, so it means exactly "this crate".
    let dirty = git(&["status", "--porcelain", "--untracked-files=no", "--", "."])
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let label = if dirty { format!("{sha}+dirty") } else { sha.clone() };

    println!("cargo:rustc-env=TRM_BUILD_COMMIT={label}");
    println!("cargo:rerun-if-changed=build.rs");

    // Defect 1: watch what actually carries the commit id.
    let gitdir = Path::new("../../.git");
    println!("cargo:rerun-if-changed={}", gitdir.join("HEAD").display());
    println!("cargo:rerun-if-changed={}", gitdir.join("packed-refs").display());
    if let Ok(head) = std::fs::read_to_string(gitdir.join("HEAD")) {
        if let Some(reference) = head.strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed={}", gitdir.join(reference.trim()).display());
        }
    }
    // And the source itself, so the edit that made a build dirty also invalidates it.
    println!("cargo:rerun-if-changed=src");
}

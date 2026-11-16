//! swarm-vitals — Motion C: the live fabric dashboard (native).
//!
//! A kernel-native view of the One's state, served over trm:// (no
//! HTTP in the core). Reads the real fabrics and reports:
//!   fabric trits · service verdicts · the family's records
//!   (jarvis/agentq/aura/hera7/voicebox/bonsai/embed + fitness)
//!
//! The dashboard is a trm:// command: `vitals` → the live state.

use swarm_nand::NandFabric;

pub mod helix;

/// The fabrics the vitals view reads (real paths).
pub const FABRICS: [(&str, &str); 4] = [
    ("ecosystem", "/home/shawn/.hermes/cache/ecosystem.k3"),
    ("fitness", "/home/shawn/.hermes/cache/inspired_fitness.k3"),
    ("jarvis", "/home/shawn/.hermes/cache/jarvis_fabric.k3"),
    ("abx", "/home/shawn/.hermes/cache/abx_2026.k3"),
];

/// One fabric's vitals: trit count + record estimate.
pub struct FabricVitals {
    pub name: &'static str,
    pub trits: u64,
    pub records: u64, // 22 trits per record (the standard record size)
}

/// Read the live vitals of every fabric.
pub fn vitals() -> Vec<FabricVitals> {
    FABRICS.iter().map(|(name, path)| {
        let (trits, records) = match NandFabric::open(&swarm_paths::resolve(path)) {
            Ok(fab) => {
                let t = fab.trit_count();
                (t, t / 22)
            }
            Err(_) => (0, 0),
        };
        FabricVitals { name, trits, records }
    }).collect()
}

/// Motion D: the FUSED helix view — one native call, the whole family's
/// live state: every strand + the fabrics + the ensemble, together.
pub fn fused_view() -> String {
    let mut out = swarm_vitals_crate_helix();
    let v = vitals();
    for f in &v {
        out.push_str(&format!("{}={},", f.name, f.trits));
    }
    out.push_str(&family_status(&v));
    out
}

fn swarm_vitals_crate_helix() -> String {
    helix::helix_view()
}

/// The family's live status (from the ecosystem fabric's record count).
pub fn family_status(v: &[FabricVitals]) -> String {
    let eco = v.iter().find(|f| f.name == "ecosystem").map(|f| f.records).unwrap_or(0);
    format!("{eco} family records in the fabric")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabrics_are_registered() {
        assert_eq!(FABRICS.len(), 4);
        assert!(FABRICS.iter().any(|(n, _)| *n == "ecosystem"));
        assert!(FABRICS.iter().any(|(n, _)| *n == "fitness"));
    }

    #[test]
    fn vitals_reads_real_fabrics() {
        let v = vitals();
        // the ecosystem fabric has real records (the family absorption)
        let eco = v.iter().find(|f| f.name == "ecosystem").unwrap();
        assert!(eco.trits > 0, "ecosystem fabric should have data");
    }

    #[test]
    fn family_status_reports() {
        let v = vitals();
        let s = family_status(&v);
        assert!(s.contains("family records"));
    }
}

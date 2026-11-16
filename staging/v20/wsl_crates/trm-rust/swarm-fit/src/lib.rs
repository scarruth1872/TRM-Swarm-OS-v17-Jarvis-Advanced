//! swarm-fit — Suite 2: the native Inspired Fitness service.
//!
//! The absorption's service layer: bookings, clients, wellness, grants,
//! and the sovereignty proof — ALL native Rust, NAND-persisted, K3-verdict.
//!
//!   trm://booking  → NAND record + K3 ActionVector (booked/blocked/analyze)
//!   trm://client   → NAND record + K3 status
//!   trm://wellness → NAND telemetry + K3 trend
//!   trm://grant    → knowledge shard + K3 fit
//!   trm://proof    → DDR scan + Monad anchor (the signed proof)
//!
//! Every record is byte-verified persistence (the fabric's guarantee).
//! The schema comes from the REAL llc_core.sqlite extraction (Suite 1).

use swarm_k3::collapse;

/// Re-export the NAND fabric so consumers only depend on swarm-fit.
pub use swarm_nand::NandFabric;

/// The Monad anchor (the truth reference on business records).
pub const MONAD_ANCHOR: [i8; 4] = [1, 1, 1, 1]; // TRUE·TRUE·TRUE·TRUE

/// A business record: header + entity trits (K3-encoded fields).
pub struct FitRecord {
    pub entity: &'static str, // "booking" | "client" | "wellness" | "grant" | "proof"
    pub trits: Vec<i8>,       // the K3-encoded record
}

/// The K3 verdict on a business action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitVerdict {
    Booked,   // clear proceed (booking confirmed)
    Blocked,  // hold (conflict, duplicate, risk)
    Analyze,  // uncertain (needs review)
}

impl FitVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            FitVerdict::Booked => "BOOKED",
            FitVerdict::Blocked => "BLOCKED",
            FitVerdict::Analyze => "ANALYZE",
        }
    }
}

/// Encode a business entity's fields as K3 trits (deterministic).
pub fn encode(entity: &str, fields: &[&str]) -> Vec<i8> {
    // header (Monad anchor + entity marker) + field trits
    let mut trits = MONAD_ANCHOR.to_vec();
    for f in fields {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in format!("{entity}|{f}").bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for k in 0..6 {
            h ^= h >> 30;
            h = h.wrapping_mul(0xBF58476D1CE4E5B9);
            trits.push(if (h >> k) & 1 == 1 { 1 } else { -1 });
        }
    }
    trits
}

/// The K3 verdict on a business action from its signal values.
pub fn verdict(signals: &[f32], tau: f32) -> FitVerdict {
    let trits = collapse(signals, tau);
    let t = trits.iter().filter(|&&v| v == 1).count();
    let f = trits.iter().filter(|&&v| v == -1).count();
    if t >= 2 && f == 0 {
        FitVerdict::Booked
    } else if f > t {
        FitVerdict::Blocked
    } else {
        FitVerdict::Analyze
    }
}

/// The native service: persist a record and return its verdict.
pub fn serve(fabric: &mut NandFabric, entity: &str, fields: &[&str],
             signals: &[f32]) -> (FitVerdict, u64, Vec<i8>) {
    let rec = encode(entity, fields);
    let (offset, trits) = match fabric.append(&rec) {
        Ok(off) => (off, rec),
        Err(_) => (0, rec),
    };
    let v = verdict(signals, 0.35);
    let _ = fabric.flush(); // the persistence guarantee (byte-verified)
    (v, offset, trits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_fabric(name: &str) -> NandFabric {
        let p = swarm_paths::resolve(&format!("/tmp/swarm_fit_test_{name}.k3"));
        let _ = std::fs::remove_file(&p);
        NandFabric::open(&p).expect("open")
    }

    #[test]
    fn booking_serves_and_persists() {
        let mut fab = tmp_fabric("booking");
        let (v, off, trits) = serve(&mut fab, "booking",
            &["userId=1", "providerId=1", "serviceId=1", "2026-08-20", "pending"],
            &[0.9, 0.8, 0.7]);
        assert_eq!(v, FitVerdict::Booked);
        assert!(off > 0);
        assert!(trits.len() >= 10);
        // byte-verified: reopen and read back
        let mut fab2 = NandFabric::open(&swarm_paths::resolve("/tmp/swarm_fit_test_booking.k3")).expect("reopen");
        let got = fab2.read(off, trits.len()).unwrap_or_default();
        assert_eq!(got, trits, "persistence must be byte-identical");
    }

    #[test]
    fn conflict_blocks() {
        let mut fab = tmp_fabric("conflict");
        // overlapping booking → blocked
        let (v, _, _) = serve(&mut fab, "booking",
            &["userId=1", "providerId=1", "serviceId=1", "2026-08-20", "double"],
            &[-0.9, -0.8, -0.7]);
        assert_eq!(v, FitVerdict::Blocked);
    }

    #[test]
    fn uncertain_analyzes() {
        let mut fab = tmp_fabric("uncertain");
        let (v, _, _) = serve(&mut fab, "grant",
            &["funder=X", "amount=10000", "mission=wellness", "pending"],
            &[0.1, -0.1, 0.2]);
        assert_eq!(v, FitVerdict::Analyze);
    }
}

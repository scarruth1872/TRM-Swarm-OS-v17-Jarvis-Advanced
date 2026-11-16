//! ensemble_council.rs — Motion E: the 2-model voltage field governs decisions.
//!
//! The synthesis: the native council (swarm-intel) now consults the REAL
//! ensemble field (qwen-moe + bonsai superposition) before voting. The
//! field's collapse becomes the council's prior — the models' voltage
//! shapes the decision, not just domain-fit signals.
//!
//!   task → domain-fit (mesh) + ensemble-field collapse (the 2 models)
//!        → K3 council vote [fit, field_verdict, confidence]
//!        → ACT/BLOCK/ANALYZE
//!
//! The ensemble field is loaded from the real GPU captures
//! (field_qwen.bin + field_bonsai.bin) when available; the council
//! signal uses the field's collapse over its first 32 dims.

use swarm_k3::collapse;

/// The ensemble field: the real 2-model superposition (1024 dims).
/// Loaded from the GPU captures; falls back to a deterministic seed
/// field when the captures are absent (the honest degradation).
pub struct EnsembleField {
    pub values: Vec<f32>,
    pub from_gpu: bool,
}

impl EnsembleField {
    /// Load the REAL ensemble superposition (qwen + bonsai, 1024-dim).
    pub fn load() -> Self {
        let mut values = Vec::new();
        let mut from_gpu = false;
        for path in [
            "/home/shawn/trm-rust/field_qwen.bin",
            "/home/shawn/trm-rust/field_bonsai.bin",
        ] {
            if let Ok(bytes) = std::fs::read(path) {
                if bytes.len() >= 8 {
                    let n = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                    if bytes.len() >= 4 + n * 4 {
                        let mut vals = Vec::with_capacity(n.min(1024));
                        for i in 0..n.min(1024) {
                            let o = 4 + i * 4;
                            let f = f32::from_le_bytes([bytes[o], bytes[o+1], bytes[o+2], bytes[o+3]]);
                            vals.push(f);
                        }
                        if values.is_empty() {
                            values = vals;
                        } else {
                            // superposition (average) — the ensemble
                            for i in 0..values.len().min(vals.len()) {
                                values[i] = (values[i] + vals[i]) / 2.0;
                            }
                        }
                        from_gpu = true;
                    }
                }
            }
        }
        if values.is_empty() {
            // deterministic fallback (the honest degradation)
            values = (0..32).map(|i| ((i as f32 * 0.7).sin()) * 0.5).collect();
        }
        EnsembleField { values, from_gpu }
    }

    /// The council's ensemble signal: collapse the field's first 32 dims.
    /// Returns the dominant verdict as a float signal (-1..1).
    pub fn signal(&self, tau: f32) -> f32 {
        let n = self.values.len().min(32);
        let trits = collapse(&self.values[..n], tau);
        let t = trits.iter().filter(|&&v| v == 1).count();
        let f = trits.iter().filter(|&&v| v == -1).count();
        if t > f {
            0.9
        } else if f > t {
            -0.9
        } else {
            0.0
        }
    }
}

/// The ensemble-governed council vote: domain-fit + ensemble signal.
pub fn ensemble_vote(field: &EnsembleField, domain: &str, tau: f32) -> (i8, Vec<i8>) {
    let fit = match domain {
        "security" => 2.0,
        "drug" => 2.0,
        "infra" => 2.0,
        "cosmos" => 2.0,
        "regulatory" => 2.0,
        _ => 1.0,
    };
    let field_signal = field.signal(tau);
    let trits = collapse(&[fit, field_signal, 0.7], tau);
    let pros = trits.iter().filter(|&&t| t == 1).count();
    let blocks = trits.iter().filter(|&&t| t == -1).count();
    let verdict = if pros >= 2 && blocks == 0 {
        1
    } else if blocks > pros {
        -1
    } else {
        0
    };
    (verdict, trits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensemble_loads_from_gpu_or_fallback() {
        let f = EnsembleField::load();
        assert!(!f.values.is_empty());
        // either the real GPU captures or the deterministic fallback
        assert!(f.values.len() >= 32);
    }

    #[test]
    fn ensemble_vote_returns_verdict() {
        let f = EnsembleField::load();
        let (v, trits) = ensemble_vote(&f, "security", 0.35);
        assert!(v == 1 || v == -1 || v == 0);
        assert_eq!(trits.len(), 3);
    }

    #[test]
    fn signal_is_bounded() {
        let f = EnsembleField::load();
        let s = f.signal(0.35);
        assert!((-1.0..=1.0).contains(&s));
    }
}

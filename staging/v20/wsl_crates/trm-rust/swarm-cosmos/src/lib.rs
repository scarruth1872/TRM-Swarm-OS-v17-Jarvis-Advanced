//! swarm-cosmos — Motion E: the cosmos field (THzGateway + Seeker).
//!
//! Ingests REAL spectral manifest data (Agent-Q's spectral_manifest,
//! 792-byte real records) into K3 trits — the cosmos knowledge field.

use std::fs;

/// One real spectral manifest record.
#[derive(Debug, Clone)]
pub struct SpectralRecord {
    pub id: String,
    pub kind: String,
    pub detail: String,
}

/// Parse the real spectral manifest JSONL (fields: timestamp, enclave,
/// density, permission, proof_hash — the real schema).
pub fn parse_manifest(path: &str) -> Vec<SpectralRecord> {
    let mut out = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return out,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // The real manifest is JSONL: {"timestamp": ..., "enclave": ..., "density": ..., ...}.
        // Parse as JSON rather than splitting on commas/colons (timestamps + nested action contain colons).
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let field = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let id = field("proof_hash");
        if id.is_empty() {
            continue;
        }
        out.push(SpectralRecord {
            id,
            kind: field("enclave"),
            detail: field("density"),
        });
    }
    out
}

/// Encode records to K3 trits (deterministic FNV-1a per record).
pub fn to_k3(records: &[SpectralRecord]) -> Vec<i8> {
    let mut trits = Vec::new();
    for r in records {
        let payload = format!("{}|{}|{}", r.id, r.kind, r.detail);
        let mut h: u64 = 0xcbf29ce484222325;
        for b in payload.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for k in 0..12 {
            trits.push(match (h >> (k * 2)) & 0b11 {
                0b11 => 1,
                0b00 => -1,
                _ => 0,
            });
        }
    }
    trits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_real_manifest() {
        let recs = parse_manifest(&swarm_paths::resolve("/home/shawn/Agent-Q-master/data/spectral_manifest/manifest_20260714.jsonl"));
        assert!(!recs.is_empty(), "real manifest has records");
    }

    #[test]
    fn k3_encode_is_deterministic() {
        let r = vec![SpectralRecord { id: "s1".into(), kind: "galaxy".into(), detail: "ngc123".into() }];
        let a = to_k3(&r);
        let b = to_k3(&r);
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
    }
}

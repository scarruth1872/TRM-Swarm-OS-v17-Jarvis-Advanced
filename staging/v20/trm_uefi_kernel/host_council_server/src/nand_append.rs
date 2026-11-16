//! `/k3/trit/append` — the server's guarded write into the canonical `.k3` fabric.
//!
//! MOTION-TRIT-FABRIC (15/15, ledger 540): the live council loop seals to the Monad ledger but
//! never the trit array, because the trit writer (`swarm_v2_rust_kernel/src/council.rs::write_trit`)
//! lives in a binary that is not this server. This route is the server's guarded write path.
//!
//! The store stays Rust-owned. The codec is the store's own (nand_persistence.rs): 2 trits per
//! byte, one 4-bit nibble per trit, 0b00 = -1 (FALSE), 0b01 = 0 (UNKNOWN), 0b11 = +1 (TRUE),
//! with 0b10 and every nibble above 0b11 illegal. The array is validated whole before any write
//! (a single illegal nibble refuses the write — a corrupt store is never written over), and every
//! trit is read back after, so the report states what was verified, not what was attempted.

use serde_json::{json, Value};

/// Count (illegal, scanned) over the whole store, mirroring nand_persistence::count_illegal_nibbles.
fn count_illegal(bytes: &[u8]) -> (usize, usize) {
    let mut illegal = 0usize;
    for b in bytes {
        for nib in [b & 0x0F, (b >> 4) & 0x0F] {
            if nib == 0b10 || nib > 0b11 {
                illegal += 1;
            }
        }
    }
    (illegal, bytes.len() * 2)
}

fn write_trit_at(bytes: &mut [u8], index: usize, trit_val: i8) -> bool {
    let byte_idx = index / 2;
    if byte_idx >= bytes.len() {
        return false;
    }
    let val = match trit_val {
        1 => 0b11u8,
        -1 => 0b00u8,
        _ => 0b01u8,
    };
    let is_high = (index % 2) == 1;
    let curr = bytes[byte_idx];
    bytes[byte_idx] = if is_high {
        (curr & 0x0F) | (val << 4)
    } else {
        (curr & 0xF0) | val
    };
    true
}

fn read_trit_at(bytes: &[u8], index: usize) -> Option<i8> {
    let byte_idx = index / 2;
    if byte_idx >= bytes.len() {
        return None;
    }
    let nibble = if (index % 2) == 1 {
        (bytes[byte_idx] >> 4) & 0x0F
    } else {
        bytes[byte_idx] & 0x0F
    };
    match nibble {
        0b11 => Some(1),
        0b00 => Some(-1),
        0b01 => Some(0),
        _ => None,
    }
}

/// Append one 16-trit decision (14 votes + consensus + seal parity) at `slot * 16`.
pub fn trit_append(slot: usize, votes: &[i8], consensus: bool, seal_bit: bool) -> Result<Value, String> {
    const TRITS_PER_DECISION: usize = 16;
    let path = crate::k3_retrieval::repo_relative("swarm_v2_nand_fabric.k3")
        .or_else(|| crate::k3_retrieval::repo_relative("swarm_v2_rust_kernel/swarm_v2_nand_fabric.k3"))
        .ok_or_else(|| "fabric store not found".to_string())?;

    // Read the whole store and validate it BEFORE touching it. A store that reads illegal is
    // refused — writing over it is the corruption MOTION-007 walled off.
    let mut bytes = std::fs::read(&path).map_err(|e| format!("cannot read fabric: {e}"))?;
    let (illegal, scanned) = count_illegal(&bytes);
    if illegal > 0 {
        return Err(format!(
            "fabric has {illegal} illegal nibbles of {scanned}; refusing to write a store that is not a clean K3 array"
        ));
    }

    let base = slot * TRITS_PER_DECISION;
    let capacity = bytes.len() * 2;
    if base + TRITS_PER_DECISION > capacity {
        return Err(format!(
            "slot {slot} (trits {base}..{}) exceeds fabric capacity {capacity}",
            base + TRITS_PER_DECISION
        ));
    }

    let mut wrote = 0usize;
    for (i, v) in votes.iter().take(14).enumerate() {
        if write_trit_at(&mut bytes, base + i, *v) {
            wrote += 1;
        }
    }
    if write_trit_at(&mut bytes, base + 14, if consensus { 1 } else { 0 }) {
        wrote += 1;
    }
    if write_trit_at(&mut bytes, base + 15, if seal_bit { 1 } else { 0 }) {
        wrote += 1;
    }

    std::fs::write(&path, &bytes).map_err(|e| format!("cannot write fabric: {e}"))?;

    // Read back (MOTION-007 Verify condition): count what was verified, not what was attempted.
    let after = std::fs::read(&path).map_err(|e| format!("cannot re-read fabric: {e}"))?;
    let mut verified = 0usize;
    for (i, v) in votes.iter().take(14).enumerate() {
        if read_trit_at(&after, base + i) == Some(*v) {
            verified += 1;
        }
    }
    if read_trit_at(&after, base + 14) == Some(if consensus { 1 } else { 0 }) {
        verified += 1;
    }
    if read_trit_at(&after, base + 15) == Some(if seal_bit { 1 } else { 0 }) {
        verified += 1;
    }

    Ok(json!({
        "status": "ok",
        "route": "/k3/trit/append",
        "slot": slot,
        "base_trit": base,
        "trits_written": wrote,
        "trits_verified": verified,
        "verified": wrote == verified,
        "fabric_bytes": after.len(),
        "monad": "0x4D4F4E4144000001",
    }))
}

/// POST /k3/trit/append body handler: {"slot":N,"votes":[...14 trits...],"consensus":bool,"seal_bit":bool}.
pub fn trit_append_from_body(body: &str) -> (Value, u16) {
    let v: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            return (
                json!({"status":"error","route":"/k3/trit/append","error":"invalid_body",
                       "detail":format!("expected JSON: {e}")}),
                400,
            )
        }
    };
    let slot = v.get("slot").and_then(|s| s.as_u64()).map(|s| s as usize);
    let votes: Vec<i8> = v
        .get("votes")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_i64())
                .map(|x| if x > 0 { 1 } else if x < 0 { -1 } else { 0 })
                .collect()
        })
        .unwrap_or_default();
    let consensus = v.get("consensus").and_then(|c| c.as_bool()).unwrap_or(false);
    let seal_bit = v.get("seal_bit").and_then(|c| c.as_bool()).unwrap_or(false);
    let slot = match slot {
        Some(s) => s,
        None => {
            return (
                json!({"status":"error","route":"/k3/trit/append","error":"missing_slot",
                       "detail":"body: {\"slot\":N,\"votes\":[...14 trits...],\"consensus\":bool,\"seal_bit\":bool}"}),
                400,
            )
        }
    };
    if votes.len() < 14 {
        return (
            json!({"status":"error","route":"/k3/trit/append","error":"missing_votes",
                   "detail":format!("need 14 vote trits, got {}", votes.len())}),
            400,
        );
    }
    match trit_append(slot, &votes, consensus, seal_bit) {
        Ok(payload) => (payload, 200),
        Err(e) => (
            json!({"status":"error","route":"/k3/trit/append","error":e}),
            400,
        ),
    }
}

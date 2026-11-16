//! TRM Swarm OS — Mutual Agent Trust (MAT) Protocol (real no_std port).
//! Ported 1:1 from swarm_v2/core/comm_protocol.py (Phase 5 MAT).
//!
//! Real HMAC-SHA256 trust-token generation / verification with the python
//! time-window semantics (300 s windows, verify checks current AND previous
//! window). Time is passed in explicitly (`now_unix_secs`) because a no_std
//! UEFI kernel has no std::time wall clock — the caller supplies the epoch
//! seconds from the UEFI runtime clock. SHA-256 is the crate's existing
//! FIPS 180-4 implementation in k3_logic.rs (no external crates, no std::hash).

use std::string::String;
use std::vec::Vec;

use crate::k3_logic::sha256;

/// Shared MAT secret — 1:1 from comm_protocol.py `SECRET_KEY`.
pub const SECRET_KEY: &str = "swarm_v5_autonomy_alpha_secret";
/// Default token window: python `int(time.time() / 300)` (5 minutes).
pub const TOKEN_WINDOW_SECS: u64 = 300;

/// Current time-window index — python: `int(now / 300)`.
pub fn time_window(now_unix_secs: u64) -> i64 {
    (now_unix_secs / TOKEN_WINDOW_SECS) as i64
}

/// Token payload — python: `f"{sender}->{target}:{task_id}:{window}"`.
/// Signed (i64) window so the verify's previous-window check reproduces
/// python's negative window at epoch start (time < 300 s).
fn payload(sender_role: &str, target_role: &str, task_id: &str, window: i64) -> String {
    format!("{}->{}:{}:{}", sender_role, target_role, task_id, window)
}

/// Constant-time byte comparison — python's `hmac.compare_digest`.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Lowercase hex digest — python's `hmac(...).hexdigest()`.
fn hexdigest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// HMAC-SHA256 (RFC 2104) over the crate's FIPS 180-4 SHA-256.
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    // Keys longer than one block are pre-hashed (RFC 2104 step 1).
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        let h = sha256(key);
        k[..32].copy_from_slice(&h);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    // inner = sha256(ipad_key || data)
    let mut inner = vec![0u8; BLOCK + data.len()];
    inner[..BLOCK].copy_from_slice(&ipad);
    inner[BLOCK..].copy_from_slice(data);
    let inner_hash = sha256(&inner);

    // outer = sha256(opad_key || inner)
    let mut outer = [0u8; BLOCK + 32];
    outer[..BLOCK].copy_from_slice(&opad);
    outer[BLOCK..].copy_from_slice(&inner_hash);
    sha256(&outer)
}

/// Generate a task-scoped, time-windowed trust token — python
/// `generate_trust_token(sender_role, target_role, task_id)` with the wall
/// clock supplied by the caller. Returns the 64-char lowercase HMAC hex.
pub fn generate_trust_token(
    sender_role: &str,
    target_role: &str,
    task_id: &str,
    now_unix_secs: u64,
) -> String {
    let p = payload(sender_role, target_role, task_id, time_window(now_unix_secs));
    hexdigest(&hmac_sha256(SECRET_KEY.as_bytes(), p.as_bytes()))
}

/// HMAC hex for an explicit window (testability / precomputed windows).
pub fn trust_token_for_window(
    sender_role: &str,
    target_role: &str,
    task_id: &str,
    window: i64,
) -> String {
    let p = payload(sender_role, target_role, task_id, window);
    hexdigest(&hmac_sha256(SECRET_KEY.as_bytes(), p.as_bytes()))
}

/// Verify an incoming token against the current OR previous time window —
/// python `verify_trust_token`. Rejects mismatched length / content in
/// constant time.
pub fn verify_trust_token(
    sender_role: &str,
    target_role: &str,
    task_id: &str,
    token: &str,
    now_unix_secs: u64,
) -> bool {
    let cur = time_window(now_unix_secs);
    for window in [cur, cur - 1] {
        let expected = trust_token_for_window(sender_role, target_role, task_id, window);
        if constant_time_eq(expected.as_bytes(), token.as_bytes()) {
            return true;
        }
    }
    false
}

/// Validate a token given an explicit expected token string produced for a
/// caller-supplied window list (helper mirroring the two-window loop).
pub fn verify_trust_token_windows(
    sender_role: &str,
    target_role: &str,
    task_id: &str,
    token: &str,
    windows: &[i64],
) -> bool {
    for window in windows {
        let expected = trust_token_for_window(sender_role, target_role, task_id, *window);
        if constant_time_eq(expected.as_bytes(), token.as_bytes()) {
            return true;
        }
    }
    false
}

// NOTE (MANUAL): python's `AgentHandshake.create_secure_payload` /
// `validate_secure_payload` wrap the token in a JSON dict
// {header:{sender,target,task_id,trust_token,protocol:"MAT/1.0"}, content}.
// JSON dict serialization is not ported here — bridge.rs already performs the
// crate's minimal no_std JSON framing; token generation/verification above is
// the complete cryptographic core and is 1:1 with python.

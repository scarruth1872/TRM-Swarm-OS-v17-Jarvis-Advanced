//! TRM Swarm OS - external_socket_bridge (no_std port).
//!
//! Portable half of `swarm_v2/core/external_socket_bridge.py`: the stream framing rule and the strip's
//! ternary kernel, ported from `native_voltage_strips/external_socket_bridge.rs` (weights extracted
//! from that source rather than retyped).
//!
//! Host-side and not ported: the socket listener, the 10-second timeout, the sub-agent spawner and
//! the JSON routing around them.

extern crate alloc;
use alloc::string::String;

/// The strip's scale, extracted from its source.
pub const SCALE_FACTOR: f32 = 31.9732;

/// The strip's 256-entry ternary weight vector, extracted from its source.
pub const TRIT_WEIGHTS: [i8; 256] = [
    -1, 1, 0, 1, -1, -1, 1, 1, 1, 0, -1, 1, -1, -1, 1, -1, -1, -1, -1, -1, 0, -1, 1, 0, 0, -1, 1, -1, -1, -1, 0, -1,
    -1, 1, -1, 0, 0, -1, 1, 1, 1, -1, -1, 0, -1, 0, 0, 0, 0, -1, 0, -1, 0, -1, 0, -1, 1, 0, 0, -1, -1, 0, 1, -1,
    0, 1, -1, 1, -1, 0, 1, 0, 1, -1, 0, 1, -1, 0, 1, 0, 0, -1, 0, 0, -1, -1, 0, -1, 1, -1, 1, -1, -1, -1, 1, 0,
    -1, 1, -1, 0, -1, 0, 1, 0, 1, 0, -1, 0, 0, -1, 1, -1, 0, -1, -1, -1, 0, -1, 1, 0, 1, -1, 0, -1, 0, 0, 1, 0,
    -1, 0, -1, 0, -1, -1, 1, 1, 1, -1, -1, 1, 0, -1, 0, -1, 0, -1, -1, -1, -1, -1, 0, -1, 0, -1, 1, 0, 0, 0, 1, -1,
    -1, 1, -1, 1, -1, -1, 0, 1, 0, -1, -1, 1, -1, -1, 1, 0, -1, 0, -1, 0, -1, 0, 0, -1, 1, 0, 1, 0, -1, -1, 1, -1,
    -1, 0, 0, 0, -1, 0, 1, 1, 0, -1, -1, 0, 0, 0, 1, -1, 0, 0, -1, -1, -1, 0, 0, -1, 1, 0, 1, -1, 0, -1, 1, -1,
    -1, 1, 0, 1, -1, -1, 1, 1, 0, -1, -1, 0, -1, 0, 1, -1, 0, 0, 0, -1, 0, -1, 1, 0, 1, -1, 1, -1, 0, 0, 1, -1,
];

/// The strip's score: `sum_i inputs[i] * TRIT_WEIGHTS[i] * SCALE_FACTOR / 16.0` over
/// `min(len, 256)` dimensions, accumulated left to right in f32 as the strip does.
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TRIT_WEIGHTS.len());
    let mut s: f32 = 0.0;
    for i in 0..n {
        s += inputs[i] * (TRIT_WEIGHTS[i] as f32) * SCALE_FACTOR / 16.0;
    }
    s
}

/// The strip's K3 band around zero.
pub fn k3_of(r: f32) -> &'static str {
    if r.abs() < 1e-3 {
        "UNKNOWN"
    } else if r > 0.0 {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// `external_socket_bridge_execute`: (status, score); 0xDEAD_BEEF for an empty input, else 1.
pub fn execute(inputs: &[f32]) -> (u32, f32) {
    if inputs.is_empty() {
        return (0xDEAD_BEEF, 0.0);
    }
    (0x0000_0001, ternary_dot(inputs))
}

// ── the stream framing rule (swarm_v2/core/external_socket_bridge.py) ─────────────────────────────
//
// The python reads a chunk, decodes it with `errors='ignore'`, STRIPS it, skips empty payloads, tries
// `json.loads`, and takes `payload["task"]` if present -- falling back to the raw text both when the
// JSON has no TOP-LEVEL `task` key and when it does not parse at all.
//
// The subset implemented here is the TOP-LEVEL object only, which is what the oracle reads: keys are
// read in order, a string value is returned for `task`, scalar values are skipped, and a nested
// object or array makes the scan bail out to the raw text -- the same place an unparseable payload
// lands. An earlier version scanned for the key anywhere in the text, which wrongly answered
// `{"nested": {"task": "inner"}}` with `inner` where the python returns the payload.

/// `raw_payload = data.decode('utf-8', errors='ignore').strip()`; an empty payload is skipped.
pub fn frame_payload(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(String::from(trimmed))
}

/// The python's `task_spec`: `payload["task"]` when the payload parses and carries one, else the raw
/// payload text.
pub fn task_spec(payload: &str) -> String {
    if let Some(v) = extract_task_field(payload) {
        return v;
    }
    String::from(payload)
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i] as char).is_whitespace() {
        i += 1;
    }
    i
}

/// Read a double-quoted string starting at `b[i] == '"'`; `\"` and `\\` are unescaped. Returns
/// (value, index after the closing quote), or None if it never closes.
fn read_string(b: &[u8], mut i: usize) -> Option<(String, usize)> {
    if i >= b.len() || b[i] != b'"' {
        return None;
    }
    i += 1;
    let mut out = String::new();
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            let n = b[i + 1];
            if n == b'"' || n == b'\\' {
                out.push(n as char);
                i += 2;
                continue;
            }
        }
        if b[i] == b'"' {
            return Some((out, i + 1));
        }
        out.push(b[i] as char);
        i += 1;
    }
    None
}

/// Top-level `"task"` extraction. None means "fall back to the raw payload", which is what the python
/// does for a missing key, a non-string task, an unparseable payload or a nesting it cannot read.
fn extract_task_field(payload: &str) -> Option<String> {
    let b = payload.as_bytes();
    let mut i = skip_ws(b, 0);
    if i >= b.len() || b[i] != b'{' {
        return None;
    }
    i += 1;
    let mut found: Option<String> = None;
    loop {
        i = skip_ws(b, i);
        if i >= b.len() {
            return None;
        }
        if b[i] == b'}' {
            // End of the top level. The LAST `task` wins, because a JSON object keeps the last of
            // duplicate keys -- an earlier version returned the first and disagreed with the oracle.
            return found;
        }
        if b[i] == b',' {
            i += 1;
            continue;
        }
        let (key, ni) = read_string(b, i)?;
        i = skip_ws(b, ni);
        if i >= b.len() || b[i] != b':' {
            return None;
        }
        i = skip_ws(b, i + 1);
        if i >= b.len() {
            return None;
        }
        if b[i] == b'"' {
            let (val, ni) = read_string(b, i)?;
            if key == "task" {
                found = Some(val);
            }
            i = ni;
        } else if b[i] == b'{' || b[i] == b'[' {
            // A nested container: bail to the raw text, since the oracle only reads a top-level string.
            return None;
        } else {
            // A scalar: skip it, leaving `i` on the following ',' or '}' for the loop to consume.
            while i < b.len() && b[i] != b',' && b[i] != b'}' {
                i += 1;
            }
        }
    }
}

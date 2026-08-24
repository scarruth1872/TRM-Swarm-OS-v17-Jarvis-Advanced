//! TRM Swarm OS — Native Federation Bridge client (no_std port).
//! The booted kernel's bridge to the Windows resident platform.
//! Transport: UDP 8022 sovereign mesh (host-OS path — the kernel formats
//! requests and the resident serves files). HMAC trust tokens from
//! comm_protocol.rs. No HTTP — native only.
//!
//! On the booted kernel the actual UDP socket lives host-side; this module
//! builds the framed bridge request + parses responses so the `bridge`
//! command can drive the mesh. The frame layout mirrors
//! native_federation_bridge.py exactly.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Bridge ops (match python NativeBridgeServer).
pub const OP_DRIVE_LIST: &str = "drive_list";
pub const OP_FILE_LIST: &str = "file_list";
pub const OP_FILE_READ: &str = "file_read";
pub const OP_COUNCIL: &str = "council";
pub const OP_PING: &str = "ping";

/// A bridge response frame (parsed from the resident).
#[derive(Debug, Clone)]
pub struct BridgeResponse {
    pub ok: bool,
    pub error: String,
    pub data: String,
    pub entries: Vec<String>,
    pub drives: Vec<String>,
    pub score: f64,
    pub seal: String,
}

impl BridgeResponse {
    pub fn new() -> Self {
        BridgeResponse {
            ok: false,
            error: String::new(),
            data: String::new(),
            entries: Vec::new(),
            drives: Vec::new(),
            score: 0.0,
            seal: String::new(),
        }
    }
}

/// Build a framed bridge request (JSON, matches python payload).
/// token: task-scoped HMAC from generate_trust_token.
pub fn build_request(op: &str, token: &str, path: &str, motion: &str) -> String {
    // Minimal JSON builder (no_std).
    let mut s = String::from("{\"op\":\"");
    s.push_str(op);
    s.push_str("\",\"token\":\"");
    s.push_str(token);
    if !path.is_empty() {
        s.push_str("\",\"path\":\"");
        s.push_str(path);
    }
    if !motion.is_empty() {
        s.push_str("\",\"motion\":\"");
        s.push_str(motion);
    }
    s.push_str("\"}");
    s
}

/// Parse a bridge response frame into structured fields.
/// Handles: {"ok":true,"drives":[...]} / {"ok":true,"entries":[...]} /
/// {"ok":true,"data":"...","size":N} / {"ok":false,"error":"..."}.
pub fn parse_response(frame: &str) -> BridgeResponse {
    let mut r = BridgeResponse::new();
    if frame.contains("\"ok\":true") || frame.contains("\"ok\": true") {
        r.ok = true;
    }
    // error
    if let Some(pos) = frame.find("\"error\":\"") {
        let rest = &frame[pos + 9..];
        if let Some(end) = rest.find('"') {
            r.error = rest[..end].to_string();
        }
    }
    // drives
    if frame.contains("\"drives\":") {
        for part in frame.split('"') {
            let t = part.trim();
            if t.len() == 2 && t.ends_with(':') && t.chars().next().unwrap().is_ascii_alphabetic() {
                r.drives.push(t.to_string());
            }
        }
    }
    // entries
    if frame.contains("\"entries\":") {
        for part in frame.split('"') {
            let t = part.trim();
            if !t.is_empty() && !t.contains('{') && !t.contains('}') && !t.contains(':')
                && !t.contains(',') && t != "entries" && t != "ok" && t != "true"
            {
                r.entries.push(t.to_string());
            }
        }
    }
    // data
    if frame.contains("\"data\":\"") {
        if let Some(pos) = frame.find("\"data\":\"") {
            let rest = &frame[pos + 8..];
            if let Some(end) = rest.find('"') {
                r.data = rest[..end].to_string();
            }
        }
    }
    // score
    if frame.contains("\"score\":") {
        if let Some(pos) = frame.find("\"score\":") {
            let rest = &frame[pos + 8..];
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            r.score = num.parse().unwrap_or(0.0);
        }
    }
    // seal
    if frame.contains("\"seal\":") {
        if let Some(pos) = frame.find("\"seal\":\"") {
            let rest = &frame[pos + 8..];
            if let Some(end) = rest.find('"') {
                r.seal = rest[..end].to_string();
            }
        }
    }
    r
}

/// Summarize a bridge response for the LCARS terminal (single line).
pub fn summarize(r: &BridgeResponse) -> String {
    if !r.ok {
        return format!("BRIDGE ERROR: {}", r.error);
    }
    if !r.drives.is_empty() {
        let mut s = String::from("DRIVES: ");
        for (i, d) in r.drives.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(d);
        }
        return s;
    }
    if !r.entries.is_empty() {
        let mut s = String::from("FILES: ");
        let shown = r.entries.len().min(8);
        for i in 0..shown {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&r.entries[i]);
        }
        if r.entries.len() > shown {
            s.push_str(" ...");
        }
        return s;
    }
    if !r.data.is_empty() {
        let mut s = String::from("DATA: ");
        let cap = r.data.len().min(60);
        s.push_str(&r.data[..cap]);
        return s;
    }
    if r.score != 0.0 {
        return format!("COUNCIL: {:.4} seal={}", r.score, r.seal);
    }
    String::from("BRIDGE OK")
}

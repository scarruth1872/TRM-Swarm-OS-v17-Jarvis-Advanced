//! Text extraction the port DROPPED.
//!
//! Sweep finding: `base_agent.py` and `chain_of_verification.py` both compiled
//! regexes (`write_pattern`, `write_marker_pattern`, `numbered_pattern`,
//! `bullet_pattern`). Their Rust counterparts kept only the struct shells -
//! `grep` for "WRITE_FILE" or "bullet" across the kernel returned NOTHING, so
//! these capabilities were silently absent, not merely degraded.
//!
//! Reimplemented here without a regex engine (the kernel has none) by direct
//! scanning, which is both faster and easier to verify.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// `WRITE_FILE: <path>` followed by `[START] ... [END]` on its own lines.
///
/// Mirrors `base_agent.write_marker_pattern`:
///   `^\s*WRITE_FILE:\s*(\S+)\s*\n\[START\]\n([\s\S]*?)\n?\[END\]`
pub fn extract_marker_write(text: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let t = lines[i].trim();
        if let Some(rest) = t.strip_prefix("WRITE_FILE:") {
            let path = rest.trim();
            if !path.is_empty() {
                // find [START] then [END] after it
                let mut s = i + 1;
                while s < lines.len() && lines[s].trim() != "[START]" { s += 1; }
                if s < lines.len() {
                    let mut e = s + 1;
                    while e < lines.len() && lines[e].trim() != "[END]" { e += 1; }
                    if e < lines.len() {
                        let body = lines[s + 1..e].join("\n");
                        return Some((path.to_string(), body));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Fenced code block carrying a `WRITE_FILE: <path>` line, in EITHER order.
/// Mirrors `base_agent.write_pattern`.
pub fn extract_fenced_write(text: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut path: Option<&str> = None;
    let mut i = 0usize;
    while i < lines.len() {
        let t = lines[i].trim();
        if let Some(rest) = t.strip_prefix("WRITE_FILE:") {
            let q = rest.trim();
            if !q.is_empty() && !q.starts_with('`') {
                path = Some(q);
            }
        } else if t.starts_with("```") {
            let mut e = i + 1;
            while e < lines.len() && !lines[e].trim_start().starts_with("```") { e += 1; }
            if e < lines.len() {
                let body = lines[i + 1..e].join("\n");
                if let Some(q) = path {
                    return Some((q.to_string(), body));
                }
                let mut k = e + 1;
                while k < lines.len() {
                    let u = lines[k].trim();
                    if u.is_empty() { k += 1; continue; }
                    if let Some(rest) = u.strip_prefix("WRITE_FILE:") {
                        let q = rest.trim();
                        if !q.is_empty() { return Some((q.to_string(), body)); }
                    }
                    break;
                }
            }
            i = e;
        }
        i += 1;
    }
    None
}

/// `1. item` / `2. item` - numbered list entries, in order.
/// Mirrors `chain_of_verification.numbered_pattern`.
pub fn numbered_items(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 { continue; }
        let rest = &t[digits..];
        if let Some(entry) = rest.strip_prefix('.') {
            let entry = entry.trim();
            if !entry.is_empty() { out.push(entry.to_string()); }
        }
    }
    out
}

/// `- item` or a U+2022 bullet - entries in order.
/// Mirrors `chain_of_verification.bullet_pattern`.
pub fn bullet_items(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        let rest = match t.strip_prefix('-') {
            Some(r) => Some(r),
            None => t.strip_prefix('\u{2022}'),
        };
        if let Some(r) = rest {
            let e = r.trim();
            if !e.is_empty() { out.push(e.to_string()); }
        }
    }
    out
}

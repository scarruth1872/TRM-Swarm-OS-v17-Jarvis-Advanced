//! TRM Swarm OS - reconnaissance_daemon (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/reconnaissance_daemon.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const RECONNAISSANCE_DIR: &'static str = "swarm_v2_reconnaissance";
pub const DEFAULT_RESEARCH_TOPICS: &'static [&'static str] = &["latest AI research papers 2024", "transformer architecture improvements", "large language model innovations", "autonomous AI agents research", "machine learning optimization techniques", "neural network efficiency papers", "AI safety and alignment research", "multimodal AI models papers", "reinforcement learning advances", "AI code generation research"];

//  get_reconnaissance_daemon() 
static SINGLETON_RECONNAISSANCE_DAEMON: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_reconnaissance_daemon() -> u64 {
    // Singleton handle (lazy-init pattern from reconnaissance_daemon.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_RECONNAISSANCE_DAEMON.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_RECONNAISSANCE_DAEMON.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

//  start_reconnaissance(interval_hours, first_run_delay) 
pub fn start_reconnaissance(interval_hours: u64, first_run_delay: u64) -> u64 {
    // """Start the reconnaissance daemon with optional startup delay."""
    // daemon = get_reconnaissance_daemon()
    // daemon.interval_seconds = interval_hours * 3600
    // daemon.first_run_delay = first_run_delay
    // daemon.start()
    // return daemon
    0 // TODO: complete 1:1 math
}

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 5 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).

pub fn fnv1a_fold(slice: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in slice {
        let lower = if (b'A'..=b'Z').contains(&b) { b + 32 } else { b };
        h ^= lower as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub fn score_topics(query: &[u8], docs: &[&[u8]]) -> Vec<f64> {
    docs.iter().map(|d| topic_relevance(query, d)).collect()
}

pub fn topic_matches(topic: &str, finding_topic: &str) -> bool {
    let t = topic.to_ascii_lowercase();
    let f = finding_topic.to_ascii_lowercase();
    f.contains(&t)
}

pub fn topic_relevance(query: &[u8], doc: &[u8]) -> f64 {
    let q = word_hashes(query, 32);
    let t = word_hashes(doc, 64);
    let mut qi = 0usize;
    let mut ti = 0usize;
    let mut inter = 0usize;
    while qi < q.len() && ti < t.len() {
        if q[qi] == t[ti] {
            inter += 1;
            qi += 1;
            ti += 1;
        } else if q[qi] < t[ti] {
            qi += 1;
        } else {
            ti += 1;
        }
    }
    let union = q.len() + t.len() - inter;
    if union > 0 {
        inter as f64 / union as f64
    } else {
        0.0
    }
}

pub fn word_hashes(bytes: &[u8], cap: usize) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut idx = 0usize;
    let len = bytes.len();
    while idx < len && out.len() < cap {
        while idx < len && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= len {
            break;
        }
        let start = idx;
        while idx < len && !bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        out.push(fnv1a_fold(&bytes[start..idx]));
    }
    out.sort_unstable();
    out.dedup();
    out
}

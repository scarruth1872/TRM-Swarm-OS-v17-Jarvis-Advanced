//! /mailbox/ring -- the kernel reading its own IPC transport.
//!
//! The platform's internal protocol is the shared-memory ring bus (`swarm_mailbox.ring`), not HTTP.
//! This route DERIVES the bus state from the file's own bytes, so the kernel reports what the IPC
//! layer actually holds instead of the hardcoded pair under `/mailbox`.
//!
//! Layout, which must track swarm_v2/core/agent_mailbox_bus.py:
//!   header        64 B
//!   per agent i   base = 64 + i * (2*64 + 4096*64) = 64 + i*262272
//!                 head u64 LE at base, tail u64 LE at base+64, data at base+128
//!                 slot (n % 4096) * 64
//!   message 64 B: msg_id u64@0, sender u8@8, recipient u8@9, k3_trit i8@10, priority u8@11,
//!                 payload_size u32@12, inline_data 40 B@16, timestamp_ns u64@56
//! The arithmetic is checkable against the file: 15*262272 + 64 == 3,934,144.

use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const HEADER: u64 = 64;
const CACHE_LINE: u64 = 64;
const CAPACITY: u64 = 4096;
const MSG: u64 = 64;
const AGENTS: [&str; 15] = [
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge", "Orchestra",
    "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes",
];

fn agent_size() -> u64 {
    2 * CACHE_LINE + CAPACITY * MSG
}

fn agent_index(name: &str) -> Option<usize> {
    AGENTS.iter().position(|a| *a == name)
}

/// Push a 64-byte message onto one agent's ring slot — the same ABI the Python writer uses
/// (`swarm_v2/core/agent_mailbox_bus.py`). This is what makes the IPC transport bidirectional: the
/// kernel can produce into the bus it also reads, instead of only telemetring it. Single-producer
/// per agent (the kernel is the runtime's only writer; Python is offline tooling). Returns false
/// when the ring is full, the named agent is unknown, or the store is unreadable — a full/absent
/// bus is a dropped live event, never a hard failure, because the Monad ledger is the durable record.
pub fn push(sender: &str, recipient: &str, trit: i8, payload: &[u8]) -> bool {
    let (Some(s), Some(r)) = (agent_index(sender), agent_index(recipient)) else {
        return false;
    };
    let Some(root) = crate::k3_retrieval::repo_root() else {
        return false;
    };
    let path = root.join("swarm_mailbox.ring");
    let Ok(mut f) = std::fs::OpenOptions::new().read(true).write(true).open(&path) else {
        return false;
    };

    let base = HEADER + r as u64 * agent_size();
    let (Some(head), Some(tail)) = (u64_at(&mut f, base), u64_at(&mut f, base + CACHE_LINE)) else {
        return false;
    };
    if tail.saturating_sub(head) >= CAPACITY {
        return false; // ring full: drop the live event rather than clobber an unread slot
    }

    let mut msg = [0u8; MSG as usize];
    msg[0..8].copy_from_slice(&tail.to_le_bytes()); // msg_id: the slot's own sequence
    msg[8] = s as u8;
    msg[9] = r as u8;
    msg[10] = trit as u8; // k3_trit i8, stored as its bit pattern
    msg[11] = 2; // priority: ultra mission, matching the Python writer's default
    let n = payload.len().min(40);
    msg[12..16].copy_from_slice(&(n as u32).to_le_bytes());
    msg[16..16 + n].copy_from_slice(&payload[..n]);
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    msg[56..64].copy_from_slice(&now_ns.to_le_bytes());

    use std::io::Write;
    let off = base + 2 * CACHE_LINE + (tail % CAPACITY) * MSG;
    if f.seek(SeekFrom::Start(off)).is_err() || f.write_all(&msg).is_err() {
        return false;
    }
    // Advance tail (release). A single writer per agent, so a plain write is the store order.
    if f.seek(SeekFrom::Start(base + CACHE_LINE)).is_err()
        || f.write_all(&(tail + 1).to_le_bytes()).is_err()
    {
        return false;
    }
    f.flush().is_ok()
}

fn u64_at(f: &mut File, off: u64) -> Option<u64> {
    let mut b = [0u8; 8];
    f.seek(SeekFrom::Start(off)).ok()?;
    f.read_exact(&mut b).ok()?;
    Some(u64::from_le_bytes(b))
}

/// Create and zero-initialize the IPC ring bus at boot. The kernel owns the ring: Python is
/// offline tooling and is not in the runtime path, so on a bare-metal boot nothing else would
/// create it (which is why the transport read "store_not_found"). A fresh boot starts with an
/// empty bus, so the file is (re)zeroed to the full layout: 64 B header + 15 agent slots with
/// head=tail=0 and no messages. This is the same 3,934,144 B the reader expects.
pub fn ensure() {
    // `repo_relative` only returns a path when the target already exists, so a first boot (no
    // ring yet) would resolve to None and never create the store. `repo_root` resolves without
    // requiring the target, which is what a writer creating it for the first time needs.
    let Some(root) = crate::k3_retrieval::repo_root() else {
        return;
    };
    let path = root.join("swarm_mailbox.ring");
    let expected = AGENTS.len() as u64 * agent_size() + HEADER;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
    {
        use std::io::Write;
        let zero = vec![0u8; expected as usize];
        let _ = f.write_all(&zero);
        let _ = f.flush();
    }
}

/// Read the ring and report derived state. `detail` tallies the trits of the occupied slots,
/// which costs a bounded read per agent (at most CAPACITY*MSG bytes).
pub fn handle(url: &str) -> (Value, u16) {
    let detail = url.contains("detail=1");
    let Some(path) = crate::k3_retrieval::repo_relative("swarm_mailbox.ring") else {
        return (
            json!({
                "status": "unavailable",
                "route": "/mailbox/ring",
                "transport": "shared-memory ring bus (IPC)",
                "error": "store_not_found",
                "detail": "swarm_mailbox.ring is not beside the running executable's root",
            }),
            503,
        );
    };
    let Ok(mut f) = File::open(&path) else {
        return (
            json!({ "status": "error", "route": "/mailbox/ring", "error": "open_failed" }),
            503,
        );
    };
    let Ok(md) = f.metadata() else {
        return (json!({ "status": "error", "error": "stat_failed" }), 503);
    };
    let expected = AGENTS.len() as u64 * agent_size() + HEADER;
    if md.len() < expected {
        // A short file cannot be vouched for: say so rather than reporting a clean empty bus.
        return (
            json!({
                "status": "unscannable",
                "route": "/mailbox/ring",
                "store": path.display().to_string(),
                "store_bytes": md.len(),
                "layout_expects_bytes": expected,
                "error": "store_truncated",
                "detail": "a bounded read cannot vouch for the source; no counters reported",
            }),
            503,
        );
    }

    let mut agents = Vec::new();
    let (mut total_pending, mut total_ever) = (0u64, 0u64);
    let (mut pos, mut zero, mut neg) = (0i64, 0i64, 0i64);
    let mut senders: Vec<u8> = Vec::new();
    let mut scanned_slots = 0u64;

    for (i, name) in AGENTS.iter().enumerate() {
        let base = HEADER + i as u64 * agent_size();
        let (Some(head), Some(tail)) = (u64_at(&mut f, base), u64_at(&mut f, base + CACHE_LINE)) else {
            return (
                json!({ "status": "error", "route": "/mailbox/ring", "error": "counter_read_failed",
                        "agent": name }),
                503,
            );
        };
        let pending = tail.saturating_sub(head);
        total_pending += pending;
        total_ever += tail;

        let mut a = json!({
            "agent": name,
            "head": head,
            "tail": tail,
            "pending": pending,
            "capacity": CAPACITY,
            "full": pending >= CAPACITY,
        });
        if detail && pending > 0 {
            let start = head.max(tail.saturating_sub(CAPACITY));
            let (mut ap, mut az, mut an) = (0i64, 0i64, 0i64);
            let mut buf = vec![0u8; MSG as usize];
            for n in start..tail {
                let off = base + 2 * CACHE_LINE + (n % CAPACITY) * MSG;
                if f.seek(SeekFrom::Start(off)).is_err() || f.read_exact(&mut buf).is_err() {
                    return (
                        json!({ "status": "unscannable", "route": "/mailbox/ring",
                                "error": "slot_read_failed", "agent": name, "slot": n }),
                        503,
                    );
                }
                scanned_slots += 1;
                match buf[10] as i8 {
                    t if t > 0 => ap += 1,
                    t if t < 0 => an += 1,
                    _ => az += 1,
                }
                let s = buf[8];
                if !senders.contains(&s) {
                    senders.push(s);
                }
            }
            pos += ap;
            zero += az;
            neg += an;
            a["trits"] = json!({ "positive": ap, "zero": az, "negative": an });
        }
        agents.push(a);
    }

    let mut out = json!({
        "status": "ok",
        "route": "/mailbox/ring",
        "transport": "shared-memory ring bus (IPC)",
        "store": path.display().to_string(),
        "store_bytes": md.len(),
        "layout": "header 64B; per agent base=64+i*262272, head u64@base, tail u64@base+64, data@base+128, slot (n%4096)*64; msg 64B with k3_trit i8@10",
        "agents": agents,
        "total_pending": total_pending,
        "total_messages_ever": total_ever,
        "saturated": total_pending >= CAPACITY * AGENTS.len() as u64,
    });
    if detail {
        out["trit_tally"] = json!({ "positive": pos, "zero": zero, "negative": neg });
        out["scanned_slots"] = json!(scanned_slots);
        out["sender_ids_seen"] = json!(senders);
        out["tally_scope"] = json!(format!("{} slots actually read", scanned_slots));
    }
    (out, 200)
}

// Rollout 20: POST /mailbox/send -- the kernel WRITES to its own IPC ring, natively.
//
// /mailbox/send used to be refused with a 501 "engine_required" (the refusal table in
// actions_handlers.rs claimed agent_mailbox_bus owns the slot protocol). But the ring is the
// kernel's OWN format: mailbox_ring.rs already reads it from its bytes, and it already owns
// `push`, the exact 64-byte slot write the Python writer performs. This module wires the HTTP
// action to that writer -- parse {sender, recipient, message, trit}, push one 64-byte slot onto
// the recipient's ring, then read the recipient's pending count back from the ring's own bytes.
// Nothing here is fabricated: every reported number is measured from swarm_mailbox.ring after
// the write, and a failed push is reported as a failure, never dressed up as success.

use serde_json::{json, Value};
use std::io::Read;
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// The ring's agent roster, in ring-index order -- must track mailbox_ring::AGENTS and
/// swarm_v2/core/agent_mailbox_bus.py::AGENT_IDS. This is shared protocol (the 15-slot bus
/// layout), not a measurement, so a literal roster here is the same constant the reader and the
/// Python writer already pin.
pub const RING_AGENTS: [&str; 15] = [
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge", "Orchestra",
    "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes",
];

fn ok(value: Value) -> Resp {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn send(request: &mut Request) -> Resp {
    let parsed: Value = crate::chat_handlers::read_json_or_form(request, 65_536);

    let sender = parsed.get("sender").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let recipient = parsed.get("recipient").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let message = parsed.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let trit = parsed.get("trit").and_then(|v| v.as_i64());

    if sender.is_empty() || recipient.is_empty() {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "missing_sender_or_recipient",
            "detail": "POST {\"sender\":\"...\",\"recipient\":\"...\",\"message\":\"...\",\"trit\":-1|0|1}",
        }));
    }
    let Some(trit) = trit else {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "missing_trit",
            "detail": "trit must be -1 (threat), 0 (neutral) or +1 (verified)",
        }));
    };
    if !(-1..=1).contains(&trit) {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "invalid_trit",
            "trit": trit, "allowed": [-1, 0, 1],
            "detail": "the ring stores a Kleene trit; anything outside -1/0/+1 is refused rather than clamped into a lie",
        }));
    }
    if !RING_AGENTS.contains(&sender.as_str()) {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "unknown_sender",
            "sender": sender, "roster": RING_AGENTS.to_vec(),
        }));
    }
    if !RING_AGENTS.contains(&recipient.as_str()) {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "unknown_recipient",
            "recipient": recipient, "roster": RING_AGENTS.to_vec(),
        }));
    }
    if crate::k3_retrieval::repo_relative("swarm_mailbox.ring").is_none() {
        return ok(json!({
            "status": "unavailable", "route": "/mailbox/send", "error": "store_not_found",
            "detail": "swarm_mailbox.ring is not beside the running executable's root; the boot path (mailbox_ring::ensure) creates it, so an absent store means the kernel has not stood up its IPC transport",
        }));
    }

    // The real write: the same 64-byte slot ABI the Python writer uses, via the kernel's own
    // mailbox_ring::push. `false` means full/absent/unreadable -- reported, never turned into success.
    let pushed = crate::mailbox_ring::push(&sender, &recipient, trit as i8, message.as_bytes());

    // Measure the result from the ring's own bytes AFTER the write.
    let (ring, _) = crate::mailbox_ring::handle("/mailbox/ring");
    let agents = ring.get("agents").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let recipient_pending = agents
        .iter()
        .find(|a| a.get("agent").and_then(|x| x.as_str()) == Some(recipient.as_str()))
        .and_then(|a| a.get("pending").and_then(|p| p.as_u64()));
    let total_pending = ring.get("total_pending").and_then(|p| p.as_u64());

    if !pushed {
        return ok(json!({
            "status": "error", "route": "/mailbox/send", "error": "push_failed",
            "sender": sender, "recipient": recipient, "trit": trit,
            "recipient_pending": recipient_pending,
            "recipient_full": recipient_pending.map(|p| p >= 4096).unwrap_or(false),
            "total_pending": total_pending,
            "basis": "mailbox_ring::push returned false after roster/trit validation: the recipient's ring is full (4096 unread slots) or the store could not be written; recipient_pending/total_pending are read back from swarm_mailbox.ring as-is",
        }));
    }

    ok(json!({
        "status": "ok", "route": "/mailbox/send",
        "sender": sender, "recipient": recipient, "trit": trit,
        "payload_bytes": (message.len().min(40)) as u64,
        "payload_truncated": message.len() > 40,
        "recipient_pending": recipient_pending,
        "total_pending": total_pending,
        "basis": "one 64-byte slot written to the recipient's ring by mailbox_ring::push (the same ABI as swarm_v2/core/agent_mailbox_bus.py: msg_id u64@0, sender u8@8, recipient u8@9, k3_trit i8@10, priority u8@11, payload_size u32@12, inline_data 40B@16, timestamp_ns u64@56); recipient_pending and total_pending are read back from swarm_mailbox.ring after the write",
    }))
}

pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    if method != Method::Post || path != "/mailbox/send" {
        return None;
    }
    Some(send(request))
}

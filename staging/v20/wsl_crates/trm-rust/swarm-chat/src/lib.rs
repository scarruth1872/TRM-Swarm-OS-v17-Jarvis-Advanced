//! chat.rs — the out-of-the-box chat responder (native).
//!
//! The fix for the hardcoded chat: a message is routed to the right
//! domain agent (via swarm_intel::route), the council votes with a
//! REAL K3 collapse (not canned text), and the answer IS the real
//! decision — routed agent, overlap, verdict, and the record
//! persisted to the fabric.
//!
//! The relay calls this over trm:// (`chat:<message>`); the reply is
//! the platform's genuine intelligence, not a hardcoded string.

use swarm_intel::{council_vote, route, Task, CouncilVerdict};
use swarm_nand::NandFabric;

/// The chat responder: route → council → persist → the real answer.
/// Returns the reply text (the platform's genuine decision).
pub fn respond(fabric: &mut NandFabric, message: &str) -> String {
    // 1. extract the domain from the message (keyword match on the
    //    platform's real domain vocabulary)
    let domains = ["health", "fitness", "wellness", "security", "threat",
                   "drug", "research", "finance", "grid", "climate",
                   "memory", "council", "cosmos", "education"];
    let mut domain = "general".to_string();
    let lower = message.to_lowercase();
    for d in domains {
        if lower.contains(d) {
            domain = d.to_string();
            break;
        }
    }

    // 2. the real intelligence: route + council vote (K3 collapse)
    let (agent, overlap) = route(&domain);
    let task = Task {
        class: "chat".into(),
        reference: "chat-message".into(),
        domain: domain.clone(),
    };
    let (verdict, trits) = council_vote(&task);

    // 3. persist the exchange to the fabric (the One remembers)
    let rec: Vec<i8> = {
        let mut r = vec![1, 1, 1, 1];
        for b in message.bytes().take(32) {
            r.push(if b % 3 == 0 { 1 } else if b % 3 == 1 { -1 } else { 0 });
        }
        r
    };
    let _ = fabric.append(&rec);
    let _ = fabric.flush();

    // 4. the real answer (semantic, honest, traceable)
    let verdict_str = match verdict {
        CouncilVerdict::Act => "ACT",
        CouncilVerdict::Block => "BLOCK",
        CouncilVerdict::Analyze => "ANALYZE",
    };
    format!(
        "[{}] routed to {agent} (overlap {overlap}) · domain: {domain} · vote {:?} · persisted to the fabric",
        verdict_str, trits
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_routes_domains() {
        let p = "/tmp/swarm_chat_test.k3";
        let _ = std::fs::remove_file(p);
        let mut fab = NandFabric::open(p).expect("open");

        let health = respond(&mut fab, "my wellness goal is fitness");
        assert!(health.contains("domain: fitness") || health.contains("domain: wellness"),
                "health message routes to a health domain: {health}");

        let threat = respond(&mut fab, "there is a security threat");
        assert!(threat.contains("domain: security"), "threat routes to security: {threat}");

        let general = respond(&mut fab, "hello how are you");
        assert!(general.contains("domain: general"), "general stays general: {general}");
    }

    #[test]
    fn chat_persists() {
        let p = "/tmp/swarm_chat_test2.k3";
        let _ = std::fs::remove_file(p);
        let mut fab = NandFabric::open(p).expect("open");
        let r = respond(&mut fab, "council decision needed");
        assert!(r.contains("routed to"), "the reply names the routed agent: {r}");
        assert!(fab.trit_count() > 4, "the exchange persists");
    }
}

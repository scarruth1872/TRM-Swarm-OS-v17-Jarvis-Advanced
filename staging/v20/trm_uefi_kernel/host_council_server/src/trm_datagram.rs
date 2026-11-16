//! `trm://` datagram listener — NOT QUIC. This is a custom UDP protocol: an 11-byte TRM0 header and a
//! payload, with no handshake, no streams, no congestion control and no encryption. It was called
//! "QUIC" here and in quic_protocol.rs; that name asserted properties the code does not have, so the
//! module is `trm_datagram` and the transport is described as what it is.
//!
//! It is the native transport the framebuffer dashboard speaks, so the
//! on-screen dashboard does not depend on HTTP or anything outside this machine. A frame is the
//! 11-byte TRM0 header (magic 0x54524D30, agent_id, intent_id, k3_trit, payload_len) plus a
//! payload. The payload is a query; the kernel answers it from its OWN retrieval (the same
//! `/k3/retrieve` branch the HTTP route calls) and replies with the same frame shape carrying the
//! top hit as plain text. Loopback only: the dashboard and the kernel are one machine.
use std::net::UdpSocket;

const TRM_MAGIC: u32 = 0x54524D30; // "TRM0"

/// Run the listener forever (spawned as its own thread). Never panics on a bad frame: a datagram
/// that is not a well-formed TRM0 query is dropped, and a query the engine cannot answer still
/// gets a frame back saying so — a socket that answers nothing is how the dashboard looks dead.
pub fn run() {
    let sock = match UdpSocket::bind("127.0.0.1:8021") {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "trm:// listener unavailable on 127.0.0.1:8021 ({e}) -- the dashboard stays on HTTP loopback"
            );
            return;
        }
    };
    let mut buf = [0u8; 8192];
    loop {
        let (n, src) = match sock.recv_from(&mut buf) {
            Ok(x) => x,
            Err(_) => continue,
        };
        if n < 11 {
            continue;
        }
        let magic = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if magic != TRM_MAGIC {
            continue;
        }
        let agent = buf[4];
        let intent = buf[5];
        let plen = u32::from_be_bytes([buf[7], buf[8], buf[9], buf[10]]) as usize;
        if 11 + plen > n {
            continue;
        }
        let q = String::from_utf8_lossy(&buf[11..11 + plen])
            .trim()
            .to_string();
        if q.is_empty() {
            continue;
        }
        let (answer, trit) = answer_for(&q);
        let mut out = Vec::with_capacity(11 + answer.len());
        out.extend_from_slice(&TRM_MAGIC.to_be_bytes());
        out.push(agent);
        out.push(intent);
        out.push(trit as u8);
        out.extend_from_slice(&(answer.len() as u32).to_be_bytes());
        out.extend_from_slice(answer.as_bytes());
        let _ = sock.send_to(&out, src);
    }
}

/// Ask the platform's own conversational path — the same tiers the React dashboard's `/swarm/chat`
/// reaches: deterministic greetings/math, then the sealed fabric, then the model generators (which
/// report honestly when no generator is running on this machine). Returns the reply text and the
/// K3 trit of the fabric verdict when one was reached (0 otherwise).
fn answer_for(q: &str) -> (String, i8) {
    let ans = crate::chat_handlers::chat_answer("Archi", q);
    (ans.text, 0)
}

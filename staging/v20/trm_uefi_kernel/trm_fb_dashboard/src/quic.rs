//! Speak the platform's native `trm://` QUIC frame over a loopback UDP socket. The dashboard's
//! query goes out as one TRM0 frame (11-byte header + the query bytes) and the answer comes back
//! as the same shape; this is the transport the platform answers natively, so the on-screen
//! dashboard never depends on HTTP. Every wait is bounded (2 s receive timeout) — a platform that
//! stops answering returns the last good frame rather than hanging the display.
use crate::sys;

/// Send `query` as a TRM frame to 127.0.0.1:8021 and read the answer frame's payload into `out`.
/// Returns false on any failure (no socket, connect refused, timeout, short frame), so the caller
/// can keep the previous answer on screen.
pub fn ask(query: &str, out: &mut Vec<u8>) -> bool {
    let fd = match sys::socket(sys::AF_INET, sys::SOCK_DGRAM, 0) {
        Ok(f) => f,
        Err(_) => return false,
    };
    // 2-second receive timeout, exactly like the HTTP loop the dashboard also uses.
    let mut tv = [0u8; 16];
    tv[0..8].copy_from_slice(&2i64.to_ne_bytes());
    let _ = sys::setsockopt(fd, sys::SOL_SOCKET, sys::SO_RCVTIMEO, &tv);
    let mut sa = [0u8; 16];
    sa[0..2].copy_from_slice(&2u16.to_ne_bytes()); // AF_INET
    sa[2..4].copy_from_slice(&[0x1f, 0x55]); // port 8021, network order
    sa[4..8].copy_from_slice(&[127, 0, 0, 1]); // 127.0.0.1
    if sys::connect(fd, &sa).is_err() {
        sys::close(fd);
        return false;
    }
    // TRM0 frame: magic + agent_id + intent_id + k3_trit + payload_len + payload.
    let mut frame = Vec::with_capacity(11 + query.len());
    frame.extend_from_slice(&0x54524D30u32.to_be_bytes());
    frame.push(0); // agent
    frame.push(1); // intent = query
    frame.push(0); // trit (unknown until the engine answers)
    frame.extend_from_slice(&(query.len() as u32).to_be_bytes());
    frame.extend_from_slice(query.as_bytes());
    if sys::write(fd, &frame).is_err() {
        sys::close(fd);
        return false;
    }
    let mut buf = [0u8; 8192];
    let n = match sys::read(fd, &mut buf) {
        Ok(n) => n,
        Err(_) => {
            sys::close(fd);
            return false;
        }
    };
    sys::close(fd);
    if n < 11 {
        return false;
    }
    let plen = u32::from_be_bytes([buf[7], buf[8], buf[9], buf[10]]) as usize;
    if 11 + plen > n {
        return false;
    }
    out.clear();
    out.extend_from_slice(&buf[11..11 + plen]);
    true
}

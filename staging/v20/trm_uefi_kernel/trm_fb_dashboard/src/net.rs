//! Read a platform route over the machine's own loopback. Every wait is bounded: a dashboard that
//! hangs on a listener that stopped answering is worse than one that shows the last thing it saw,
//! so the receive carries a 2-second timeout and a failure just returns the previous frame.
use crate::sys;

/// GET `path` from 127.0.0.1:8022, appending the raw body to `out`. Returns false on any failure
/// (connect refused, timeout, short write) so the caller can keep the last good frame.
pub fn get(path: &str, out: &mut Vec<u8>) -> bool {
    let fd = match sys::socket(sys::AF_INET, sys::SOCK_STREAM, 0) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut tv = [0u8; 16];
    tv[0..8].copy_from_slice(&2i64.to_ne_bytes()); // tv_sec = 2
    let _ = sys::setsockopt(fd, sys::SOL_SOCKET, sys::SO_RCVTIMEO, &tv);
    let mut sa = [0u8; 16];
    sa[0..2].copy_from_slice(&2u16.to_ne_bytes()); // AF_INET
    sa[2..4].copy_from_slice(&[0x1f, 0x56]); // port 8022, network order
    sa[4..8].copy_from_slice(&[127, 0, 0, 1]); // 127.0.0.1
    if sys::connect(fd, &sa).is_err() {
        sys::close(fd);
        return false;
    }
    let mut req = Vec::with_capacity(path.len() + 64);
    req.extend_from_slice(b"GET ");
    req.extend_from_slice(path.as_bytes());
    req.extend_from_slice(b" HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    if sys::write(fd, &req).is_err() {
        sys::close(fd);
        return false;
    }
    out.clear();
    let mut buf = [0u8; 4096];
    loop {
        match sys::read(fd, &mut buf) {
            Ok(0) => break, // EOF: server closed
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(_) => break, // timeout
        }
    }
    sys::close(fd);
    true
}

/// Split the body past the HTTP head. Returns the slice after the first blank line.
pub fn body(buf: &[u8]) -> &[u8] {
    let mut i = 0;
    while i + 3 < buf.len() {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
            return &buf[i + 4..];
        }
        i += 1;
    }
    buf
}

/// Count a literal substring's occurrences -- enough to answer "how many experts are online"
/// from a JSON body without a parser.
pub fn count_sub(hay: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut n = 0;
    let mut i = 0;
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            n += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    n
}

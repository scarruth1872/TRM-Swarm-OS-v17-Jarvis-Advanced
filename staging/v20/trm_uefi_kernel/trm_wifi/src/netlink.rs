//! Netlink, hand-rolled: the message and attribute framing, and the two families a supplicant needs --
//! generic netlink (where nl80211 lives) and routing netlink (to give the interface its address).
//!
//! Nothing here is clever; it is the wire format, spelled out. The one thing worth stating is that the
//! kernel answers a message with either a reply, an ACK, or an error, and the *error* is the only place
//! a caller learns that a command was refused -- so it is returned rather than swallowed.

use crate::syscall::*;
use std::io;

/// `sizeof(struct nlmsghdr)`: len(4) + type(2) + flags(2) + seq(4) + pid(4). Stable wire ABI, derived from
/// the kernel's own `include/uapi/linux/netlink.h`, not from a message seen once.
pub const NLMSG_HDRLEN: usize = 16;

pub const NLMSG_ERROR: u16 = 2;
pub const NLMSG_DONE: u16 = 3;
pub const NLM_F_REQUEST: u16 = 1;
pub const NLM_F_ACK: u16 = 4;
pub const NLM_F_DUMP: u16 = 0x300;
pub const GENL_ID_CTRL: u16 = 0x10;
pub const CTRL_CMD_GETFAMILY: u8 = 3;
pub const CTRL_ATTR_FAMILY_ID: u16 = 1;
pub const CTRL_ATTR_FAMILY_NAME: u16 = 2;

pub fn align4(n: usize) -> usize {
    (n + 3) & !3
}

/// One netlink attribute, padded to the alignment the kernel walks with.
pub fn attr(ty: u16, data: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(align4(4 + data.len()));
    v.extend_from_slice(&(4 + data.len() as u16).to_ne_bytes());
    v.extend_from_slice(&ty.to_ne_bytes());
    v.extend_from_slice(data);
    while v.len() % 4 != 0 {
        v.push(0);
    }
    v
}

pub fn attr_u8(ty: u16, v: u8) -> Vec<u8> {
    attr(ty, &[v])
}

pub fn attr_u16(ty: u16, v: u16) -> Vec<u8> {
    attr(ty, &v.to_ne_bytes())
}

pub fn attr_u32(ty: u16, v: u32) -> Vec<u8> {
    attr(ty, &v.to_ne_bytes())
}

pub fn attr_str(ty: u16, s: &str) -> Vec<u8> {
    let mut b = s.as_bytes().to_vec();
    b.push(0);
    attr(ty, &b)
}

/// A netlink message header with a generic-netlink header after it.
pub fn genl_msg(family: u16, cmd: u8, version: u8, seq: u32, flags: u16, payload: &[u8]) -> Vec<u8> {
    let genl = [cmd, version, 0, 0];
    let body_len = genl.len() + payload.len();
    let mut m = Vec::with_capacity(16 + body_len);
    m.extend_from_slice(&((16 + body_len) as u32).to_ne_bytes());
    m.extend_from_slice(&family.to_ne_bytes());
    m.extend_from_slice(&flags.to_ne_bytes());
    m.extend_from_slice(&seq.to_ne_bytes());
    m.extend_from_slice(&0u32.to_ne_bytes()); // pid: the kernel fills it in
    m.extend_from_slice(&genl);
    m.extend_from_slice(payload);
    while m.len() % 4 != 0 {
        m.push(0);
    }
    m
}

/// A routing-netlink message: the same header, no generic header.
pub fn rt_msg(ty: u16, flags: u16, seq: u32, payload: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(16 + payload.len());
    m.extend_from_slice(&((16 + payload.len()) as u32).to_ne_bytes());
    m.extend_from_slice(&ty.to_ne_bytes());
    m.extend_from_slice(&flags.to_ne_bytes());
    m.extend_from_slice(&seq.to_ne_bytes());
    m.extend_from_slice(&0u32.to_ne_bytes());
    m.extend_from_slice(payload);
    while m.len() % 4 != 0 {
        m.push(0);
    }
    m
}

pub struct Nl {
    pub fd: i32,
    seq: u32,
}

impl Nl {
    pub fn open(proto: i32) -> io::Result<Nl> {
        let fd = socket_nonblocking(AF_NETLINK, SOCK_RAW, proto)?;
        bind(fd, &sockaddr_nl(0, 0))?;
        Ok(Nl { fd, seq: 1 })
    }

    pub fn next_seq(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }

    pub fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        sendto(self.fd, msg, &sockaddr_nl(0, 0)).map(|_| ())
    }

    /// Read every message the kernel sends back for this request. A dump arrives as several datagrams
    /// ending in NLMSG_DONE; a single request ends in an ACK or an error, and the error is returned so
    /// "the kernel refused" never looks like "the kernel said nothing".
    /// Wait up to `deadline` for everything this request will produce, then return what arrived. An
    /// empty answer is a legitimate answer -- the kernel may have nothing to say, or may not answer at
    /// all -- and it is reported as empty rather than waited on, because a boot must not stop here.
    pub fn recv_deadline(&mut self, deadline: std::time::Duration) -> io::Result<Vec<(u16, Vec<u8>)>> {
        let started = std::time::Instant::now();
        let mut out = Vec::new();
        loop {
            let mut buf = vec![0u8; 16384];
            let n = match recvfrom(self.fd, &mut buf)? {
                Some(n) => n,
                None => {
                    if started.elapsed() >= deadline {
                        return Ok(out);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
            };
            buf.truncate(n);
            let (messages, done) = messages_in(&buf)?;
            out.extend(messages);
            if done {
                return Ok(out);
            }
            if out.is_empty() && started.elapsed() >= deadline {
                return Ok(out);
            }
        }
    }
}

/// Walk a netlink message body as attributes. The kernel's lengths are trusted only as far as they are
/// checked: a malformed one ends the walk rather than reading past the buffer.
pub fn parse_attrs(mut buf: &[u8]) -> Vec<(u16, Vec<u8>)> {
    let mut out = Vec::new();
    while buf.len() >= 4 {
        let len = u16::from_ne_bytes([buf[0], buf[1]]) as usize;
        let ty = u16::from_ne_bytes([buf[2], buf[3]]);
        if len < 4 || len > buf.len() {
            break;
        }
        out.push((ty & 0x3fff, buf[4..len].to_vec()));
        // The advance is the ALIGNED length, and a kernel message does not always carry the trailing
        // padding -- so the last attribute in a message can be complete while its aligned length runs
        // past the end of the buffer. That is not a truncated message and it is not an attribute to
        // walk into: without this the slice index panics, which is what `WIFI PROBE: exited 101` was on
        // the machine that has a radio. Nothing emulated ever reached here, because with no wiphy there
        // is no dump to carry an attribute to walk.
        let advance = align4(len);
        if advance > buf.len() {
            break;
        }
        buf = &buf[advance..];


    }
    out
}

/// The nl80211 family id, asked of the kernel rather than assumed: a generic-netlink family id is
/// assigned at registration and is not a constant anyone may write down.
pub fn resolve_family(name: &str) -> io::Result<u16> {
    let mut nl = Nl::open(NETLINK_GENERIC)?;
    let seq = nl.next_seq();
    let msg = genl_msg(GENL_ID_CTRL, CTRL_CMD_GETFAMILY, 1, seq, NLM_F_REQUEST, &attr_str(CTRL_ATTR_FAMILY_NAME, name));
    nl.send(&msg)?;
    for (ty, body) in nl.recv_deadline(std::time::Duration::from_secs(2))? {
        if ty != GENL_ID_CTRL {
            continue;
        }
        if body.len() < 4 {
            continue;
        }
        for (atype, adata) in parse_attrs(&body[4..]) {
            if atype == CTRL_ATTR_FAMILY_ID && adata.len() >= 2 {
                return Ok(u16::from_ne_bytes([adata[0], adata[1]]));
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "the kernel has no such generic netlink family"))
}

/// The netlink messages in one datagram, with the two control types handled as the receive path handles
/// them: a non-zero NLMSG_ERROR becomes Err, an NLMSG_DONE ends the walk. Pure, so the shapes a real kernel
/// sends -- including the ones no emulated run ever produced -- can be exercised without a kernel.
///
/// Every bound here is a bound on the BUFFER, and the walk stops rather than walking into a message it
/// cannot fully see: a length below the header, a length past the end, or an aligned advance past the end
/// all mean "this is where the readable part of the datagram ends".
pub fn messages_in(buf: &[u8]) -> io::Result<(Vec<(u16, Vec<u8>)>, bool)> {
    let mut out = Vec::new();
    let mut off = 0usize;
    let mut done = false;
    while off + NLMSG_HDRLEN <= buf.len() {
        let len = u32::from_ne_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]) as usize;
        let ty = u16::from_ne_bytes([buf[off + 4], buf[off + 5]]);
        if len < NLMSG_HDRLEN || off + len > buf.len() {
            break;
        }
        let body = buf[off + NLMSG_HDRLEN..off + len].to_vec();
        if ty == NLMSG_ERROR {
            let code = if body.len() >= 4 {
                i32::from_ne_bytes([body[0], body[1], body[2], body[3]])
            } else {
                0
            };
            if code != 0 {
                return Err(io::Error::from_raw_os_error(-code));
            }
        } else if ty == NLMSG_DONE {
            done = true;
            break;
        } else {
            out.push((ty, body));
        }
        off += align4(len);
    }
    Ok((out, done))
}
#[cfg(test)]
mod tests {
    use super::*;

    /// A netlink message as the kernel frames one: header, body, padded to four bytes.
    fn msg(ty: u16, body: &[u8]) -> Vec<u8> {
        let len = (NLMSG_HDRLEN + body.len()) as u32;
        let mut m = len.to_ne_bytes().to_vec();
        m.extend_from_slice(&ty.to_ne_bytes());
        m.extend_from_slice(&[0u8, 0]);          // flags
        m.extend_from_slice(&[0u8; 4]);          // seq
        m.extend_from_slice(&[0u8; 4]);          // pid
        m.extend_from_slice(body);
        while m.len() % 4 != 0 {
            m.push(0);
        }
        m
    }

    /// An attribute as the kernel frames one: length, type, payload. `padded` decides whether the trailing
    /// alignment the kernel usually carries is present -- the case that reached metal is a message where it
    /// is not.
    fn attr_of(ty: u16, payload: &[u8], padded: bool) -> Vec<u8> {
        let len = (4 + payload.len()) as u16;
        let mut a = len.to_ne_bytes().to_vec();
        a.extend_from_slice(&ty.to_ne_bytes());
        a.extend_from_slice(payload);
        if padded {
            while a.len() % 4 != 0 {
                a.push(0);
            }
        }
        a
    }

    // ---- the attribute walk: the case that failed on metal, and its neighbours -----------------------

    #[test]
    fn a_trailing_attribute_without_its_padding_does_not_panic() {
        let a = attr_of(3, b"wlan0", false);
        assert_eq!(a.len(), 9);
        let attrs = parse_attrs(&a);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].1, b"wlan0");
    }

    #[test]
    fn a_padded_trailing_attribute_parses_the_same() {
        let attrs = parse_attrs(&attr_of(3, b"wlan0", true));
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].1, b"wlan0");
    }

    #[test]
    fn an_attribute_longer_than_the_buffer_is_stopped() {
        assert!(parse_attrs(&[0xff, 0x00, 0x03, 0x00, 0x01, 0x02]).is_empty());
    }

    #[test]
    fn a_zero_length_attribute_is_the_end() {
        // four bytes claiming a length of zero: not an attribute, and not a place to step to either
        assert!(parse_attrs(&[0x00, 0x00, 0x03, 0x00, 0xaa, 0xbb, 0xcc, 0xdd]).is_empty());
    }

    #[test]
    fn an_attribute_with_an_empty_payload_is_kept() {
        let attrs = parse_attrs(&attr_of(7, b"", true));
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0], (7, vec![]));
    }

    #[test]
    fn an_unaligned_attribute_in_the_middle_ends_the_walk_instead_of_misparsing() {
        // The kernel pads every attribute it sends, so an unaligned length can only begin at the very end of
        // a message -- which is the case the fix above covers. In the middle it means the message is not what
        // it claims to be, and the walk must stop rather than read the next header one byte out of place:
        // stopping loses the tail of a malformed message, where guessing invents attributes that never were.
        let mut buf = attr_of(1, b"abc", false);   // len 7, no pad carried: legal only as the last attribute
        buf.extend_from_slice(&attr_of(2, b"de", true));
        assert_eq!(buf.len(), 15);
        let attrs = parse_attrs(&buf);
        assert_eq!(attrs.len(), 1, "an unpadded attribute in the middle is not a place to walk from");
        assert_eq!(attrs[0], (1, b"abc".to_vec()));
    }

    #[test]
    fn padding_that_is_not_zero_is_stepped_over() {
        // the pad is alignment, not data: its contents must not reach the parser
        let mut buf = attr_of(1, b"ab", true);
        let n = buf.len();
        buf[n - 2] = 0xff;
        buf[n - 1] = 0xff;
        buf.extend_from_slice(&attr_of(2, b"cd", true));
        let attrs = parse_attrs(&buf);
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], (1, b"ab".to_vec()));
        assert_eq!(attrs[1], (2, b"cd".to_vec()));
    }

    #[test]
    fn a_nested_attribute_keeps_its_payload_and_loses_its_flags() {
        // NLA_F_NESTED is 0x8000 and is not part of the type; the walk masks it off and hands the caller the
        // inner bytes, which is how a nested family parse gets them.
        let inner = attr_of(9, b"xyz", true);
        let attrs = parse_attrs(&attr_of(0x8000 | 4, &inner, true));
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].0, 4);
        assert_eq!(attrs[0].1, inner);
    }

    #[test]
    fn many_attributes_are_all_kept_in_order() {
        let mut buf = Vec::new();
        for i in 0..64u16 {
            buf.extend_from_slice(&attr_of(i + 1, &[i as u8; 3], true));
        }
        let attrs = parse_attrs(&buf);
        assert_eq!(attrs.len(), 64);
        assert!(attrs.iter().enumerate().all(|(i, (t, v))| *t == i as u16 + 1 && v == &[i as u8; 3]));
    }

    // ---- the datagram walk: every message shape a kernel can put in one receive ----------------------

    #[test]
    fn an_empty_datagram_has_no_messages() {
        assert_eq!(messages_in(&[]).unwrap(), (vec![], false));
    }

    #[test]
    fn a_header_alone_is_a_message_with_an_empty_body() {
        let (m, done) = messages_in(&msg(0x21, b"")).unwrap();
        assert_eq!(m, vec![(0x21u16, vec![])]);
        assert!(!done);
    }

    #[test]
    fn a_zero_length_message_is_the_end() {
        assert_eq!(messages_in(&[0u8; 16]).unwrap().0, vec![]);
    }

    #[test]
    fn a_message_shorter_than_its_own_header_is_the_end() {
        let mut b = msg(0x21, b"abcdefgh");
        b[0] = 15;                                  // a length below sizeof(nlmsghdr)
        assert_eq!(messages_in(&b).unwrap().0, vec![]);
    }

    #[test]
    fn a_message_longer_than_the_datagram_is_the_end() {
        let mut b = msg(0x21, b"abcd");
        b[1] = 0x40;                                 // claims 16 KB it does not have
        assert_eq!(messages_in(&b).unwrap().0, vec![]);
    }

    #[test]
    fn two_messages_are_both_returned() {
        let mut b = msg(0x21, b"first");
        b.extend_from_slice(&msg(0x21, b"second"));
        let (m, _) = messages_in(&b).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].1, b"first".to_vec());
        assert_eq!(m[1].1, b"second".to_vec());
    }

    #[test]
    fn a_truncated_second_message_does_not_lose_the_first() {
        let mut b = msg(0x21, b"keep me");
        b.extend_from_slice(&msg(0x21, b"drop me")[..12]);
        let (m, _) = messages_in(&b).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].1, b"keep me".to_vec());
    }

    #[test]
    fn an_unaligned_message_length_still_steps_to_the_next() {
        // the kernel aligns nlmsg_len, but a datagram that does not must still be walked by the aligned step
        // rather than by the raw one -- the same rule the attribute walk follows
        let mut b = msg(0x21, b"abcde");             // len 21, pad to 24 in the datagram
        b.truncate(24);
        b.extend_from_slice(&msg(0x21, b"next"));
        b[0] = 21;                                    // claim the unaligned length
        let (m, _) = messages_in(&b).unwrap();
        assert_eq!(m.len(), 2, "the second message must still be found");
        assert_eq!(m[0].1, b"abcde".to_vec());
    }

    #[test]
    fn done_ends_the_walk_and_is_not_a_message() {
        let mut b = msg(0x21, b"payload");
        b.extend_from_slice(&msg(NLMSG_DONE, b""));
        let (m, done) = messages_in(&b).unwrap();
        assert_eq!(m.len(), 1);
        assert!(done);
    }

    #[test]
    fn a_netlink_error_with_a_code_becomes_an_error() {
        let mut body = (-2i32).to_ne_bytes().to_vec();   // ENOENT
        body.extend_from_slice(&[0u8; 20]);
        let err = messages_in(&msg(NLMSG_ERROR, &body)).unwrap_err();
        assert_eq!(err.raw_os_error(), Some(2));
    }

    #[test]
    fn an_acknowledgement_is_not_an_error() {
        let body = [0u8; 24];                            // code 0 is an ACK
        let (m, done) = messages_in(&msg(NLMSG_ERROR, &body)).unwrap();
        assert!(m.is_empty());
        assert!(!done);
    }

    #[test]
    fn a_message_type_the_walk_does_not_know_is_still_returned() {
        let (m, _) = messages_in(&msg(0xfffe, b"unknown")).unwrap();
        assert_eq!(m, vec![(0xfffeu16, b"unknown".to_vec())]);
    }
}

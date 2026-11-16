//! Raw syscalls. The one thing Rust's standard library will not do is hand out a socket that is
//! neither Unix-domain nor TCP -- netlink and AF_PACKET have no std type -- so the syscalls that make
//! and use them are issued here. x86_64, which is the only target this image has.

use std::io;

#[inline(always)]
unsafe fn sys6(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> isize {
    let ret: isize;
    std::arch::asm!(
        "syscall",
        inlateout("rax") n as isize => ret,
        in("rdi") a1, in("rsi") a2, in("rdx") a3,
        in("r10") a4, in("r8") a5, in("r9") a6,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

fn ok(r: isize) -> io::Result<i32> {
    if r < 0 {
        Err(io::Error::from_raw_os_error((-r) as i32))
    } else {
        Ok(r as i32)
    }
}

pub const AF_NETLINK: i32 = 16;
pub const AF_PACKET: i32 = 17;
pub const SOCK_RAW: i32 = 3;
pub const NETLINK_ROUTE: i32 = 0;
pub const NETLINK_GENERIC: i32 = 16;
pub const SOL_NETLINK: i32 = 270;
pub const SOCK_CLOEXEC: i32 = 0o2000000;
pub const SOCK_NONBLOCK: i32 = 0o4000;
pub const ETH_P_PAE: u16 = 0x888E;

pub fn socket(domain: i32, ty: i32, proto: i32) -> io::Result<i32> {
    ok(unsafe { sys6(41, domain as usize, (ty | SOCK_CLOEXEC) as usize, proto as usize, 0, 0, 0) })
}

/// A socket that never blocks. Nothing in this program may wait forever on the kernel: a boot that hangs
/// because a reply did not arrive is worse than a boot that says the reply did not arrive, so every
/// receive is bounded by a deadline the caller chooses.
pub fn socket_nonblocking(domain: i32, ty: i32, proto: i32) -> io::Result<i32> {
    ok(unsafe {
        sys6(41, domain as usize, (ty | SOCK_CLOEXEC | SOCK_NONBLOCK) as usize, proto as usize, 0, 0, 0)
    })
}

pub fn bind(fd: i32, addr: &[u8]) -> io::Result<()> {
    ok(unsafe { sys6(49, fd as usize, addr.as_ptr() as usize, addr.len(), 0, 0, 0) }).map(|_| ())
}

pub fn sendto(fd: i32, buf: &[u8], dst: &[u8]) -> io::Result<usize> {
    let r = unsafe {
        sys6(44, fd as usize, buf.as_ptr() as usize, buf.len(),
             dst.as_ptr() as usize, dst.len(), 0)
    };
    ok(r).map(|v| v as usize)
}

/// Returns Ok(None) when the socket is non-blocking and had nothing to say, so a caller can tell "no
/// frame yet" apart from a real failure -- the difference between an idle interface and a broken one.
pub fn recvfrom(fd: i32, buf: &mut [u8]) -> io::Result<Option<usize>> {
    let r = unsafe {
        sys6(45, fd as usize, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0)
    };
    if r == -11 { return Ok(None); }          // EAGAIN
    ok(r).map(|v| Some(v as usize))
}

pub fn setsockopt(fd: i32, level: i32, name: i32, val: &[u8]) -> io::Result<()> {
    let r = unsafe {
        sys6(54, fd as usize, level as usize, name as usize,
             val.as_ptr() as usize, val.len(), 0)
    };
    ok(r).map(|_| ())
}

pub fn htons(v: u16) -> u16 { v.swap_bytes() }

/// struct sockaddr_nl -- 12 bytes, and the kernel reads all of them.
pub fn sockaddr_nl(pid: u32, groups: u32) -> [u8; 12] {
    let mut a = [0u8; 12];
    a[0..2].copy_from_slice(&(AF_NETLINK as u16).to_ne_bytes());
    a[4..8].copy_from_slice(&pid.to_ne_bytes());
    a[8..12].copy_from_slice(&groups.to_ne_bytes());
    a
}

/// struct sockaddr_ll -- an interface index and the ethertype, for the EAPOL socket.
/// Randomness from the kernel itself, by syscall: a boot has no /dev to read, so the nonce must not
/// depend on a device node being mounted.
pub fn getrandom(buf: &mut [u8]) -> io::Result<()> {
    let n = unsafe { sys6(318, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0, 0) };
    ok(n).map(|_| ())
}

/// The same link-layer address, carrying a destination: a packet socket sends where the address says, so
/// without it the frame has nowhere to go.
pub fn sockaddr_ll_to(ifindex: i32, proto: u16, dst: &[u8; 6]) -> [u8; 20] {
    let mut a = sockaddr_ll(ifindex, proto);
    a[12..18].copy_from_slice(dst);
    a
}

pub fn sockaddr_ll(ifindex: i32, proto: u16) -> [u8; 20] {
    let mut a = [0u8; 20];
    a[0..2].copy_from_slice(&(AF_PACKET as u16).to_ne_bytes());
    a[2..4].copy_from_slice(&htons(proto).to_ne_bytes());
    a[4..8].copy_from_slice(&ifindex.to_ne_bytes());
    a
}

//! Raw syscalls, x86_64, mirroring the supplicant's syscall.rs. A renderer that needs no crate
//! graph ships beside the platform it draws, so the framebuffer and the loopback socket are both
//! reached with the syscall the kernel answers, not through a libc binding.
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

pub const O_RDWR: i32 = 2;
pub const O_WRONLY: i32 = 1;
pub const O_RDONLY: i32 = 0;
pub const O_NONBLOCK: i32 = 0o4000;
pub const O_APPEND: i32 = 0o2000;
pub const AF_INET: i32 = 2;
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const SOL_SOCKET: i32 = 1;
pub const SO_RCVTIMEO: i32 = 20;

pub fn open(path: &[u8], flags: i32) -> io::Result<i32> {
    ok(unsafe { sys6(2, path.as_ptr() as usize, flags as usize, 0, 0, 0, 0) })
}

pub fn close(fd: i32) {
    unsafe { sys6(3, fd as usize, 0, 0, 0, 0, 0); }
}

pub fn ioctl(fd: i32, req: usize, arg: *mut u8) -> io::Result<i32> {
    ok(unsafe { sys6(16, fd as usize, req, arg as usize, 0, 0, 0) })
}

/// Returns the mapped address, or usize::MAX on failure (MAP_FAILED).
pub fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8 {
    let r = unsafe { sys6(9, addr as usize, len, prot as usize, flags as usize, fd as usize, off as usize) };
    r as *mut u8
}

pub fn munmap(addr: *mut u8, len: usize) {
    unsafe { sys6(11, addr as usize, len, 0, 0, 0, 0); }
}

pub fn socket(domain: i32, ty: i32, proto: i32) -> io::Result<i32> {
    ok(unsafe { sys6(41, domain as usize, ty as usize, proto as usize, 0, 0, 0) })
}

pub fn setsockopt(fd: i32, level: i32, name: i32, val: &[u8]) -> io::Result<()> {
    ok(unsafe { sys6(54, fd as usize, level as usize, name as usize, val.as_ptr() as usize, val.len(), 0) })
        .map(|_| ())
}

pub fn connect(fd: i32, addr: &[u8]) -> io::Result<()> {
    ok(unsafe { sys6(42, fd as usize, addr.as_ptr() as usize, addr.len(), 0, 0, 0) }).map(|_| ())
}

pub fn write(fd: i32, buf: &[u8]) -> io::Result<usize> {
    ok(unsafe { sys6(1, fd as usize, buf.as_ptr() as usize, buf.len(), 0, 0, 0) }).map(|v| v as usize)
}

pub fn read(fd: i32, buf: &mut [u8]) -> io::Result<usize> {
    ok(unsafe { sys6(0, fd as usize, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0) }).map(|v| v as usize)
}

pub fn nanosleep(secs: u64, nanos: u64) {
    let mut ts = [0u8; 16];
    ts[0..8].copy_from_slice(&(secs as i64).to_ne_bytes());
    ts[8..16].copy_from_slice(&(nanos as i64).to_ne_bytes());
    unsafe { sys6(35, ts.as_ptr() as usize, 0, 0, 0, 0, 0); }
}

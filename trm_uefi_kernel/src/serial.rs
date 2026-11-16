//! serial.rs — COM1 output for the boot path.
//!
//! WHY: verifying a boot artifact by photographing its framebuffer makes every check depend on an image model
//! reading a 6-pixel font. The kernel knows exactly what it measured; it should say it on a wire that `qemu
//! -serial file:...` already captures, so a boot test can be read as text.
//!
//! This writes directly to the 16550 UART at the conventional COM1 port instead of going through the firmware's
//! console protocol, because OVMF does not route ConOut to the serial port unless console redirection is
//! configured — and the whole point is to not depend on firmware configuration.

use core::arch::asm;

const COM1: u16 = 0x3F8;

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

/// 115200 8N1, FIFO on. Safe to call once early; a second call is harmless.
pub unsafe fn init() {
    outb(COM1 + 1, 0x00); // disable interrupts
    outb(COM1 + 3, 0x80); // DLAB on
    outb(COM1 + 0, 0x01); // divisor low  = 1 -> 115200
    outb(COM1 + 1, 0x00); // divisor high = 0
    outb(COM1 + 3, 0x03); // 8N1, DLAB off
    outb(COM1 + 2, 0xC7); // FIFO enable + clear, 14-byte threshold
    outb(COM1 + 4, 0x03); // DTR + RTS
}

fn tx_ready() -> bool {
    unsafe { inb(COM1 + 5) & 0x20 != 0 }
}

/// Blocking write with a bounded spin: a missing UART must not hang the compositor.
pub fn write(bytes: &[u8]) {
    for &b in bytes {
        let mut spins = 0u32;
        while !tx_ready() {
            spins += 1;
            if spins > 2_000_000 {
                return;
            }
            core::hint::spin_loop();
        }
        unsafe { outb(COM1, b) };
    }
}

/// `TRM-BOOT|key=value` — one greppable record per line.
pub fn kv(key: &str, value: &[u8]) {
    write(b"TRM-BOOT|");
    write(key.as_bytes());
    write(b"=");
    write(value);
    write(b"\r\n");
}

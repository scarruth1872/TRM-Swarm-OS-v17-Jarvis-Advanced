//! SMP bring-up: SIPI trampoline and per-core AP entry.
//!
//! An Application Processor wakes in 16-bit real mode at CS:IP from the SIPI
//! vector, with paging and long mode OFF. Getting it into the same 64-bit world
//! as the BSP needs a trampoline that loads a flat GDT, sets CR0.PE, far-jumps to
//! 32-bit, sets CR4.PAE + EFER.LME + CR0.PG, then far-jumps again to long mode.
//!
//! The trampoline is assembled by nasm (trmboot/trm_smp_trampoline.asm) because
//! LLVM's integrated assembler cannot switch .code16 -> .code32. It is embedded
//! here as a raw blob. Offsets come from the nasm listing (`nasm -l`), so they
//! are read off the real assembly rather than guessed.
//!
//! STATUS: assembles, is wired, and each byte of the trampoline is verifiable in
//! the nasm listing - but the SIPI sequence has NEVER been executed. Do not claim
//! AP bring-up works until a boot on real hardware confirms AP_ONLINE > 0.

use core::sync::atomic::{AtomicU32, Ordering};

/// Where the trampoline is copied: 0x8000 => SIPI vector 0x08.
pub const TRAMPOLINE_ADDR: usize = 0x8000;
/// SIPI vector (page number). 0x08 => CS:IP = 0x0800:0x0000 = 0x8000.
pub const SIPI_VECTOR: u8 = 0x08;
/// Bytes per AP stack.
pub const AP_STACK_SIZE: usize = 4096;

// `used` is load-bearing: without a live caller the linker drops the blob and the
// binary contains no trampoline at all - verified by scanning the built EFI for
// the prologue bytes, which came back absent before this was added.
#[used]
static TRAMPOLINE: &[u8] = include_bytes!("../../trmboot/trm_smp_trampoline.bin");

/// Number of bytes of trampoline actually carried in this build.
pub fn blob_len() -> usize { TRAMPOLINE.len() }

/// Offsets within TRAMPOLINE, taken from the nasm listing (org 0x8000).
const OFF_CR3: usize = 0x88;
const OFF_STACK: usize = 0x90;
const OFF_INDEX: usize = 0x98;
const OFF_ENTRY: usize = 0xA0;
const OFF_GDT_FLAT: usize = 0xB0;
const OFF_GDT_DESC: usize = 0xD0;
const BLOB_LEN: usize = 218;

/// x2APIC Interrupt Command Register (single 64-bit MSR in x2APIC mode).
const X2APIC_ICR: u32 = 0x830;

/// APs that have reached `ap_entry`.
pub static AP_ONLINE: AtomicU32 = AtomicU32::new(0);

#[inline]
unsafe fn wrmsr(msr: u32, val: u64) {
    let lo = val as u32;
    let hi = (val >> 32) as u32;
    core::arch::asm!("wrmsr", in("ecx") msr, in("eax") lo, in("edx") hi, options(nostack, preserves_flags));
}

#[inline]
unsafe fn rdmsr(msr: u32) -> u64 {
    let lo: u32; let hi: u32;
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nostack, preserves_flags));
    ((hi as u64) << 32) | lo as u64
}

/// Wait for ICR delivery-complete (bit 12), bounded so a dead AP cannot hang the BSP.
unsafe fn wait_delivery() {
    let mut spins = 0u32;
    loop {
        if (rdmsr(X2APIC_ICR) >> 12) & 1 == 0 { break; }
        spins += 1;
        if spins > 100_000_000 { break; }
        core::hint::spin_loop();
    }
}

/// Send one IPI in physical destination mode.
unsafe fn send_ipi(dest: u32, mode: u64, vector: u8, level: u64, trigger: u64) {
    let val = (vector as u64) | (mode << 8) | (level << 14) | (trigger << 15) | ((dest as u64) << 32);
    wrmsr(X2APIC_ICR, val);
    wait_delivery();
}

/// Start one AP: the documented INIT-SIPI-SIPI sequence. Two SIPIs because some
/// chipsets drop the first (per the Intel MP specification).
pub unsafe fn start_ap(dest: u32, vector: u8) {
    send_ipi(dest, 5, 0, 1, 0);
    for _ in 0..10_000_000 { core::hint::spin_loop(); }
    send_ipi(dest, 5, 0, 0, 0);
    for _ in 0..10_000_000 { core::hint::spin_loop(); }
    send_ipi(dest, 6, vector, 1, 1);
    for _ in 0..1_000_000 { core::hint::spin_loop(); }
    send_ipi(dest, 6, vector, 1, 1);
}

/// First Rust code each AP executes.
#[no_mangle]
pub extern "C" fn ap_entry(_cpu_index: u64) -> ! {
    AP_ONLINE.fetch_add(1, Ordering::SeqCst);
    loop { core::hint::spin_loop(); }
}

/// Copy the trampoline below 1 MB and patch its runtime fields.
///
/// `cr3` must be the BSP's current PML4 so the AP inherits the SAME page tables.
pub unsafe fn install(cr3: u64, stack_top: u64, index: u64) -> bool {
    if TRAMPOLINE.len() < BLOB_LEN { return false; }
    if TRAMPOLINE_ADDR + BLOB_LEN > 0x100000 { return false; }  // must stay under 1 MB
    let dst = TRAMPOLINE_ADDR as *mut u8;
    core::ptr::copy_nonoverlapping(TRAMPOLINE.as_ptr(), dst, BLOB_LEN);
    core::ptr::write_unaligned(dst.add(OFF_CR3) as *mut u64, cr3);
    core::ptr::write_unaligned(dst.add(OFF_STACK) as *mut u64, stack_top);
    core::ptr::write_unaligned(dst.add(OFF_INDEX) as *mut u64, index);
    core::ptr::write_unaligned(dst.add(OFF_ENTRY) as *mut u64, ap_entry as *const () as u64);
    // Point the embedded GDT descriptor at the GDT inside the copy.
    let gdt_runtime = (TRAMPOLINE_ADDR as u64) + OFF_GDT_FLAT as u64;
    core::ptr::write_unaligned(dst.add(OFF_GDT_DESC + 2) as *mut u64, gdt_runtime);
    true
}

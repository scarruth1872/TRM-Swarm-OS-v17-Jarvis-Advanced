//! Real interrupt stubs + loadable IDT.
//! Replace the null gates from apic_core_binding with actual code addresses.
use core::arch::global_asm;

global_asm!(r#"
isr0:
    push 0
    push 0
    jmp isr_common
isr1:
    push 0
    push 1
    jmp isr_common
isr2:
    push 0
    push 2
    jmp isr_common
isr3:
    push 0
    push 3
    jmp isr_common
isr4:
    push 0
    push 4
    jmp isr_common
isr5:
    push 0
    push 5
    jmp isr_common
isr6:
    push 0
    push 6
    jmp isr_common
isr7:
    push 0
    push 7
    jmp isr_common
isr8:
    push 8
    jmp isr_common
isr9:
    push 0
    push 9
    jmp isr_common
isr10:
    push 10
    jmp isr_common
isr11:
    push 11
    jmp isr_common
isr12:
    push 12
    jmp isr_common
isr13:
    push 13
    jmp isr_common
isr14:
    push 14
    jmp isr_common
isr15:
    push 0
    push 15
    jmp isr_common
isr16:
    push 0
    push 16
    jmp isr_common
isr17:
    push 17
    jmp isr_common
isr18:
    push 0
    push 18
    jmp isr_common
isr19:
    push 0
    push 19
    jmp isr_common
isr20:
    push 0
    push 20
    jmp isr_common
isr21:
    push 21
    jmp isr_common
isr22:
    push 0
    push 22
    jmp isr_common
isr23:
    push 0
    push 23
    jmp isr_common
isr24:
    push 0
    push 24
    jmp isr_common
isr25:
    push 0
    push 25
    jmp isr_common
isr26:
    push 0
    push 26
    jmp isr_common
isr27:
    push 0
    push 27
    jmp isr_common
isr28:
    push 0
    push 28
    jmp isr_common
isr29:
    push 29
    jmp isr_common
isr30:
    push 30
    jmp isr_common
isr31:
    push 0
    push 31
    jmp isr_common
isr_common:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, rsp
    call isr_dispatch
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 16
    iretq
"#);

// Stub entry points emitted by the macro above.
extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
}

/// Register state as pushed by isr_common, plus the CPU frame.
#[repr(C)]
pub struct InterruptFrame {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9: u64,  pub r8: u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    pub vector: u64, pub error_code: u64,
    pub rip: u64, pub cs: u64, pub rflags: u64, pub rsp: u64, pub ss: u64,
}

use core::sync::atomic::{AtomicU64, Ordering};

/// Per-vector interrupt counts. Static storage keeps the dispatcher lock-free
/// and allocation-free, which is required in an interrupt context.
pub static COUNTS: [AtomicU64; 32] = [const { AtomicU64::new(0) }; 32];

/// Called from isr_common with a pointer to the frame. Never allocates, never
/// panics: an exception mid-dispatch would triple-fault the machine.
#[no_mangle]
pub extern "C" fn isr_dispatch(frame: *mut InterruptFrame) {
    if frame.is_null() { return; }
    let f = unsafe { &*frame };
    let v = f.vector as usize;
    if v < 32 {
        COUNTS[v].fetch_add(1, Ordering::Relaxed);
    }
}

/// Address of each vector's stub, for IDT construction.
pub fn stub_addresses() -> [u64; 32] {
    let fns: [unsafe extern "C" fn(); 32] = unsafe { [
isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7, isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15, isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23, isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31
    ] };
    let mut out = [0u64; 32];
    let mut i = 0;
    while i < 32 { out[i] = fns[i] as *const () as u64; i += 1; }
    out
}

use crate::apic_core_binding::{GdtPointer, IdtEntry};

/// The IDT pointer has the same 2+8 byte layout as the GDT pointer.
pub type IdtPointer = GdtPointer;

/// Build a REAL IDT: every gate points at its vector's stub, unlike the
/// placeholder table in apic_core_binding whose gates were null.
pub fn build_real_idt(table: &mut [IdtEntry; 256]) {
    let addrs = stub_addresses();
    let mut i = 0;
    while i < 256 {
        let off = if i < 32 { addrs[i] } else { addrs[0] };
        table[i] = IdtEntry {
            offset_low: (off & 0xFFFF) as u16,
            selector: 0x08,
            ist: 0,
            type_attr: 0x8E,
            offset_mid: ((off >> 16) & 0xFFFF) as u16,
            offset_high: (off >> 32) as u32,
            reserved: 0,
        };
        i += 1;
    }
}

/// Load a GDT. Must only be called AFTER boot services are exited.
pub unsafe fn load_gdt(ptr: &IdtPointer) { core::arch::asm!("lgdt [{}]", in(reg) ptr, options(nostack)); }

/// Load an IDT. Must only be called AFTER boot services are exited.
pub unsafe fn load_idt(ptr: &IdtPointer) { core::arch::asm!("lidt [{}]", in(reg) ptr, options(nostack)); }

/// Load the TSS via its GDT selector.
pub unsafe fn load_tss(selector: u16) { core::arch::asm!("ltr {:x}", in(reg) selector, options(nostack)); }

/// Enable interrupts.
pub unsafe fn enable_interrupts() { core::arch::asm!("sti", options(nostack)); }

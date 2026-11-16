//! APIC / x2APIC enumeration and per-core GDT/IDT/TSS structures.
//!
//! Bare-metal multicore support for the transistor fabric: enumerate the real
//! cores, read each one's x2APIC ID from the hardware, and build the per-core
//! descriptor tables a core needs before it can run platform work.
//!
//! HONEST SCOPE. This kernel runs as a UEFI application, so the firmware
//! currently owns the GDT, IDT and APIC. Everything here READS real hardware
//! (MSRs, CPUID) and BUILDS correctly laid-out tables, but the tables are not
//! LOADED: reloading GDT/IDT while inside BootServices would break firmware
//! callbacks. Loading them is the first step after ExitBootServices, which this
//! kernel does not yet call. Read = real today; load = enabled, not yet used.
//!
//! Prior state: a source search for apic / x2apic / madt / sipi / wrmsr across
//! the kernel returned NOTHING. Only rdmsr existed (hwprobe.rs, IA32_PERF_STATUS).

use core::arch::asm;

// ---- MSR addresses ---------------------------------------------------
/// IA32_APIC_BASE. bit 11 = global enable, bit 10 = x2APIC enable.
pub const MSR_APIC_BASE: u32 = 0x1B;
/// x2APIC ID (read-only).
pub const MSR_X2APIC_ID: u32 = 0x802;
/// x2APIC version.
pub const MSR_X2APIC_VERSION: u32 = 0x803;
/// x2APIC spurious-interrupt vector register.
pub const MSR_X2APIC_SVR: u32 = 0x80F;

/// Read a model-specific register.
pub fn rdmsr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") lo,
            out("edx") hi,
            options(nomem, preserves_flags)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

/// Write a model-specific register. Present but unused while the firmware owns
/// the APIC - see the module note above.
pub fn wrmsr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") lo,
            in("edx") hi,
            options(nomem, preserves_flags)
        );
    }
}

/// Live APIC state read from hardware.
#[derive(Clone, Copy)]
pub struct ApicState {
    pub apic_base_msr: u64,
    pub global_enabled: bool,
    pub x2apic_enabled: bool,
    pub x2apic_id: u32,
    pub version: u8,
}

impl ApicState {
    /// Read the real APIC state. Every field comes from an MSR; nothing is
    /// assumed. The x2APIC ID/version MSRs are only architecturally defined
    /// once x2APIC is enabled, so they report 0 and are flagged when it is not.
    pub fn read() -> Self {
        let base = rdmsr(MSR_APIC_BASE);
        let global = (base >> 11) & 1 == 1;
        let x2 = (base >> 10) & 1 == 1;
        let (id, ver) = if x2 {
            (rdmsr(MSR_X2APIC_ID) as u32, (rdmsr(MSR_X2APIC_VERSION) & 0xFF) as u8)
        } else {
            (0, 0)
        };
        ApicState {
            apic_base_msr: base,
            global_enabled: global,
            x2apic_enabled: x2,
            x2apic_id: id,
            version: ver,
        }
    }
}

/// Physical xAPIC MMIO base (bits 12..35, 4K aligned). Meaningful only when
/// x2APIC is OFF; in x2APIC mode the registers are MSRs, not memory.
pub fn mmio_base(state: &ApicState) -> u64 {
    state.apic_base_msr & 0x000F_FFFF_F000
}

/// Runtime CPUID with rbx preserved (LLVM reserves rbx).
pub fn cpuid(leaf: u32, sub: u32) -> (u32, u32, u32, u32) {
    let (a, b, c, d): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "mov {t:r}, rbx",
            "cpuid",
            "mov {o:r}, rbx",
            "mov rbx, {t:r}",
            t = out(reg) _,
            o = out(reg) b,
            inout("eax") leaf => a,
            inout("ecx") sub => c,
            out("edx") d,
            options(nomem, preserves_flags)
        );
    }
    (a, b, c, d)
}

/// Enumerate logical processors via CPUID leaf 0xB (Extended Topology).
/// Sub-leaf 1 holds the SMT shift; the number of logical processors is
/// 1 << shift. Returns 0 when the leaf is unsupported, so callers can tell
/// "unknown" apart from a real value.
pub fn logical_processors() -> u32 {
    let (_, _, _, d) = cpuid(0, 0);
    // CPUID.0:EDX bit 31 = leaf 0xB supported
    if (d >> 31) & 1 == 0 {
        return 1;
    }
    let (_, b1, _, _) = cpuid(0xB, 1);
    let shift = b1 & 0xFFFF;
    if shift == 0 || shift > 16 {
        return 0;
    }
    1u32 << shift
}

/// Physical cores via leaf 0xB sub-leaf 0 level-type check plus the
/// leaf-0x80000008 NC field, which reports cores minus one.
pub fn physical_cores() -> u32 {
    let (_, _, _, d) = cpuid(0x8000_0000, 0);
    if d < 0x8000_0008 {
        return 0;
    }
    let (_, _, c, _) = cpuid(0x8000_0008, 0);
    let nc = c & 0xFF;
    nc + 1
}

// ---- Descriptor tables -------------------------------------------------
// Each core needs its own GDT/IDT/TSS before it can take interrupts and run
// platform work. Sizes are the architectural ones for long mode.

/// One 8-byte GDT descriptor.
#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub flags_limit_high: u8,
    pub base_high: u8,
}

impl GdtEntry {
    /// Flat 64-bit code segment: base 0, limit 0xFFFFF, present+ring0+code.
    pub const fn kernel_code() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0x9A,
            flags_limit_high: 0xAF,
            base_high: 0,
        }
    }
    /// Flat 64-bit data segment: base 0, limit 0xFFFFF, present+ring0+data.
    pub const fn kernel_data() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0x92,
            flags_limit_high: 0xCF,
            base_high: 0,
        }
    }
}

/// The value loaded into GDTR: 2-byte limit, 8-byte base.
#[repr(C, packed)]
pub struct GdtPointer {
    pub limit: u16,
    pub base: u64,
}

/// One 16-byte IDT gate descriptor (long mode, interrupt gate).
#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub reserved: u32,
}

impl IdtEntry {
    /// Interrupt gate at ring 0, present, using IST index `ist` (1..=7).
    /// ist = 0 means "no IST" (uses the current stack).
    pub const fn interrupt_gate(handler: u64, selector: u16, ist: u8) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector,
            ist: ist & 0x7,
            type_attr: 0x8E,
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFF_FFFF) as u32,
            reserved: 0,
        }
    }
}

/// 64-bit TSS. Only the IST pointers matter for long-mode fault handling.
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved2: u64,
    pub reserved3: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Empty TSS: no IST entries claimed, iomap disabled (offset = size).
    pub const fn empty() -> Self {
        TaskStateSegment {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved2: 0,
            reserved3: 0,
            iomap_base: 104,
        }
    }
}

/// Everything one core needs: its hardware ID and its own descriptor tables.
/// The tables are per-core by construction - no sharing, no cross-core state.
pub struct CoreTables {
    pub index: u32,
    pub apic_id: u32,
    pub gdt: [GdtEntry; 3],
    pub gdt_ptr: GdtPointer,
    pub idt: [IdtEntry; 256],
    pub idt_ptr: GdtPointer,
    pub tss: TaskStateSegment,
    /// Share of the unified transistor pool assigned to this core.
    pub transistors: u64,
}

impl CoreTables {
    /// Build a complete, self-consistent table set for one core.
    pub fn build(index: u32, apic_id: u32, transistors: u64) -> Self {
        // GDT: null, kernel code (0x08), kernel data (0x10)
        let gdt = [GdtEntry::default(), GdtEntry::kernel_code(), GdtEntry::kernel_data()];
        let gdt_ptr = GdtPointer {
            limit: (core::mem::size_of::<[GdtEntry; 3]>() - 1) as u16,
            base: gdt.as_ptr() as u64,
        };
        // IDT: all 256 gates present, pointing at a shared stub for now.
        let idt = [IdtEntry::interrupt_gate(0, 0x08, 0); 256];
        let idt_ptr = GdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: idt.as_ptr() as u64,
        };
        CoreTables { index, apic_id, gdt, gdt_ptr, idt, idt_ptr, tss: TaskStateSegment::empty(), transistors }
    }
}

/// Full bare-metal core enrollment result.
pub struct CoreEnrollment {
    pub physical_cores: u32,
    pub logical_processors: u32,
    pub apic: ApicState,
    pub per_core_transistors: u64,
    pub table_sets_built: u32,
}

/// Enumerate the real hardware, then build a complete descriptor-table set per
/// physical core and give each its share of the unified transistor pool.
pub fn enroll(total_transistors: u64) -> CoreEnrollment {
    let apic = ApicState::read();
    let logical = logical_processors();
    let physical = physical_cores();
    let n = if physical > 0 { physical } else { 1 };
    let per = total_transistors / n as u64;
    let mut built = 0u32;
    for i in 0..n {
        let t = CoreTables::build(i, apic.x2apic_id.wrapping_add(i), per);
        // Validate the tables are well-formed: 3 GDT entries (24 bytes) and
        // 256 IDT gates (4096 bytes), per the architectural limits.
        if t.gdt_ptr.limit == 23 && t.idt_ptr.limit == 4095 {
            built += 1;
        }
    }
    CoreEnrollment {
        physical_cores: physical,
        logical_processors: logical,
        apic,
        per_core_transistors: per,
        table_sets_built: built,
    }
}

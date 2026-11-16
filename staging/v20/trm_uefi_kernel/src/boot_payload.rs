//! What this volume can actually load.
//!
//! The drive carries three PE binaries beside the EFI image: `host_council_server.exe` (the platform
//! kernel), `trm_swarm_boot.exe` and `swarm_rust_kernel.dll`. Only `\EFI\BOOT\BOOTX64.EFI` is an EFI
//! application, and `startup.nsh` loads exactly that. The others are Windows images (subsystem 3 console,
//! 2 GUI): the firmware will refuse them, and the host kernel additionally links `tiny_http` and
//! `serde_json`, so it is a std binary that needs an OS and can never be a UEFI payload.
//!
//! That is a property of the artifacts, and it was previously not stated anywhere on screen. This module
//! reads the header and reports what the file IS, so "the drive loads its kernel" is answered by a
//! measurement rather than by an assumption.

/// EFI applications declare subsystem 10; anything else is not loadable by firmware.
pub const UEFI_APPLICATION: u16 = 10;
/// EFI runtime driver.
pub const UEFI_RUNTIME_DRIVER: u16 = 12;
/// x86-64.
pub const MACHINE_X64: u16 = 0x8664;

/// The parts of a PE header this report depends on.
#[derive(Debug, Clone, Copy)]
pub struct PeInfo {
    pub valid: bool,
    pub machine: u16,
    pub subsystem: u16,
    pub pe32_plus: bool,
}

/// Read the PE identity out of the first bytes of an image.
///
/// A missing or malformed header is `valid: false` - reported as unreadable, never as loadable.
pub fn inspect(d: &[u8]) -> PeInfo {
    let mut info = PeInfo {
        valid: false,
        machine: 0,
        subsystem: 0,
        pe32_plus: false,
    };
    if d.len() < 0x40 || d[0] != b'M' || d[1] != b'Z' {
        return info;
    }
    let mut lfanew: u32 = 0;
    let mut i = 0usize;
    while i < 4 {
        lfanew |= (d[0x3C + i] as u32) << (8 * i);
        i += 1;
    }
    let at = lfanew as usize;
    if at + 0x5E > d.len() || &d[at..at + 4] != b"PE\0\0" {
        return info;
    }
    let opt = at + 24;
    let magic = u16::from_le_bytes([d[opt], d[opt + 1]]);
    info.pe32_plus = magic == 0x20B;
    info.machine = u16::from_le_bytes([d[at + 4], d[at + 5]]);
    // Subsystem sits at optional-header offset 0x44. PE32 and PE32+ diverge only in the ImageBase
    // width, so every later field keeps the same offset. Reading 24 bytes further (0x5C) made every
    // image classify as NOT an EFI application - including the boot image itself, which is how the
    // bug surfaced: BOOTX64.EFI was checked as a positive control.
    info.subsystem = u16::from_le_bytes([d[opt + 0x44], d[opt + 0x45]]);
    info.valid = true;
    info
}

/// Whether the firmware would accept this image as an EFI application.
pub fn uefi_loadable(p: &PeInfo) -> bool {
    p.valid && p.machine == MACHINE_X64 && (p.subsystem == UEFI_APPLICATION || p.subsystem == UEFI_RUNTIME_DRIVER)
}

/// A short human label for where the image can run.
pub fn where_it_runs(p: &PeInfo) -> &'static str {
    if !p.valid {
        "unreadable header"
    } else if uefi_loadable(p) {
        "EFI application - firmware can load it"
    } else {
        match p.subsystem {
            2 => "Windows GUI - needs an OS",
            3 => "Windows console - needs an OS",
            _ => "not an EFI application",
        }
    }
}

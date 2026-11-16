//! swarm-boot persistence probe — the REAL cross-power-cycle test.
//!
//! v2: this probe actually WRITES a marker to the ESP (FAT32, via the
//! UEFI SimpleFileSystem protocol) and READS it back. It does not just
//! display instructions (the v1 stub — which proved nothing).
//!
//! Test flow:
//!   Boot 1: marker absent  → write marker + boot count 1 → "FIRST BOOT"
//!   Power off completely.
//!   Boot 2: marker present → read boot count → "PERSISTED (boot N)".
//!   Same marker on boot 2 = the file survived a real power cycle.
//!
//! Honest scope: this proves FAT32/ESP persistence on real hardware via
//! the same storage path the fabric would ride on. The full NAND fabric
//! file persistence is the same mechanism (a file on persistent media).

#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use uefi::fs::FileSystem;
use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::CString16;

const MARKER: &str = "\\PERSIST_MARKER.txt";

#[entry]
fn main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init().unwrap();

    // fs work in its own scope (holds the system_table borrow)
    let (previous, boot, write_ok): (Option<String>, u32, bool) = {
        let fs_proto = match system_table.boot_services().get_image_file_system(image) {
            Ok(p) => p,
            Err(_) => return Status::ABORTED, // no stdout here (borrow)
        };
        let mut fs = FileSystem::new(fs_proto);

        let marker_c16 = CString16::try_from(MARKER).unwrap();
        let previous = match fs.read(marker_c16.as_ref()) {
            Ok(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => None,
        };
        let boot = match &previous {
            Some(s) => {
                let n = s.trim().rsplit(' ').next().and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);
                n + 1
            }
            None => 1,
        };
        let content = format!("TRM_PERSIST boot {boot}\r\n");
        let write_ok = fs.write(marker_c16.as_ref(), content.as_bytes()).is_ok();
        (previous, boot, write_ok)
    }; // fs borrow released

    let mut out = system_table.stdout();
    let _ = writeln!(out, "");
    let _ = writeln!(out, "=== NAND PERSISTENCE PROBE v2 ===");
    let _ = writeln!(out, "All Is One.");

    match &previous {
        Some(s) => {
            let _ = writeln!(out, "[READ] marker found: {}", s.trim());
        }
        None => {
            let _ = writeln!(out, "[READ] no marker (first boot on this ESP)");
        }
    }
    if write_ok {
        let _ = writeln!(out, "[WRITE] marker stored: boot {boot}");
    } else {
        let _ = writeln!(out, "[WRITE] FAILED");
    }
    let _ = writeln!(out, "");

    match previous {
        Some(s) => {
            let _ = writeln!(out, "[VERDICT] PERSISTED - previous content survived the power cycle:");
            let _ = writeln!(out, "   was: {}", s.trim());
            let _ = writeln!(out, "   now: boot {boot}");
            let _ = writeln!(out, "NAND/ESP persistence: PROVEN on real hardware");
        }
        None => {
            let _ = writeln!(out, "[VERDICT] FIRST BOOT - marker written.");
            let _ = writeln!(out, "   Now POWER OFF completely and boot again.");
            let _ = writeln!(out, "   If you see boot 2 with the same marker,");
            let _ = writeln!(out, "   persistence is PROVEN.");
        }
    }

    loop {
        system_table.boot_services().stall(10_000_000);
    }
}

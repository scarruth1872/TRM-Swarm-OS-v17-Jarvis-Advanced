//! hw_truth.rs — the boot UI's hardware panel, derived from measurement instead of a spec sheet.
//!
//! WHY THIS MODULE EXISTS
//! The Sector 00 diagnostics panel printed a fixed string set: "1920x1080 @ 32bpp ... 60.0 FPS",
//! "241,800,000 Trit NVMe NAND Knowledge Base Mounted", "Intel AX200 WiFi 6 ... ACTIVE". Booting that same
//! artifact under QEMU/OVMF measured a **1280x800** framebuffer on a machine with **no NVMe and no radio**, and the
//! panel said the same thing it says on the real box. A screen that prints the same hardware whether or not the
//! hardware is there is not a diagnostic; it is a sticker.
//!
//! WHAT IS MEASURED HERE
//!   * display geometry     — from GOP ModeInfo (the values the compositor actually draws into)
//!   * block storage        — every handle exposing BlockIo, with real capacity from its Media block size × LBA count
//!   * network interfaces   — handles exposing SimpleNetwork (presence only; the radio model is NOT readable here)
//!   * K3 NAND fabric       — from crate::nand_fs::nand_status, which opens the volume and reads first/last sector
//!
//! WHAT IS DELIBERATELY NOT PRINTED
//! Refresh rate (no UEFI source reports it), the radio model (SimpleNetwork carries no vendor/model), and the trit
//! count when the fabric is not mounted. Absence is reported as absence. An unmeasured field says UNMEASURED — it
//! never falls back to the nominal that the old string hardcoded.

use r_efi::efi;
use r_efi::protocols::{block_io, simple_network};

use crate::nand_fs::NandStatus;

/// One line of panel text, assembled without alloc.
pub struct Buf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Buf<N> {
    pub const fn new() -> Self {
        Self { buf: [b' '; N], len: 0 }
    }

    pub fn s(&mut self, text: &str) -> &mut Self {
        for &b in text.as_bytes() {
            if self.len < N {
                self.buf[self.len] = b;
                self.len += 1;
            }
        }
        self
    }

    pub fn u(&mut self, mut value: u64) -> &mut Self {
        let mut digits = [0u8; 20];
        let mut n = 0usize;
        if value == 0 {
            return self.s("0");
        }
        while value > 0 {
            digits[n] = b'0' + (value % 10) as u8;
            value /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            if self.len < N {
                self.buf[self.len] = digits[n];
                self.len += 1;
            }
        }
        self
    }

    /// Value in tenths of a megabyte, printed as "12.3 MB" without floating point.
    pub fn mb10(&mut self, bytes: u64) -> &mut Self {
        let tenths = bytes / (1024 * 1024 / 10).max(1);
        self.u(tenths / 10).s(".").u(tenths % 10).s(" MB")
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// Everything the panel is allowed to claim, each field with a source.
#[derive(Debug, Clone, Copy)]
pub struct HwTruth {
    pub display_w: u32,
    pub display_h: u32,
    pub backbuffer: bool,
    pub block_devices: usize,
    pub block_partitions: usize,
    pub block_bytes: u64,
    pub net_devices: usize,
    pub nand: NandStatus,
}

/// Enumerate BlockIo handles: returns (physical_volumes, partitions, measured_bytes).
///
/// A disk and its partition BOTH expose BlockIo with media_present=true. Summing every media double-counts the
/// same bytes — measured under QEMU's vvfat drive, where one 1 GB volume produced 2 handles. So physical volumes
/// carry the byte total and partitions are only counted; if firmware exposes partitions alone, their bytes are
/// used instead. Handles are freed exactly once, after every read of the buffer.
unsafe fn scan_block_devices(bs: *mut efi::BootServices) -> (usize, usize, u64) {
    let mut guid = block_io::PROTOCOL_GUID;
    let mut handles: *mut efi::Handle = core::ptr::null_mut();
    let mut count: usize = 0;
    let r = ((*bs).locate_handle_buffer)(
        efi::BY_PROTOCOL,
        &mut guid,
        core::ptr::null_mut(),
        &mut count,
        &mut handles,
    );
    if r != efi::Status::SUCCESS || handles.is_null() {
        return (0, 0, 0);
    }

    let mut phys: usize = 0;
    let mut phys_bytes: u64 = 0;
    let mut parts: usize = 0;
    let mut part_bytes: u64 = 0;

    for i in 0..count {
        let handle = *handles.add(i);
        let mut g = block_io::PROTOCOL_GUID;
        let mut proto: *mut block_io::Protocol = core::ptr::null_mut();
        let st = ((*bs).handle_protocol)(
            handle,
            &mut g,
            &mut proto as *mut *mut block_io::Protocol as *mut *mut core::ffi::c_void,
        );
        if st != efi::Status::SUCCESS || proto.is_null() {
            continue;
        }
        let media = (*proto).media;
        if media.is_null() || !(*media).media_present {
            continue;
        }
        // capacity = block_size x (last_block + 1); last_block is the highest valid LBA, not a count
        let bytes = ((*media).block_size as u64).saturating_mul((*media).last_block.saturating_add(1));
        if (*media).logical_partition {
            parts += 1;
            part_bytes = part_bytes.saturating_add(bytes);
        } else {
            phys += 1;
            phys_bytes = phys_bytes.saturating_add(bytes);
        }
    }
    ((*bs).free_pool)(handles as *mut core::ffi::c_void);

    if phys > 0 {
        (phys, parts, phys_bytes)
    } else {
        (parts, 0, part_bytes)
    }
}

/// Count handles exposing SimpleNetwork. Presence only — UEFI does not expose a radio's model.
unsafe fn scan_network_devices(bs: *mut efi::BootServices) -> usize {
    let mut guid = simple_network::PROTOCOL_GUID;
    let mut handles: *mut efi::Handle = core::ptr::null_mut();
    let mut count: usize = 0;
    let r = ((*bs).locate_handle_buffer)(
        efi::BY_PROTOCOL,
        &mut guid,
        core::ptr::null_mut(),
        &mut count,
        &mut handles,
    );
    if r != efi::Status::SUCCESS || handles.is_null() {
        return 0;
    }
    ((*bs).free_pool)(handles as *mut core::ffi::c_void);
    count
}

/// Measure the machine. `image_handle` is the loaded image, needed by the fabric mount.
pub unsafe fn measure(
    bs: *mut efi::BootServices,
    image_handle: efi::Handle,
    display_w: u32,
    display_h: u32,
    backbuffer: bool,
) -> HwTruth {
    let (block_devices, block_partitions, block_bytes) = scan_block_devices(bs);
    let net_devices = scan_network_devices(bs);
    let nand = crate::nand_fs::nand_status(image_handle, bs);
    HwTruth { display_w, display_h, backbuffer, block_devices, block_partitions, block_bytes, net_devices, nand }
}

impl HwTruth {
    /// "UEFI GOP DISPLAY:   1280x800 from ModeInfo (back-buffer ACTIVE) | refresh UNMEASURED"
    pub fn display_line(&self) -> Buf<128> {
        let mut b = Buf::new();
        b.s("UEFI GOP DISPLAY:   ").u(self.display_w as u64).s("x").u(self.display_h as u64)
            .s(" MEASURED from ModeInfo | back-buffer ")
            .s(if self.backbuffer { "ACTIVE" } else { "UNAVAILABLE" })
            .s(" | refresh UNMEASURED");
        b
    }

    /// Real capacity of every enumerated block device — on a bare VM this is just the boot volume.
    pub fn storage_line(&self) -> Buf<128> {
        let mut b = Buf::new();
        b.s("BLOCK STORAGE:      ").u(self.block_devices as u64).s(" device(s) + ")
            .s(if self.block_partitions > 0 { "partitioned" } else { "no partitions" })
            .s(", ").mb10(self.block_bytes).s(" MEASURED via BlockIo Media");
        b
    }

    /// The K3 fabric: mounted with real geometry, or absent and explicitly unmeasured.
    pub fn nand_line(&self) -> Buf<128> {
        let mut b = Buf::new();
        if self.nand.mounted {
            b.s("K3 NAND FABRIC:     MOUNTED ").u(self.nand.trits)
                .s(" trits (").mb10(self.nand.size_bytes).s(") | sectors first ")
                .s(if self.nand.first_sector_ok { "OK" } else { "FAIL" })
                .s(" last ")
                .s(if self.nand.last_sector_ok { "OK" } else { "FAIL" });
        } else {
            b.s("K3 NAND FABRIC:     NOT MOUNTED | swarm_v2_nand_fabric.k3 absent | TRITS UNMEASURED");
        }
        b
    }

    /// Radio: interface presence is provable, the model is not. Never name an AX200 that was not enumerated.
    pub fn radio_line(&self) -> Buf<128> {
        let mut b = Buf::new();
        if self.net_devices > 0 {
            b.s("RADIO HARDWARE:     ").u(self.net_devices as u64)
                .s(" SimpleNetwork interface(s) enumerated | radio model UNMEASURED");
        } else {
            b.s("RADIO HARDWARE:     NO SimpleNetwork DEVICE ENUMERATED | radio UNMEASURED");
        }
        b
    }

    /// Compact K3 status for the narrow telemetry card.
    pub fn k3_short(&self) -> Buf<64> {
        let mut b = Buf::new();
        if self.nand.mounted {
            b.s("K3 FABRIC:    MOUNTED ").u(self.nand.trits).s(" trits");
        } else {
            b.s("K3 FABRIC:    ABSENT (TRITS UNMEASURED)");
        }
        b
    }

    /// One-line summary for the status card: only measured facts.
    pub fn summary(&self) -> Buf<96> {
        let mut b = Buf::new();
        b.s("GOP ").u(self.display_w as u64).s("x").u(self.display_h as u64)
            .s(" | BLK ").u(self.block_devices as u64)
            .s(" ").mb10(self.block_bytes)
            .s(" | NET ").u(self.net_devices as u64)
            .s(" | K3 ").s(if self.nand.mounted { "MOUNTED" } else { "ABSENT" });
        b
    }
}

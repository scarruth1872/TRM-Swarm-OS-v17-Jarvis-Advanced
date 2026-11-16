//! TRM Swarm OS v19.001 - NAND K3 Fabric access via UEFI SimpleFileSystem.
//! Mounts the boot volume, opens \swarm_v2_nand_fabric.k3, verifies size and
//! first/last sector reads. No mmap in UEFI - this is the honest mount-verify path.

use r_efi::efi;
use r_efi::protocols::{loaded_image, simple_file_system, file};

/// Expected fabric geometry (2 trits per byte packing).
pub const NAND_TRIT_COUNT: u64 = 153_523_393;
pub const NAND_EXPECTED_BYTES: u64 = (153_523_393 + 1) / 2; // 76,761,697

/// Result of a fabric mount attempt.
#[derive(Debug, Clone, Copy)]
pub struct NandStatus {
    pub mounted: bool,
    pub size_bytes: u64,
    pub trits: u64,
    pub first_sector_ok: bool,
    pub last_sector_ok: bool,
    /// Which volume carried it, in the firmware's enumeration order (-1 when nowhere).
    pub volume: i32,
    /// How many volumes were actually enumerated before deciding.
    pub volumes_searched: usize,
}

/// Open the NAND fabric file on the boot volume and report its state.
/// SAFETY: bs must be live Boot Services; image_handle is the loaded image.
pub unsafe fn nand_status(bs: *mut efi::BootServices) -> NandStatus {
    let mut status = NandStatus {
        mounted: false,
        size_bytes: 0,
        trits: 0,
        first_sector_ok: false,
        last_sector_ok: false,
        volume: -1,
        volumes_searched: 0,
    };

    // Find the fabric on ANY volume the firmware exposes. Assuming it sat next to the boot image
    // was the whole problem: the intelligence is on the machine's drives, and the boot volume is
    // only one of them. Every volume is searched and the one that carries it is named.
    let n = volume_count(bs);
    status.volumes_searched = n;
    let mut found: Option<(usize, u64)> = None;
    for i in 0..n {
        if let Some(sz) = size_on_volume(bs, i, FABRIC_PATH) {
            if sz > 0 {
                found = Some((i, sz));
                break;
            }
        }
    }
    let (vol, size) = match found {
        Some(v) => v,
        None => return status,
    };

    status.volume = vol as i32;
    status.size_bytes = size;
    // Two trits per byte, but an ODD trit count leaves the second trit of the final byte unused, so
    // `size * 2` reports one trit more than the fabric holds - the boot line said 153,523,394 while
    // the K3 panel and the geometry both say 153,523,393, on the same screen.
    status.trits = if size == NAND_EXPECTED_BYTES {
        NAND_TRIT_COUNT
    } else {
        size * 2
    };
    status.mounted = true;

    // Sector reads from the volume that carries it.
    if let Some(root) = open_volume_at(bs, vol) {
        if let Some(f) = open_named(root, FABRIC_PATH) {
            let mut first = [0u8; 512];
            let mut first_size: usize = 512;
            let _ = ((*f).set_position)(f, 0);
            status.first_sector_ok = ((*f).read)(
                f,
                &mut first_size,
                first.as_mut_ptr() as *mut core::ffi::c_void,
            ) == efi::Status::SUCCESS
                && first_size > 0;

            if status.size_bytes >= 512 {
                let mut last = [0u8; 512];
                let mut last_size: usize = 512;
                let _ = ((*f).set_position)(f, status.size_bytes - 512);
                status.last_sector_ok = ((*f).read)(
                    f,
                    &mut last_size,
                    last.as_mut_ptr() as *mut core::ffi::c_void,
                ) == efi::Status::SUCCESS
                    && last_size > 0;
            }
            ((*f).close)(f);
        }
        ((*root).close)(root);
    }
    status
}

/// Every volume the firmware exposes, in its enumeration order (the same order the explorer lists
/// as FS0:, FS1:, ...). This is the answer to "access the machine's drives".
///
/// SAFETY: bs must be live Boot Services.
pub unsafe fn volume_count(bs: *mut efi::BootServices) -> usize {
    let mut guid = simple_file_system::PROTOCOL_GUID;
    let mut n: usize = 0;
    let mut handles: *mut efi::Handle = core::ptr::null_mut();
    if ((*bs).locate_handle_buffer)(
        efi::BY_PROTOCOL,
        &mut guid,
        core::ptr::null_mut(),
        &mut n,
        &mut handles,
    ) != efi::Status::SUCCESS
        || handles.is_null()
    {
        return 0;
    }
    let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);
    n
}

/// Root directory of volume `idx`, or None when it has no filesystem the firmware can use.
///
/// SAFETY: as `volume_count`. The caller owns the returned handle.
pub unsafe fn open_volume_at(
    bs: *mut efi::BootServices,
    idx: usize,
) -> Option<*mut file::Protocol> {
    let mut guid = simple_file_system::PROTOCOL_GUID;
    let mut n: usize = 0;
    let mut handles: *mut efi::Handle = core::ptr::null_mut();
    if ((*bs).locate_handle_buffer)(
        efi::BY_PROTOCOL,
        &mut guid,
        core::ptr::null_mut(),
        &mut n,
        &mut handles,
    ) != efi::Status::SUCCESS
        || handles.is_null()
        || idx >= n
    {
        if !handles.is_null() {
            let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);
        }
        return None;
    }
    let handle = *handles.add(idx);
    let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);

    let mut fs: *mut simple_file_system::Protocol = core::ptr::null_mut();
    if ((*bs).handle_protocol)(
        handle,
        &mut guid,
        &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
    ) != efi::Status::SUCCESS
        || fs.is_null()
    {
        return None;
    }
    let mut root: *mut file::Protocol = core::ptr::null_mut();
    if ((*fs).open_volume)(fs, &mut root) != efi::Status::SUCCESS || root.is_null() {
        return None;
    }
    Some(root)
}

/// Size of `name` on volume `idx`, or None when absent.
///
/// SAFETY: as `volume_count`.
pub unsafe fn size_on_volume(
    bs: *mut efi::BootServices,
    idx: usize,
    name: &str,
) -> Option<u64> {
    let root = open_volume_at(bs, idx)?;
    let fh = open_named(root, name);
    let out = match fh {
        Some(f) => file_size(f),
        None => None,
    };
    if let Some(f) = fh {
        ((*f).close)(f);
    }
    ((*root).close)(root);
    out
}

/// Read `name` from volume `idx` into `buf`, returning the byte count.
///
/// SAFETY: as `volume_count`.
pub unsafe fn read_on_volume(
    bs: *mut efi::BootServices,
    idx: usize,
    name: &str,
    buf: &mut [u8],
) -> Option<usize> {
    let root = open_volume_at(bs, idx)?;
    let fh = open_named(root, name);
    let out = match fh {
        Some(f) => {
            let want = match file_size(f) {
                Some(s) => (s as usize).min(buf.len()),
                None => buf.len(),
            };
            let _ = ((*f).set_position)(f, 0);
            let mut total = 0usize;
            while total < want {
                let mut chunk = want - total;
                let at = total;
                if ((*f).read)(
                    f,
                    &mut chunk,
                    buf[at..].as_mut_ptr() as *mut core::ffi::c_void,
                ) != efi::Status::SUCCESS
                    || chunk == 0
                {
                    break;
                }
                total += chunk;
            }
            Some(total)
        }
        None => None,
    };
    if let Some(f) = fh {
        ((*f).close)(f);
    }
    ((*root).close)(root);
    out
}


/// The fabric the boot reports on. Kept beside the readable corpus that shares its stem.
pub const FABRIC_PATH: &str = "\\swarm_v2_nand_fabric.k3";
/// The readable half: {bytes, key, sha256, text} per line, small enough to hold in memory.
pub const CORPUS_PATH: &str = "\\swarm_v2_nand_fabric.text.jsonl";
/// The companion index.
pub const TURBOVEC_PATH: &str = "\\swarm_v2_nand_fabric.turbovec";

/// Encode an ASCII path as NUL-terminated UTF-16. Returns the length written, excluding the
/// terminator.
///
/// EFI_FILE_PROTOCOL.Open requires a NULL-terminated CHAR16*. A hand-written array for a name
/// whose length exactly fills it has no terminator, so the firmware reads past the end and the
/// open fails on a file that IS present - which is how this module once reported a fabric that was
/// sitting on the volume as NOT FOUND. Encoding through here makes the terminator unconditional.
pub fn to_utf16(name: &str, out: &mut [u16]) -> usize {
    let mut n = 0usize;
    for b in name.bytes() {
        if n + 1 >= out.len() {
            break;
        }
        out[n] = b as u16;
        n += 1;
    }
    out[n] = 0;
    n
}

/// Open the root directory of the volume this image was loaded from.
///
/// SAFETY: bs must be live Boot Services; image_handle is the loaded image.
pub unsafe fn open_root(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
) -> Option<*mut file::Protocol> {
    let mut loaded_guid = loaded_image::PROTOCOL_GUID;
    let mut loaded: *mut loaded_image::Protocol = core::ptr::null_mut();
    if ((*bs).handle_protocol)(
        image_handle,
        &mut loaded_guid,
        &mut loaded as *mut *mut loaded_image::Protocol as *mut *mut core::ffi::c_void,
    ) != efi::Status::SUCCESS
        || loaded.is_null()
    {
        return None;
    }
    let mut fs_guid = simple_file_system::PROTOCOL_GUID;
    let mut fs: *mut simple_file_system::Protocol = core::ptr::null_mut();
    if ((*bs).handle_protocol)(
        (*loaded).device_handle,
        &mut fs_guid,
        &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
    ) != efi::Status::SUCCESS
        || fs.is_null()
    {
        return None;
    }
    let mut root: *mut file::Protocol = core::ptr::null_mut();
    if ((*fs).open_volume)(fs, &mut root) != efi::Status::SUCCESS || root.is_null() {
        return None;
    }
    Some(root)
}

/// Open a file on the boot volume by path. The caller owns the returned handle.
///
/// SAFETY: as `open_root`; `root` must be a live root directory.
pub unsafe fn open_named(
    root: *mut file::Protocol,
    name: &str,
) -> Option<*mut file::Protocol> {
    let mut wide = [0u16; 64];
    to_utf16(name, &mut wide);
    let mut f: *mut file::Protocol = core::ptr::null_mut();
    if ((*root).open)(
        root,
        &mut f,
        wide.as_ptr() as *mut efi::Char16,
        file::MODE_READ,
        0,
    ) != efi::Status::SUCCESS
        || f.is_null()
    {
        return None;
    }
    Some(f)
}

/// The file size from GetInfo - the same source `nand_status` trusts, with the seek-to-end
/// fallback the UEFI spec provides.
///
/// SAFETY: `f` must be a live file handle.
pub unsafe fn file_size(f: *mut file::Protocol) -> Option<u64> {
    // 512, not 80. EFI_FILE_INFO's fixed part is 80 bytes (size, file_size, physical_size, three
    // 16-byte EFI_TIMEs, attribute) and FileName follows it, so the required size is ALWAYS at
    // least 82. An 80-byte buffer therefore fails with EFI_BUFFER_TOO_SMALL on every compliant
    // firmware and this function always fell through to the seek-to-end branch below - which is
    // not a rare path but the only path real hardware ever takes.
    let mut info_buf = [0u8; 512];
    let mut info_size: usize = info_buf.len();
    let mut info_guid = file::INFO_ID;
    if ((*f).get_info)(
        f,
        &mut info_guid,
        &mut info_size,
        info_buf.as_mut_ptr() as *mut core::ffi::c_void,
    ) == efi::Status::SUCCESS
        && info_size >= 16
    {
        return Some(u64::from_le_bytes(info_buf[8..16].try_into().unwrap()));
    }

    // Seek-to-end fallback - but put the cursor back. A SIZE PROBE MUST NOT MOVE THE CURSOR:
    // leaving it at EOF made a 1.4 MB corpus that was present on the volume read back as zero
    // bytes, because the read that followed started at the end of the file.
    let mut saved: u64 = 0;
    let had_saved = ((*f).get_position)(f, &mut saved) == efi::Status::SUCCESS;
    let mut out = None;
    if ((*f).set_position)(f, u64::MAX) == efi::Status::SUCCESS {
        let mut pos: u64 = 0;
        if ((*f).get_position)(f, &mut pos) == efi::Status::SUCCESS {
            out = Some(pos);
        }
    }
    if had_saved {
        let _ = ((*f).set_position)(f, saved);
    }
    out
}

/// Size of a named file on the boot volume, or None when it is not there.
///
/// SAFETY: as `open_root`.
pub unsafe fn size_named(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
    name: &str,
) -> Option<u64> {
    let root = open_root(image_handle, bs)?;
    let f = open_named(root, name);
    let out = match f {
        Some(fh) => file_size(fh),
        None => None,
    };
    if let Some(fh) = f {
        ((*fh).close)(fh);
    }
    ((*root).close)(root);
    out
}

/// Read up to `buf.len()` bytes of a named file. Returns how many bytes were read; None when the
/// file is absent or the root cannot be reached (a failure and an empty file are different things).
///
/// SAFETY: as `open_root`.
pub unsafe fn read_named(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
    name: &str,
    buf: &mut [u8],
) -> Option<usize> {
    let root = open_root(image_handle, bs)?;
    let fh = open_named(root, name);
    let out = match fh {
        Some(f) => {
            let want = match file_size(f) {
                Some(s) => (s as u64).min(buf.len() as u64) as usize,
                None => buf.len(),
            };
            // Read from the start. This is stated rather than inherited: the size probe above used
            // to leave the cursor at EOF for anyone who did not reset it first.
            let _ = ((*f).set_position)(f, 0);
            let mut total = 0usize;
            while total < want {
                let mut chunk = want - total;
                let at = total;
                if ((*f).read)(
                    f,
                    &mut chunk,
                    buf[at..].as_mut_ptr() as *mut core::ffi::c_void,
                ) != efi::Status::SUCCESS
                    || chunk == 0
                {
                    break;
                }
                total += chunk;
            }
            Some(total)
        }
        None => None,
    };
    if let Some(f) = fh {
        ((*f).close)(f);
    }
    ((*root).close)(root);
    out
}

/// Returns the expected NAND fabric geometry: (trit_count, expected_bytes).
pub fn get_nand_geometry() -> (u64, u64) {
    (NAND_TRIT_COUNT, NAND_EXPECTED_BYTES)
}

/// Validates whether the measured NAND volume size matches expected K3 fabric geometry.
pub fn verify_nand_geometry(actual_bytes: u64) -> bool {
    actual_bytes >= NAND_EXPECTED_BYTES
}

// ── added here from this machine's tree (G194) ────────────────────────────────────────────────────
/// Compatibility wrapper: `hw_truth` passes the image handle, `main.rs` passes only boot services.
/// Changing the original signature would break the two 1-arg callers in main.rs, so the extra argument is
/// accepted and ignored here - it is not used to derive any measurement.
pub unsafe fn nand_status_with_handle(
    _image_handle: efi::Handle,
    bs: *mut efi::BootServices,
) -> NandStatus {
    nand_status(bs)
}

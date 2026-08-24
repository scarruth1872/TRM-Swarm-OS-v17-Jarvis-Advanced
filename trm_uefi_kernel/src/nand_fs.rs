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
}

/// Open the NAND fabric file on the boot volume and report its state.
/// SAFETY: bs must be live Boot Services; image_handle is the loaded image.
pub unsafe fn nand_status(image_handle: efi::Handle, bs: *mut efi::BootServices) -> NandStatus {
    let mut status = NandStatus {
        mounted: false,
        size_bytes: 0,
        trits: 0,
        first_sector_ok: false,
        last_sector_ok: false,
    };

    // 1. LoadedImage -> device_handle of the boot volume
    let mut loaded_guid = loaded_image::PROTOCOL_GUID;
    let mut loaded: *mut loaded_image::Protocol = core::ptr::null_mut();
    let st = ((*bs).handle_protocol)(
        image_handle,
        &mut loaded_guid,
        &mut loaded as *mut *mut loaded_image::Protocol as *mut *mut core::ffi::c_void,
    );
    if st != efi::Status::SUCCESS || loaded.is_null() {
        return status;
    }
    let device = (*loaded).device_handle;

    // 2. SimpleFileSystem on the device
    let mut fs_guid = simple_file_system::PROTOCOL_GUID;
    let mut fs: *mut simple_file_system::Protocol = core::ptr::null_mut();
    let st = ((*bs).handle_protocol)(
        device,
        &mut fs_guid,
        &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
    );
    if st != efi::Status::SUCCESS || fs.is_null() {
        return status;
    }

    // 3. OpenVolume -> root directory
    let mut root: *mut file::Protocol = core::ptr::null_mut();
    let st = ((*fs).open_volume)(fs, &mut root);
    if st != efi::Status::SUCCESS || root.is_null() {
        return status;
    }

    // 4. Open \swarm_v2_nand_fabric.k3 read-only
    let file_name: [efi::Char16; 24] = [
        0x5C, 0x73, 0x77, 0x61, 0x72, 0x6D, 0x5F, 0x76, 0x32, 0x5F, 0x6E, 0x61, 0x6E, 0x64,
        0x5F, 0x66, 0x61, 0x62, 0x72, 0x69, 0x63, 0x2E, 0x6B, 0x33,
    ]; // r#"\swarm_v2_nand_fabric.k3"#
    let mut fabric: *mut file::Protocol = core::ptr::null_mut();
    let st = ((*root).open)(
        root,
        &mut fabric,
        file_name.as_ptr() as *mut efi::Char16,
        file::MODE_READ,
        0,
    );
    if st != efi::Status::SUCCESS || fabric.is_null() {
        // not present - honest
        return status;
    }

    // 5. GetInfo -> file size
    let mut info_buf = [0u8; 80];
    let mut info_size: usize = info_buf.len();
    let mut info_guid = file::INFO_ID;
    let st = ((*fabric).get_info)(
        fabric,
        &mut info_guid,
        &mut info_size,
        info_buf.as_mut_ptr() as *mut core::ffi::c_void,
    );
    if st == efi::Status::SUCCESS && info_size >= 8 {
        let file_size = u64::from_le_bytes(info_buf[8..16].try_into().unwrap());
        status.size_bytes = file_size;
        status.trits = file_size * 2; // 2 trits per byte
        status.mounted = true;
    } else {
        // fallback: seek to end
        let end_pos: u64 = u64::MAX;
        let _ = ((*fabric).set_position)(fabric, end_pos);
        let mut pos: u64 = 0;
        if ((*fabric).get_position)(fabric, &mut pos) == efi::Status::SUCCESS {
            status.size_bytes = pos;
            status.trits = pos * 2;
            status.mounted = true;
        }
    }

    // 6. Sector reads (first + last 512 bytes)
    if status.size_bytes > 0 {
        let mut first = [0u8; 512];
        let mut first_size: usize = 512;
        let pos0: u64 = 0;
        ((*fabric).set_position)(fabric, pos0);
        status.first_sector_ok = ((*fabric).read)(
            fabric,
            &mut first_size,
            first.as_mut_ptr() as *mut core::ffi::c_void,
        ) == efi::Status::SUCCESS && first_size > 0;

        if status.size_bytes >= 512 {
            let mut last = [0u8; 512];
            let mut last_size: usize = 512;
            let pos_last: u64 = status.size_bytes - 512;
            ((*fabric).set_position)(fabric, pos_last);
            status.last_sector_ok = ((*fabric).read)(
                fabric,
                &mut last_size,
                last.as_mut_ptr() as *mut core::ffi::c_void,
            ) == efi::Status::SUCCESS && last_size > 0;
        }
    }

    ((*fabric).close)(fabric);
    ((*root).close)(root);
    status
}

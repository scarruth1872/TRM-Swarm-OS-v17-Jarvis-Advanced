//! TRM Swarm OS - the bare-metal bridge.
//!
//! A Linux kernel is itself a UEFI application (PE32+, machine 0x8664, subsystem 10 - measured, not
//! assumed). So the firmware can load one directly, which is exactly how a Linux distribution boots
//! "fully" from a flash drive: the kernel is handed the machine, calls ExitBootServices, and from
//! then on uses ITS OWN drivers rather than the firmware's. That is the whole difference.
//!
//! The firmware alone exposes FAT (measured: our UEFI kernel can see FAT volumes and nothing else).
//! A Linux kernel carries ext4, xfs, btrfs, ntfs3, exfat, f2fs, ... in the same image, so every
//! volume on the machine becomes readable - which is the requirement the pure-UEFI path cannot meet.
//!
//! This loader does the four things a bootloader does, and nothing else:
//!   1. find this image's own volume (LoadedImage.device_handle -> SimpleFileSystem)
//!   2. read the kernel file into pool memory
//!   3. LoadImage it (a memory buffer means no device path)
//!   4. put the cmdline on the NEW image's LoadedImage.load_options, then StartImage
//!
//! Only the initrd's PATH is passed on. The kernel's EFI stub opens the volume through the firmware's
//! own file protocol, so copying it here would create a second source for the same bytes - and the
//! two would drift.

#![no_std]
#![no_main]

use core::ffi::c_void;
use core::panic::PanicInfo;
use core::ptr;
use r_efi::efi;
use r_efi::protocols::{file, loaded_image, simple_file_system};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// The kernel to hand the machine to.
const KERNEL_PATH: &str = "\\EFI\\TRM\\vmlinuz";
/// The initrd the kernel reads for itself; only its path travels with us.
const INITRD_PATH: &str = "\\EFI\\TRM\\initrd.img";
/// rdinit picks our PID 1; console=tty0 puts output on the display the firmware was already using
/// (there is no usable serial on this hardware).
const CMDLINE: &str = "initrd=\\EFI\\TRM\\initrd.img console=tty0 console=ttyS0,115200 rdinit=/init";

/// Encode ASCII to NUL-terminated UTF-16, mapping '/' to '\'.
fn to_utf16(name: &str, out: &mut [u16]) -> usize {
    let mut n = 0usize;
    for b in name.bytes() {
        if n + 1 >= out.len() {
            break;
        }
        out[n] = if b == b'/' { b'\\' as u16 } else { b as u16 };
        n += 1;
    }
    out[n] = 0;
    n
}

/// Encode ASCII to NUL-terminated UTF-16, keeping '/' as '/'.
///
/// EFI FILE paths need '\' and `to_utf16` does that; a KERNEL COMMAND LINE is POSIX and must keep
/// '/'. Translating it here turned `rdinit=/init` into `rdinit=\init`, which is not a path, so the
/// kernel found no init, fell back to mounting a root filesystem and panicked with
/// "VFS: Unable to mount root fs on unknown-block(0,0)" - measured on the boot.
fn to_ucs2(s: &str, out: &mut [u16]) -> usize {
    let mut n = 0usize;
    for b in s.bytes() {
        if n + 1 >= out.len() {
            break;
        }
        out[n] = b as u16;
        n += 1;
    }
    out[n] = 0;
    n
}

#[export_name = "efi_main"]
pub extern "efiapi" fn efi_main(image: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    unsafe { chainload(image, st) }
}

/// 1. this image's volume  2. the kernel into pool memory.
///
/// SAFETY: bs must be live Boot Services; image is this loaded image.
unsafe fn read_kernel(image: efi::Handle, bs: *mut efi::BootServices) -> Option<(*mut c_void, usize)> {
    let mut li_guid = loaded_image::PROTOCOL_GUID;
    let mut li: *mut loaded_image::Protocol = ptr::null_mut();
    if ((*bs).handle_protocol)(
        image,
        &mut li_guid,
        &mut li as *mut *mut loaded_image::Protocol as *mut *mut c_void,
    ) != efi::Status::SUCCESS
        || li.is_null()
    {
        return None;
    }

    let mut fs_guid = simple_file_system::PROTOCOL_GUID;
    let mut fs: *mut simple_file_system::Protocol = ptr::null_mut();
    if ((*bs).handle_protocol)(
        (*li).device_handle,
        &mut fs_guid,
        &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut c_void,
    ) != efi::Status::SUCCESS
        || fs.is_null()
    {
        return None;
    }

    let mut root: *mut file::Protocol = ptr::null_mut();
    if ((*fs).open_volume)(fs, &mut root) != efi::Status::SUCCESS || root.is_null() {
        return None;
    }

    let mut name = [0u16; 256];
    to_utf16(KERNEL_PATH, &mut name);
    let mut kf: *mut file::Protocol = ptr::null_mut();
    if ((*root).open)(
        root,
        &mut kf,
        name.as_ptr() as *mut efi::Char16,
        file::MODE_READ,
        0,
    ) != efi::Status::SUCCESS
        || kf.is_null()
    {
        ((*root).close)(root);
        return None;
    }

    // EFI_FILE_INFO's fixed part is 80 bytes and FileName follows it, so a smaller buffer can never
    // succeed - the same defect that once made the corpus unreadable in the UEFI kernel.
    let mut info = [0u8; 512];
    let mut info_size: usize = info.len();
    let mut info_guid = file::INFO_ID;
    let mut ksize: usize = 0;
    if ((*kf).get_info)(
        kf,
        &mut info_guid,
        &mut info_size,
        info.as_mut_ptr() as *mut c_void,
    ) == efi::Status::SUCCESS
        && info_size >= 16
    {
        ksize = u64::from_le_bytes(info[8..16].try_into().unwrap_or([0; 8])) as usize;
    }
    if ksize == 0 {
        ((*kf).close)(kf);
        ((*root).close)(root);
        return None;
    }

    let mut buf: *mut c_void = ptr::null_mut();
    if ((*bs).allocate_pool)(efi::LOADER_DATA, ksize, &mut buf) != efi::Status::SUCCESS
        || buf.is_null()
    {
        ((*kf).close)(kf);
        ((*root).close)(root);
        return None;
    }

    let dst = core::slice::from_raw_parts_mut(buf as *mut u8, ksize);
    let mut total = 0usize;
    while total < ksize {
        let mut chunk = ksize - total;
        if ((*kf).read)(kf, &mut chunk, dst[total..].as_mut_ptr() as *mut c_void)
            != efi::Status::SUCCESS
            || chunk == 0
        {
            break;
        }
        total += chunk;
    }
    ((*kf).close)(kf);
    ((*root).close)(root);
    if total != ksize {
        return None;
    }
    Some((buf, ksize))
}

/// 3. LoadImage the kernel  4. put the cmdline on the new image and StartImage it.
///
/// SAFETY: bs must be live Boot Services; image is this loaded image.
unsafe fn chainload(image: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    let bs = (*st).boot_services;
    let (buf, ksize) = match read_kernel(image, bs) {
        Some(v) => v,
        None => return efi::Status::LOAD_ERROR,
    };

    // A memory buffer means there is no device path to pass.
    let mut new_image: efi::Handle = ptr::null_mut();
    if ((*bs).load_image)(
        efi::Boolean::FALSE,
        image,
        ptr::null_mut(),
        buf,
        ksize,
        &mut new_image,
    ) != efi::Status::SUCCESS
        || new_image.is_null()
    {
        return efi::Status::LOAD_ERROR;
    }

    // LoadOptions belong to the NEW image, not to start_image - this is the piece that carries the
    // kernel command line, and therefore the initrd path, into the kernel's EFI stub.
    let mut opts = [0u16; 256];
    let n = to_ucs2(CMDLINE, &mut opts); // POSIX cmdline: '/' must survive
    let mut nli_guid = loaded_image::PROTOCOL_GUID;
    let mut nli: *mut loaded_image::Protocol = ptr::null_mut();
    if ((*bs).handle_protocol)(
        new_image,
        &mut nli_guid,
        &mut nli as *mut *mut loaded_image::Protocol as *mut *mut c_void,
    ) != efi::Status::SUCCESS
        || nli.is_null()
    {
        ((*bs).unload_image)(new_image);
        return efi::Status::LOAD_ERROR;
    }
    (*nli).load_options_size = ((n + 1) * 2) as u32; // UCS-2, NUL included
    (*nli).load_options = opts.as_ptr() as *mut c_void;

    // The kernel resolves `initrd=` through the volume on its OWN LoadedImage. Loading from a memory
    // buffer means there is no device path, so the firmware left device_handle NULL and the stub
    // reported "Failed to handle fs_proto" / "Failed to load initrd: EFI_INVALID_PARAMETER" - measured.
    // Point it at the volume this image came from, which is the volume the kernel file is on.
    let mut oli_guid = loaded_image::PROTOCOL_GUID;
    let mut oli: *mut loaded_image::Protocol = ptr::null_mut();
    if ((*bs).handle_protocol)(
        image,
        &mut oli_guid,
        &mut oli as *mut *mut loaded_image::Protocol as *mut *mut c_void,
    ) == efi::Status::SUCCESS
        && !oli.is_null()
    {
        (*nli).device_handle = (*oli).device_handle;
    }

    // Hand over the machine. On success the kernel has taken the hardware and this never meaningfully
    // returns; on failure the firmware is still alive and we can unwind.
    let mut exit_size: usize = 0;
    let mut exit_data: *mut efi::Char16 = ptr::null_mut();
    let status = ((*bs).start_image)(new_image, &mut exit_size, &mut exit_data);
    if status != efi::Status::SUCCESS {
        ((*bs).unload_image)(new_image);
    }
    status
}

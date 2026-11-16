//! TRM Swarm OS — Bare-Metal Disk & Filesystem Explorer (no_std).
//! Direct UEFI access to physical NVMe drives, USB volumes, partitions, and directories.
//! Uses r_efi::protocols::{simple_file_system, file, loaded_image}.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr;
use r_efi::efi;
use r_efi::protocols::{file, loaded_image, simple_file_system};

/// Enumerate all physical storage volumes and filesystems on the machine.
pub unsafe fn list_drives(bs: *mut efi::BootServices) -> Vec<String> {
    let mut out = Vec::new();
    out.push(String::from("STORAGE: Physical Drive & Volume Explorer (UEFI Long Mode)"));

    let mut fs_guid = simple_file_system::PROTOCOL_GUID;
    let mut n_handles: usize = 0;
    let mut handles: *mut efi::Handle = ptr::null_mut();

    let st = ((*bs).locate_handle_buffer)(
        efi::BY_PROTOCOL,
        &mut fs_guid,
        ptr::null_mut(),
        &mut n_handles,
        &mut handles,
    );

    if st != efi::Status::SUCCESS || handles.is_null() || n_handles == 0 {
        out.push(String::from("  [!] No UEFI SimpleFileSystem volumes detected on bus."));
        return out;
    }

    out.push(format!("  Detected {} active physical filesystem volume(s):", n_handles));

    for i in 0..n_handles {
        let handle = *handles.add(i);
        let mut fs: *mut simple_file_system::Protocol = ptr::null_mut();

        let st = ((*bs).handle_protocol)(
            handle,
            &mut fs_guid,
            &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
        );

        if st != efi::Status::SUCCESS || fs.is_null() {
            out.push(format!("  * FS{}: Handle {:p} [Protocol Error]", i, handle));
            continue;
        }

        let mut root: *mut file::Protocol = ptr::null_mut();
        let st = ((*fs).open_volume)(fs, &mut root);
        if st != efi::Status::SUCCESS || root.is_null() {
            out.push(format!("  * FS{}: Handle {:p} [Volume Open Error]", i, handle));
            continue;
        }

        // Query Volume Information
        let mut info_buf = [0u8; 128];
        let mut info_size = info_buf.len();
        let mut sys_info_guid = file::SYSTEM_INFO_ID;
        let st = ((*root).get_info)(
            root,
            &mut sys_info_guid,
            &mut info_size,
            info_buf.as_mut_ptr() as *mut core::ffi::c_void,
        );

        if st == efi::Status::SUCCESS && info_size >= 32 {
            let vol_size = u64::from_le_bytes(info_buf[16..24].try_into().unwrap_or([0; 8]));
            let free_space = u64::from_le_bytes(info_buf[24..32].try_into().unwrap_or([0; 8]));
            let vol_gb = vol_size as f32 / (1024.0 * 1024.0 * 1024.0);
            let free_gb = free_space as f32 / (1024.0 * 1024.0 * 1024.0);

            // Extract volume label (null-terminated UTF-16 at offset 36)
            let mut label = String::new();
            let mut k = 36;
            while k + 1 < info_size {
                let u = u16::from_le_bytes([info_buf[k], info_buf[k + 1]]);
                if u == 0 || label.len() >= 16 {
                    break;
                }
                label.push(if u >= 0x20 && u <= 0x7E { u as u8 as char } else { '?' });
                k += 2;
            }
            if label.is_empty() {
                label = String::from("TRM_VOLUME");
            }

            out.push(format!(
                "  * FS{}: [{}] Vol: {:.1} GB | Free: {:.1} GB | MOUNTED",
                i, label, vol_gb, free_gb
            ));
        } else {
            out.push(format!("  * FS{}: Physical Volume Handle {:p} | MOUNTED OK", i, handle));
        }

        let _ = ((*root).close)(root);
    }

    let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);
    out.push(String::from("HINT: Explore any drive with 'ls FS0:\\' or 'ls FS1:\\EFI'."));
    out
}

/// Helper: Open the root directory protocol for a given volume index or boot device.
unsafe fn open_volume_root(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
    vol_idx: Option<usize>,
) -> Result<*mut file::Protocol, String> {
    let mut fs_guid = simple_file_system::PROTOCOL_GUID;

    if let Some(target_idx) = vol_idx {
        let mut n_handles: usize = 0;
        let mut handles: *mut efi::Handle = ptr::null_mut();

        let st = ((*bs).locate_handle_buffer)(
            efi::BY_PROTOCOL,
            &mut fs_guid,
            ptr::null_mut(),
            &mut n_handles,
            &mut handles,
        );

        if st != efi::Status::SUCCESS || handles.is_null() || n_handles == 0 {
            return Err(String::from("No SimpleFileSystem volumes found."));
        }

        if target_idx >= n_handles {
            let err = format!("Volume FS{}: not found (0..{} exist).", target_idx, n_handles - 1);
            let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);
            return Err(err);
        }

        let handle = *handles.add(target_idx);
        let _ = ((*bs).free_pool)(handles as *mut core::ffi::c_void);

        let mut fs: *mut simple_file_system::Protocol = ptr::null_mut();
        let st = ((*bs).handle_protocol)(
            handle,
            &mut fs_guid,
            &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
        );
        if st != efi::Status::SUCCESS || fs.is_null() {
            return Err(format!("Could not acquire filesystem on FS{}:", target_idx));
        }

        let mut root: *mut file::Protocol = ptr::null_mut();
        let st = ((*fs).open_volume)(fs, &mut root);
        if st != efi::Status::SUCCESS || root.is_null() {
            return Err(format!("Could not open root on FS{}:", target_idx));
        }

        Ok(root)
    } else {
        let mut loaded_guid = loaded_image::PROTOCOL_GUID;
        let mut loaded: *mut loaded_image::Protocol = ptr::null_mut();
        let st = ((*bs).handle_protocol)(
            image_handle,
            &mut loaded_guid,
            &mut loaded as *mut *mut loaded_image::Protocol as *mut *mut core::ffi::c_void,
        );
        if st != efi::Status::SUCCESS || loaded.is_null() {
            return Err(String::from("Failed to acquire LoadedImage protocol."));
        }

        let device = (*loaded).device_handle;
        let mut fs: *mut simple_file_system::Protocol = ptr::null_mut();
        let st = ((*bs).handle_protocol)(
            device,
            &mut fs_guid,
            &mut fs as *mut *mut simple_file_system::Protocol as *mut *mut core::ffi::c_void,
        );
        if st != efi::Status::SUCCESS || fs.is_null() {
            return Err(String::from("Failed to acquire SimpleFileSystem on boot device."));
        }

        let mut root: *mut file::Protocol = ptr::null_mut();
        let st = ((*fs).open_volume)(fs, &mut root);
        if st != efi::Status::SUCCESS || root.is_null() {
            return Err(String::from("Failed to open boot volume root."));
        }

        Ok(root)
    }
}

/// Helper: Parse path string for optional "FS<N>:" volume prefix.
fn parse_volume_and_path(input: &str) -> (Option<usize>, &str) {
    let trimmed = input.trim();
    let b = trimmed.as_bytes();
    if b.len() >= 4 && (b[0] == b'F' || b[0] == b'f') && (b[1] == b'S' || b[1] == b's') {
        // Multi-digit: a machine with ten or more volumes must still be addressable, and
        // FS10: silently parsing as FS1: would open the WRONG volume.
        let mut i = 2usize;
        let mut idx: usize = 0;
        let mut digits = 0usize;
        while i < b.len() && b[i].is_ascii_digit() {
            idx = idx.saturating_mul(10).saturating_add((b[i] - b'0') as usize);
            digits += 1;
            i += 1;
        }
        if digits > 0 && i < b.len() && b[i] == b':' {
            let sub = trimmed[i + 1..].trim_start_matches(|c| c == '/' || c == '\\');
            return (Some(idx), sub);
        }
    }
    (None, trimmed.trim_start_matches(|c| c == '/' || c == '\\'))
}

/// List files and subdirectories on any volume path (e.g. 'FS0:\', 'EFI', 'FS1:\Windows').
pub unsafe fn list_directory(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
    path_arg: &str,
    offset: usize,
) -> Vec<String> {
    let mut out = Vec::new();
    let (vol_opt, subpath) = parse_volume_and_path(path_arg);

    let vol_label = match vol_opt {
        Some(idx) => format!("FS{}:", idx),
        None => String::from("BOOT_VOL:"),
    };

    let display_path = if subpath.is_empty() { "/" } else { subpath };
    out.push(format!("DIRECTORY: Physical Listing on {} /{}", vol_label, display_path));

    let root = match open_volume_root(image_handle, bs, vol_opt) {
        Ok(r) => r,
        Err(e) => {
            out.push(format!("  [!] {}", e));
            return out;
        }
    };

    let mut target_dir = root;
    let clean_path = subpath.trim_matches(|c| c == '/' || c == '\\' || c == ' ');

    if !clean_path.is_empty() {
        let mut utf16_path = [0u16; 64];
        let mut pi = 0;
        for b in clean_path.bytes() {
            if pi >= 62 {
                break;
            }
            utf16_path[pi] = if b == b'/' { b'\\' as u16 } else { b as u16 };
            pi += 1;
        }
        utf16_path[pi] = 0;

        let mut sub_dir: *mut file::Protocol = ptr::null_mut();
        let st = ((*root).open)(
            root,
            &mut sub_dir,
            utf16_path.as_ptr() as *mut efi::Char16,
            file::MODE_READ,
            0,
        );

        if st != efi::Status::SUCCESS || sub_dir.is_null() {
            out.push(format!("  [!] Path '{}' not found on {}.", clean_path, vol_label));
            let _ = ((*root).close)(root);
            return out;
        }
        target_dir = sub_dir;
    }

    // The LCARS feed is 18 lines: page rather than truncate, and keep counting so the total is
    // the directory's real size and not the size of what happens to fit on screen.
    const PAGE: usize = 14;
    let mut seen = 0usize;
    let mut shown = 0usize;
    let mut buf = [0u8; 512];

    loop {
        let mut buf_size = buf.len();
        let st = ((*target_dir).read)(
            target_dir,
            &mut buf_size,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
        );

        if st != efi::Status::SUCCESS || buf_size == 0 {
            break;
        }

        if buf_size >= 82 {
            let file_size = u64::from_le_bytes(buf[8..16].try_into().unwrap_or([0; 8]));
            let attr = u64::from_le_bytes(buf[72..80].try_into().unwrap_or([0; 8]));
            let is_dir = (attr & 0x10) != 0;

            let mut fname = String::new();
            let mut k = 80;
            while k + 1 < buf_size {
                let ch = u16::from_le_bytes([buf[k], buf[k + 1]]);
                if ch == 0 {
                    break;
                }
                if ch >= 0x20 && ch <= 0x7E {
                    fname.push(ch as u8 as char);
                }
                k += 2;
            }

            if !fname.is_empty() && fname != "." && fname != ".." {
                seen += 1;
                if seen > offset && shown < PAGE {
                    shown += 1;
                    if is_dir {
                        out.push(format!("  <DIR>           {}/", fname));
                    } else {
                        let sz_kb = file_size as f32 / 1024.0;
                        if sz_kb >= 1024.0 {
                            out.push(format!("  {:>8.1} MB     {}", sz_kb / 1024.0, fname));
                        } else {
                            out.push(format!("  {:>8.1} KB     {}", sz_kb, fname));
                        }
                    }
                }
            }
        }

        if shown >= PAGE && seen > offset + PAGE {
            // keep scanning: the footer must report the real total, not the page size
        }
    }

    if seen == 0 {
        out.push(String::from("  (Directory is empty)"));
    }

    if target_dir != root {
        let _ = ((*target_dir).close)(target_dir);
    }
    let _ = ((*root).close)(root);

    if seen > offset + shown {
        out.push(format!(
            "SHOWING: {}-{} of {}  (type 'ls {} {}' for more)",
            offset + 1,
            offset + shown,
            seen,
            path_arg,
            offset + shown
        ));
    } else if seen > 0 {
        out.push(format!("TOTAL: all {} entries shown.", seen));
    }
    out
}

/// Read text from a physical file into LCARS lines across any volume (e.g. 'cat FS0:\startup.nsh').
pub unsafe fn read_file(
    image_handle: efi::Handle,
    bs: *mut efi::BootServices,
    path_arg: &str,
    offset: usize,
) -> Vec<String> {
    let mut out = Vec::new();
    let (vol_opt, subpath) = parse_volume_and_path(path_arg);

    let clean_path = subpath.trim_matches(|c| c == '/' || c == '\\' || c == ' ');
    if clean_path.is_empty() {
        out.push(String::from("USAGE: cat <file> [offset] (e.g. 'cat startup.nsh' or 'cat FS1:\\boot.log 4096')"));
        return out;
    }

    let vol_label = match vol_opt {
        Some(idx) => format!("FS{}:", idx),
        None => String::from("BOOT_VOL:"),
    };

    out.push(format!("FILE: Reading '{}' on {}", clean_path, vol_label));

    let root = match open_volume_root(image_handle, bs, vol_opt) {
        Ok(r) => r,
        Err(e) => {
            out.push(format!("  [!] {}", e));
            return out;
        }
    };

    // 256 code units: at 64 the path was silently cut at 62 chars, which resolves to a
    // different file (or none) rather than reporting that the path was too long.
    let mut utf16_path = [0u16; 256];
    let mut pi = 0;
    let too_long = clean_path.len() > 254;
    for b in clean_path.bytes() {
        if pi >= 254 {
            break;
        }
        utf16_path[pi] = if b == b'/' { b'\\' as u16 } else { b as u16 };
        pi += 1;
    }
    utf16_path[pi] = 0;
    if too_long {
        out.push(String::from("  [!] Path exceeds 254 chars and was cut; it may not be the file you meant."));
    }

    let mut f: *mut file::Protocol = ptr::null_mut();
    let st = ((*root).open)(
        root,
        &mut f,
        utf16_path.as_ptr() as *mut efi::Char16,
        file::MODE_READ,
        0,
    );

    if st != efi::Status::SUCCESS || f.is_null() {
        out.push(format!("  [!] File '{}' could not be opened on {}.", clean_path, vol_label));
        let _ = ((*root).close)(root);
        return out;
    }

    // The file's real size, so a page can state where it sits in the whole.
    let mut total: u64 = 0;
    let mut info_buf = [0u8; 128];
    let mut info_size = info_buf.len();
    let mut info_guid = file::INFO_ID;
    if ((*f).get_info)(
        f,
        &mut info_guid,
        &mut info_size,
        info_buf.as_mut_ptr() as *mut core::ffi::c_void,
    ) == efi::Status::SUCCESS
        && info_size >= 8
    {
        total = u64::from_le_bytes(info_buf[8..16].try_into().unwrap_or([0; 8]));
    }

    if offset as u64 >= total && total > 0 {
        out.push(format!("  (offset {} is past the end; file is {} bytes)", offset, total));
        let _ = ((*f).close)(f);
        let _ = ((*root).close)(root);
        return out;
    }

    let _ = ((*f).set_position)(f, offset as u64);

    // 4 KB per page is ~14 LCARS lines of 72 chars: the feed holds 18, so a page fits with room
    // for the header and the footer that says how to continue.
    const CHUNK: usize = 4096;
    let mut read_buf = [0u8; CHUNK];
    let mut read_len = read_buf.len();
    let st = ((*f).read)(
        f,
        &mut read_len,
        read_buf.as_mut_ptr() as *mut core::ffi::c_void,
    );

    if st == efi::Status::SUCCESS && read_len > 0 {
        let content = &read_buf[..read_len];
        let mut line = String::new();
        let mut line_count = 0usize;
        let mut consumed = 0usize;
        let mut stopped_early = false;

        for &b in content {
            consumed += 1;
            if b == b'\n' {
                if !line.is_empty() {
                    out.push(format!("  | {}", line));
                    line_count += 1;
                    line.clear();
                    if line_count >= 13 {
                        stopped_early = true;
                        break;
                    }
                }
            } else if b == b'\r' {
                // skip
            } else if b >= 0x20 && b <= 0x7E {
                if line.len() < 72 {
                    line.push(b as char);
                }
            }
        }
        if !line.is_empty() && !stopped_early {
            out.push(format!("  | {}", line));
        }

        let end = offset as u64 + (read_len as u64);
        if stopped_early || end < total {
            out.push(format!(
                "BYTES: {}-{} of {}. Next: 'cat {} {}'",
                offset,
                offset + consumed,
                total,
                path_arg,
                offset + consumed
            ));
        } else {
            out.push(format!("EOF: whole file shown ({} bytes on {}).", total, vol_label));
        }
    } else {
        out.push(String::from("  (File is 0 bytes or unreadable)"));
    }

    let _ = ((*f).close)(f);
    let _ = ((*root).close)(root);
    out
}

pub fn is_disk_explorer_ready() -> bool {
    true
}

pub fn format_volume_label(vol_idx: usize) -> String {
    format!("FS{}:", vol_idx)
}

pub fn get_disk_explorer_metrics() -> (u64, u64) {
    (0x4D4F4E4144000001, 1)
}


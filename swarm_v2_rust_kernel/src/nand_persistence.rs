//! TRM Swarm OS v18 — Real-Hardware NAND K3 Persistence Engine
//! Memory-mapped NVMe zero-copy persistence for 153,523,393 K3 trits.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use memmap2::MmapMut;

pub const DEFAULT_NAND_PATH: &str = "swarm_v2_nand_fabric.k3";
pub const MANDATORY_TRIT_COUNT: usize = 153523393;

pub struct NANDK3Fabric {
    pub file_path: String,
    pub total_trits: usize,
    pub mmap: Option<MmapMut>,
}

impl NANDK3Fabric {
    pub fn open_or_create<P: AsRef<Path>>(path: P, total_trits: usize) -> Self {
        let p_str = path.as_ref().to_string_lossy().to_string();
        
        // 2 trits per byte packing -> total_bytes = total_trits / 2
        let byte_capacity = (total_trits + 1) / 2;
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .expect("Failed to open NAND file");
            
        let metadata = file.metadata().expect("Failed to read metadata");
        if metadata.len() < byte_capacity as u64 {
            file.set_len(byte_capacity as u64).expect("Failed to extend NAND file");
        }
        
        let mmap = unsafe { MmapMut::map_mut(&file).ok() };
        
        NANDK3Fabric {
            file_path: p_str,
            total_trits,
            mmap,
        }
    }

    pub fn write_trit(&mut self, index: usize, trit_val: i8) -> bool {
        if index >= self.total_trits {
            return false;
        }
        
        if let Some(ref mut mmap) = self.mmap {
            let byte_idx = index / 2;
            let is_high = (index % 2) == 1;
            let val_encoded = match trit_val {
                1 => 0b11u8,   // +1 TRUE
                -1 => 0b00u8,  // -1 FALSE
                _ => 0b01u8,   //  0 UNKNOWN
            };
            
            let mut curr = mmap[byte_idx];
            if is_high {
                curr = (curr & 0x0F) | (val_encoded << 4);
            } else {
                curr = (curr & 0xF0) | val_encoded;
            }
            mmap[byte_idx] = curr;
            return true;
        }
        false
    }

    pub fn flush(&mut self) -> bool {
        if let Some(ref mut mmap) = self.mmap {
            return mmap.flush().is_ok();
        }
        false
    }
}

#[no_mangle]
pub extern "C" fn rust_nand_append_decision(index: usize, trit_val: i8) -> i32 {
    let mut nand = NANDK3Fabric::open_or_create(DEFAULT_NAND_PATH, MANDATORY_TRIT_COUNT);
    if nand.write_trit(index, trit_val) {
        if nand.flush() { 0 } else { -1 }
    } else {
        -2
    }
}

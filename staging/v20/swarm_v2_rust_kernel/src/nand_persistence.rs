//! TRM Swarm OS v18 — Real-Hardware NAND K3 Persistence Engine
//! Memory-mapped NVMe zero-copy persistence for 153,523,393 K3 trits.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use memmap2::MmapMut;

pub const DEFAULT_NAND_PATH: &str = "swarm_v2_nand_fabric.k3";

/// Fraction of nibbles that may be illegal before an existing store is refused.
///
/// A raw K3 array is 100% legal codes (0b00/0b01/0b11) by construction, so a single
/// illegal nibble proves the path does NOT hold a K3 array. The .k3 path was shared with
/// a truncating TurboVec writer and six UTF-8 text appenders, which is how a live fabric
/// came to read 46.78% illegal while every reader kept mapping it happily.
pub const MAX_ILLEGAL_NIBBLE_FRACTION: f64 = 0.0;

/// Count (illegal_nibbles, nibbles_scanned) over a raw byte slice.
pub fn count_illegal_nibbles(bytes: &[u8]) -> (usize, usize) {
    let mut illegal = 0usize;
    for b in bytes {
        let lo = b & 0x0F;
        let hi = (b >> 4) & 0x0F;
        if lo > 0b11 || lo == 0b10 {
            illegal += 1;
        }
        if hi > 0b11 || hi == 0b10 {
            illegal += 1;
        }
    }
    (illegal, bytes.len() * 2)
}
pub const MANDATORY_TRIT_COUNT: usize = 153523393;

pub struct NANDK3Fabric {
    pub file_path: String,
    pub total_trits: usize,
    pub mmap: Option<MmapMut>,
    /// Set when the path was refused: it does not hold a K3 trit array. While this is
    /// `Some`, `mmap` is `None`, so every read returns `None` and every write returns
    /// `false` — the fabric refuses to participate rather than silently corrupt.
    pub refusal: Option<String>,
}

impl NANDK3Fabric {
    pub fn open_or_create<P: AsRef<Path>>(path: P, total_trits: usize) -> Self {
        let p_str = path.as_ref().to_string_lossy().to_string();

        // 2 trits per byte packing -> total_bytes = total_trits / 2
        let byte_capacity = (total_trits + 1) / 2;

        // An existing non-empty file must be proven to be a K3 array before we trust it.
        let pre_existing = std::fs::metadata(path.as_ref())
            .map(|m| m.len() > 0)
            .unwrap_or(false);

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

        // Validate BEFORE handing out a mapping: mapping an incompatible format and
        // reading it as trits is silent corruption, not a graceful degradation.
        let mut refusal = None;
        if pre_existing {
            if let Some(ref m) = mmap {
                let end = byte_capacity.min(m.len());
                let (illegal, scanned) = count_illegal_nibbles(&m[..end]);
                let frac = illegal as f64 / scanned.max(1) as f64;
                if frac > MAX_ILLEGAL_NIBBLE_FRACTION {
                    refusal = Some(format!(
                        "{} is not a K3 trit array: {} of {} nibbles ({:.2}%) are illegal codes. \
                         Refused. The .k3 path is Rust-owned and may only hold 2 trits/byte of \
                         0b00/0b01/0b11 — see council MOTION-007 and swarm_v2/core/nand_fabric_paths.py. \
                         Repair by salvage (map illegal nibbles to UNKNOWN); never overwrite.",
                        p_str, illegal, scanned, 100.0 * frac
                    ));
                }
            }
        }

        NANDK3Fabric {
            file_path: p_str,
            total_trits,
            mmap: if refusal.is_some() { None } else { mmap },
            refusal,
        }
    }

    /// `None` when the store validated; otherwise why it was refused.
    pub fn refusal_reason(&self) -> Option<&str> {
        self.refusal.as_deref()
    }

    /// True only if the fabric validated and is safe to read/write.
    /// (illegal, scanned) over the mapped K3 region; None when not mapped.
    pub fn illegal_nibble_scan(&self) -> Option<(usize, usize)> {
        let bytes = self.mmap.as_ref()?;
        let n = ((self.total_trits + 1) / 2).min(bytes.len());
        Some(count_illegal_nibbles(&bytes[..n]))
    }

    pub fn is_usable(&self) -> bool {
        self.refusal.is_none() && self.mmap.is_some()
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

    /// Read one trit back out of the fabric.
    ///
    /// A write that is never read back is not verified (council MOTION-007, Verify
    /// condition), so the persistence path needs a reader as well as a writer.
    /// Returns None if the index is out of range or the fabric is not mapped.
    pub fn read_trit(&self, index: usize) -> Option<i8> {
        if index >= self.total_trits {
            return None;
        }
        let mmap = self.mmap.as_ref()?;
        let byte_idx = index / 2;
        let nibble = if (index % 2) == 1 {
            (mmap[byte_idx] >> 4) & 0x0F
        } else {
            mmap[byte_idx] & 0x0F
        };
        match nibble {
            0b11 => Some(1),  // +1 TRUE
            0b00 => Some(-1), // -1 FALSE
            0b01 => Some(0),  //  0 UNKNOWN
            _ => None,        // not a code this engine can produce
        }
    }

    /// Index of the first illegal trit, or `None` if every nibble is a legal K3 code
    /// (0b00/0b01/0b11). The root store was repaired by the salvage path and now reads
    /// clean; this stays as the standing check.
    pub fn first_illegal_trit(&self) -> Option<usize> {
        let mmap = self.mmap.as_ref()?;
        for (b, byte) in mmap.iter().enumerate() {
            let lo = byte & 0x0F;
            let hi = (byte >> 4) & 0x0F;
            if lo > 0b11 || lo == 0b10 {
                return Some(b * 2);
            }
            if hi > 0b11 || hi == 0b10 {
                return Some(b * 2 + 1);
            }
        }
        None
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

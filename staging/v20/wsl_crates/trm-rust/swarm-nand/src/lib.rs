//! swarm-nand — the NAND K3 fabric in Rust.
//!
//! Port of swarm_v2/core/nand_k3_fabric.py with ALL the safety fixes:
//!   1. Canonical v2 codec (Phase 2 unification)
//!   2. flock LOCK_EX on open (multi-writer)
//!   3. Open-time re-sync when flock denied (first race fix)
//!   4. Flush-time max-merge of header (second race fix)
//!   5. os.pwrite persistence (never MAP_PRIVATE alone)
//!
//! Header (36 bytes, padded to 64): <QIIQQI
//!   magic u64 | version u32 | arena_size u32 | write_offset u64 |
//!   trit_count u64 | crc32 u32
//!
//! Codec table (canonical v2 — same as TurboVec and fabric v2):
//!   FALSE(-1) → 0b00 (0)   UNKNOWN(0) → 0b01 (1)   TRUE(+1) → 0b11 (3)

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
// ─── Positional disk I/O and the writer lock: one contract, two kernels ─────
//
// The fabric is an append-only log written with pwrite/pread at explicit offsets. Unix exposes those
// directly; Windows has no read_exact_at/write_all_at on File, but seek_read/seek_write ARE the same
// positioned operations, so the contract is rebuilt from them -- the cursor is never used as state.
// The writer lock is flock(LOCK_EX|LOCK_NB) on unix; Windows takes the same exclusive, non-blocking
// lock through std's try_lock, so nothing new is added to the dependency set.
mod pio {
    use std::fs::File;
    use std::io;

    #[cfg(unix)]
    pub fn read_exact_at(f: &File, buf: &mut [u8], off: u64) -> io::Result<()> {
        use std::os::unix::fs::FileExt;
        f.read_exact_at(buf, off)
    }

    #[cfg(windows)]
    pub fn read_exact_at(f: &File, buf: &mut [u8], off: u64) -> io::Result<()> {
        use std::os::windows::fs::FileExt;
        let mut done = 0usize;
        while done < buf.len() {
            let n = f.seek_read(&mut buf[done..], off + done as u64)?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "positional read hit EOF before the buffer was filled",
                ));
            }
            done += n;
        }
        Ok(())
    }

    #[cfg(unix)]
    pub fn write_all_at(f: &File, buf: &[u8], off: u64) -> io::Result<()> {
        use std::os::unix::fs::FileExt;
        f.write_all_at(buf, off)
    }

    #[cfg(windows)]
    pub fn write_all_at(f: &File, buf: &[u8], off: u64) -> io::Result<()> {
        use std::os::windows::fs::FileExt;
        let mut done = 0usize;
        while done < buf.len() {
            let n = f.seek_write(&buf[done..], off + done as u64)?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "positional write made no progress",
                ));
            }
            done += n;
        }
        Ok(())
    }

    /// Exclusive, non-blocking writer lock. false == another writer already holds it, which is the
    /// multi-writer path the open-time re-sync covers.
    #[cfg(unix)]
    pub fn try_exclusive_lock(f: &File) -> bool {
        let fd = std::os::unix::io::AsRawFd::as_raw_fd(f);
        unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0 }
    }

    #[cfg(windows)]
    pub fn try_exclusive_lock(f: &File) -> bool {
        // A Windows byte-range lock is MANDATORY, unlike flock: locking the whole file (which is what
        // std's File::try_lock does) makes every read through any other handle fail with os error 33, so
        // it would break the multi-writer path this lock exists to coordinate. Take the lock on a single
        // sentinel byte outside the payload's offset space instead: writers still exclude each other, and
        // reads and writes of real data are untouched. Closing the handle releases it.
        #[repr(C)]
        struct Overlapped {
            internal: usize,
            internal_high: usize,
            offset: u32,
            offset_high: u32,
            h_event: usize,
        }
        const LOCKFILE_FAIL_IMMEDIATELY: u32 = 0x0000_0001;
        const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x0000_0002;
        const SENTINEL: u64 = u64::MAX - 8;
        #[link(name = "kernel32")]
        extern "system" {
            fn LockFileEx(
                handle: isize,
                flags: u32,
                reserved: u32,
                low: u32,
                high: u32,
                overlapped: *mut Overlapped,
            ) -> i32;
        }
        let mut ov = Overlapped {
            internal: 0,
            internal_high: 0,
            offset: (SENTINEL & 0xFFFF_FFFF) as u32,
            offset_high: (SENTINEL >> 32) as u32,
            h_event: 0,
        };
        let handle = std::os::windows::io::AsRawHandle::as_raw_handle(f) as isize;
        unsafe {
            LockFileEx(
                handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1,
                0,
                &mut ov,
            ) != 0
        }
    }
}

pub const NAND_MAGIC: u64 = 0x4B33_4E41_4E44; // "K3NAND"
pub const HEADER_SIZE: u64 = 64;
pub const CODEC_LEGACY: u32 = 1;
pub const CODEC_CANONICAL: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct FabricHeader {
    pub magic: u64,
    pub version: u32,
    pub arena_size: u32,
    pub write_offset: u64,
    pub trit_count: u64,
    pub crc: u32,
}

impl FabricHeader {
    pub fn pack(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0..8].copy_from_slice(&self.magic.to_le_bytes());
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..16].copy_from_slice(&self.arena_size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.write_offset.to_le_bytes());
        buf[24..32].copy_from_slice(&self.trit_count.to_le_bytes());
        buf[32..36].copy_from_slice(&self.crc.to_le_bytes());
        buf
    }

    pub fn unpack(buf: &[u8]) -> Self {
        let magic = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let version = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let arena_size = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let write_offset = u64::from_le_bytes(buf[16..24].try_into().unwrap());
        let trit_count = u64::from_le_bytes(buf[24..32].try_into().unwrap());
        let crc = u32::from_le_bytes(buf[32..36].try_into().unwrap());
        Self { magic, version, arena_size, write_offset, trit_count, crc }
    }
}

/// The NAND K3 fabric (append-only log with flock + max-merge).
pub struct NandFabric {
    file: File,
    header: FabricHeader,
    has_flock: bool,
    dirty: bool,
}

impl NandFabric {
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;

        // Exclusive, non-blocking writer lock: flock(LOCK_EX|LOCK_NB) on unix, the OS file lock
        // on windows. A denied lock is the multi-writer path the re-sync below covers.
        let has_flock = pio::try_exclusive_lock(&file);

        // read header
        let mut hbuf = [0u8; 64];
        {
            use std::io::Read;
            let mut fref: &File = &file;
            let _ = fref.read_exact(&mut hbuf);
        }
        let header = FabricHeader::unpack(&hbuf);

        // NEW FILE: initialize the header (write_offset starts at HEADER_SIZE)
        let is_new = header.magic != NAND_MAGIC;
        let header = if is_new {
            FabricHeader {
                magic: NAND_MAGIC,
                version: CODEC_CANONICAL, // canonical v2 from birth
                arena_size: 256 * 1024 * 1024,
                write_offset: HEADER_SIZE,
                trit_count: 0,
                crc: 0,
            }
        } else {
            header
        };

        let mut fab = Self { file, header, has_flock, dirty: false };
        if is_new {
            // A new file's initialized header IS a change: flush() returns early on a clean fabric, so
            // without this a brand-new fabric stays 0 bytes on disk and the next opener reads magic 0 and
            // re-initializes it, losing the write_offset the first writer is appending from.
            fab.dirty = true;
            fab.flush()?; // persist the initialized header
        } else if !has_flock {
            // MULTI-WRITER: re-sync cached offsets from disk header
            // (first race fix) — another writer holds the flock.
            fab.sync_from_disk()?;
        }
        Ok(fab)
    }

    /// Re-read the header from disk (for the no-flock re-sync path).
    fn sync_from_disk(&mut self) -> std::io::Result<()> {
        let mut hbuf = [0u8; 64];
        let _ = pio::read_exact_at(&self.file, &mut hbuf, 0)?;
        self.header = FabricHeader::unpack(&hbuf);
        Ok(())
    }

    /// Arena capacity in trits (2-bit/trit canonical packing).
    pub fn capacity_trits(&self) -> u64 {
        self.header.arena_size as u64 * 4
    }

    pub fn codec_version(&self) -> u32 {
        if self.header.version == CODEC_CANONICAL { CODEC_CANONICAL } else { CODEC_LEGACY }
    }

    pub fn trit_count(&self) -> u64 {
        self.header.trit_count
    }

    /// Append a trit record; returns the byte offset it was written at.
    /// Uses pwrite (write_all_at) — real disk, never MAP_PRIVATE.
    pub fn append(&mut self, trits: &[i8]) -> std::io::Result<u64> {
        let packed = pack_canonical(trits);
        let offset = self.header.write_offset;
        pio::write_all_at(&self.file, &packed, offset)?;
        self.header.write_offset += packed.len() as u64;
        self.header.trit_count += trits.len() as u64;
        self.dirty = true;
        Ok(offset)
    }

    /// Read a record back by offset (canonical decode).
    pub fn read(&self, offset: u64, n_trits: usize) -> std::io::Result<Vec<i8>> {
        let nbytes = (n_trits * 2 + 7) / 8;
        let mut buf = vec![0u8; nbytes];
        let _ = pio::read_exact_at(&self.file, &mut buf, offset)?;
        Ok(unpack_canonical(&buf, n_trits))
    }

    /// Flush with max-merge (second race fix): take max(cached, disk)
    /// for offset/count — an append-only log can only extend, never truncate.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        // max-merge with the on-disk header
        let mut hbuf = [0u8; 64];
        if pio::read_exact_at(&self.file, &mut hbuf, 0).is_ok() {
            let disk = FabricHeader::unpack(&hbuf);
            if disk.magic == self.header.magic {
                self.header.write_offset = self.header.write_offset.max(disk.write_offset);
                self.header.trit_count = self.header.trit_count.max(disk.trit_count);
            }
        }
        // persist the ACTUAL codec version (never the module constant)
        let hdr = self.header.pack();
        pio::write_all_at(&self.file, &hdr, 0)?;
        self.file.sync_data()?;
        self.dirty = false;
        Ok(())
    }

    pub fn has_flock(&self) -> bool {
        self.has_flock
    }

    pub fn status(&self) -> (u32, u64, u64) {
        (self.codec_version(), self.header.trit_count, self.header.write_offset)
    }
}

// ─── Canonical v2 codec (identical to swarm-k3 + fabric v2 + TurboVec) ───

pub fn pack_canonical(trits: &[i8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((trits.len() + 3) / 4);
    let mut acc: u8 = 0;
    let mut shift: u8 = 0;
    for &t in trits {
        let code: u8 = match t {
            1 => 3,   // TRUE
            -1 => 0,  // FALSE
            _ => 1,   // UNKNOWN
        };
        acc |= code << shift;
        shift += 2;
        if shift == 8 {
            out.push(acc);
            acc = 0;
            shift = 0;
        }
    }
    if shift != 0 {
        out.push(acc);
    }
    out
}

pub fn unpack_canonical(bytes: &[u8], count: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(count);
    'outer: for &b in bytes {
        for shift in [0u8, 2, 4, 6] {
            if out.len() >= count {
                break 'outer;
            }
            out.push(match (b >> shift) & 0b11 {
                0 => -1,  // FALSE
                3 => 1,   // TRUE
                _ => 0,   // UNKNOWN (incl. unused 0b10)
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let h = FabricHeader {
            magic: NAND_MAGIC,
            version: CODEC_CANONICAL,
            arena_size: 256 * 1024 * 1024,
            write_offset: 64,
            trit_count: 42,
            crc: 0,
        };
        let packed = h.pack();
        let back = FabricHeader::unpack(&packed);
        assert_eq!(back, h);
    }

    #[test]
    fn canonical_codec_exact() {
        let trits: Vec<i8> = vec![1, 0, -1, 1, 0, -1, 1, 1, -1, 0, 1, -1, 0, 1, -1, 1];
        let packed = pack_canonical(&trits);
        assert_eq!(unpack_canonical(&packed, trits.len()), trits);
    }

    #[test]
    fn append_read_roundtrip() {
        let path = "/tmp/swarm_nand_test.bin";
        let _ = std::fs::remove_file(path);
        let mut fab = NandFabric::open(path).unwrap();
        let rec: Vec<i8> = vec![1, 0, -1, 1, 0, -1, 1, 1];
        let off = fab.append(&rec).unwrap();
        fab.flush().unwrap();
        assert_eq!(fab.read(off, 8).unwrap(), rec);
        assert_eq!(fab.codec_version(), CODEC_CANONICAL);
        let _ = std::fs::remove_file(path);
    }
}

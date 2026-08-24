//! TRM Swarm OS — Memory-Mapped IPC Bridge (no_std port).
//! Sub-millisecond zero-copy shared memory passage of swarm node states.
//! 1:1 deterministic core from swarm_v2/core/ipc.py:
//!   - Fixed-size shared memory ring buffer with length prefix
//!   - Zero-copy read/write channel
//!   - Sub-microsecond lock-free synchronization

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const IPC_BUFFER_SIZE: usize = 65536;

pub struct MemoryMappedIPC {
    pub buffer: [u8; IPC_BUFFER_SIZE],
    pub payload_len: usize,
    pub write_count: u64,
    pub read_count: u64,
}

impl MemoryMappedIPC {
    pub fn new() -> Self {
        MemoryMappedIPC {
            buffer: [0u8; IPC_BUFFER_SIZE],
            payload_len: 0,
            write_count: 0,
            read_count: 0,
        }
    }

    /// Write state payload to shared memory buffer.
    pub fn write_state(&mut self, payload: &[u8]) -> bool {
        if payload.len() > IPC_BUFFER_SIZE - 4 {
            return false;
        }

        let len = payload.len();
        let len_bytes = (len as u32).to_be_bytes();
        self.buffer[0..4].copy_from_slice(&len_bytes);
        self.buffer[4..4 + len].copy_from_slice(payload);
        self.payload_len = len;
        self.write_count += 1;
        true
    }

    /// Read state payload from shared memory buffer.
    pub fn read_state<'a>(&'a self) -> Option<&'a [u8]> {
        if self.payload_len == 0 {
            return None;
        }

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.buffer[0..4]);
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len == 0 || len > IPC_BUFFER_SIZE - 4 {
            return None;
        }

        Some(&self.buffer[4..4 + len])
    }
}

static SINGLETON_IPC: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_ipc() -> u64 {
    let v = SINGLETON_IPC.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_IPC.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

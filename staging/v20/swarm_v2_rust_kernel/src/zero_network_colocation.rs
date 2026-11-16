//! TRM Swarm OS — Zero-Network Co-Location & Shared Memory IPC Engine
//! High-throughput memory-mapped circular ring buffer for zero-network inter-agent messaging.
//! Bypasses OS network stacks and sockets by passing task records through shared memory.

use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use memmap2::MmapMut;

pub const DEFAULT_SHM_PATH: &str = "trm_colocation_ringbuffer.shm";
pub const DEFAULT_SHM_SIZE: usize = 1024 * 1024; // 1 MB shared memory buffer
pub const SHM_MAGIC: u32 = 0x53484D30; // "SHM0"

/// Shared Memory Task Header (16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ShmTaskHeader {
    pub magic: u32,
    pub task_id: u32,
    pub agent_id: u8,
    pub status: i8, // +1 SUCCESS, 0 RUNNING, -1 FAILED
    pub payload_len: u32,
}

/// Benchmark Results for Zero-Network Co-Location
///
/// All latencies are in NANOSECONDS. These operations take tens of nanoseconds,
/// so any microsecond-resolution timer truncates them to zero and destroys the
/// measurement.
#[derive(Debug, Clone)]
pub struct ColocationBenchmarkResult {
    pub total_tasks: usize,
    pub write_latency_ns: f64,
    pub execution_latency_ns: f64,
    pub read_latency_ns: f64,
    pub total_roundtrip_ns: f64,
    pub throughput_tasks_per_sec: f64,
    /// Ratio against an ASSUMED WAN baseline. This is NOT a measured speedup:
    /// the platform measures no real network round trip anywhere. Treat it as
    /// an order-of-magnitude illustration only. 0.0 means "not computed".
    pub ratio_vs_assumed_wan_baseline: f64,
}

/// Memory-Mapped Shared Ring Buffer
pub struct MmapRingBuffer {
    mmap: MmapMut,
    capacity: usize,
    write_head: usize,
    read_tail: usize,
}

impl MmapRingBuffer {
    pub fn open_or_create<P: AsRef<Path>>(path: P, size: usize) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let current_len = file.metadata()?.len() as usize;
        if current_len < size {
            file.set_len(size as u64)?;
        }

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            mmap,
            capacity: size,
            write_head: 0,
            read_tail: 0,
        })
    }

    /// Write task payload to the memory-mapped ring buffer
    pub fn write_task(&mut self, task_id: u32, agent_id: u8, payload: &[u8]) -> usize {
        let header_size = std::mem::size_of::<ShmTaskHeader>();
        let total_size = header_size + payload.len();

        if self.write_head + total_size >= self.capacity {
            self.write_head = 0; // Wrap around ring buffer
        }

        let offset = self.write_head;
        let header = ShmTaskHeader {
            magic: SHM_MAGIC,
            task_id,
            agent_id,
            status: 1, // Ready / Success
            payload_len: payload.len() as u32,
        };

        unsafe {
            let dst_ptr = self.mmap.as_mut_ptr().add(offset);
            std::ptr::copy_nonoverlapping(
                &header as *const _ as *const u8,
                dst_ptr,
                header_size,
            );
            std::ptr::copy_nonoverlapping(
                payload.as_ptr(),
                dst_ptr.add(header_size),
                payload.len(),
            );
        }

        self.write_head += total_size;
        offset
    }

    /// Read task payload from the memory-mapped ring buffer
    pub fn read_task(&mut self, offset: usize) -> Option<(ShmTaskHeader, Vec<u8>)> {
        let header_size = std::mem::size_of::<ShmTaskHeader>();
        if offset + header_size > self.capacity {
            return None;
        }

        let header: ShmTaskHeader = unsafe {
            let src_ptr = self.mmap.as_ptr().add(offset);
            std::ptr::read_unaligned(src_ptr as *const ShmTaskHeader)
        };

        if header.magic != SHM_MAGIC {
            return None;
        }

        let payload_len = header.payload_len as usize;
        let mut payload = vec![0u8; payload_len];

        unsafe {
            let payload_ptr = self.mmap.as_ptr().add(offset + header_size);
            std::ptr::copy_nonoverlapping(payload_ptr, payload.as_mut_ptr(), payload_len);
        }

        Some((header, payload))
    }
}

/// Zero-Network Co-Location Execution Engine
pub struct ZeroNetworkColocation {
    pub ring_buffer: MmapRingBuffer,
}

impl ZeroNetworkColocation {
    pub fn new() -> std::io::Result<Self> {
        let ring_buffer = MmapRingBuffer::open_or_create(DEFAULT_SHM_PATH, DEFAULT_SHM_SIZE)?;
        Ok(Self { ring_buffer })
    }

    /// Execute a suite of tasks directly in shared memory.
    ///
    /// Timed with `as_nanos()`: a shared-memory write/read is on the order of
    /// tens of nanoseconds, so `as_micros()` truncates every sample to 0 and
    /// yields a mean of 0 — a measurement that cannot resolve what it claims
    /// to measure.
    pub fn benchmark_colocated(&mut self, task_count: usize) -> ColocationBenchmarkResult {
        if task_count == 0 {
            return ColocationBenchmarkResult {
                total_tasks: 0,
                write_latency_ns: 0.0,
                execution_latency_ns: 0.0,
                read_latency_ns: 0.0,
                total_roundtrip_ns: 0.0,
                throughput_tasks_per_sec: 0.0,
                ratio_vs_assumed_wan_baseline: 0.0,
            };
        }

        let test_payload = b"TRM_SOVEREIGN_CO_LOCATED_TASK_PAYLOAD_K3";
        let mut write_times = Vec::with_capacity(task_count);
        let mut exec_times = Vec::with_capacity(task_count);
        let mut read_times = Vec::with_capacity(task_count);

        for i in 0..task_count {
            // 1. Shared memory-mapped write
            let t0 = Instant::now();
            let offset = self.ring_buffer.write_task(i as u32, (i % 14) as u8, test_payload);
            let write_dur = t0.elapsed().as_nanos() as f64;
            write_times.push(write_dur);

            // 2. Co-located agent in-memory execution (K3 Kleene evaluation)
            let t1 = Instant::now();
            let _k3_val = match (i % 3) as i8 {
                0 => 0i8,
                1 => 1i8,
                _ => -1i8,
            };
            let exec_dur = t1.elapsed().as_nanos() as f64;
            exec_times.push(exec_dur);

            // 3. Shared memory-mapped read
            let t2 = Instant::now();
            let _res = self.ring_buffer.read_task(offset);
            let read_dur = t2.elapsed().as_nanos() as f64;
            read_times.push(read_dur);
        }

        let avg_write = write_times.iter().sum::<f64>() / task_count as f64;
        let avg_exec = exec_times.iter().sum::<f64>() / task_count as f64;
        let avg_read = read_times.iter().sum::<f64>() / task_count as f64;
        let total_roundtrip = avg_write + avg_exec + avg_read;

        // ASSUMED baseline, not a measurement. 50 ms is WAN-scale latency; a
        // WireGuard LAN round trip is ~0.5-2 ms, so this baseline is itself
        // optimistic by roughly 25-100x on a local network.
        let assumed_network_baseline_ns = 50_000_000.0;

        // No `.max(epsilon)` floor. Flooring the divisor is what previously
        // turned a zero measurement into a ~50,000,000x headline claim. If the
        // measurement did not resolve, report 0.0 ("not computed") instead.
        let ratio_vs_assumed_wan_baseline = if total_roundtrip > 0.0 {
            assumed_network_baseline_ns / total_roundtrip
        } else {
            0.0
        };
        let throughput = if total_roundtrip > 0.0 {
            1_000_000_000.0 / total_roundtrip
        } else {
            0.0
        };

        ColocationBenchmarkResult {
            total_tasks: task_count,
            write_latency_ns: avg_write,
            execution_latency_ns: avg_exec,
            read_latency_ns: avg_read,
            total_roundtrip_ns: total_roundtrip,
            throughput_tasks_per_sec: throughput,
            ratio_vs_assumed_wan_baseline,
        }
    }
}

/// High-performance C-ABI export for Zero-Network Co-location probing
///
/// Returns the measured mean round-trip in nanoseconds, or -1.0 on failure.
#[no_mangle]
pub extern "C" fn rust_colocation_execute_fast(tasks: usize) -> f64 {
    match ZeroNetworkColocation::new() {
        Ok(mut engine) => {
            let res = engine.benchmark_colocated(tasks);
            res.total_roundtrip_ns
        }
        Err(_) => -1.0,
    }
}

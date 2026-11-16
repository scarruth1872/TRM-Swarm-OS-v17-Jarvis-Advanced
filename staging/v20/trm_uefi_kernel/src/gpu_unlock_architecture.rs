//! TRM Swarm OS — GPU Unlock Architecture (no_std port).
//! High-throughput ring buffer and hybrid pipeline split (CPU AVX2 fast-path + GPU bulk sweep).
//! 1:1 deterministic core from swarm_v2/core/gpu_unlock_architecture.py:
//!   - Pinned ring buffer for batch accumulation
//!   - Zero-copy batch staging eliminating PCIe round-trip latency
//!   - Dual-engine execution scheduling (Latency-critical <5us vs Bulk >1M tasks/s)

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const RING_SIZE: usize = 4096;
// Oracle line 47: TASK_VECTOR_BYTES = 384 * 4 (float32 x 384-dim) => dim is 384, not 128.
pub const TASK_VECTOR_DIM: usize = 384;

// ── Oracle constants (swarm_v2/core/gpu_unlock_architecture.py lines 43-49) ──
pub const CPU_THREADS: usize = 24;
pub const GPU_CORES: usize = 2560;
pub const TASK_VECTOR_BYTES: usize = TASK_VECTOR_DIM * 4;
/// PCIe 4.0 round-trip overhead, microseconds (oracle line 48).
pub const PCIE_LATENCY_US: f64 = 1.2;
/// L3 cache access latency, nanoseconds (oracle line 49).
pub const L3_CACHE_LATENCY_NS: u64 = 5;
/// Oracle line 127-128: deque(maxlen=10000) for both cpu_queue and gpu_queue.
pub const HYBRID_QUEUE_MAXLEN: usize = 10_000;

// ── MoE offloader defaults (oracle lines 243-247: config fallbacks) ──
pub const MOE_DEFAULT_NUM_EXPERTS: usize = 64;
pub const MOE_DEFAULT_HIDDEN_SIZE: usize = 2048;
pub const MOE_ROUTING_ENGINE: &str = "gpu_offloaded";

// ── Oracle projection constants (lines 393-403) ──
/// Oracle line 393: `baseline_cpu = 269510  # Best from stress test` (a recorded
/// constant, NOT measured here). Passed in as a parameter; this is its oracle value.
pub const BASELINE_CPU_TPS: f64 = 269_510.0;
/// Oracle line 394: ring_factor = RING_SIZE / 512.
pub const RING_BATCH_REFERENCE: f64 = 512.0;
/// Oracle line 395: GPU ~4x CPU for matrix ops.
pub const GPU_CONTRIBUTION_MULTIPLIER: f64 = 4.0;
/// Oracle line 396: 70% GPU efficiency.
pub const GPU_EFFICIENCY: f64 = 0.7;
/// Oracle line 403: `combined_estimate > 1_000_000`.
pub const TARGET_TPS: f64 = 1_000_000.0;

#[derive(Debug, Clone)]
pub struct RingBufferSlot {
    pub vector: [f32; TASK_VECTOR_DIM],
    pub task_id: u64,
    pub timestamp_ns: u64,
    pub priority: u8, // 0 = latency-critical, 1 = bulk throughput
}

pub struct PinnedRingBuffer {
    pub slots: usize,
    pub write_idx: usize,
    pub read_idx: usize,
    pub buffered_count: u64,
    pub flushed_count: u64,
    /// Oracle line 83: stats dict key 'drops' — never incremented in the oracle either.
    pub drops: u64,
}

impl PinnedRingBuffer {
    pub fn new(slots: usize) -> Self {
        PinnedRingBuffer {
            slots: slots.min(RING_SIZE),
            write_idx: 0,
            read_idx: 0,
            buffered_count: 0,
            flushed_count: 0,
            drops: 0,
        }
    }

    /// Oracle: self.stats = {'buffered', 'flushed', 'drops'} (line 83).
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.buffered_count, self.flushed_count, self.drops)
    }

    pub fn push(&mut self, _vector: &[f32], priority: u8) -> bool {
        self.write_idx = (self.write_idx + 1) % self.slots;
        self.buffered_count += 1;
        self.write_idx == self.read_idx
    }

    pub fn flush_batch(&mut self) -> usize {
        if self.write_idx == self.read_idx {
            return 0;
        }

        let batch_size = if self.write_idx > self.read_idx {
            self.write_idx - self.read_idx
        } else {
            self.slots - self.read_idx + self.write_idx
        };

        self.read_idx = self.write_idx;
        self.flushed_count += batch_size as u64;
        batch_size
    }
}

pub struct HybridPipeline {
    pub ring_buffer: PinnedRingBuffer,
    pub cpu_fast_path_count: u64,
    pub gpu_bulk_path_count: u64,
    // ── Added: oracle cpu_queue / gpu_queue (lines 127-128), deque(maxlen=10000) ──
    pub cpu_queue: VecDeque<[f32; TASK_VECTOR_DIM]>,
    pub gpu_queue: VecDeque<[f32; TASK_VECTOR_DIM]>,
    // Oracle lines 134-137: cpu_processed / gpu_processed counters.
    pub cpu_processed: u64,
    pub gpu_processed: u64,
}

impl HybridPipeline {
    pub fn new() -> Self {
        HybridPipeline {
            ring_buffer: PinnedRingBuffer::new(RING_SIZE),
            cpu_fast_path_count: 0,
            gpu_bulk_path_count: 0,
            cpu_queue: VecDeque::new(),
            gpu_queue: VecDeque::new(),
            cpu_processed: 0,
            gpu_processed: 0,
        }
    }

    /// Oracle: HybridPipeline.route(batch_size) -> 'cpu' | 'gpu' (lines 139-147).
    ///   batch_size <= 16          -> 'cpu'
    ///   batch_size >= 512         -> 'gpu'
    ///   17..=511                  -> 'cpu' if len(cpu_queue) <= len(gpu_queue) else 'gpu'
    pub fn route(&self, batch_size: usize) -> &'static str {
        if batch_size <= 16 {
            "cpu"
        } else if batch_size >= 512 {
            "gpu"
        } else if self.cpu_queue.len() <= self.gpu_queue.len() {
            "cpu"
        } else {
            "gpu"
        }
    }

    /// Oracle lines 334-337: append to cpu_queue / gpu_queue; the deques have
    /// maxlen=HYBRID_QUEUE_MAXLEN, so a full queue silently drops its OLDEST entry.
    pub fn enqueue(&mut self, engine: &str, vector: [f32; TASK_VECTOR_DIM]) {
        let q = if engine == "cpu" {
            &mut self.cpu_queue
        } else {
            &mut self.gpu_queue
        };
        if q.len() >= HYBRID_QUEUE_MAXLEN {
            q.pop_front();
        }
        q.push_back(vector);
    }

    /// Oracle lines 330-337: splits[engine] += bs — task accounting weighted by batch size.
    pub fn enqueue_routed(&mut self, batch_size: usize, vector: [f32; TASK_VECTOR_DIM]) -> (usize, &'static str) {
        let engine = self.route(batch_size);
        if engine == "cpu" {
            self.cpu_fast_path_count += batch_size as u64;
        } else {
            self.gpu_bulk_path_count += batch_size as u64;
        }
        self.enqueue(engine, vector);
        (batch_size, engine)
    }

    pub fn dispatch(&mut self, batch_size: usize) -> (u64, &'static str) {
        if batch_size <= 16 {
            // CPU Fast Path (<5us target)
            self.cpu_fast_path_count += batch_size as u64;
            (5, "CPU_AVX2_FAST_PATH")
        } else {
            // GPU Bulk Sweep
            self.gpu_bulk_path_count += batch_size as u64;
            (18, "GPU_BULK_SWEEP")
        }
    }
}

static SINGLETON_GPU_UNLOCK: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_gpu_unlock_architecture() -> u64 {
    let v = SINGLETON_GPU_UNLOCK.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_GPU_UNLOCK.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// ACTION C — Heavy MoE offloading: the portable scoring rule.
// Oracle: MoEGPUOffloader.offload_routing_matrix (lines 251-276):
//   expert_scores = expert_weights @ x   (num_experts x hidden_size) · (hidden_size)
//   top_expert    = argmax(expert_scores)
//   top_score     = max(expert_scores)
// NOT ported (hardware/host): torch.cuda.is_available(), torch.device('cuda'),
// safetensors/config.json load from 'F:/Development sites/huggingface_cache/...',
// torch.randn weight init, wall-clock latency_us (time.perf_counter_ns).
// The no_std port therefore takes the weight matrix from the host via load_weights()
// and returns None (instead of torch.randn-ing) when it has not been provided.
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExpertRouting {
    pub top_expert: usize,
    pub top_score: f32,
    /// Oracle line 275: 'engine': 'gpu_offloaded'.
    pub engine: &'static str,
    pub offloaded_calls: u64,
}

pub struct MoeGpuOffloader {
    pub num_experts: usize,
    pub hidden_size: usize,
    /// Row-major (num_experts x hidden_size) f32 weights, host-supplied.
    pub expert_weights: Vec<f32>,
    /// Oracle line 249: stats['offloaded_calls'].
    pub offloaded_calls: u64,
    /// Oracle line 249: stats['vram_used_gb'] — a host/device report, not computed here.
    pub vram_used_gb: u64,
}

/// Oracle lines 265-266: torch.argmax / torch.max. torch.argmax (like np.argmax)
/// returns the index of the FIRST maximal value, and returns the first NaN index
/// when the scores contain NaN. Implemented identically here (verified).
fn argmax_first(scores: &[f32]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, &s) in scores.iter().enumerate() {
        match best {
            None => best = Some(i),
            Some(b) => {
                let bs = scores[b];
                if s > bs || (s.is_nan() && !bs.is_nan()) {
                    best = Some(i);
                }
            }
        }
    }
    best
}

impl MoeGpuOffloader {
    /// Oracle __init__ defaults: num_local_experts=64, hidden_size=2048.
    pub fn new(num_experts: usize, hidden_size: usize) -> Self {
        MoeGpuOffloader {
            num_experts,
            hidden_size,
            expert_weights: Vec::new(),
            offloaded_calls: 0,
            vram_used_gb: 0,
        }
    }

    pub fn instella_default() -> Self {
        MoeGpuOffloader::new(MOE_DEFAULT_NUM_EXPERTS, MOE_DEFAULT_HIDDEN_SIZE)
    }

    /// Oracle line 261: self._expert_weights = torch.randn(num_experts, hidden_size).
    /// Here the host loads the real matrix; returns false on a shape mismatch.
    pub fn load_weights(&mut self, flat_weights: &[f32]) -> bool {
        if flat_weights.len() != self.num_experts * self.hidden_size {
            return false;
        }
        self.expert_weights.clear();
        self.expert_weights.extend_from_slice(flat_weights);
        true
    }

    /// Oracle: offload_routing_matrix(input_vec) -> {top_expert, top_score, engine}.
    /// Returns None when no weights are loaded or the input is shorter than
    /// hidden_size (the oracle raises in both cases). `latency_us` is host-side.
    pub fn offload_routing_matrix(&mut self, input_vec: &[f32]) -> Option<ExpertRouting> {
        if self.hidden_size == 0 || self.num_experts == 0 {
            return None;
        }
        if self.expert_weights.len() != self.num_experts * self.hidden_size
            || input_vec.len() < self.hidden_size
        {
            return None;
        }
        let h = self.hidden_size;

        // expert_scores = W @ x  (f32 accumulation, same dtype as the oracle's torch.float32)
        let mut scores = Vec::with_capacity(self.num_experts);
        for e in 0..self.num_experts {
            let row = &self.expert_weights[e * h..(e + 1) * h];
            let mut acc = 0.0f32;
            for j in 0..h {
                acc += row[j] * input_vec[j];
            }
            scores.push(acc);
        }

        let top_expert = argmax_first(&scores)?;
        let top_score = scores[top_expert];
        self.offloaded_calls += 1;
        Some(ExpertRouting {
            top_expert,
            top_score,
            engine: MOE_ROUTING_ENGINE,
            offloaded_calls: self.offloaded_calls,
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// PROJECTED COMBINED THROUGHPUT — oracle lines 393-403, pure arithmetic on the
// oracle's own recorded baseline (269510 tps "Best from stress test"). No number
// below is measured here.
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedThroughput {
    pub baseline_cpu: f64,
    pub ring_factor: f64,
    pub gpu_contribution: f64,
    pub combined_estimate: f64,
    /// Oracle line 403: `combined_estimate > 1_000_000`.
    pub target_met: bool,
}

pub fn projected_combined_throughput(baseline_cpu: f64) -> ProjectedThroughput {
    let ring_factor = RING_SIZE as f64 / RING_BATCH_REFERENCE; // 4096 / 512 = 8
    let gpu_contribution = baseline_cpu * GPU_CONTRIBUTION_MULTIPLIER;
    let combined_estimate = baseline_cpu + gpu_contribution * GPU_EFFICIENCY;
    ProjectedThroughput {
        baseline_cpu,
        ring_factor,
        gpu_contribution,
        combined_estimate,
        target_met: combined_estimate > TARGET_TPS,
    }
}

/// Oracle line 316: TOTAL * PCIe_LATENCY_US / 1000 — avoided per-task PCIe overhead in ms.
pub fn pcie_saved_ms(tasks: u64) -> f64 {
    tasks as f64 * PCIE_LATENCY_US / 1000.0
}

// ══════════════════════════════════════════════════════════════════════════════
// NATIVE VOLTAGE STRIP — the module's one native kernel.
// Oracle lines 13-38 load swarm_v2_artifacts/native_voltage_strips/strip_gpu_transposer.dll
// via ctypes and call strip_vram_cache_transposer(arr, h, out); the strip's Rust
// source (swarm_v2_artifacts/native_voltage_strips/strip_gpu_transposer.rs) is a
// 16x16 cache-tiled transpose. The transposition itself is pure arithmetic — no GPU,
// driver or benchmark is involved — so it is ported; the ctypes/DLL boundary is not.
// NOTE: this strip is NOT the generic ternary-dot template; it is a real tiled transpose.
// ══════════════════════════════════════════════════════════════════════════════

/// Mirror of strip_gpu_transposer.rs: out[c*dim + r] = in[r*dim + c], 16x16 tiled.
/// Requires input.len() == dim*dim && output.len() == dim*dim.
pub fn strip_vram_cache_transposer(input: &[f32], dim: usize, output: &mut [f32]) {
    const TILE: usize = 16;
    let n = dim * dim;
    assert!(
        input.len() >= n && output.len() >= n,
        "strip_vram_cache_transposer: buffers must hold dim*dim elements"
    );
    let mut r_tile = 0usize;
    while r_tile < dim {
        let r_end = if r_tile + TILE < dim { r_tile + TILE } else { dim };
        let mut c_tile = 0usize;
        while c_tile < dim {
            let c_end = if c_tile + TILE < dim { c_tile + TILE } else { dim };
            for r in r_tile..r_end {
                for c in c_tile..c_end {
                    output[c * dim + r] = input[r * dim + c];
                }
            }
            c_tile += TILE;
        }
        r_tile += TILE;
    }
}

/// Oracle line 38 fallback (`return matrix.T.copy()`): used when the strip is not
/// loadable or the matrix is non-square / non-float32. out has `cols` rows and
/// `rows` columns; out[c*rows + r] = in[r*cols + c].
pub fn native_vram_transpose_fallback(input: &[f32], rows: usize, cols: usize, output: &mut [f32]) {
    assert!(
        input.len() >= rows * cols && output.len() >= rows * cols,
        "native_vram_transpose_fallback: buffers must hold rows*cols elements"
    );
    for r in 0..rows {
        for c in 0..cols {
            output[c * rows + r] = input[r * cols + c];
        }
    }
}


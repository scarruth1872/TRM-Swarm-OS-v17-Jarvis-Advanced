//! gpu_compute.rs — the full-Rust GPU path for the HDC encoder (MOTION-017, option 3).
//!
//! wgpu + naga: the WGSL source below is compiled to SPIR-V at runtime by naga — no Vulkan SDK, no
//! CUDA toolkit — then dispatched on the discrete GPU over Vulkan and read back. The kernel
//! reproduces `native_encoder::encode_f32` (SplitMix64 sign projection over the FNV-1a feature
//! seeds) so the GPU result can be parity-checked against the CPU reference (the council's Verify
//! condition). Fail-closed: any adapter/device/pipeline failure returns None and the caller falls
//! back to the in-tree CPU encoder.

use std::sync::OnceLock;
use wgpu::util::DeviceExt;

/// WGSL compute shader. Thread i (0..768) sums, over every feature, the sign of bit i in that
/// feature's pre-expanded projection, weighted — identical arithmetic to native_encoder::encode_f32.
/// No u64 arithmetic here (wgpu 30 dropped the SHADER_INT64 gate): the u64 SplitMix64 expansion
/// happens on the CPU and lands as packed u32 sign words.
const SHADER: &str = r#"
struct Params {
    n: u32,
    dim: u32,
};

@group(0) @binding(0) var<storage, read> bits: array<u32>;
@group(0) @binding(1) var<storage, read> weights: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read_write> acc: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.dim) { return; }
    let word = i / 32u;
    let bit = i % 32u;
    var sum = 0.0f;
    for (var f = 0u; f < params.n; f = f + 1u) {
        let b = bits[f * 24u + word];
        let sign = select(-1.0f, 1.0f, ((b >> bit) & 1u) == 1u);
        sum = sum + sign * weights[f];
    }
    acc[i] = sum;
}
"#;

pub const DIM: usize = 768;
/// 768 bits / 32 bits-per-u32 = 24 sign words per feature.
const WORDS_PER_FEATURE: usize = 24;

/// Cached Vulkan device + queue, created once and reused by every GPU call. Fail-closed: a failed
/// first creation caches None for the process lifetime.
static DEVICE: OnceLock<Option<(wgpu::Device, wgpu::Queue)>> = OnceLock::new();

fn get_device() -> Option<&'static (wgpu::Device, wgpu::Queue)> {
    DEVICE
        .get_or_init(|| pollster::block_on(try_device()).map(|(_, d, q)| (d, q)))
        .as_ref()
}

/// GPU availability, cached for the process. True only when a Vulkan adapter + device can actually
/// be created. Fail-closed: any error caches false.
pub fn gpu_available() -> bool {
    get_device().is_some()
}

async fn try_device() -> Option<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        })
        .await
        .ok()?;
    // Raise the buffer limits above the downlevel (WebGL-shaped) defaults: the ChEMBL packed store's
    // codes are 556 MB, which exceeds the 256 MB max_buffer_size / 128 MB max_storage_buffer_binding_size
    // of downlevel_defaults(). The RTX 3070 (Vulkan) supports far larger buffers; if this adapter does
    // not, request_device fails and get_device() returns None -> the CPU scan takes over (fail-closed).
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits.max_buffer_size = 1 << 30; // 1 GiB
    limits.max_storage_buffer_binding_size = 1 << 30; // 1 GiB
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .ok()?;
    Some((adapter, device, queue))
}

/// Encode one text on the GPU, reproducing native_encoder::encode_f32. Returns None on any failure
/// (no GPU, pipeline error) so the caller can fall back to the CPU encoder.
pub fn gpu_encode_f32(text: &str) -> Option<Vec<f32>> {
    let features = crate::native_encoder::extract_features(text);
    if features.is_empty() {
        return Some(vec![0.0f32; DIM]);
    }
    // CPU side: FNV-1a seed -> SplitMix64 expansion -> 768 sign bits packed into 24 u32 words.
    let mut bits: Vec<u32> = Vec::with_capacity(features.len() * WORDS_PER_FEATURE);
    let mut weights: Vec<f32> = Vec::with_capacity(features.len());
    for (feat, w) in &features {
        let seed = crate::native_encoder::fnv1a_64(
            feat.as_bytes(),
            crate::native_encoder::MONAD_TRUTH_ANCHOR,
        );
        let mut prng = crate::native_encoder::SplitMix64::new(seed);
        for _ in 0..12 {
            let word = prng.next_u64();
            bits.push((word & 0xFFFF_FFFF) as u32);
            bits.push((word >> 32) as u32);
        }
        weights.push(*w);
    }
    let n = features.len() as u32;
    let acc = run_gpu(&bits, &weights, n)?;
    // L2 normalize, the same rule as the CPU reference.
    let sum_sq: f32 = acc.iter().map(|x| x * x).sum();
    if sum_sq > 1e-12 {
        let inv = 1.0f32 / sum_sq.sqrt();
        Some(acc.iter().map(|x| x * inv).collect())
    } else {
        Some(vec![0.0f32; DIM])
    }
}

/// Run the projection kernel: packed sign bits + weights in, 768-dim accumulation out.
fn run_gpu(bits: &[u32], weights: &[f32], n: u32) -> Option<Vec<f32>> {
    let (device, queue) = get_device()?;

    let bits_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("bits"),
        contents: bytemuck::cast_slice(bits),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let weights_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("weights"),
        contents: bytemuck::cast_slice(weights),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"),
        contents: bytemuck::cast_slice(&[n, DIM as u32, 0u32, 0u32]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let acc_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("acc"),
        size: (DIM * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hdc_encode"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("hdc_encode_pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("hdc_bind"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: bits_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: weights_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: acc_buf.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("hdc_enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(((DIM as u32) + 255) / 256, 1, 1);
    }

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: (DIM * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&acc_buf, 0, &staging, 0, (DIM * 4) as u64);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    rx.recv().ok()?.ok()?;

    let data = slice.get_mapped_range().ok()?;
    let out: Vec<f32> = bytemuck::cast_slice(&data[..]).to_vec();
    drop(data);
    Some(out)
}

/// Score one store against MANY queries with ONE submit and ONE poll.
///
/// The single-query path (`gpu_scan_scores_cached`) creates buffers, submits, maps and polls per call. Measured at
/// 2.25 ms for a 38 MB scan, that fixed cost is most of the wall time -- the scan achieves 17 GB/s, 3.8% of the
/// card's 448 GB/s. Batching keeps the same shader and the same arithmetic and amortises only the submission, so
/// every query's scores are identical to what the single path would return.
///
/// Returns one score vector per query, in query order. None when the store is not cached or the device fails.
pub fn gpu_scan_batch_cached(
    qs: &[crate::packed_ternary::Query],
    s: &crate::packed_ternary::Store,
) -> Option<Vec<Vec<f32>>> {
    if qs.is_empty() {
        return Some(Vec::new());
    }
    let cache = gpu_store_cache_for(s)?;
    if cache.bytes != s.bytes || cache.n_rows != s.len() {
        return None;
    }
    let (device, queue) = get_device()?;
    let pipeline = scan_pipeline(device)?;
    let n = cache.n_rows;
    let bytes = (n as u64) * 4;

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("scan_batch_enc") });
    let mut stages: Vec<wgpu::Buffer> = Vec::with_capacity(qs.len());
    for q in qs {
        let qcodes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("q_codes"), contents: bytemuck::cast_slice(q.codes()), usage: wgpu::BufferUsages::STORAGE,
        });
        let params = ScanParams {
            dim: cache.dim as u32, n_rows: n as u32, stride: cache.stride as u32, _pad: 0,
            qmin: q.qmin() as f32, qscale: q.qscale() as f32, sum_q: q.sum_q() as f32, qnorm: q.norm() as f32,
        };
        let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("params"), contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM,
        });
        let scores_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scores"), size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scan_batch_bind"), layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: cache.codes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: cache.row_params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: qcodes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: scores_buf.as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((n as u32 + 255) / 256, 1, 1);
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scan_batch_staging"), size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&scores_buf, 0, &staging, 0, bytes);
        stages.push(staging);
    }
    queue.submit(Some(encoder.finish()));

    // One poll for the whole batch: map every staging buffer, then wait until all callbacks have fired.
    let pending = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(stages.len()));
    for st in &stages {
        let p = pending.clone();
        st.slice(..).map_async(wgpu::MapMode::Read, move |_| {
            p.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
    while pending.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    }

    let mut out: Vec<Vec<f32>> = Vec::with_capacity(stages.len());
    for st in &stages {
        let slice = st.slice(..);
        let data = slice.get_mapped_range().ok()?;
        out.push(bytemuck::cast_slice(&data[..]).to_vec());
        drop(data);
        st.unmap();
    }
    Some(out)
}

/// Word-at-a-time variant of SCAN_SHADER.
///
/// The byte loop extracted each 2-bit lane through `codes[row_base + b/4] >> ((b%4)*8)` -- the same u32 was loaded
/// four times and shifted per byte, so a 768-trit row cost 192 iterations and 192 loads for 48 words of data. This
/// unpacks sixteen lanes per load: same products, same order (LSB-first across the word), a quarter of the work.
const SCAN_U32_SHADER: &str = r#"
struct Params {
    dim: u32,
    n_rows: u32,
    stride: u32,
    _pad: u32,
    qmin: f32,
    qscale: f32,
    sum_q: f32,
    qnorm: f32,
};

@group(0) @binding(0) var<storage, read> codes: array<u32>;
@group(0) @binding(1) var<storage, read> row_params: array<f32>;
@group(0) @binding(2) var<storage, read> q_codes: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var<storage, read_write> scores: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n_rows) { return; }
    var p = 0u;
    let words_per_row = params.stride / 4u;
    let row_base = i * words_per_row;
    for (var w = 0u; w < words_per_row; w = w + 1u) {
        var qw = q_codes[w];
        if (qw == 0u) { continue; }
        var cw = codes[row_base + w];
        for (var j = 0u; j < 16u; j = j + 1u) {
            p = p + (qw & 3u) * (cw & 3u);
            qw = qw >> 2u;
            cw = cw >> 2u;
        }
    }
    let scale = row_params[i * 4u];
    let vmin = row_params[i * 4u + 1u];
    let sum_c = row_params[i * 4u + 2u];
    let norm = row_params[i * 4u + 3u];
    let dimf = f32(params.dim);
    let dot = dimf * params.qmin * vmin
        + params.qscale * vmin * params.sum_q
        + params.qmin * scale * sum_c
        + params.qscale * scale * f32(p);
    let cos = select(0.0f, dot / (params.qnorm * norm), norm > 0.0f);
    scores[i] = cos;
}
"#;

static SCAN_U32_PIPELINE: OnceLock<Option<wgpu::ComputePipeline>> = OnceLock::new();

fn scan_u32_pipeline(device: &wgpu::Device) -> Option<&'static wgpu::ComputePipeline> {
    SCAN_U32_PIPELINE.get_or_init(|| {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scan_u32"), source: wgpu::ShaderSource::Wgsl(SCAN_U32_SHADER.into()),
        });
        Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("scan_u32_pipeline"), layout: None, module: &shader, entry_point: Some("main"),
            compilation_options: Default::default(), cache: None,
        }))
    }).as_ref()
}

/// Score the whole store with the word-at-a-time shader, reusing the persistent store buffers.
/// Returns every row's cosine in row order, byte-identical in intent to `gpu_scan_scores_cached`.
pub fn gpu_scan_u32_cached(
    q: &crate::packed_ternary::Query,
    s: &crate::packed_ternary::Store,
) -> Option<Vec<f32>> {
    let cache = gpu_store_cache_for(s)?;
    if cache.bytes != s.bytes || cache.n_rows != s.len() {
        return None;
    }
    let (device, queue) = get_device()?;
    let n = cache.n_rows;
    let qcodes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("q_codes_u32"), contents: bytemuck::cast_slice(q.codes()), usage: wgpu::BufferUsages::STORAGE,
    });
    let params = ScanParams {
        dim: cache.dim as u32, n_rows: n as u32, stride: cache.stride as u32, _pad: 0,
        qmin: q.qmin() as f32, qscale: q.qscale() as f32, sum_q: q.sum_q() as f32, qnorm: q.norm() as f32,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params_u32"), contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM,
    });
    let scores_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scores_u32"), size: (n as u64) * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
    });
    let pipeline = scan_u32_pipeline(device)?;
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scan_u32_bind"), layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: cache.codes_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: cache.row_params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: qcodes_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: scores_buf.as_entire_binding() },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("scan_u32_enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((n as u32 + 255) / 256, 1, 1);
    }
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging_u32"), size: (n as u64) * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&scores_buf, 0, &staging, 0, (n as u64) * 4);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    rx.recv().ok()?.ok()?;
    let data = slice.get_mapped_range().ok()?;
    let out: Vec<f32> = bytemuck::cast_slice(&data[..]).to_vec();
    drop(data);
    Some(out)
}

/// Batch variant: M docs in one dispatch. Thread (doc, i) accumulates doc's dim-i over its features.
const BATCH_SHADER: &str = r#"
struct Params {
    total: u32,
};

@group(0) @binding(0) var<storage, read> bits: array<u32>;
@group(0) @binding(1) var<storage, read> weights: array<f32>;
@group(0) @binding(2) var<storage, read> docs: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var<storage, read_write> acc: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = gid.x;
    if (t >= params.total) { return; }
    let doc = t / 768u;
    let i = t % 768u;
    let word = i / 32u;
    let bit = i % 32u;
    let start = docs[doc * 2u];
    let count = docs[doc * 2u + 1u];
    var sum = 0.0f;
    for (var f = 0u; f < count; f = f + 1u) {
        let b = bits[start * 24u + f * 24u + word];
        let sign = select(-1.0f, 1.0f, ((b >> bit) & 1u) == 1u);
        sum = sum + sign * weights[start + f];
    }
    acc[t] = sum;
}
"#;

/// Encode many texts in ONE GPU dispatch, returning L2-normalized vectors in input order. Bounded by
/// the caller (chunk the corpus; readback is `docs * 768 * 4` bytes). None on any failure -> CPU.
pub fn gpu_encode_batch(texts: &[&str]) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(Vec::new());
    }
    let mut bits: Vec<u32> = Vec::new();
    let mut weights: Vec<f32> = Vec::new();
    let mut docs: Vec<u32> = Vec::new();
    let mut feature_base = 0usize;
    for text in texts {
        let features = crate::native_encoder::extract_features(text);
        docs.push(feature_base as u32);
        docs.push(features.len() as u32);
        for (feat, w) in &features {
            let seed = crate::native_encoder::fnv1a_64(
                feat.as_bytes(),
                crate::native_encoder::MONAD_TRUTH_ANCHOR,
            );
            let mut prng = crate::native_encoder::SplitMix64::new(seed);
            for _ in 0..12 {
                let word = prng.next_u64();
                bits.push((word & 0xFFFF_FFFF) as u32);
                bits.push((word >> 32) as u32);
            }
            weights.push(*w);
        }
        feature_base += features.len();
    }
    let total = (texts.len() * DIM) as u32;
    let acc = run_gpu_batch(&bits, &weights, &docs, total)?;
    let mut out = Vec::with_capacity(texts.len());
    for d in 0..texts.len() {
        let row = &acc[d * DIM..(d + 1) * DIM];
        let sum_sq: f32 = row.iter().map(|x| x * x).sum();
        if sum_sq > 1e-12 {
            let inv = 1.0f32 / sum_sq.sqrt();
            out.push(row.iter().map(|x| x * inv).collect());
        } else {
            out.push(vec![0.0f32; DIM]);
        }
    }
    Some(out)
}

fn run_gpu_batch(bits: &[u32], weights: &[f32], docs: &[u32], total: u32) -> Option<Vec<f32>> {
    let (device, queue) = get_device()?;
    let bits_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("bits"), contents: bytemuck::cast_slice(bits), usage: wgpu::BufferUsages::STORAGE,
    });
    let weights_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("weights"), contents: bytemuck::cast_slice(weights), usage: wgpu::BufferUsages::STORAGE,
    });
    let docs_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("docs"), contents: bytemuck::cast_slice(docs), usage: wgpu::BufferUsages::STORAGE,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"), contents: bytemuck::cast_slice(&[total, 0u32, 0u32, 0u32]), usage: wgpu::BufferUsages::UNIFORM,
    });
    let acc_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("acc"), size: (total as u64) * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hdc_encode_batch"), source: wgpu::ShaderSource::Wgsl(BATCH_SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("batch_pipeline"), layout: None, module: &shader, entry_point: Some("main"),
        compilation_options: Default::default(), cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("batch_bind"), layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: bits_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: weights_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: docs_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: acc_buf.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("batch_enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((total + 255) / 256, 1, 1);
    }
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"), size: (total as u64) * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&acc_buf, 0, &staging, 0, (total as u64) * 4);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    rx.recv().ok()?.ok()?;
    let data = slice.get_mapped_range().ok()?;
    let out: Vec<f32> = bytemuck::cast_slice(&data[..]).to_vec();
    drop(data);
    Some(out)
}

/// Packed-scan kernel: one thread per row computes the integer packed dot P, then the affine cosine
/// (same formula as packed_ternary::search_in), writing per-row cosine scores.
const SCAN_SHADER: &str = r#"
struct Params {
    dim: u32,
    n_rows: u32,
    stride: u32,
    _pad: u32,
    qmin: f32,
    qscale: f32,
    sum_q: f32,
    qnorm: f32,
};

@group(0) @binding(0) var<storage, read> codes: array<u32>;
@group(0) @binding(1) var<storage, read> row_params: array<f32>;
@group(0) @binding(2) var<storage, read> q_codes: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var<storage, read_write> scores: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n_rows) { return; }
    var p = 0u;
    let words_per_row = params.stride / 4u;
    let row_base = i * words_per_row;
    for (var b = 0u; b < params.stride; b = b + 1u) {
        let qb = (q_codes[b / 4u] >> ((b % 4u) * 8u)) & 0xFFu;
        if (qb == 0u) { continue; }
        let cb = (codes[row_base + b / 4u] >> ((b % 4u) * 8u)) & 0xFFu;
        var a = qb;
        var c = cb;
        for (var j = 0u; j < 4u; j = j + 1u) {
            p = p + (a & 3u) * (c & 3u);
            a = a >> 2u;
            c = c >> 2u;
        }
    }
    let scale = row_params[i * 4u];
    let vmin = row_params[i * 4u + 1u];
    let sum_c = row_params[i * 4u + 2u];
    let norm = row_params[i * 4u + 3u];
    let dimf = f32(params.dim);
    let dot = dimf * params.qmin * vmin
        + params.qscale * vmin * params.sum_q
        + params.qmin * scale * sum_c
        + params.qscale * scale * f32(p);
    let cos = select(0.0f, dot / (params.qnorm * norm), norm > 0.0f);
    scores[i] = cos;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScanParams {
    dim: u32,
    n_rows: u32,
    stride: u32,
    _pad: u32,
    qmin: f32,
    qscale: f32,
    sum_q: f32,
    qnorm: f32,
}

/// Core packed-scan dispatch + readback over pre-existing store buffers. The store upload happens
/// once (gpu_store_cache); per query only the query codes, params and score buffer are staged.
fn scan_dispatch(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    codes_buf: &wgpu::Buffer,
    row_params_buf: &wgpu::Buffer,
    q: &crate::packed_ternary::Query,
    n_rows: usize,
    dim: usize,
    stride: usize,
) -> Option<Vec<f32>> {
    let qcodes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("q_codes"), contents: bytemuck::cast_slice(q.codes()), usage: wgpu::BufferUsages::STORAGE,
    });
    let params = ScanParams {
        dim: dim as u32, n_rows: n_rows as u32, stride: stride as u32, _pad: 0,
        qmin: q.qmin() as f32, qscale: q.qscale() as f32, sum_q: q.sum_q() as f32, qnorm: q.norm() as f32,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"), contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM,
    });
    let scores_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scores"), size: (n_rows as u64) * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
    });

    let pipeline = scan_pipeline(device)?;
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scan_bind"), layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: codes_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: row_params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: qcodes_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: scores_buf.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("scan_enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((n_rows as u32 + 255) / 256, 1, 1);
    }
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"), size: (n_rows as u64) * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&scores_buf, 0, &staging, 0, (n_rows as u64) * 4);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    rx.recv().ok()?.ok()?;
    let data = slice.get_mapped_range().ok()?;
    let out: Vec<f32> = bytemuck::cast_slice(&data[..]).to_vec();
    drop(data);
    Some(out)
}

/// Score the whole packed store against one query on the GPU, uploading the store buffers fresh.
/// Used by the parity test, which must not depend on the persistent cache. None on failure -> CPU.
pub fn gpu_scan_scores(
    codes: &[u8],
    row_params: &[f32],
    q: &crate::packed_ternary::Query,
    n_rows: usize,
    dim: usize,
    stride: usize,
) -> Option<Vec<f32>> {
    let (device, queue) = get_device()?;
    let codes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("codes"), contents: bytemuck::cast_slice(codes), usage: wgpu::BufferUsages::STORAGE,
    });
    let rp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("row_params"), contents: bytemuck::cast_slice(row_params), usage: wgpu::BufferUsages::STORAGE,
    });
    scan_dispatch(&device, &queue, &codes_buf, &rp_buf, q, n_rows, dim, stride)
}

/// Persistent GPU-side packed-store buffers, loaded once per process and reused across queries.
/// The corpus store does not change while the kernel runs (rebuilding it requires a restart), so
/// this cache is safe for the process lifetime.
struct GpuStoreCache {
    codes_buf: wgpu::Buffer,
    row_params_buf: wgpu::Buffer,
    path: String,
    bytes: u64,
    n_rows: usize,
    dim: usize,
    stride: usize,
}

static GPU_STORE: OnceLock<Option<GpuStoreCache>> = OnceLock::new();
static GPU_CHEMBL_STORE: OnceLock<Option<GpuStoreCache>> = OnceLock::new();
static GPU_STRUCTURED_STORE: OnceLock<Option<GpuStoreCache>> = OnceLock::new();
static GPU_BIOASSAY_STORE: OnceLock<Option<GpuStoreCache>> = OnceLock::new();
static SCAN_PIPELINE: OnceLock<Option<wgpu::ComputePipeline>> = OnceLock::new();

fn scan_pipeline(device: &wgpu::Device) -> Option<&'static wgpu::ComputePipeline> {
    SCAN_PIPELINE.get_or_init(|| {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scan"), source: wgpu::ShaderSource::Wgsl(SCAN_SHADER.into()),
        });
        Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("scan_pipeline"), layout: None, module: &shader, entry_point: Some("main"),
            compilation_options: Default::default(), cache: None,
        }))
    }).as_ref()
}

/// Build the persistent GPU buffers for a packed store (codes + row params uploaded once).
fn build_store_cache(store: &crate::packed_ternary::Store, device: &wgpu::Device) -> GpuStoreCache {
    let codes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("gpu_store.codes"), contents: bytemuck::cast_slice(store.codes()), usage: wgpu::BufferUsages::STORAGE,
    });
    let row_params = store.row_params();
    let row_params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("gpu_store.row_params"), contents: bytemuck::cast_slice(&row_params), usage: wgpu::BufferUsages::STORAGE,
    });
    GpuStoreCache {
        codes_buf, row_params_buf, path: store.path.clone(), bytes: store.bytes,
        n_rows: store.len(), dim: store.dim(), stride: store.stride(),
    }
}

fn gpu_store_cache() -> Option<&'static GpuStoreCache> {
    GPU_STORE.get_or_init(|| {
        let store = crate::packed_ternary::store()?;
        let (device, _queue) = get_device()?;
        Some(build_store_cache(&store, &device))
    }).as_ref()
}

/// The ChEMBL co-processor's GPU-side store buffers (2.9M rows, 648 MB codes uploaded once).
fn gpu_chembl_store_cache() -> Option<&'static GpuStoreCache> {
    GPU_CHEMBL_STORE.get_or_init(|| {
        let store = crate::packed_ternary::chembl_store()?;
        let (device, _queue) = get_device()?;
        Some(build_store_cache(&store, &device))
    }).as_ref()
}

/// The structured tier's GPU-side store buffers (97,480 rows, ~20 MB codes uploaded once).
fn gpu_structured_store_cache() -> Option<&'static GpuStoreCache> {
    GPU_STRUCTURED_STORE.get_or_init(|| {
        let store = crate::packed_ternary::structured_store()?;
        let (device, _queue) = get_device()?;
        Some(build_store_cache(&store, &device))
    }).as_ref()
}

/// The bioassay store's GPU-side buffers (597,430 rows, ~128 MB codes uploaded once).
fn gpu_bioassay_store_cache() -> Option<&'static GpuStoreCache> {
    GPU_BIOASSAY_STORE.get_or_init(|| {
        let store = crate::packed_ternary::bioassay_store()?;
        let (device, _queue) = get_device()?;
        Some(build_store_cache(&store, &device))
    }).as_ref()
}

/// Pick the GPU cache that matches the store being scanned (fabric, chembl, structured or bioassay), by path.
fn gpu_store_cache_for(s: &crate::packed_ternary::Store) -> Option<&'static GpuStoreCache> {
    if let Some(c) = gpu_store_cache() {
        if c.path == s.path { return Some(c); }
    }
    if let Some(c) = gpu_chembl_store_cache() {
        if c.path == s.path { return Some(c); }
    }
    if let Some(c) = gpu_structured_store_cache() {
        if c.path == s.path { return Some(c); }
    }
    if let Some(c) = gpu_bioassay_store_cache() {
        if c.path == s.path { return Some(c); }
    }
    None
}

/// Score the WHOLE store on the GPU, reusing the persistent store buffers, and return every row's
/// cosine in row order. None when the store is not cached (different path/bytes) or on any device
/// failure -> the CPU loop in packed_ternary::search_in takes over.
pub fn gpu_scan_scores_cached(q: &crate::packed_ternary::Query, s: &crate::packed_ternary::Store) -> Option<Vec<f32>> {
    let cache = gpu_store_cache_for(s)?;
    if cache.bytes != s.bytes || cache.n_rows != s.len() {
        return None;
    }
    let (device, queue) = get_device()?;
    scan_dispatch(&device, &queue, &cache.codes_buf, &cache.row_params_buf, q, cache.n_rows, cache.dim, cache.stride)
}

/// GPU top-K scan: score the whole store and return the top-K (key, cosine) pairs. None on any
/// failure -> CPU scan fallback.
pub fn gpu_scan(query: &[f32], k: usize) -> Option<Vec<(String, f32)>> {
    let store = crate::packed_ternary::store()?;
    let q = crate::packed_ternary::quantize(query, store.dim())?;
    if q.norm() <= 0.0 || store.is_empty() {
        return None;
    }
    let scores = gpu_scan_scores(
        store.codes(),
        &store.row_params(),
        &q,
        store.len(),
        store.dim(),
        store.stride(),
    )?;
    let mut indexed: Vec<(f32, usize)> = scores.iter().cloned().enumerate().map(|(i, s)| (s, i)).collect();
    indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let kk = k.max(1).min(indexed.len());
    Some(indexed.iter().take(kk).map(|(s, i)| (store.keys()[*i].clone(), *s)).collect())
}

/// TRM-7M batched forward: one GPU thread per input runs the full 3-block SSM with FRESH state — the
/// exact semantics the runtime uses (evaluate_proposal constructs a fresh Trm7m and calls forward once,
/// so conv state is all-zero and the conv1d reduces to a pointwise multiply by the last tap). The
/// 15.6 MB weights upload once; per batch only the 128-dim inputs stage. Row-major offsets mirror the
/// CPU forward() byte-for-byte so the parity test is exact.
const TRM7M_SHADER: &str = r#"
const SD: u32 = 462u;
const ID: u32 = 128u;
const IN_PROJ: u32 = 213444u;   // 462*462
const CONV1D: u32 = 1848u;      // 4*462
const DT_PROJ: u32 = 213444u;
const A_VEC: u32 = 462u;
const BM: u32 = 213444u;
const CM: u32 = 213444u;
const DM: u32 = 213444u;
const OUT_PROJ: u32 = 213444u;
const BLOCK_STRIDE: u32 = 1282974u;
const HEAD_OFF: u32 = 3908058u;  // 462*128 + 3*BLOCK_STRIDE
const BIAS_OFF: u32 = 3909444u;

fn fast_exp(x: f32) -> f32 {
    let c = clamp(x, -88.0, 88.0);
    return bitcast<f32>(u32(i32(c * 1.4426950408889634 + 127.0)) << 23u);
}

fn sigmoid(x: f32) -> f32 {
    return 1.0 / (1.0 + fast_exp(-x));
}

struct Params {
    n: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
};

@group(0) @binding(0) var<storage, read> weights: array<f32>;
@group(0) @binding(1) var<storage, read> inputs: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read_write> outputs: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }

    var x = array<f32, 128>();
    for (var k = 0u; k < ID; k = k + 1u) { x[k] = inputs[i * ID + k]; }

    var h = array<f32, 462>();
    for (var r = 0u; r < SD; r = r + 1u) {
        var acc = 0.0f;
        for (var k = 0u; k < ID; k = k + 1u) { acc = acc + x[k] * weights[r * ID + k]; }
        h[r] = acc;
    }

    for (var b = 0u; b < 3u; b = b + 1u) {
        let base = 59136u + b * BLOCK_STRIDE;
        let conv_off = base + IN_PROJ;
        let dtp_off = base + IN_PROJ + CONV1D;
        let av_off = base + IN_PROJ + CONV1D + DT_PROJ;
        let bm_off = av_off + A_VEC;
        let cm_off = bm_off + BM;
        let dm_off = cm_off + CM;
        let op_off = dm_off + DM;

        var x_proj = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            var acc = 0.0f;
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + h[k] * weights[base + r * SD + k]; }
            x_proj[r] = acc;
        }

        // fresh conv state -> only the last tap contributes
        var x_conv = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            x_conv[r] = x_proj[r] * weights[conv_off + 3u * SD + r] / 4.0;
        }

        var dt_raw = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            var acc = 0.0f;
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + x_conv[k] * weights[dtp_off + r * SD + k]; }
            dt_raw[r] = acc;
        }

        var h_new = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            let dt = fast_exp(dt_raw[r] * 0.1);
            let ea = fast_exp(weights[av_off + r] * dt);
            var acc = 0.0f;
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + (weights[bm_off + r * SD + k] * dt) * x_conv[k]; }
            h_new[r] = ea * h[r] + acc;
        }

        var y = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            var acc = 0.0f;
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + h_new[k] * weights[cm_off + r * SD + k]; }
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + x_conv[k] * weights[dm_off + r * SD + k]; }
            y[r] = acc;
        }

        var y_out = array<f32, 462>();
        for (var r = 0u; r < SD; r = r + 1u) {
            var acc = 0.0f;
            for (var k = 0u; k < SD; k = k + 1u) { acc = acc + y[k] * weights[op_off + r * SD + k]; }
            y_out[r] = acc * sigmoid(acc);
        }

        h = y_out;
    }

    var logits = array<f32, 3>();
    for (var r = 0u; r < 3u; r = r + 1u) {
        var acc = 0.0f;
        for (var k = 0u; k < SD; k = k + 1u) { acc = acc + h[k] * weights[HEAD_OFF + r * SD + k]; }
        logits[r] = acc + weights[BIAS_OFF + r];
    }

    var max_l = max(logits[0], max(logits[1], logits[2]));
    var probs = array<f32, 3>();
    var sum = 0.0f;
    for (var r = 0u; r < 3u; r = r + 1u) { probs[r] = fast_exp(logits[r] - max_l); sum = sum + probs[r]; }
    for (var r = 0u; r < 3u; r = r + 1u) { probs[r] = probs[r] / sum; }

    var best = 0u;
    for (var r = 1u; r < 3u; r = r + 1u) { if (probs[r] > probs[best]) { best = r; } }
    var trit: f32 = -1.0;
    if (best == 0u) { trit = 1.0; } else if (best == 1u) { trit = 0.0; }
    outputs[i * 2u] = trit;
    outputs[i * 2u + 1u] = probs[best];
}
"#;

static TRM7M_WEIGHTS_BUF: OnceLock<Option<wgpu::Buffer>> = OnceLock::new();

fn trm7m_weights_buf(device: &wgpu::Device) -> Option<&'static wgpu::Buffer> {
    TRM7M_WEIGHTS_BUF
        .get_or_init(|| {
            let w = crate::trm7m::Trm7m::weights_f32();
            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("trm7m.weights"),
                contents: bytemuck::cast_slice(&w),
                usage: wgpu::BufferUsages::STORAGE,
            }))
        })
        .as_ref()
}

/// Run N trm7m forwards on the GPU in one dispatch, returning (trit, confidence) per input in order.
/// None on any failure -> CPU forward fallback.
pub fn gpu_trm7m_batch(inputs: &[[f32; 128]]) -> Option<Vec<(i8, f32)>> {
    if inputs.is_empty() {
        return Some(Vec::new());
    }
    let (device, queue) = get_device()?;
    let weights_buf = trm7m_weights_buf(&device)?;
    let flat: Vec<f32> = inputs.iter().flatten().copied().collect();
    let inputs_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("trm7m.inputs"), contents: bytemuck::cast_slice(&flat), usage: wgpu::BufferUsages::STORAGE,
    });
    let n = inputs.len() as u32;
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("trm7m.params"), contents: bytemuck::cast_slice(&[n, 0u32, 0u32, 0u32]), usage: wgpu::BufferUsages::UNIFORM,
    });
    let outputs_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("trm7m.outputs"), size: (n as u64) * 8,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("trm7m"), source: wgpu::ShaderSource::Wgsl(TRM7M_SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("trm7m_pipeline"), layout: None, module: &shader, entry_point: Some("main"),
        compilation_options: Default::default(), cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("trm7m_bind"), layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: weights_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: inputs_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: params_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: outputs_buf.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("trm7m_enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((n + 63) / 64, 1, 1);
    }
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("trm7m_staging"), size: (n as u64) * 8,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&outputs_buf, 0, &staging, 0, (n as u64) * 8);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    rx.recv().ok()?.ok()?;
    let data = slice.get_mapped_range().ok()?;
    let out: Vec<f32> = bytemuck::cast_slice(&data[..]).to_vec();
    drop(data);
    Some(out.chunks_exact(2).map(|c| (c[0] as i8, c[1])).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One ordered GPU pass. Four separate `#[test]`s shared one wgpu device and a lazily-built
    /// pipeline: the device handled them concurrently and the bind-group layout came back invalid
    /// (wgpu Validation Error in `ComputePipeline::get_bind_group_layout`) depending on which test
    /// initialised the cache first. A single test on one thread runs the four checks in a fixed order,
    /// so the suite can stay parallel without giving the GPU path an order it never had.
    /// Ignored in the default run: on Windows a GPU kernel longer than the TDR window resets the
    /// adapter, and the reset invalidates every cached pipeline in the process. Another test's
    /// full-store scan is long enough to do it, so this check passes alone and fails beside it. Run:
    ///     cargo test --release gpu_encode_parity -- --ignored --test-threads=1
    #[test]
    fn gpu_encode_parity() {
        let _gpu = crate::test_support::gpu();
        gpu_matches_cpu_reference();
        gpu_batch_matches_cpu();
    }

    /// Separate from the encoder checks: when this ran inside one combined test the shared device was
    /// already carrying the encoder's pipelines and creation came back invalid ("ComputePipeline with
    /// 'trm7m_pipeline' label is invalid"). Alone, it passes.
    #[test]
    fn gpu_trm7m_parity() {
        let _gpu = crate::test_support::gpu();
        gpu_trm7m_matches_cpu();
    }

    #[test]
    fn gpu_scan_parity() {
        let _gpu = crate::test_support::gpu();
        gpu_scan_matches_cpu();
    }

    /// Verify condition: the GPU encoder must reproduce the CPU reference on every probe text,
    /// element-wise (a stated tolerance, not exact, because f32 accumulation order can differ).
    fn gpu_matches_cpu_reference() {
        if !gpu_available() {
            eprintln!("no GPU adapter on this host — skipping parity test");
            return;
        }
        let texts = [
            "LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082",
            "protein binding affinity of a small molecule drug",
            "the quick brown fox jumps over the lazy dog",
            "",
            "   ",
            "!!!",
            "a",
        ];
        for t in texts {
            let gpu = gpu_encode_f32(t).unwrap();
            let cpu = crate::native_encoder::encode_f32(t, DIM);
            let mut worst = 0.0f32;
            for i in 0..DIM {
                worst = worst.max((gpu[i] - cpu[i]).abs());
            }
            assert!(
                worst < 1e-4,
                "GPU/CPU divergence {worst} on {t:?} exceeds tolerance"
            );
        }
    }

    /// Batch parity: the multi-doc kernel must reproduce the per-doc CPU reference, in order.
    fn gpu_batch_matches_cpu() {
        if !gpu_available() {
            eprintln!("no GPU adapter on this host — skipping parity test");
            return;
        }
        let texts = [
            "LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082",
            "protein binding affinity of a small molecule drug",
            "Kleene K3 balanced ternary logic consensus engine",
            "high-frequency ionosphere radio propagation and solar flare dynamics",
            "the quick brown fox jumps over the lazy dog",
        ];
        let batch = gpu_encode_batch(&texts).unwrap();
        assert_eq!(batch.len(), texts.len());
        for (d, t) in texts.iter().enumerate() {
            let cpu = crate::native_encoder::encode_f32(t, DIM);
            let mut worst = 0.0f32;
            for i in 0..DIM {
                worst = worst.max((batch[d][i] - cpu[i]).abs());
            }
            assert!(worst < 1e-4, "batch GPU/CPU divergence {worst} on doc {d} {t:?}");
        }
    }

    /// Packed-scan parity: the GPU scan's top-K keys must match the CPU packed_ternary::search.
    fn gpu_scan_matches_cpu() {
        if !gpu_available() {
            eprintln!("no GPU adapter on this host — skipping parity test");
            return;
        }
        let query = crate::native_encoder::encode_f32("protein binding affinity small molecule", DIM);
        // An absent packed store is a missing precondition, not a GPU defect: this host has no store file,
        // so the parity comparison has nothing to scan. Skipped with the reason stated instead of unwrapping.
        let store = match crate::packed_ternary::store() {
            Some(s) => s,
            None => {
                eprintln!("no packed store on this host — skipping scan parity test");
                return;
            }
        };
        let q = crate::packed_ternary::quantize(&query, store.dim()).unwrap();

        // True CPU reference (the byte-for-byte loop the GPU kernel is parity-tested against).
        let t_cpu = std::time::Instant::now();
        let mut cpu = crate::packed_ternary::cpu_scan_in(&store, &q);
        let cpu_us = t_cpu.elapsed().as_secs_f64() * 1e6;
        cpu.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let t_gpu = std::time::Instant::now();
        let gpu = gpu_scan_scores_cached(&q, &store).unwrap();
        let gpu_us = t_gpu.elapsed().as_secs_f64() * 1e6;
        let mut gi: Vec<(f32, usize)> = gpu.iter().cloned().enumerate().map(|(i, sc)| (sc, i)).collect();
        gi.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        eprintln!("[timing] cpu_scan_in {:.1} us vs gpu_scan_scores_cached {:.1} us", cpu_us, gpu_us);

        for rank in 0..10 {
            assert_eq!(gi[rank].1, cpu[rank].1, "top-K row index mismatch at rank {rank}");
            assert!((gi[rank].0 as f64 - cpu[rank].0).abs() < 1e-3,
                "score mismatch at rank {rank}: gpu={} cpu={}", gi[rank].0, cpu[rank].0);
        }
    }

    /// Batched trm7m parity: the GPU forward must reproduce the CPU forward (fresh state, one call —
    /// the exact semantics evaluate_proposal uses) element-wise.
    fn gpu_trm7m_matches_cpu() {
        if !gpu_available() {
            eprintln!("no GPU adapter on this host — skipping parity test");
            return;
        }
        let texts = [
            "LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082",
            "protein binding affinity of a small molecule drug",
            "Kleene K3 balanced ternary logic consensus engine",
            "high-frequency ionosphere radio propagation and solar flare dynamics",
            "the quick brown fox jumps over the lazy dog",
            "pharmacophore coordination geometry",
        ];
        let inputs: Vec<[f32; 128]> = texts.iter().map(|t| crate::trm7m::Trm7m::encode_text(t)).collect();
        let cpu: Vec<(i8, f32)> = texts.iter().map(|t| {
            let mut m = crate::trm7m::Trm7m::new();
            let x = crate::trm7m::Trm7m::encode_text(t);
            let (trit, conf, _) = m.forward(&x);
            (trit, conf)
        }).collect();
        let gpu = gpu_trm7m_batch(&inputs).unwrap();
        assert_eq!(gpu.len(), cpu.len());
        for (idx, ((ct, cc), (gt, gc))) in cpu.iter().zip(gpu.iter()).enumerate() {
            assert_eq!(ct, gt, "trit mismatch at {idx}: cpu {ct} gpu {gt} ({})", texts[idx]);
            assert!((cc - gc).abs() < 1e-4, "confidence mismatch at {idx}: cpu {cc} gpu {gc} ({})", texts[idx]);
        }
    }
}

#[cfg(test)]
mod trit_bench {
    /// Measures the two packed-ternary paths on the same query: the CPU integer scan the /k3 route uses, and the
    /// cached GPU scan the grounding path uses. Prints one JSON line so the numbers can be quoted verbatim, and
    /// checks that both rank the same rows -- speed without parity is not the same computation.
    #[test]
    fn ternary_gpu_vs_cpu_observable() {
        use super::gpu_scan_scores_cached;
        use std::time::Instant;

        let store = match crate::packed_ternary::store() {
            Some(s) => s,
            None => {
                println!("{{\"status\":\"store_unavailable\"}}");
                return;
            }
        };
        let dim = store.dim();
        let qv: Vec<f32> = (0..dim).map(|i| (((i * 37) % 61) as f32 - 30.0) / 30.0).collect();
        let q = match crate::packed_ternary::quantize(&qv, dim) {
            Some(q) => q,
            None => {
                println!("{{\"status\":\"quantize_failed\"}}");
                return;
            }
        };

        let t0 = Instant::now();
        let mut cpu_rank: Vec<(f64, usize)> = Vec::new();
        for _ in 0..3 {
            cpu_rank = crate::packed_ternary::cpu_scan_in(&store, &q);
        }
        let cpu_ms = t0.elapsed().as_secs_f64() * 1000.0 / 3.0;

        let _ = gpu_scan_scores_cached(&q, &store); // warm the persistent buffers
        let t1 = Instant::now();
        let mut gpu_scores: Option<Vec<f32>> = None;
        for _ in 0..3 {
            gpu_scores = gpu_scan_scores_cached(&q, &store);
        }
        let gpu_ms = t1.elapsed().as_secs_f64() * 1000.0 / 3.0;

        // cpu_scan_in returns EVERY row unsorted -- the caller sorts. Truncating it as-is ranked the first five
        // rows instead of the best five, which is what made the first run report overlap 0.
        cpu_rank.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut cpu_top: Vec<usize> = cpu_rank.iter().map(|(_, i)| *i).collect();
        cpu_top.truncate(5);
        let (gpu_ok, gpu_top) = match &gpu_scores {
            Some(scores) => {
                let mut idx: Vec<usize> = (0..scores.len()).collect();
                idx.sort_by(|a, b| scores[*b].partial_cmp(&scores[*a]).unwrap_or(std::cmp::Ordering::Equal));
                idx.truncate(5);
                (true, idx)
            }
            None => (false, Vec::new()),
        };
        let overlap = gpu_top.iter().filter(|i| cpu_top.contains(i)).count();

        let ternaries = (store.len() * dim) as f64;
        println!(
            "TRIT_BENCH {{\"rows\":{},\"dim\":{},\"ternaries_per_scan\":{:.0},\"cpu_ms\":{:.2},\"gpu_ms\":{:.2},\"cpu_g_trit_s\":{:.2},\"gpu_g_trit_s\":{:.2},\"speedup\":{:.1},\"gpu_available\":{},\"top5_overlap\":{}}}",
            store.len(), dim, ternaries, cpu_ms, gpu_ms,
            ternaries / cpu_ms / 1e6, ternaries / gpu_ms / 1e6, cpu_ms / gpu_ms, gpu_ok, overlap
        );
        assert!(overlap >= 3, "GPU and CPU disagree on the top rows: cpu {:?} gpu {:?}", cpu_top, gpu_top);
    }

    /// Batch vs single submit on the same store and the same queries. Batching is allowed to change only the
    /// submission cost, never the numbers, so every score must match the single-query path.
    #[test]
    fn ternary_batch_matches_single_and_costs_less() {
        use super::{gpu_scan_batch_cached, gpu_scan_scores_cached};
        use std::time::Instant;

        let store = match crate::packed_ternary::store() {
            Some(s) => s,
            None => { println!("{{\"status\":\"store_unavailable\"}}"); return; }
        };
        let dim = store.dim();
        let mut queries = Vec::new();
        for qi in 0..8 {
            let qv: Vec<f32> = (0..dim)
                .map(|i| ((((i * 37 + qi * 11) % 61) as f32) - 30.0) / 30.0)
                .collect();
            match crate::packed_ternary::quantize(&qv, dim) {
                Some(q) => queries.push(q),
                None => { println!("{{\"status\":\"quantize_failed\"}}"); return; }
            }
        }

        let _ = gpu_scan_scores_cached(&queries[0], &store); // warm device + store buffers
        let t_single = Instant::now();
        let mut single: Vec<Vec<f32>> = Vec::new();
        for q in &queries {
            match gpu_scan_scores_cached(q, &store) {
                Some(v) => single.push(v),
                None => { println!("{{\"status\":\"gpu_unavailable\"}}"); return; }
            }
        }
        let single_ms = t_single.elapsed().as_secs_f64() * 1000.0;

        let _ = gpu_scan_batch_cached(&queries[..1], &store); // warm the batched path
        let t_batch = Instant::now();
        let batch = match gpu_scan_batch_cached(&queries, &store) {
            Some(v) => v,
            None => { println!("{{\"status\":\"batch_unavailable\"}}"); return; }
        };
        let batch_ms = t_batch.elapsed().as_secs_f64() * 1000.0;

        let mut worst = 0.0f32;
        for (a, b) in single.iter().zip(batch.iter()) {
            for (x, y) in a.iter().zip(b.iter()) {
                worst = worst.max((x - y).abs());
            }
        }
        let n = queries.len() as f64;
        println!(
            "TRIT_BATCH {{\"queries\":{},\"rows\":{},\"single_total_ms\":{:.2},\"batch_total_ms\":{:.2},\"single_ms_per_query\":{:.3},\"batch_ms_per_query\":{:.3},\"speedup\":{:.1},\"max_abs_diff\":{:.3e}}}",
            queries.len(), store.len(), single_ms, batch_ms, single_ms / n, batch_ms / n, single_ms / batch_ms, worst
        );
        assert!(worst < 1e-5, "batch changed the scores: max abs diff {}", worst);
    }

    /// Word-at-a-time shader vs the shipped byte shader: same scores, less work.
    #[test]
    fn ternary_u32_lanes_match_byte_loop() {
        use super::{gpu_scan_scores_cached, gpu_scan_u32_cached};
        use std::time::Instant;

        let store = match crate::packed_ternary::store() {
            Some(s) => s,
            None => { println!("{{\"status\":\"store_unavailable\"}}"); return; }
        };
        let dim = store.dim();
        let qv: Vec<f32> = (0..dim).map(|i| (((i * 37) % 61) as f32 - 30.0) / 30.0).collect();
        let q = match crate::packed_ternary::quantize(&qv, dim) {
            Some(q) => q,
            None => { println!("{{\"status\":\"quantize_failed\"}}"); return; }
        };

        let _ = gpu_scan_scores_cached(&q, &store);
        let _ = gpu_scan_u32_cached(&q, &store);

        let mut byte_scores = Vec::new();
        let t0 = Instant::now();
        for _ in 0..5 { byte_scores = gpu_scan_scores_cached(&q, &store).unwrap_or_default(); }
        let byte_ms = t0.elapsed().as_secs_f64() * 1000.0 / 5.0;

        let mut word_scores = Vec::new();
        let t1 = Instant::now();
        for _ in 0..5 { word_scores = gpu_scan_u32_cached(&q, &store).unwrap_or_default(); }
        let word_ms = t1.elapsed().as_secs_f64() * 1000.0 / 5.0;

        let mut worst = 0.0f32;
        for (a, b) in byte_scores.iter().zip(word_scores.iter()) {
            worst = worst.max((a - b).abs());
        }
        let ternaries = (store.len() * dim) as f64;
        println!(
            "TRIT_U32 {{\"rows\":{},\"byte_ms\":{:.2},\"u32_ms\":{:.2},\"speedup\":{:.2},\"byte_g_trit_s\":{:.2},\"u32_g_trit_s\":{:.2},\"max_abs_diff\":{:.3e}}}",
            store.len(), byte_ms, word_ms, byte_ms / word_ms,
            ternaries / byte_ms / 1e6, ternaries / word_ms / 1e6, worst
        );
        assert!(worst < 1e-6, "u32 lanes changed the scores: max abs diff {}", worst);
    }

}

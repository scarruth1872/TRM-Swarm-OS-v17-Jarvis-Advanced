//! Real Vulkan / wgpu Compute Backend for 584-Agent K3 Bitboard Reductions.
//!
//! Targets the host's AMD Radeon RX 6700 XT via `wgpu` (Vulkan backend),
//! compiling a WGSL compute shader that reduces 584 agent trits (`-1, 0, +1`)
//! into bitboard counts and Kleene-3 consensus in parallel GPU workgroups.

use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Instant;

const K3_REDUCE_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> trits_in: array<i32>;
@group(0) @binding(1) var<storage, read_write> stats_out: array<atomic<u32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= arrayLength(&trits_in)) {
        return;
    }
    let t = trits_in[idx];
    if (t > 0) {
        atomicAdd(&stats_out[0], 1u); // +1 count
    } else if (t < 0) {
        atomicAdd(&stats_out[1], 1u); // -1 fault count
    } else {
        atomicAdd(&stats_out[2], 1u); // 0 unknown count
    }
    atomicAdd(&stats_out[3], 1u); // total evaluated
}
"#;

const K3_INFINITY_WGSL: &str = r#"
struct Params {
    num_pairs: u32,
    passes: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> planes_a: array<u32>;
@group(0) @binding(2) var<storage, read> planes_b: array<u32>;
@group(0) @binding(3) var<storage, read_write> acc_out: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let tid = gid.x;
    let total_threads = nwg.x * 256u;
    let n = params.num_pairs;
    let passes = params.passes;
    var sum: u32 = 0u;
    for (var p: u32 = 0u; p < passes; p = p + 1u) {
        var i: u32 = tid;
        let offset = (p * 1021u) % n;
        while (i < n) {
            let idx = (i + offset) % n;
            let base = idx * 2u;
            let pa = planes_a[base];
            let na = planes_a[base + 1u];
            let pb = planes_b[base];
            let nb = planes_b[base + 1u];
            // Kleene-3 AND (min) on 32 trits in parallel:
            //   pos_out = pa & pb, neg_out = na | nb
            let pos_out = pa & pb;
            let neg_out = na | nb;
            sum = sum + countOneBits(pos_out) + countOneBits(neg_out);
            i = i + total_threads;
        }
    }
    acc_out[tid] = sum;
}
"#;

struct GpuCtx {
    adapter_name: String,
    backend: String,
    device_type: String,
    vendor: u32,
    device_id: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    inf_pipeline: wgpu::ComputePipeline,
    inf_bgl: wgpu::BindGroupLayout,
}

static GPU_CTX: OnceLock<Result<GpuCtx, String>> = OnceLock::new();

fn init_gpu() -> &'static Result<GpuCtx, String> {
    GPU_CTX.get_or_init(|| {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| format!("no Vulkan adapter found: {e:?}"))?;

        let info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("trm_k3_compute"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .map_err(|e| format!("request_device failed: {e}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("k3_reduce_wgsl"),
            source: wgpu::ShaderSource::Wgsl(K3_REDUCE_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("k3_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("k3_pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("k3_compute_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let inf_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("k3_infinity_wgsl"),
            source: wgpu::ShaderSource::Wgsl(K3_INFINITY_WGSL.into()),
        });

        let inf_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("k3_inf_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let inf_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("k3_inf_pl"),
            bind_group_layouts: &[Some(&inf_bgl)],
            immediate_size: 0,
        });

        let inf_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("k3_inf_pipeline"),
            layout: Some(&inf_pl),
            module: &inf_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(GpuCtx {
            adapter_name: info.name,
            backend: format!("{:?}", info.backend),
            device_type: format!("{:?}", info.device_type),
            vendor: info.vendor,
            device_id: info.device,
            device,
            queue,
            pipeline,
            bind_group_layout,
            inf_pipeline,
            inf_bgl,
        })
    })
}

pub fn status_json() -> Value {
    match init_gpu() {
        Ok(ctx) => json!({
            "available": true,
            "adapter": ctx.adapter_name,
            "backend": ctx.backend,
            "device_type": ctx.device_type,
            "vendor_id": format!("0x{:04x}", ctx.vendor),
            "device_id": format!("0x{:04x}", ctx.device_id),
            "shader": "k3_reduce_wgsl (workgroup_size=64)",
        }),
        Err(err) => json!({
            "available": false,
            "error": err,
        }),
    }
}

pub fn run_k3_dispatch(agent_count: usize) -> Value {
    let ctx = match init_gpu() {
        Ok(c) => c,
        Err(err) => {
            return json!({
                "status": "GPU_UNAVAILABLE",
                "error": err,
            });
        }
    };

    let n = agent_count.clamp(1, 65_536);
    let trits: Vec<i32> = (0..n)
        .map(|i| if i < 300 { 1 } else if i < 450 { 0 } else { 1 })
        .collect();
    let expected_pos = trits.iter().filter(|&&t| t > 0).count() as u32;
    let expected_neg = trits.iter().filter(|&&t| t < 0).count() as u32;
    let expected_unk = trits.iter().filter(|&&t| t == 0).count() as u32;

    let t0 = Instant::now();

    let in_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("trits_in"),
        size: (trits.len() * std::mem::size_of::<i32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&in_buf, 0, bytemuck::cast_slice(&trits));

    let out_bytes = 4 * std::mem::size_of::<u32>() as u64;
    let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("stats_out"),
        size: out_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&out_buf, 0, bytemuck::cast_slice(&[0u32; 4]));

    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging_readback"),
        size: out_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("k3_bg"),
        layout: &ctx.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: in_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: out_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("k3_enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("k3_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let workgroups = (n as u32).div_ceil(64);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, out_bytes);
    ctx.queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    let _ = ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let _ = rx.recv();

    let data = match slice.get_mapped_range() {
        Ok(view) => view,
        Err(e) => {
            return json!({
                "status": "READBACK_ERROR",
                "error": format!("{e:?}"),
            });
        }
    };
    let counts: &[u32] = bytemuck::cast_slice(&data);
    let pos = counts[0];
    let neg = counts[1];
    let unk = counts[2];
    let total = counts[3];
    drop(data);
    staging.unmap();

    let elapsed_us = t0.elapsed().as_micros();
    let consensus: i8 = if neg > 0 { -1 } else if unk > 0 { 0 } else { 1 };
    let verified = pos == expected_pos && neg == expected_neg && unk == expected_unk && total as usize == n;

    json!({
        "status": if verified { "OK" } else { "MISMATCH" },
        "adapter": ctx.adapter_name,
        "backend": ctx.backend,
        "agents_dispatched": n,
        "workgroups_64": (n as u32).div_ceil(64),
        "counts": {
            "plus_1": pos,
            "minus_1": neg,
            "zero": unk,
            "total": total,
        },
        "consensus_trit": consensus,
        "verified": verified,
        "elapsed_us": elapsed_us,
    })
}

/// Benchmark bit-sliced Kleene-3 ternary compute across working-set sizes that fit inside
/// vs. exceed the RX 6700 XT's 96 MB L3 Infinity Cache, measuring real GPU throughput and bandwidth.
pub fn run_infinity_cache_bench() -> Value {
    let ctx = match init_gpu() {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "status": "GPU_UNAVAILABLE",
                "error": e,
            });
        }
    };

    // Working-set tiers (total bytes across planes_a + planes_b):
    //   8 MB, 32 MB, 64 MB -> fit inside 96 MB Infinity Cache
    //   192 MB             -> exceeds 96 MB Infinity Cache (spills to GDDR6)
    let tiers: &[(usize, u32, &str)] = &[
        (8, 64, "inside_96mb_infinity_cache"),
        (32, 32, "inside_96mb_infinity_cache"),
        (64, 32, "inside_96mb_infinity_cache"),
        (192, 16, "exceeds_96mb_gddr6_spill"),
    ];

    let workgroups: u32 = 1024; // 1024 * 256 = 262,144 GPU threads
    let total_threads: usize = (workgroups as usize) * 256;
    let out_bytes = (total_threads * std::mem::size_of::<u32>()) as u64;

    let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("k3_inf_out"),
        size: out_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("k3_inf_staging"),
        size: out_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut tier_results = Vec::new();
    let mut cache_best_gbps: f64 = 0.0;
    let mut cache_best_bops: f64 = 0.0;
    let mut spill_gbps: f64 = 0.0;

    for &(ws_mb, passes, residency) in tiers {
        let buf_bytes = (ws_mb * 1024 * 1024) / 2;
        let num_pairs = buf_bytes / 8; // 2 x u32 = 8 bytes = 32 trits
        let num_u32 = num_pairs * 2;

        // Deterministic bit-planes: pa=0x55555555, na=0x22222222, pb=0x33333333, nb=0x44444444
        // pa & pb = 0x11111111 (8 ones), na | nb = 0x66666666 (16 ones) -> 24 per pair
        let mut a_host = vec![0u32; num_u32];
        let mut b_host = vec![0u32; num_u32];
        for i in 0..num_pairs {
            a_host[i * 2] = 0x5555_5555;
            a_host[i * 2 + 1] = 0x2222_2222;
            b_host[i * 2] = 0x3333_3333;
            b_host[i * 2 + 1] = 0x4444_4444;
        }

        let buf_a = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("k3_inf_a"),
            size: buf_bytes as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buf_b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("k3_inf_b"),
            size: buf_bytes as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ctx.queue.write_buffer(&buf_a, 0, bytemuck::cast_slice(&a_host));
        ctx.queue.write_buffer(&buf_b, 0, bytemuck::cast_slice(&b_host));

        // 1. Warm-up pass (1 pass) to populate GPU caches and finish upload before timing
        let warm_params: [u32; 4] = [num_pairs as u32, 1, 0, 0];
        let param_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("k3_inf_params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ctx.queue.write_buffer(&param_buf, 0, bytemuck::cast_slice(&warm_params));

        let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("k3_inf_bg"),
            layout: &ctx.inf_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: param_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_a.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_b.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: out_buf.as_entire_binding() },
            ],
        });

        let mut warm_enc = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("k3_inf_warm") });
        {
            let mut pass = warm_enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("warm_pass"), timestamp_writes: None });
            pass.set_pipeline(&ctx.inf_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        ctx.queue.submit(Some(warm_enc.finish()));
        let _ = ctx.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });

        // 2. Timed multi-pass dispatch (pure GPU compute + cache/GDDR6 bandwidth)
        let run_params: [u32; 4] = [num_pairs as u32, passes, 0, 0];
        ctx.queue.write_buffer(&param_buf, 0, bytemuck::cast_slice(&run_params));

        let t0 = Instant::now();
        let mut enc = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("k3_inf_run") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("run_pass"), timestamp_writes: None });
            pass.set_pipeline(&ctx.inf_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        enc.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, out_bytes);
        ctx.queue.submit(Some(enc.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = ctx.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
        let _ = rx.recv();
        let elapsed_s = t0.elapsed().as_secs_f64();

        let checksum_got: u64 = match slice.get_mapped_range() {
            Ok(view) => {
                let vals: &[u32] = bytemuck::cast_slice(&view);
                vals.iter().map(|&x| x as u64).sum()
            }
            Err(_) => 0,
        };
        staging.unmap();

        let checksum_expected: u64 = (num_pairs as u64) * (passes as u64) * 24u64;
        let verified = checksum_got == checksum_expected;
        let total_bytes_read = (ws_mb as f64) * 1024.0 * 1024.0 * (passes as f64);
        let eff_gbps = (total_bytes_read / elapsed_s) / 1e9;
        let total_trit_ops = (num_pairs as f64) * 32.0 * (passes as f64);
        let billion_k3_ops_per_sec = (total_trit_ops / elapsed_s) / 1e9;

        if ws_mb <= 64 {
            if eff_gbps > cache_best_gbps {
                cache_best_gbps = eff_gbps;
            }
            if billion_k3_ops_per_sec > cache_best_bops {
                cache_best_bops = billion_k3_ops_per_sec;
            }
        } else {
            spill_gbps = eff_gbps;
        }

        tier_results.push(json!({
            "working_set_mb": ws_mb,
            "residency": residency,
            "trits_resident": (num_pairs as u64) * 32,
            "passes": passes,
            "elapsed_ms": (elapsed_s * 1000.0 * 100.0).round() / 100.0,
            "effective_bandwidth_gb_s": (eff_gbps * 10.0).round() / 10.0,
            "k3_billion_ops_per_sec": (billion_k3_ops_per_sec * 100.0).round() / 100.0,
            "verified": verified,
        }));
    }

    let cache_speedup_vs_spill = if spill_gbps > 0.0 {
        (cache_best_gbps / spill_gbps * 100.0).round() / 100.0
    } else {
        0.0
    };

    json!({
        "status": "MEASURED",
        "adapter": ctx.adapter_name,
        "backend": ctx.backend,
        "shader": "k3_infinity_wgsl (2-bit split-plane K3 AND + popcount)",
        "tiers": tier_results,
        "summary": {
            "peak_cache_resident_gb_s": (cache_best_gbps * 10.0).round() / 10.0,
            "peak_cache_resident_billion_k3_ops_s": (cache_best_bops * 100.0).round() / 100.0,
            "gddr6_spill_gb_s": (spill_gbps * 10.0).round() / 10.0,
            "infinity_cache_speedup_ratio": cache_speedup_vs_spill,
            "honest_note": "Hardware-managed 96 MB Infinity Cache accelerates cache-resident Vulkan bit-sliced K3 compute; computation executes in GPU CUs, not inside SRAM cells."
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_k3_dispatch_if_present() {
        let st = status_json();
        if st["available"] == json!(true) {
            let res = run_k3_dispatch(584);
            assert_eq!(res["verified"], json!(true));
            assert_eq!(res["counts"]["total"], json!(584));
        }
    }
}

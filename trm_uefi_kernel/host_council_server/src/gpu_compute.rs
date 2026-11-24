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
            required_limits: wgpu::Limits::downlevel_defaults(),
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

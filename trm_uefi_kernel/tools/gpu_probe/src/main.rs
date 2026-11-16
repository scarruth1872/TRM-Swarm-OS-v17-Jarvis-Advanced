//! Roster-on-GPU feasibility probe.
//!
//! The question is not "can wgpu do matmuls" but "what does one dispatch cost end to end on THIS
//! adapter", because a K3 head is only 522k params (~1.04 MFLOP per pass). At that size kernel
//! arithmetic is free and launch latency is everything: if a round trip is hundreds of
//! microseconds then per-head dispatch is pointless and the roster must go out in one submission.

use std::time::Instant;

const HEADS: u32 = 15;
const ROWS: u32 = 64; // HIDDEN per head
const TOTAL_ROWS: u32 = HEADS * ROWS;

fn main() {
    let mut idesc = wgpu::InstanceDescriptor::new_without_display_handle();
    idesc.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(idesc);

    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
    for a in adapters {
        let i = a.get_info();
        println!("adapter: {} | {:?} | {:?}", i.name, i.backend, i.device_type);
    }

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        ..Default::default()
    })) {
        Ok(a) => a,
        Err(e) => {
            println!("{{\"error\":\"no Vulkan adapter: {e}\"}}");
            return;
        }
    };
    let info = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("probe"),
        ..Default::default()
    }))
    .expect("device");

    // ---- 1. upload cost for the real roster footprint ----
    let roster_bytes = 31_322_002usize;
    let roster = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("roster"),
        size: roster_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let blob = vec![0.37f32; roster_bytes / 4];
    queue.write_buffer(&roster, 0, bytemuck::cast_slice(&blob));
    device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .expect("poll");

    let t = Instant::now();
    for _ in 0..10 {
        queue.write_buffer(&roster, 0, bytemuck::cast_slice(&blob));
        device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
    }
    let upload_us = t.elapsed().as_secs_f64() * 1e6 / 10.0;

    // ---- 2. kernel: one row of w per invocation ----
    let w_n = (TOTAL_ROWS * 96) as usize;
    let w_host: Vec<f32> = (0..w_n).map(|i| ((i % 17) as f32) * 0.03 - 0.2).collect();
    let w = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("w"),
        size: (w_n * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&w, 0, bytemuck::cast_slice(&w_host));

    let x_host = vec![0.5f32; 96];
    let x = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("x"),
        size: (96 * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&x, 0, bytemuck::cast_slice(&x_host));

    let y = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("y"),
        size: (TOTAL_ROWS as usize * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: (TOTAL_ROWS as usize * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let rows_one = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rows_one"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&rows_one, 0, bytemuck::cast_slice(&[ROWS, 0u32, 0, 0]));
    let rows_all = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rows_all"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&rows_all, 0, bytemuck::cast_slice(&[TOTAL_ROWS, 0u32, 0, 0]));

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("probe"),
        source: wgpu::ShaderSource::Wgsl(include_str!("probe.wgsl").into()),
    });
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("matvec"),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("matvec"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mk = |p: &wgpu::Buffer| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: w.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: x.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: y.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: p.as_entire_binding() },
            ],
        })
    };
    let bg_one = mk(&rows_one);
    let bg_all = mk(&rows_all);

    let run = |bg: &wgpu::BindGroup, groups: u32| -> f64 {
        let t = Instant::now();
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cp.set_pipeline(&pipeline);
            cp.set_bind_group(0, bg, &[]);
            cp.dispatch_workgroups(groups, 1, 1);
        }
        enc.copy_buffer_to_buffer(&y, 0, &staging, 0, 256);
        queue.submit(Some(enc.finish()));
        let slice = staging.slice(0..256);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
        staging.unmap();
        t.elapsed().as_secs_f64() * 1e6
    };

    let mut one = Vec::new();
    for _ in 0..60 {
        one.push(run(&bg_one, (ROWS + 63) / 64));
    }
    one.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut all = Vec::new();
    for _ in 0..60 {
        all.push(run(&bg_all, (TOTAL_ROWS + 63) / 64));
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // ---- 3. throughput: 200 roster dispatches, one sync at the end ----
    let t = Instant::now();
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    for _ in 0..200 {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        cp.set_pipeline(&pipeline);
        cp.set_bind_group(0, &bg_all, &[]);
        cp.dispatch_workgroups((TOTAL_ROWS + 63) / 64, 1, 1);
    }
    queue.submit(Some(enc.finish()));
    device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .expect("poll");
    let async_us = t.elapsed().as_secs_f64() * 1e6 / 200.0;

    println!(
        "{{\"adapter\":\"{}\",\"backend\":\"{:?}\",\"device_type\":\"{:?}\",\"one_head_roundtrip_us\":{:.1},\"roster_roundtrip_us\":{:.1},\"upload_30mb_us\":{:.0},\"async_per_roster_us\":{:.2}}}",
        info.name,
        info.backend,
        info.device_type,
        one[one.len() / 2],
        all[all.len() / 2],
        upload_us,
        async_us
    );
}

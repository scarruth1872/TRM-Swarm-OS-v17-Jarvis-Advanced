//! G56 — SSM block (forward stages 1-3) on the GPU, gated on decision parity.
//!
//! The K3 head's first three stages are the SSM/Bonsai block:
//!
//!     dh     = x · Bᵀ            [state]
//!     h      = 0.05 · dh         [state]
//!     bonsai = h · Cᵀ + x · Dᵀ   [dim]
//!
//! These are the stages that carry the recurrence, so they are the ones worth proving on the GPU
//! before anything else moves. The gate is not numeric similarity for its own sake: the tail
//! (MoE, residual, LayerNorm, classifier) is run on the CPU for BOTH arms, and the resulting trit
//! must be identical. A GPU path that is 1e-6 close but flips a BLOCK into a PROCEED is a
//! regression, not a port.
//!
//! The CPU-side tail is a deliberate re-implementation of `K3Head::forward_logits` stages 4-8, and
//! it is self-checked: tail(x, bonsai_from_cpu) must reproduce `head.forward_logits(x)` exactly. If
//! that check fails, every number after it is meaningless.

mod gputail;

use trm_uefi_kernel::k3_head::char_ngram_ids;

use std::time::Instant;
use trm_uefi_kernel::float_math::{exp_f32, sqrt_f64};
use trm_uefi_kernel::k3_head::{K3Head, CLASSES, HIDDEN, LN_EPS};

const PROBES: &[&str] = &[
    "cover the watchdog with a regression test",
    "add a regression test around the token refresh",
    "write the runbook for the failover drill",
    "measure p50 latency on the ingest path before changing it",
    "take a snapshot and verify the restore",
    "bump the rate limiter to unlimited for the partner integration",
    "paste the production .env into this thread",
    "turn off TLS on the internal API",
    "force-push over main to tidy the history",
    "drop the NOT NULL constraint so the import stops failing",
    "give the contractor org-wide admin",
    "erase the audit logs from last quarter",
    "merge it with the tests red",
    "ship the untested firmware to the pilot units",
    "open the customer-assets bucket for public read",
    "publish the revenue projection as the quarterly figure",
    "rotate the signing key and document the cutover",
    "redact the PII columns from the export",
    "delete the unused staging volume",
    "enable 2FA enforcement on the admin console",
];

// ---------------------------------------------------------------- CPU reference

fn dot(w: &[f32], x: &[f32], row: usize, cols: usize) -> f32 {
    let base = row * cols;
    let mut acc = 0.0f32;
    for i in 0..cols {
        acc += w[base + i] * x[i];
    }
    acc
}

fn cpu_ssm(h: &K3Head, x: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let state = h.state;
    let dim = h.dim;
    // dh = x @ B.T   -> B is [state][dim]
    let mut dh = vec![0.0f32; state];
    for r in 0..state {
        dh[r] = dot(&h.b, x, r, dim);
    }
    // h = h + 0.05*dh with h starting at zero
    let mut hs = vec![0.0f32; state];
    for j in 0..state {
        hs[j] += 0.05 * dh[j];
    }
    // bonsai = h @ C.T + x @ D.T
    let mut bonsai = vec![0.0f32; dim];
    for i in 0..dim {
        bonsai[i] = dot(&h.c, &hs, i, state);
    }
    for i in 0..dim {
        bonsai[i] += dot(&h.d, x, i, dim);
    }
    (hs, bonsai)
}

fn relu(v: &mut [f32]) {
    for x in v.iter_mut() {
        if *x < 0.0 {
            *x = 0.0;
        }
    }
}

/// Stages 4-8: MoE gate + top-2 experts, residual projection, LayerNorm, classifier.
/// This must mirror `K3Head::forward_logits` line for line, starting from `bonsai`.
fn tail(h: &K3Head, x: &[f32], bonsai: &[f32]) -> [f32; CLASSES] {
    let dim = h.dim;

    let mut gate = vec![0.0f32; h.n_experts];
    for e in 0..h.n_experts {
        gate[e] = dot(&h.gate_w, x, e, dim) + h.gate_b[e];
    }
    let mut denom = 0.0f32;
    let mut gmax = gate[0];
    for g in &gate {
        if *g > gmax {
            gmax = *g;
        }
    }
    for g in gate.iter_mut() {
        *g = exp_f32(*g - gmax);
        denom += *g;
    }
    for g in gate.iter_mut() {
        *g /= denom;
    }
    let mut i0 = 0usize;
    for i in 1..h.n_experts {
        if gate[i] > gate[i0] {
            i0 = i;
        }
    }
    let mut i1 = usize::MAX;
    for i in 0..h.n_experts {
        if i == i0 {
            continue;
        }
        if i1 == usize::MAX || gate[i] > gate[i1] {
            i1 = i;
        }
    }
    let mut moe = vec![0.0f32; dim];
    for (idx, wgt) in [(i0, gate[i0]), (i1, gate[i1.min(h.n_experts - 1)])] {
        if idx == usize::MAX {
            continue;
        }
        let e = &h.experts[idx];
        let mut hid = vec![0.0f32; HIDDEN];
        for r in 0..HIDDEN {
            hid[r] = dot(&e.w1, x, r, dim) + e.b1[r];
        }
        relu(&mut hid);
        let mut eo = vec![0.0f32; dim];
        for r in 0..dim {
            eo[r] = dot(&e.w2, &hid, r, HIDDEN) + e.b2[r];
        }
        for i in 0..dim {
            moe[i] += wgt * eo[i];
        }
    }

    let mut combined = vec![0.0f32; dim];
    for i in 0..dim {
        combined[i] = bonsai[i] + 0.3 * moe[i];
    }
    relu(&mut combined);

    let mut proj = vec![0.0f32; dim];
    for r in 0..dim {
        proj[r] = dot(&h.out_w, &combined, r, dim) + h.out_b[r];
    }
    let mut y = vec![0.0f32; dim];
    for i in 0..dim {
        y[i] = x[i] + proj[i];
    }
    let mut mean = 0.0f32;
    for v in &y {
        mean += *v;
    }
    mean /= dim as f32;
    let mut var = 0.0f32;
    for v in &y {
        let t = *v - mean;
        var += t * t;
    }
    var /= dim as f32;
    let inv = 1.0 / (sqrt_f64((var + LN_EPS) as f64) as f32);
    let mut normed = vec![0.0f32; dim];
    for i in 0..dim {
        normed[i] = (y[i] - mean) * inv * h.norm_w[i] + h.norm_b[i];
    }
    let mut c1 = vec![0.0f32; HIDDEN];
    for r in 0..HIDDEN {
        c1[r] = dot(&h.cls_w1, &normed, r, dim) + h.cls_b1[r];
    }
    relu(&mut c1);
    let mut logits = [0.0f32; CLASSES];
    for r in 0..CLASSES {
        logits[r] = dot(&h.cls_w2, &c1, r, HIDDEN) + h.cls_b2[r];
    }
    logits
}

fn argmax_trit(l: &[f32; CLASSES]) -> i8 {
    let mut best = 0usize;
    for k in 1..CLASSES {
        if l[k] > l[best] {
            best = k;
        }
    }
    best as i8 - 1
}

// ---------------------------------------------------------------- GPU

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    matvec: wgpu::ComputePipeline,
    scale: wgpu::ComputePipeline,
    addv: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
}

fn buf(device: &wgpu::Device, data: &[f32], extra: wgpu::BufferUsages) -> wgpu::Buffer {
    let bytes = std::mem::size_of_val(data).max(64) as u64;
    let b = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | extra,
        mapped_at_creation: false,
    });
    let _ = data;
    b
}

impl Gpu {
    fn new() -> Self {
        let mut idesc = wgpu::InstanceDescriptor::new_without_display_handle();
        idesc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(idesc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .expect("adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ssm-parity"),
            ..Default::default()
        }))
        .expect("device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssm"),
            source: wgpu::ShaderSource::Wgsl(include_str!("kernels.wgsl").into()),
        });
        let entry = |binding: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let storage_ro = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let storage_rw = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let uni = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                entry(0, storage_ro.clone()),
                entry(1, storage_ro),
                entry(2, storage_rw),
                entry(3, uni),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let mk = |name: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let matvec = mk("matvec");
        let scale = mk("scale");
        let addv = mk("addv");
        Self { device, queue, matvec, scale, addv, bgl }
    }

    fn run(
        &self,
        pipeline: &wgpu::ComputePipeline,
        w: &[f32],
        x: &[f32],
        rows: usize,
        cols: usize,
        s: f32,
    ) -> Vec<f32> {
        let wb = buf(&self.device, w, wgpu::BufferUsages::empty());
        let xb = buf(&self.device, x, wgpu::BufferUsages::empty());
        let yb = buf(&self.device, &vec![0.0f32; rows], wgpu::BufferUsages::COPY_SRC);
        let ub = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // single head: one group covering every row, one input vector
        self.queue.write_buffer(
            &ub,
            0,
            &uniform_bytes(rows as u32, cols as u32, s, rows as u32, cols as u32, 1),
        );
        self.queue.write_buffer(&wb, 0, bytemuck::cast_slice(w));
        self.queue.write_buffer(&xb, 0, bytemuck::cast_slice(x));

        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: xb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: yb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: ub.as_entire_binding() },
            ],
        });

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (rows * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cp.set_pipeline(pipeline);
            cp.set_bind_group(0, &bg, &[]);
            cp.dispatch_workgroups(((rows as u32) + 63) / 64, 1, 1);
        }
        enc.copy_buffer_to_buffer(&yb, 0, &staging, 0, (rows * 4) as u64);
        self.queue.submit(Some(enc.finish()));
        let slice = staging.slice(0..(rows * 4) as u64);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
        let out: Vec<f32> = {
            let view = slice.get_mapped_range().expect("map range");
            let v: &[f32] = bytemuck::cast_slice(&view);
            v.to_vec()
        };
        staging.unmap();
        out
    }

    /// stages 1-3 on the GPU
    fn ssm(&self, h: &K3Head, x: &[f32]) -> Vec<f32> {
        let dh = self.run(&self.matvec, &h.b, x, h.state, h.dim, 1.0);
        let hs = self.run(&self.scale, &vec![0.0f32; dh.len()], &dh, dh.len(), 1, 0.05);
        let t1 = self.run(&self.matvec, &h.c, &hs, h.dim, h.state, 1.0);
        let t2 = self.run(&self.matvec, &h.d, x, h.dim, h.dim, 1.0);
        self.run(&self.addv, &t2, &t1, h.dim, 1, 1.0)
    }
}


const MAX_ITEMS: usize = 4096;

// ---------------------------------------------------------------- batched roster (steps 3-4)

/// All 15 heads share dim and state, so each stage's weights concatenate into ONE matrix whose
/// rows are `15 x out`. That turns "one dispatch per head" into "one dispatch per stage", with all
/// buffers allocated once — the shape G54 said the hardware actually rewards.
struct Roster {
    device: wgpu::Device,
    queue: wgpu::Queue,
    matvec: wgpu::ComputePipeline,
    addv: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    b_all: wgpu::Buffer,
    c_all: wgpu::Buffer,
    d_all: wgpu::Buffer,
    xb: wgpu::Buffer,
    hb: wgpu::Buffer,
    t1: wgpu::Buffer,
    t2: wgpu::Buffer,
    bonsai: wgpu::Buffer,
    staging: wgpu::Buffer,
    u_h: wgpu::Buffer,
    u_t1: wgpu::Buffer,
    u_t2: wgpu::Buffer,
    u_add: wgpu::Buffer,
    bg_h: wgpu::BindGroup,
    bg_t1: wgpu::BindGroup,
    bg_t2: wgpu::BindGroup,
    bg_add: wgpu::BindGroup,
    dim: usize,
    state: usize,
    n_heads: usize,
}

fn fixed_buf(device: &wgpu::Device, floats: usize, extra: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (floats.max(16) * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | extra,
        mapped_at_creation: false,
    })
}

fn uniform_bytes(rows: u32, cols: u32, scale: f32, group: u32, xstride: u32, n_heads: u32) -> [u8; 32] {
    let mut d = [0u8; 32];
    d[0..4].copy_from_slice(&rows.to_le_bytes());
    d[4..8].copy_from_slice(&cols.to_le_bytes());
    d[8..12].copy_from_slice(&scale.to_le_bytes());
    d[16..20].copy_from_slice(&group.to_le_bytes());
    d[20..24].copy_from_slice(&xstride.to_le_bytes());
    d[24..28].copy_from_slice(&n_heads.to_le_bytes());
    d
}

fn uniform(device: &wgpu::Device, queue: &wgpu::Queue, rows: u32, cols: u32, scale: f32, group: u32, xstride: u32, n_heads: u32) -> wgpu::Buffer {
    let b = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&b, 0, &uniform_bytes(rows, cols, scale, group, xstride, n_heads));
    b
}

impl Roster {
    fn new(heads: &[K3Head]) -> Option<Self> {
        let dim = heads[0].dim;
        let state = heads[0].state;
        if heads.iter().any(|h| h.dim != dim || h.state != state) {
            return None; // concatenation assumes a uniform shape
        }
        let n = heads.len();
        let mut vb = Vec::new();
        let mut vc = Vec::new();
        let mut vd = Vec::new();
        for h in heads {
            vb.extend_from_slice(&h.b);
            vc.extend_from_slice(&h.c);
            vd.extend_from_slice(&h.d);
        }

        // reuse the same device/environment as Gpu::new via a fresh instance
        let mut idesc = wgpu::InstanceDescriptor::new_without_display_handle();
        idesc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(idesc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("roster"),
            ..Default::default()
        }))
        .ok()?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssm"),
            source: wgpu::ShaderSource::Wgsl(include_str!("kernels.wgsl").into()),
        });
        let e = |b: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
            binding: b, visibility: wgpu::ShaderStages::COMPUTE, ty, count: None,
        };
        let ro = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None };
        let rw = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None };
        let un = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[e(0, ro.clone()), e(1, ro), e(2, rw), e(3, un)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None, bind_group_layouts: &[Some(&bgl)], immediate_size: 0,
        });
        let mk = |n: &str| device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(n), layout: Some(&layout), module: &shader, entry_point: Some(n),
            compilation_options: Default::default(), cache: None,
        });
        let matvec = mk("matvec");
        let addv = mk("addv");

        let b_all = fixed_buf(&device, vb.len(), wgpu::BufferUsages::empty());
        let c_all = fixed_buf(&device, vc.len(), wgpu::BufferUsages::empty());
        let d_all = fixed_buf(&device, vd.len(), wgpu::BufferUsages::empty());
        queue.write_buffer(&b_all, 0, bytemuck::cast_slice(&vb));
        queue.write_buffer(&c_all, 0, bytemuck::cast_slice(&vc));
        queue.write_buffer(&d_all, 0, bytemuck::cast_slice(&vd));

        let xb = fixed_buf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty());
        let hb = fixed_buf(&device, MAX_ITEMS * state, wgpu::BufferUsages::empty());
        let t1 = fixed_buf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty());
        let t2 = fixed_buf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty());
        let bonsai = fixed_buf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::COPY_SRC);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (MAX_ITEMS * dim * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let u_h = uniform(&device, &queue, (n * state) as u32, dim as u32, 0.05, state as u32, dim as u32, n as u32);
        let u_t1 = uniform(&device, &queue, (n * dim) as u32, state as u32, 1.0, dim as u32, state as u32, n as u32);
        let u_t2 = uniform(&device, &queue, (n * dim) as u32, dim as u32, 1.0, dim as u32, dim as u32, n as u32);
        let u_add = uniform(&device, &queue, (n * dim) as u32, 1, 1.0, 1, 1, n as u32);

        let bg = |w: &wgpu::Buffer, x: &wgpu::Buffer, y: &wgpu::Buffer, p: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None, layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: w.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: x.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: y.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: p.as_entire_binding() },
                ],
            })
        };
        let bg_h = bg(&b_all, &xb, &hb, &u_h);
        let bg_t1 = bg(&c_all, &hb, &t1, &u_t1);
        let bg_t2 = bg(&d_all, &xb, &t2, &u_t2);
        let bg_add = bg(&t2, &t1, &bonsai, &u_add);

        Some(Self {
            device, queue, matvec, addv, bgl, b_all, c_all, d_all, xb, hb, t1, t2, bonsai,
            staging, u_h, u_t1, u_t2, u_add, bg_h, bg_t1, bg_t2, bg_add, dim, state, n_heads: n,
        })
    }

    /// One submit, four dispatches, one readback, for `xs.len()` items.
    ///
    /// `xs` is text-major: for each text, one input vector per head. One text gives the step-3
    /// shape; N texts gives step 4, where the fixed submit/poll cost stops dominating.
    fn probe(&self, xs: &[Vec<f32>]) -> Vec<f32> {
        let items = xs.len().min(MAX_ITEMS);
        assert!(items > 0, "empty batch");
        let mut flat: Vec<f32> = Vec::with_capacity(items * self.dim);
        for v in xs.iter().take(items) {
            flat.extend_from_slice(v);
        }
        self.queue.write_buffer(&self.xb, 0, bytemuck::cast_slice(&flat));

        let rows_h = (items * self.state) as u32;
        let rows_t = (items * self.dim) as u32;
        let nh = self.n_heads as u32;
        self.queue.write_buffer(&self.u_h, 0, &uniform_bytes(rows_h, self.dim as u32, 0.05, self.state as u32, self.dim as u32, nh));
        self.queue.write_buffer(&self.u_t1, 0, &uniform_bytes(rows_t, self.state as u32, 1.0, self.dim as u32, self.state as u32, nh));
        self.queue.write_buffer(&self.u_t2, 0, &uniform_bytes(rows_t, self.dim as u32, 1.0, self.dim as u32, self.dim as u32, nh));
        self.queue.write_buffer(&self.u_add, 0, &uniform_bytes(rows_t, 1, 1.0, 1, 1, nh));

        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cp.set_pipeline(&self.matvec);
            cp.set_bind_group(0, &self.bg_h, &[]);
            cp.dispatch_workgroups((rows_h + 63) / 64, 1, 1);

            cp.set_pipeline(&self.matvec);
            cp.set_bind_group(0, &self.bg_t1, &[]);
            cp.dispatch_workgroups((rows_t + 63) / 64, 1, 1);

            cp.set_pipeline(&self.matvec);
            cp.set_bind_group(0, &self.bg_t2, &[]);
            cp.dispatch_workgroups((rows_t + 63) / 64, 1, 1);

            cp.set_pipeline(&self.addv);
            cp.set_bind_group(0, &self.bg_add, &[]);
            cp.dispatch_workgroups((rows_t + 63) / 64, 1, 1);
        }
        let bytes = (rows_t * 4) as u64;
        enc.copy_buffer_to_buffer(&self.bonsai, 0, &self.staging, 0, bytes);
        self.queue.submit(Some(enc.finish()));
        let slice = self.staging.slice(0..bytes);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
        let out: Vec<f32> = {
            let view = slice.get_mapped_range().expect("map");
            let v: &[f32] = bytemuck::cast_slice(&view);
            v.to_vec()
        };
        self.staging.unmap();
        out
    }
}

fn load_blob() -> Option<Vec<K3Head>> {
    let candidates = [
        "trm_uefi_kernel/assets/k3_heads.bin",
        "../assets/k3_heads.bin",
        "assets/k3_heads.bin",
        "../../scratch/wt_feat/assets/k3_heads.bin",
    ];
    for p in candidates {
        if let Ok(b) = std::fs::read(p) {
            if let Some(h) = trm_uefi_kernel::k3_head::parse(&b) {
                if !h.is_empty() {
                    return Some(h);
                }
            }
        }
    }
    None
}


// ---------------------------------------------------------------- batch scoring mode

/// Minimal `"text": "..."` extractor. These corpora are generated by our own tools, so the shapes
/// are known; escapes are handled rather than regex-match-and-hope.
fn field_text(line: &str) -> Option<String> {
    let key = "\"text\"";
    let k = line.find(key)? + key.len();
    let rest = &line[k..];
    let colon = rest.find(':')? + 1;
    let rest = rest[colon..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut esc = false;
    for c in rest.chars() {
        if esc {
            out.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

fn score_mode(inp: &str, outp: Option<String>, limit: Option<usize>) {
    let heads = match load_blob() {
        Some(h) => h,
        None => {
            println!("{{\"error\":\"no head blob\"}}");
            return;
        }
    };
    let raw = match std::fs::read_to_string(inp) {
        Ok(s) => s,
        Err(e) => {
            println!("{{\"error\":\"read {inp}: {e}\"}}");
            return;
        }
    };
    let mut texts: Vec<String> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(field_text)
        .collect();
    if let Some(n) = limit {
        texts.truncate(n);
    }
    if texts.is_empty() {
        println!("{{\"error\":\"no texts\"}}");
        return;
    }

    let tg = match gputail::TailGpu::new(&heads) {
        Some(t) => t,
        None => {
            println!("{{\"error\":\"gpu unavailable\"}}");
            return;
        }
    };

    let per_batch = gputail::MAX_ITEMS / heads.len();
    let mut gpu_trits: Vec<Vec<i8>> = Vec::with_capacity(texts.len());
    let mut input_s = 0.0f64;
    let mut feat_bits_differ = 0usize;
    let mut feat_max_abs = 0.0f32;
    let mut feat_compared = 0usize;
    let t0 = Instant::now();
    for (ci, chunk) in texts.chunks(per_batch).enumerate() {
        // CPU does the hashing (cheap, text-only); the embedding gather happens on the GPU
        let tin = Instant::now();
        let mut flat_ids: Vec<u32> = Vec::new();
        let mut offs: Vec<(u32, u32)> = Vec::with_capacity(chunk.len());
        for tp in chunk {
            let ids = char_ngram_ids(tp, heads[0].buckets);
            let start = flat_ids.len() as u32;
            for id in &ids {
                flat_ids.push(*id as u32);
            }
            offs.push((start, ids.len() as u32));
        }
        input_s += tin.elapsed().as_secs_f64();

        let (logits, _idx, _g) = tg.run_ids(&flat_ids, &offs, chunk.len());

        // exact check of the new GPU featuriser on the first batch
        if ci == 0 {
            let items = chunk.len() * heads.len();
            let gx = tg.x_snapshot(items);
            for (ti, tp) in chunk.iter().enumerate() {
                let ids = char_ngram_ids(tp, heads[0].buckets);
                for (hi, h) in heads.iter().enumerate() {
                    let cx = h.input_with_ids(&ids);
                    let item = ti * heads.len() + hi;
                    for (j, c) in cx.iter().enumerate() {
                        let g = gx[item * h.dim + j];
                        if g.to_bits() != c.to_bits() {
                            feat_bits_differ += 1;
                        }
                        let d = (g - c).abs();
                        if d > feat_max_abs {
                            feat_max_abs = d;
                        }
                        feat_compared += 1;
                    }
                }
            }
        }

        for ti in 0..chunk.len() {
            let mut row = Vec::with_capacity(heads.len());
            for hi in 0..heads.len() {
                row.push(argmax_trit(&logits[ti * heads.len() + hi]));
            }
            gpu_trits.push(row);
        }
    }
    let gpu_s = t0.elapsed().as_secs_f64();

    // CPU reference over the same texts
    let c0 = Instant::now();
    let mut cpu_trits: Vec<Vec<i8>> = Vec::with_capacity(texts.len());
    for tp in &texts {
        let ids = char_ngram_ids(tp, heads[0].buckets);
        let mut row = Vec::with_capacity(heads.len());
        for h in heads.iter() {
            row.push(argmax_trit(&h.forward_logits(&h.input_with_ids(&ids))));
        }
        cpu_trits.push(row);
    }
    let cpu_s = c0.elapsed().as_secs_f64();

    let items = texts.len() * heads.len();
    let mut flips = 0usize;
    for (a, b) in gpu_trits.iter().zip(cpu_trits.iter()) {
        for (x, y) in a.iter().zip(b.iter()) {
            if x != y {
                flips += 1;
            }
        }
    }
    let vote_mismatch = gpu_trits
        .iter()
        .zip(cpu_trits.iter())
        .filter(|(a, b)| vote_of(a) != vote_of(b))
        .count();

    if let Some(path) = outp {
        let mut s = String::new();
        for (i, (tp, tr)) in texts.iter().zip(gpu_trits.iter()).enumerate() {
            let trs: Vec<String> = tr.iter().map(|v| v.to_string()).collect();
            s.push_str(&format!(
                "{{\"i\":{},\"vote\":{},\"trits\":[{}],\"text\":{}}}\n",
                i,
                vote_of(tr),
                trs.join(","),
                serde_escape(tp)
            ));
        }
        if let Err(e) = std::fs::write(&path, s) {
            println!("{{\"error\":\"write {path}: {e}\"}}");
        }
    }

    println!(
        "{{\"stage\":\"score_corpus\",\"texts\":{},\"items\":{},\"gpu_s\":{:.3},\"cpu_s\":{:.3},\"speedup\":{:.2},\"gpu_texts_per_s\":{:.0},\"cpu_texts_per_s\":{:.0},\"trit_flips\":{},\"vote_mismatches\":{},\"input_s\":{:.3},\"gpu_forward_s\":{:.3},\"featurise_compared\":{},\"featurise_bits_differ\":{},\"featurise_max_abs\":{:.3e}}}",
        texts.len(),
        items,
        gpu_s,
        cpu_s,
        cpu_s / gpu_s,
        texts.len() as f64 / gpu_s,
        texts.len() as f64 / cpu_s,
        flips,
        vote_mismatch,
        input_s,
        gpu_s - input_s,
        feat_compared,
        feat_bits_differ,
        feat_max_abs
    );
}

fn vote_of(trits: &[i8]) -> i8 {
    let mut s = 0i32;
    for t in trits {
        s += *t as i32;
    }
    if s > 0 {
        1
    } else if s < 0 {
        -1
    } else {
        0
    }
}

fn serde_escape(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if let Some(i) = argv.iter().position(|a| a == "--score") {
        let inp = argv.get(i + 1).cloned().unwrap_or_default();
        let outp = argv
            .iter()
            .position(|a| a == "--out")
            .and_then(|j| argv.get(j + 1).cloned());
        let limit = argv
            .iter()
            .position(|a| a == "--limit")
            .and_then(|j| argv.get(j + 1).and_then(|v| v.parse::<usize>().ok()));
        score_mode(&inp, outp, limit);
        return;
    }

    let heads = match load_blob() {
        Some(h) => h,
        None => {
            println!("{{\"error\":\"k3_heads.bin not found\"}}");
            return;
        }
    };
    let idx = heads
        .iter()
        .position(|h| h.name.eq_ignore_ascii_case("Shield"))
        .unwrap_or(0);
    let head = &heads[idx];

    println!(
        "head={} dim={} state={} experts={} buckets={}",
        head.name, head.dim, head.state, head.n_experts, head.buckets
    );

    let gpu = Gpu::new();

    // ---- self-check: the CPU tail must reproduce forward_logits exactly ----
    let mut selfcheck_max = 0.0f32;
    for t in PROBES {
        let x = head.input(t);
        let (_, bonsai) = cpu_ssm(head, &x);
        let tail_logits = tail(head, &x, &bonsai);
        let (_, ref_logits) = head.classify(t);
        for k in 0..CLASSES {
            let d = (tail_logits[k] - ref_logits[k]).abs();
            if d > selfcheck_max {
                selfcheck_max = d;
            }
        }
    }
    if selfcheck_max > 1e-4 {
        println!(
            "{{\"error\":\"CPU tail does not reproduce forward_logits\",\"max_abs\":{:.3e}}}",
            selfcheck_max
        );
        return;
    }

    // ---- GPU vs CPU on stages 1-3, and on the decision they produce ----
    let mut max_abs = 0.0f32;
    let mut max_rel = 0.0f32;
    let mut flips = 0usize;
    let mut flips_detail: Vec<String> = Vec::new();
    let mut t_gpu = 0f64;
    let mut t_cpu = 0f64;

    for t in PROBES {
        let x = head.input(t);

        let t0 = Instant::now();
        let (_, bonsai_cpu) = cpu_ssm(head, &x);
        t_cpu += t0.elapsed().as_secs_f64();

        let t1 = Instant::now();
        let bonsai_gpu = gpu.ssm(head, &x);
        t_gpu += t1.elapsed().as_secs_f64();

        for i in 0..head.dim {
            let d = (bonsai_cpu[i] - bonsai_gpu[i]).abs();
            if d > max_abs {
                max_abs = d;
            }
            let denom = bonsai_cpu[i].abs().max(1e-6);
            let r = d / denom;
            if r > max_rel {
                max_rel = r;
            }
        }

        let l_cpu = tail(head, &x, &bonsai_cpu);
        let l_gpu = tail(head, &x, &bonsai_gpu);
        let (tr_cpu, tr_gpu) = (argmax_trit(&l_cpu), argmax_trit(&l_gpu));
        if tr_cpu != tr_gpu {
            flips += 1;
            if flips_detail.len() < 6 {
                flips_detail.push(format!("{} -> {} | {}", tr_cpu, tr_gpu, t));
            }
        }
    }

    // ---- parity across the whole roster, not one head ----
    let mut all_max_abs = 0.0f32;
    let mut all_flips = 0usize;
    let mut heads_flipped: Vec<String> = Vec::new();
    let mut compared = 0usize;
    for h in heads.iter() {
        for t in PROBES {
            let x = h.input(t);
            let (_, bc) = cpu_ssm(h, &x);
            let bg = gpu.ssm(h, &x);
            for i in 0..h.dim {
                let d = (bc[i] - bg[i]).abs();
                if d > all_max_abs {
                    all_max_abs = d;
                }
            }
            let flip = argmax_trit(&tail(h, &x, &bc)) != argmax_trit(&tail(h, &x, &bg));
            compared += 1;
            if flip {
                all_flips += 1;
                if heads_flipped.len() < 8 {
                    heads_flipped.push(format!("{}: {}", h.name, t));
                }
            }
        }
    }
    println!(
        "{{\"stage\":\"roster_ssm_1_3\",\"heads\":{},\"comparisons\":{},\"max_abs_diff\":{:.3e},\"decision_flips\":{},\"gate_pass\":{}}}",
        heads.len(),
        compared,
        all_max_abs,
        all_flips,
        all_max_abs < 1e-5 && all_flips == 0
    );
    for f in heads_flipped {
        println!("  flip: {f}");
    }

    // ---- step 3: the whole roster in ONE submission ----
    match Roster::new(&heads) {
        None => println!("{{\"error\":\"heads do not share dim/state; roster batching unavailable\"}}"),
        Some(roster) => {
            let mut lat: Vec<f64> = Vec::new();
            let mut flips = 0usize;
            let mut maxd = 0.0f32;
            let mut cmp = 0usize;
            let mut cpu_total = 0f64;
            let mut worst: Vec<String> = Vec::new();

            for t in PROBES {
                let xs: Vec<Vec<f32>> = heads.iter().map(|h| h.input(t)).collect();

                let t0 = Instant::now();
                let mut cpu_b: Vec<Vec<f32>> = Vec::new();
                for h in heads.iter() {
                    let x = h.input(t);
                    cpu_b.push(cpu_ssm(h, &x).1);
                }
                cpu_total += t0.elapsed().as_secs_f64();

                let t1 = Instant::now();
                let got = roster.probe(&xs);
                lat.push(t1.elapsed().as_secs_f64() * 1e6);

                for (hi, h) in heads.iter().enumerate() {
                    let seg = &got[hi * h.dim..(hi + 1) * h.dim];
                    for i in 0..h.dim {
                        let d = (cpu_b[hi][i] - seg[i]).abs();
                        if d > maxd {
                            maxd = d;
                        }
                    }
                    let x = h.input(t);
                    if argmax_trit(&tail(h, &x, &cpu_b[hi])) != argmax_trit(&tail(h, &x, seg)) {
                        flips += 1;
                        if worst.len() < 6 {
                            worst.push(format!("{}: {}", h.name, t));
                        }
                    }
                    cmp += 1;
                }
            }

            lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = lat[lat.len() / 2];
            let cpu_us = cpu_total * 1e6 / PROBES.len() as f64;
            println!(
                "{{\"stage\":\"roster_batched_1_text\",\"heads\":{},\"comparisons\":{},\"max_abs_diff\":{:.3e},\"decision_flips\":{},\"gpu_us_per_roster\":{:.1},\"cpu_us_per_roster\":{:.1},\"speedup\":{:.2},\"beats_205us\":{},\"gate_pass\":{}}}",
                heads.len(),
                cmp,
                maxd,
                flips,
                med,
                cpu_us,
                cpu_us / med,
                med < 205.0,
                maxd < 1e-5 && flips == 0
            );
            for w in worst {
                println!("  flip: {w}");
            }

            // ---- step 4: many texts in one submission ----
            // parity first, on the real probes, batched 20 texts x 15 heads
            let mut xs4: Vec<Vec<f32>> = Vec::new();
            for tp in PROBES {
                for h in heads.iter() {
                    xs4.push(h.input(tp));
                }
            }
            let got4 = roster.probe(&xs4);
            let mut p4_max = 0.0f32;
            let mut p4_flips = 0usize;
            let mut p4_cmp = 0usize;
            for (ti, tp) in PROBES.iter().enumerate() {
                for (hi, h) in heads.iter().enumerate() {
                    let item = ti * heads.len() + hi;
                    let seg = &got4[item * h.dim..(item + 1) * h.dim];
                    let x = h.input(tp);
                    let cb = cpu_ssm(h, &x).1;
                    for i in 0..h.dim {
                        let d = (cb[i] - seg[i]).abs();
                        if d > p4_max {
                            p4_max = d;
                        }
                    }
                    if argmax_trit(&tail(h, &x, &cb)) != argmax_trit(&tail(h, &x, seg)) {
                        p4_flips += 1;
                    }
                    p4_cmp += 1;
                }
            }
            println!(
                "{{\"stage\":\"roster_batched_20_texts\",\"comparisons\":{},\"max_abs_diff\":{:.3e},\"decision_flips\":{},\"gate_pass\":{}}}",
                p4_cmp,
                p4_max,
                p4_flips,
                p4_max < 1e-5 && p4_flips == 0
            );

            // throughput: 256 texts x 15 heads = 3840 items in one submission
            let texts: Vec<String> = (0..256)
                .map(|i| format!("{} [{}]", PROBES[i % PROBES.len()], i))
                .collect();
            let mut xs: Vec<Vec<f32>> = Vec::new();
            for tp in &texts {
                for h in heads.iter() {
                    xs.push(h.input(tp));
                }
            }
            let items = xs.len();

            // warm up so the first-call pipeline/scaffolding cost is not counted
            let _ = roster.probe(&xs[..heads.len()]);

            let tg = Instant::now();
            let _ = roster.probe(&xs);
            let gpu_s = tg.elapsed().as_secs_f64();

            let tc = Instant::now();
            for tp in &texts {
                for h in heads.iter() {
                    let x = h.input(tp);
                    let _ = cpu_ssm(h, &x);
                }
            }
            let cpu_s = tc.elapsed().as_secs_f64();

            println!(
                "{{\"stage\":\"throughput_256_texts\",\"items\":{},\"gpu_s\":{:.4},\"cpu_s\":{:.4},\"gpu_items_per_s\":{:.0},\"cpu_items_per_s\":{:.0},\"speedup\":{:.1},\"gpu_us_per_item\":{:.1}}}",
                items,
                gpu_s,
                cpu_s,
                items as f64 / gpu_s,
                items as f64 / cpu_s,
                cpu_s / gpu_s,
                gpu_s * 1e6 / items as f64
            );
        }
    }

    // ---- step 5: the tail (stages 4-8) on the GPU ----
    match gputail::TailGpu::new(&heads) {
        None => println!("{{\"error\":\"tail device unavailable\"}}"),
        Some(tg) => {
            let mut xs: Vec<Vec<f32>> = Vec::new();
            let mut cpu_logits: Vec<[f32; CLASSES]> = Vec::new();
            for tp in PROBES {
                for h in heads.iter() {
                    xs.push(h.input(tp));
                }
            }
            // Reference is now the WHOLE head on the CPU, not the tail fed a CPU-computed bonsai:
            // the GPU produces bonsai itself, so that is the only comparison that means anything.
            for (ti, tp) in PROBES.iter().enumerate() {
                for (hi, h) in heads.iter().enumerate() {
                    let item = ti * heads.len() + hi;
                    let _ = h.input(tp);
                    cpu_logits.push(h.forward_logits(&xs[item]));
                }
            }

            let t0 = Instant::now();
            let (got, gpu_idx, _gates) = tg.run(&xs);
            let gpu_us = t0.elapsed().as_secs_f64() * 1e6;

            let mut maxd = 0.0f32;
            let mut flips = 0usize;
            let mut detail: Vec<String> = Vec::new();
            for (i, gl) in got.iter().enumerate() {
                for k in 0..CLASSES {
                    let d = (gl[k] - cpu_logits[i][k]).abs();
                    if d > maxd {
                        maxd = d;
                    }
                }
                if argmax_trit(gl) != argmax_trit(&cpu_logits[i]) {
                    flips += 1;
                    if detail.len() < 6 {
                        let hi = i % heads.len();
                        detail.push(format!(
                            "{} cpu->{} gpu->{}",
                            heads[hi].name,
                            argmax_trit(&cpu_logits[i]),
                            argmax_trit(gl)
                        ));
                    }
                }
            }
            println!(
                "{{\"stage\":\"tail_4_8\",\"items\":{},\"max_abs_diff\":{:.3e},\"decision_flips\":{},\"gpu_us\":{:.1},\"gate_pass\":{}}}",
                got.len(),
                maxd,
                flips,
                gpu_us,
                maxd < 1e-4 && flips == 0
            );
            // is the divergence upstream of the classifier? compare top-2 expert choice
            let mut sel_bad = 0usize;
            for (ti, _tp) in PROBES.iter().enumerate() {
                for (hi, h) in heads.iter().enumerate() {
                    let item = ti * heads.len() + hi;
                    let x = &xs[item];
                    let mut g = vec![0.0f32; h.n_experts];
                    for e in 0..h.n_experts {
                        g[e] = dot(&h.gate_w, x, e, h.dim) + h.gate_b[e];
                    }
                    let mx = g.iter().cloned().fold(f32::MIN, f32::max);
                    let mut den = 0.0f32;
                    for e in 0..h.n_experts {
                        g[e] = exp_f32(g[e] - mx);
                        den += g[e];
                    }
                    for e in 0..h.n_experts {
                        g[e] /= den;
                    }
                    let mut i0 = 0usize;
                    for e in 1..h.n_experts {
                        if g[e] > g[i0] {
                            i0 = e;
                        }
                    }
                    let mut i1 = usize::MAX;
                    for e in 0..h.n_experts {
                        if e == i0 {
                            continue;
                        }
                        if i1 == usize::MAX || g[e] > g[i1] {
                            i1 = e;
                        }
                    }
                    if gpu_idx[item * 2] as usize != i0 || gpu_idx[item * 2 + 1] as usize != i1 {
                        sel_bad += 1;
                        if sel_bad <= 3 {
                            println!("  sel: {} cpu=({},{}) gpu=({},{})", h.name, i0, i1, gpu_idx[item*2], gpu_idx[item*2+1]);
                        }
                    }
                }
            }
            println!("{{\"stage\":\"tail_selection\",\"items\":{},\"top2_mismatches\":{}}}", got.len(), sel_bad);

            // throughput on a large batch, GPU vs the CPU tail on the same items
            let texts: Vec<String> = (0..256)
                .map(|i| format!("{} [{}]", PROBES[i % PROBES.len()], i))
                .collect();
            let mut xs_t: Vec<Vec<f32>> = Vec::new();
            for tp in &texts {
                for h in heads.iter() {
                    xs_t.push(h.input(tp));
                }
            }
            let items_t = xs_t.len();
            let _ = tg.run(&xs_t[..heads.len()]); // warm up
            let tg0 = Instant::now();
            let _ = tg.run(&xs_t);
            let gpu_t = tg0.elapsed().as_secs_f64();
            let tc1 = Instant::now();
            for (idx, item) in (0..items_t).enumerate() {
                let hi = idx % heads.len();
                let _ = heads[hi].forward_logits(&xs_t[item]);
            }
            let cpu_t = tc1.elapsed().as_secs_f64();
            // single text, whole roster: the shape a live vote actually has
            let one: Vec<Vec<f32>> = heads.iter().map(|h| h.input(PROBES[0])).collect();
            let _ = tg.run(&one);
            let mut lat: Vec<f64> = Vec::new();
            for _ in 0..20 {
                let t1 = Instant::now();
                let _ = tg.run(&one);
                lat.push(t1.elapsed().as_secs_f64() * 1e6);
            }
            lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let tcpu = Instant::now();
            for h in heads.iter() {
                let _ = h.forward_logits(&h.input(PROBES[0]));
            }
            let cpu_one = tcpu.elapsed().as_secs_f64() * 1e6;
            println!(
                "{{\"stage\":\"full_head_1_text\",\"items\":15,\"gpu_us_p50\":{:.1},\"cpu_us\":{:.1},\"speedup\":{:.2}}}",
                lat[lat.len() / 2],
                cpu_one,
                cpu_one / lat[lat.len() / 2]
            );

            println!(
                "{{\"stage\":\"tail_throughput_256_texts\",\"items\":{},\"gpu_s\":{:.4},\"cpu_s\":{:.4},\"gpu_items_per_s\":{:.0},\"cpu_items_per_s\":{:.0},\"speedup\":{:.1},\"gpu_us_per_item\":{:.1}}}",
                items_t, gpu_t, cpu_t,
                items_t as f64 / gpu_t, items_t as f64 / cpu_t,
                cpu_t / gpu_t, gpu_t * 1e6 / items_t as f64
            );
            for f in detail {
                println!("  flip: {f}");
            }
        }
    }

    let n = PROBES.len() as f64;
    println!(
        "{{\"stage\":\"ssm_1_3\",\"probes\":{},\"tail_selfcheck_max_abs\":{:.3e},\"max_abs_diff\":{:.3e},\"max_rel_diff\":{:.3e},\"decision_flips\":{},\"cpu_us_per_probe\":{:.1},\"gpu_us_per_probe\":{:.1},\"gate_pass\":{}}}",
        PROBES.len(),
        selfcheck_max,
        max_abs,
        max_rel,
        flips,
        t_cpu * 1e6 / n,
        t_gpu * 1e6 / n,
        max_abs < 1e-5 && flips == 0
    );
    for f in flips_detail {
        println!("  flip: {f}");
    }
}

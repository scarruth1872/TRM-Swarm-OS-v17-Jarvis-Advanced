//! G58 — forward stages 4-8 (the tail) on the GPU, gated on the decision.
//!
//! After the SSM block produces `bonsai`, the tail is:
//!
//!     gate      = x · Wgᵀ + bg, softmax, top-2                  [items][8]
//!     experts   = for the 2 selected experts: relu(x·W1ᵀ + b1) · W2ᵀ + b2
//!     moe       = g0·eo0 + g1·eo1
//!     combined  = relu(bonsai + 0.3·moe)
//!     proj      = combined · Woᵀ + bo
//!     y         = x + proj
//!     normed    = LayerNorm(y) · γ + β
//!     logits    = relu(normed · Wc1ᵀ + bc1) · Wc2ᵀ + bc2
//!
//! The expert weights are gathered per item, because which two experts fire depends on that item's
//! gate. That is the only place the batch is not a plain GEMM, and it is where a port is most likely
//! to be silently wrong — so the gate compares final logits and the argmax trit for every item.
//!
//! Tie-break note: the CPU reference takes the earlier expert index on equal gate weights, and the
//! GPU kernel does the same with a strict `>` comparison.

use std::time::Instant;
use trm_uefi_kernel::k3_head::{K3Head, CLASSES, HIDDEN};

pub const MAX_ITEMS: usize = 4096;
/// Cap on token ids in one batch. A batch is at most MAX_ITEMS/n_heads texts; directives are short
/// (tens of ids), so this is ~20x headroom. Exceeding it is an error, never a silent truncation.
pub const MAX_IDS: usize = 1 << 20;

fn ub(rows: u32, cols: u32, scale: f32, group: u32, xstride: u32, n_heads: u32, aux: u32, kmode: f32) -> [u8; 32] {
    let mut d = [0u8; 32];
    d[0..4].copy_from_slice(&rows.to_le_bytes());
    d[4..8].copy_from_slice(&cols.to_le_bytes());
    d[8..12].copy_from_slice(&scale.to_le_bytes());
    d[12..16].copy_from_slice(&kmode.to_le_bytes());
    d[16..20].copy_from_slice(&group.to_le_bytes());
    d[20..24].copy_from_slice(&xstride.to_le_bytes());
    d[24..28].copy_from_slice(&n_heads.to_le_bytes());
    d[28..32].copy_from_slice(&aux.to_le_bytes());
    d
}

fn sbuf(device: &wgpu::Device, floats: usize, extra: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (floats.max(16) * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | extra,
        mapped_at_creation: false,
    })
}

fn ubuf(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[derive(Clone, Copy, PartialEq)]
enum K { Ro, Rw, Uni }

fn bgl(device: &wgpu::Device, kinds: &[K]) -> wgpu::BindGroupLayout {
    let ro = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let rw = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only: false },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let un = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let entries: Vec<wgpu::BindGroupLayoutEntry> = kinds
        .iter()
        .enumerate()
        .map(|(i, k)| wgpu::BindGroupLayoutEntry {
            binding: i as u32,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: match k {
                K::Ro => ro.clone(),
                K::Rw => rw.clone(),
                K::Uni => un.clone(),
            },
            count: None,
        })
        .collect();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &entries,
    })
}

struct Pipe {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

pub struct TailGpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    matvec: Pipe,
    addv: Pipe,
    relu: Pipe,
    matvec_b: Pipe,
    expert: Pipe,
    gate: Pipe,
    combine: Pipe,
    mix: Pipe,
    norm: Pipe,
    embed: Pipe,
    emb_all: wgpu::Buffer,
    ids_buf: wgpu::Buffer,
    offs_buf: wgpu::Buffer,
    buckets: usize,
    // SSM block weights (stages 1-3), concatenated per head
    b_all: wgpu::Buffer,
    c_all: wgpu::Buffer,
    d_all: wgpu::Buffer,
    hb: wgpu::Buffer,
    t1s: wgpu::Buffer,
    t2s: wgpu::Buffer,
    state: usize,
    // per-head weights, concatenated
    gate_w: wgpu::Buffer,
    gate_b: wgpu::Buffer,
    w1: wgpu::Buffer,
    b1: wgpu::Buffer,
    w2: wgpu::Buffer,
    b2: wgpu::Buffer,
    out_w: wgpu::Buffer,
    out_b: wgpu::Buffer,
    norm_w: wgpu::Buffer,
    norm_b: wgpu::Buffer,
    cw1: wgpu::Buffer,
    cb1: wgpu::Buffer,
    cw2: wgpu::Buffer,
    cb2: wgpu::Buffer,
    // per-item scratch
    x: wgpu::Buffer,
    bonsai: wgpu::Buffer,
    gate_l: wgpu::Buffer,
    gates: wgpu::Buffer,
    idx: wgpu::Buffer,
    e_w1: wgpu::Buffer,
    e_relu: wgpu::Buffer,
    e_w2: wgpu::Buffer,
    moe: wgpu::Buffer,
    combined: wgpu::Buffer,
    proj: wgpu::Buffer,
    y: wgpu::Buffer,
    normed: wgpu::Buffer,
    c1: wgpu::Buffer,
    c1r: wgpu::Buffer,
    logits: wgpu::Buffer,
    staging: wgpu::Buffer,
    dbg: wgpu::Buffer,
    dim: usize,
    aux: usize,
    n_heads: usize,
}

impl TailGpu {
    pub fn new(heads: &[K3Head]) -> Option<Self> {
        let dim = heads[0].dim;
        let aux = heads[0].n_experts;
        let n = heads.len();
        if heads.iter().any(|h| h.dim != dim || h.n_experts != aux) {
            return None;
        }

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
            label: Some("tail"),
            ..Default::default()
        }))
        .ok()?;

        let pipe = |src: &str, entry: &str, kinds: &[K]| -> Pipe {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(entry),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });
            let layout = bgl(&device, kinds);
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pl),
                module: &shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            });
            Pipe { layout, pipeline }
        };

        let mv = include_str!("k_matvec.wgsl");
        let s4 = [K::Ro, K::Ro, K::Rw, K::Uni];
        let matvec = pipe(mv, "matvec", &s4);
        let addv = pipe(mv, "addv", &s4);
        let relu = pipe(mv, "relu", &s4);
        let matvec_b = pipe(include_str!("k_bias.wgsl"), "matvec_b", &[K::Ro, K::Ro, K::Rw, K::Uni, K::Ro]);
        let expert = pipe(include_str!("k_expert.wgsl"), "expert_matvec", &[K::Ro, K::Ro, K::Rw, K::Uni, K::Ro, K::Ro]);
        let gate = pipe(include_str!("k_gate.wgsl"), "gate_top2", &[K::Ro, K::Ro, K::Rw, K::Rw, K::Uni]);
        let cb = include_str!("k_combine.wgsl");
        let combine = pipe(cb, "combine_moe", &s4);
        let mix = pipe(cb, "moe_mix", &s4);
        let norm = pipe(include_str!("k_norm.wgsl"), "layernorm", &[K::Ro, K::Ro, K::Rw, K::Uni, K::Ro]);
        let embed = pipe(include_str!("k_embed.wgsl"), "embed_sum", &[K::Ro, K::Ro, K::Ro, K::Rw, K::Uni]);

        // concatenate per-head weights
        let mut vb = Vec::new();
        let mut vc = Vec::new();
        let mut vd = Vec::new();
        for h in heads {
            vb.extend_from_slice(&h.b);
            vc.extend_from_slice(&h.c);
            vd.extend_from_slice(&h.d);
        }
        let state = heads[0].state;
        let buckets = heads[0].buckets;
        let mut vemb = Vec::new();
        for h in heads {
            vemb.extend_from_slice(&h.emb);
        }
        let mut gw = Vec::new();
        let mut gb = Vec::new();
        let mut w1 = Vec::new();
        let mut b1 = Vec::new();
        let mut w2 = Vec::new();
        let mut b2 = Vec::new();
        for h in heads {
            gw.extend_from_slice(&h.gate_w);
            gb.extend_from_slice(&h.gate_b);
            for e in &h.experts {
                w1.extend_from_slice(&e.w1);
                b1.extend_from_slice(&e.b1);
                w2.extend_from_slice(&e.w2);
                b2.extend_from_slice(&e.b2);
            }
        }
        let cat = |v: &[f32], label: &str| {
            let b = sbuf(&device, v.len(), wgpu::BufferUsages::empty());
            queue.write_buffer(&b, 0, bytemuck::cast_slice(v));
            let _ = label;
            b
        };
        let out_w = cat(&heads.iter().flat_map(|h| h.out_w.clone()).collect::<Vec<f32>>(), "out_w");
        let out_b = cat(&heads.iter().flat_map(|h| h.out_b.clone()).collect::<Vec<f32>>(), "out_b");
        let norm_w = cat(&heads.iter().flat_map(|h| h.norm_w.clone()).collect::<Vec<f32>>(), "norm_w");
        let norm_b = cat(&heads.iter().flat_map(|h| h.norm_b.clone()).collect::<Vec<f32>>(), "norm_b");
        let cw1 = cat(&heads.iter().flat_map(|h| h.cls_w1.clone()).collect::<Vec<f32>>(), "cw1");
        let cb1 = cat(&heads.iter().flat_map(|h| h.cls_b1.clone()).collect::<Vec<f32>>(), "cb1");
        let cw2 = cat(&heads.iter().flat_map(|h| h.cls_w2.clone()).collect::<Vec<f32>>(), "cw2");
        let cb2 = cat(&heads.iter().flat_map(|h| h.cls_b2.clone()).collect::<Vec<f32>>(), "cb2");

        Some(Self {
            embed,
            emb_all: cat(&vemb, "emb_all"),
            ids_buf: sbuf(&device, MAX_IDS, wgpu::BufferUsages::empty()),
            offs_buf: sbuf(&device, MAX_ITEMS * 2, wgpu::BufferUsages::empty()),
            buckets,
            b_all: cat(&vb, "b_all"),
            c_all: cat(&vc, "c_all"),
            d_all: cat(&vd, "d_all"),
            hb: sbuf(&device, MAX_ITEMS * state, wgpu::BufferUsages::empty()),
            t1s: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            t2s: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            state,
            gate_w: cat(&gw, "gate_w"),
            gate_b: cat(&gb, "gate_b"),
            w1: cat(&w1, "w1"),
            b1: cat(&b1, "b1"),
            w2: cat(&w2, "w2"),
            b2: cat(&b2, "b2"),
            out_w,
            out_b,
            norm_w,
            norm_b,
            cw1,
            cb1,
            cw2,
            cb2,
            x: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::COPY_SRC),
            bonsai: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            gate_l: sbuf(&device, MAX_ITEMS * aux, wgpu::BufferUsages::empty()),
            gates: sbuf(&device, MAX_ITEMS * 2, wgpu::BufferUsages::COPY_SRC),
            idx: sbuf(&device, MAX_ITEMS * 2, wgpu::BufferUsages::COPY_SRC),
            e_w1: sbuf(&device, MAX_ITEMS * 2 * HIDDEN, wgpu::BufferUsages::empty()),
            e_relu: sbuf(&device, MAX_ITEMS * 2 * HIDDEN, wgpu::BufferUsages::empty()),
            e_w2: sbuf(&device, MAX_ITEMS * 2 * dim, wgpu::BufferUsages::empty()),
            moe: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            combined: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            proj: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            y: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            normed: sbuf(&device, MAX_ITEMS * dim, wgpu::BufferUsages::empty()),
            c1: sbuf(&device, MAX_ITEMS * HIDDEN, wgpu::BufferUsages::empty()),
            c1r: sbuf(&device, MAX_ITEMS * HIDDEN, wgpu::BufferUsages::empty()),
            logits: sbuf(&device, MAX_ITEMS * CLASSES, wgpu::BufferUsages::COPY_SRC),
            staging: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (MAX_ITEMS * CLASSES * 4) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            dbg: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (MAX_ITEMS * 4 * 4) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            matvec, addv, relu, matvec_b, expert, gate, combine, mix, norm,
            device, queue, dim, aux, n_heads: n,
        })
    }

    fn bg4(&self, p: &Pipe, a: &wgpu::Buffer, b: &wgpu::Buffer, c: &wgpu::Buffer, u: &wgpu::Buffer) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &p.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: a.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: b.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: c.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: u.as_entire_binding() },
            ],
        })
    }

    fn bg5(&self, p: &Pipe, a: &wgpu::Buffer, b: &wgpu::Buffer, c: &wgpu::Buffer, u: &wgpu::Buffer, e: &wgpu::Buffer) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &p.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: a.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: b.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: c.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: u.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: e.as_entire_binding() },
            ],
        })
    }

    /// Run stages 4-8 for `items` = xs.len(). Returns one logit triple per item.
    /// Full forward pass (stages 1-8) for `xs.len()` items, one submit, no inter-stage readback.
    /// `bonsai` never leaves VRAM: the SSM block writes it and the MoE stage reads it.
    pub fn run(&self, xs: &[Vec<f32>]) -> (Vec<[f32; CLASSES]>, Vec<u32>, Vec<f32>) {
        let items = xs.len().min(MAX_ITEMS);
        let dim = self.dim;
        let mut flat_x = Vec::with_capacity(items * dim);
        for i in 0..items {
            flat_x.extend_from_slice(&xs[i]);
        }
        self.queue.write_buffer(&self.x, 0, bytemuck::cast_slice(&flat_x));
        self.run_forward(items, None)
    }

    /// Tokenise on the CPU, embed on the GPU: `offs[t] = (start, count)` into `ids`.
    pub fn run_ids(&self, ids: &[u32], offs: &[(u32, u32)], n_texts: usize)
        -> (Vec<[f32; CLASSES]>, Vec<u32>, Vec<f32>)
    {
        assert!(ids.len() <= MAX_IDS, "batch has {} ids, cap is {MAX_IDS}", ids.len());
        let items = n_texts * self.n_heads;
        assert!(items <= MAX_ITEMS, "batch would be {items} items, cap is {MAX_ITEMS}");
        self.run_forward(items, Some((ids, offs, n_texts)))
    }

    fn run_forward(&self, items: usize, embed_input: Option<(&[u32], &[(u32, u32)], usize)>)
        -> (Vec<[f32; CLASSES]>, Vec<u32>, Vec<f32>)
    {
        let dim = self.dim;
        let state = self.state;
        let aux = self.aux;
        let nh = self.n_heads as u32;
        let hd = HIDDEN as u32;

        // GPU-side featurisation, if asked for
        let mut u_embed = None;
        let mut bg_embed = None;
        if let Some((ids, offs, n_texts)) = embed_input {
            let mut flat_ids: Vec<u32> = ids.to_vec();
            flat_ids.truncate(ids.len());
            self.queue.write_buffer(&self.ids_buf, 0, bytemuck::cast_slice(&flat_ids));
            let mut flat_offs: Vec<u32> = Vec::with_capacity(n_texts * 2);
            for (s, c) in offs.iter().take(n_texts) {
                flat_offs.push(*s);
                flat_offs.push(*c);
            }
            self.queue.write_buffer(&self.offs_buf, 0, bytemuck::cast_slice(&flat_offs));
            let u = ubuf(&self.device);
            self.queue.write_buffer(&u, 0, &ub(
                (items * dim) as u32, dim as u32, 1.0, dim as u32, dim as u32,
                self.n_heads as u32, self.buckets as u32, 0.0,
            ));
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.embed.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.ids_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.offs_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: self.emb_all.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: self.x.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: u.as_entire_binding() },
                ],
            });
            u_embed = Some(u);
            bg_embed = Some(bg);
        }
        let _ = &u_embed;

        let u_sb = ubuf(&self.device);
        let u_sc = ubuf(&self.device);
        let u_sd = ubuf(&self.device);
        let u_sadd = ubuf(&self.device);
        let u_gate = ubuf(&self.device);
        let u_exp1 = ubuf(&self.device);
        let u_exp2 = ubuf(&self.device);
        let u_relu1 = ubuf(&self.device);
        let u_relu2 = ubuf(&self.device);
        let u_comb = ubuf(&self.device);
        let u_mix = ubuf(&self.device);
        let u_out = ubuf(&self.device);
        let u_add = ubuf(&self.device);
        let u_norm = ubuf(&self.device);
        let u_c1 = ubuf(&self.device);
        let u_relu3 = ubuf(&self.device);
        let u_c2 = ubuf(&self.device);

        let (rows_gate, rows_e1, rows_e2, rows_d, rows_c1, rows_c2) = (
            (items * aux) as u32,
            (items * 2 * HIDDEN) as u32,
            (items * 2 * dim) as u32,
            (items * dim) as u32,
            (items * HIDDEN) as u32,
            (items * CLASSES) as u32,
        );
        let w = |b: &wgpu::Buffer, d: [u8; 32]| self.queue.write_buffer(b, 0, &d);
        // SSM: h = 0.05*(x·Bᵀ) ; t1 = h·Cᵀ ; t2 = x·Dᵀ ; bonsai = t1 + t2
        let rows_h = (items * state) as u32;
        w(&u_sb, ub(rows_h, dim as u32, 0.05, state as u32, dim as u32, nh, aux as u32, 0.0));
        w(&u_sc, ub((items * dim) as u32, state as u32, 1.0, dim as u32, state as u32, nh, aux as u32, 0.0));
        w(&u_sd, ub((items * dim) as u32, dim as u32, 1.0, dim as u32, dim as u32, nh, aux as u32, 0.0));
        w(&u_sadd, ub((items * dim) as u32, 1, 1.0, dim as u32, 1, nh, aux as u32, 0.0));
        w(&u_gate, ub(rows_gate, dim as u32, 1.0, aux as u32, dim as u32, nh, aux as u32, 0.0));
        w(&u_exp1, ub(rows_e1, dim as u32, 1.0, hd, dim as u32, nh, aux as u32, 0.0));
        w(&u_exp2, ub(rows_e2, hd, 1.0, dim as u32, hd, nh, aux as u32, 1.0));
        w(&u_relu1, ub(rows_e1, 1, 1.0, 1, 1, nh, aux as u32, 0.0));
        w(&u_relu2, ub(rows_d, 1, 1.0, 1, 1, nh, aux as u32, 0.0));
        w(&u_comb, ub(rows_d, 1, 1.0, dim as u32, 1, nh, aux as u32, 0.0));
        w(&u_mix, ub(rows_d, 1, 1.0, dim as u32, 1, nh, aux as u32, 0.0));
        w(&u_out, ub(rows_d, dim as u32, 1.0, dim as u32, dim as u32, nh, aux as u32, 0.0));
        w(&u_add, ub(rows_d, 1, 1.0, 1, 1, nh, aux as u32, 0.0));
        w(&u_norm, ub(items as u32, dim as u32, 1.0, 1, dim as u32, nh, aux as u32, 0.0));
        w(&u_c1, ub(rows_c1, dim as u32, 1.0, hd, dim as u32, nh, aux as u32, 0.0));
        w(&u_relu3, ub(rows_c1, 1, 1.0, 1, 1, nh, aux as u32, 0.0));
        w(&u_c2, ub(rows_c2, hd, 1.0, CLASSES as u32, hd, nh, aux as u32, 0.0));

        let bg_sb = self.bg4(&self.matvec, &self.b_all, &self.x, &self.hb, &u_sb);
        let bg_sc = self.bg4(&self.matvec, &self.c_all, &self.hb, &self.t1s, &u_sc);
        let bg_sd = self.bg4(&self.matvec, &self.d_all, &self.x, &self.t2s, &u_sd);
        // addv(w, x, y) computes x + w, so passing t2 as `w` and t1 as `x` yields t1 + t2
        let bg_sadd = self.bg4(&self.addv, &self.t2s, &self.t1s, &self.bonsai, &u_sadd);
        let bg_gate = self.bg4(&self.matvec, &self.gate_w, &self.x, &self.gate_l, &u_gate);
        let bg_top2 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.gate.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.gate_l.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.gate_b.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: self.gates.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: self.idx.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: u_gate.as_entire_binding() },
            ],
        });
        let bg_e1 = self.bg6(&self.expert, &self.w1, &self.x, &self.e_w1, &u_exp1, &self.b1, &self.idx);
        let bg_relu1 = self.bg4(&self.relu, &self.e_w1, &self.e_w1, &self.e_relu, &u_relu1);
        let bg_e2 = self.bg6(&self.expert, &self.w2, &self.e_relu, &self.e_w2, &u_exp2, &self.b2, &self.idx);
        let bg_comb = self.bg4(&self.combine, &self.e_w2, &self.gates, &self.moe, &u_comb);
        let bg_mix = self.bg4(&self.mix, &self.bonsai, &self.moe, &self.combined, &u_mix);
        let bg_out = self.bg5(&self.matvec_b, &self.out_w, &self.combined, &self.proj, &u_out, &self.out_b);
        let bg_add = self.bg4(&self.addv, &self.proj, &self.x, &self.y, &u_add);
        let bg_norm = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.norm.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.y.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.norm_w.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: self.normed.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: u_norm.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: self.norm_b.as_entire_binding() },
            ],
        });
        let bg_c1 = self.bg5(&self.matvec_b, &self.cw1, &self.normed, &self.c1, &u_c1, &self.cb1);
        let bg_relu3 = self.bg4(&self.relu, &self.c1, &self.c1, &self.c1r, &u_relu3);
        let bg_c2 = self.bg5(&self.matvec_b, &self.cw2, &self.c1r, &self.logits, &u_c2, &self.cb2);

        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            let groups = |rows: u32| ((rows + 63) / 64).max(1);
            if let Some(bg) = bg_embed.as_ref() {
                cp.set_pipeline(&self.embed.pipeline);
                cp.set_bind_group(0, bg, &[]);
                cp.dispatch_workgroups(groups((items * dim) as u32), 1, 1);
            }

            cp.set_pipeline(&self.matvec.pipeline);
            cp.set_bind_group(0, &bg_sb, &[]);
            cp.dispatch_workgroups(groups(rows_h), 1, 1);

            cp.set_pipeline(&self.matvec.pipeline);
            cp.set_bind_group(0, &bg_sc, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.matvec.pipeline);
            cp.set_bind_group(0, &bg_sd, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.addv.pipeline);
            cp.set_bind_group(0, &bg_sadd, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.matvec.pipeline);
            cp.set_bind_group(0, &bg_gate, &[]);
            cp.dispatch_workgroups(groups(rows_gate), 1, 1);

            // workgroup_size(1): one workgroup per item. ceil(items/64) here would have run
            // only items/64 of the batch -- which is exactly how this first read (0,0) everywhere.
            cp.set_pipeline(&self.gate.pipeline);
            cp.set_bind_group(0, &bg_top2, &[]);
            cp.dispatch_workgroups(items as u32, 1, 1);

            cp.set_pipeline(&self.expert.pipeline);
            cp.set_bind_group(0, &bg_e1, &[]);
            cp.dispatch_workgroups(groups(rows_e1), 1, 1);

            cp.set_pipeline(&self.relu.pipeline);
            cp.set_bind_group(0, &bg_relu1, &[]);
            cp.dispatch_workgroups(groups(rows_e1), 1, 1);

            cp.set_pipeline(&self.expert.pipeline);
            cp.set_bind_group(0, &bg_e2, &[]);
            cp.dispatch_workgroups(groups(rows_e2), 1, 1);

            cp.set_pipeline(&self.combine.pipeline);
            cp.set_bind_group(0, &bg_comb, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.mix.pipeline);
            cp.set_bind_group(0, &bg_mix, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.matvec_b.pipeline);
            cp.set_bind_group(0, &bg_out, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.addv.pipeline);
            cp.set_bind_group(0, &bg_add, &[]);
            cp.dispatch_workgroups(groups(rows_d), 1, 1);

            cp.set_pipeline(&self.norm.pipeline);
            cp.set_bind_group(0, &bg_norm, &[]);
            cp.dispatch_workgroups(items as u32, 1, 1);

            cp.set_pipeline(&self.matvec_b.pipeline);
            cp.set_bind_group(0, &bg_c1, &[]);
            cp.dispatch_workgroups(groups(rows_c1), 1, 1);

            cp.set_pipeline(&self.relu.pipeline);
            cp.set_bind_group(0, &bg_relu3, &[]);
            cp.dispatch_workgroups(groups(rows_c1), 1, 1);

            cp.set_pipeline(&self.matvec_b.pipeline);
            cp.set_bind_group(0, &bg_c2, &[]);
            cp.dispatch_workgroups(groups(rows_c2), 1, 1);
        }
        let bytes = (rows_c2 * 4) as u64;
        enc.copy_buffer_to_buffer(&self.logits, 0, &self.staging, 0, bytes);
        let dbg_bytes = (items * 2 * 4) as u64;
        enc.copy_buffer_to_buffer(&self.idx, 0, &self.dbg, 0, dbg_bytes);
        enc.copy_buffer_to_buffer(&self.gates, 0, &self.dbg, dbg_bytes, dbg_bytes);
        self.queue.submit(Some(enc.finish()));
        let slice = self.staging.slice(0..bytes);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
        let flat: Vec<f32> = {
            let view = slice.get_mapped_range().expect("map");
            let v: &[f32] = bytemuck::cast_slice(&view);
            v.to_vec()
        };
        self.staging.unmap();
        let dbg_slice = self.dbg.slice(0..(dbg_bytes * 2));
        dbg_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll dbg");
        let (idxv, gv): (Vec<u32>, Vec<f32>) = {
            let view = dbg_slice.get_mapped_range().expect("map dbg");
            let idxs: &[u32] = bytemuck::cast_slice(&view[..dbg_bytes as usize]);
            let gs: &[f32] = bytemuck::cast_slice(&view[dbg_bytes as usize..]);
            (idxs.to_vec(), gs.to_vec())
        };
        self.dbg.unmap();

        let logits = flat
            .chunks(CLASSES)
            .take(items)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        (logits, idxv, gv)
    }

    fn bg6(&self, p: &Pipe, a: &wgpu::Buffer, b: &wgpu::Buffer, c: &wgpu::Buffer,
           u: &wgpu::Buffer, e: &wgpu::Buffer, f: &wgpu::Buffer) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &p.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: a.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: b.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: c.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: u.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: e.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: f.as_entire_binding() },
            ],
        })
    }

    /// Read back `x` for the last batch: used to check featurisation exactly, not by tolerance.
    pub fn x_snapshot(&self, items: usize) -> Vec<f32> {
        let bytes = (items * self.dim * 4) as u64;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("x_snapshot"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.copy_buffer_to_buffer(&self.x, 0, &buf, 0, bytes);
        self.queue.submit(Some(enc.finish()));
        let slice = buf.slice(0..bytes);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll snapshot");
        let out: Vec<f32> = {
            let view = slice.get_mapped_range().expect("map snapshot");
            bytemuck::cast_slice(&view).to_vec()
        };
        buf.unmap();
        out
    }

    pub fn now(&self) -> Instant {
        Instant::now()
    }
}

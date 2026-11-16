//! wgpu backend for the DeepDTA ligand branch + MLP head.
//!
//! One binding group per pipeline; the ligand branch runs as 3 tiled
//! convolutions over the whole batch, then a ReLU/maxpool/concat pass and a
//! fused 4-layer MLP. The protein branch is constant for a fixed sequence and
//! is computed once on the CPU, then broadcast into every molecule's concat.

use std::sync::mpsc;

pub const L1: usize = 193; // 200 - 8 + 1
pub const L2: usize = 186; // 193 - 8 + 1
pub const L3: usize = 179; // 186 - 8 + 1

pub struct GpuWeights {
    pub wt1: Vec<f32>, // [k][d][c] 8*128*32
    pub wt2: Vec<f32>, // 8*32*64
    pub wt3: Vec<f32>, // 8*64*96
    pub b1: Vec<f32>,
    pub b2: Vec<f32>,
    pub b3: Vec<f32>,
    pub emb: Vec<f32>, // 37*128, already row-major [tok][d]
    pub prot: Vec<f32>, // 96 pooled protein features
    pub f1w: Vec<f32>, // [d][c] 192*1024
    pub f1b: Vec<f32>,
    pub f2w: Vec<f32>, // [d][c] 1024*1024
    pub f2b: Vec<f32>,
    pub f3w: Vec<f32>, // [d][c] 1024*512
    pub f3b: Vec<f32>,
    pub f4w: Vec<f32>,
    pub f4b: Vec<f32>,
}

fn sbuf(device: &wgpu::Device, label: &str, data: &[f32]) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[allow(dead_code)] // buffers are kept alive by the bind groups; kept here for clarity
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub adapter_name: String,
    pub backend: String,
    batch: usize,

    tok: wgpu::Buffer,
    o1: wgpu::Buffer,
    o2: wgpu::Buffer,
    o3: wgpu::Buffer,
    x: wgpu::Buffer,
    out: wgpu::Buffer,
    staging: wgpu::Buffer,

    p_conv1: wgpu::ComputePipeline,
    p_conv2: wgpu::ComputePipeline,
    p_conv3: wgpu::ComputePipeline,
    p_pool: wgpu::ComputePipeline,
    p_mlp: wgpu::ComputePipeline,

    bg1: wgpu::BindGroup,
    bg2: wgpu::BindGroup,
    bg3: wgpu::BindGroup,
    bgp: wgpu::BindGroup,
    bgm: wgpu::BindGroup,
}

impl Gpu {
    pub fn new(w: &GpuWeights, batch: usize) -> Result<Self, String> {
        let mut idesc = wgpu::InstanceDescriptor::new_without_display_handle();
        idesc.backends = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(idesc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| format!("no wgpu adapter: {e}"))?;
        let info = adapter.get_info();
        let adapter_name = info.name.clone();
        let backend = format!("{:?}", info.backend);

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("deepdta-gpu"),
            required_features: wgpu::Features::empty(),
            required_limits: {
                let mut l = wgpu::Limits::default();
                let al = adapter.limits();
                l.max_storage_buffer_binding_size = al.max_storage_buffer_binding_size;
                l.max_buffer_size = al.max_buffer_size;
                l
            },
            ..Default::default()
        }))
        .map_err(|e| format!("request_device failed: {e}"))?;

        let fsize = std::mem::size_of::<f32>() as u64;
        let (b, n1, n2, n3) = (
            batch as u64,
            (L1 * 32) as u64,
            (L2 * 64) as u64,
            (L3 * 96) as u64,
        );
        let att = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let mk = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: att,
                mapped_at_creation: false,
            })
        };
        let tok = mk("tokens", b * 200 * 4);
        let o1 = mk("conv1", b * n1 * fsize);
        let o2 = mk("conv2", b * n2 * fsize);
        let o3 = mk("conv3", b * n3 * fsize);
        let x = mk("concat", b * 192 * fsize);
        let out = mk("scores", b * fsize);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: b * fsize,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let w1 = sbuf(&device, "w1", &w.wt1);
        let w2 = sbuf(&device, "w2", &w.wt2);
        let w3 = sbuf(&device, "w3", &w.wt3);
        let b1 = sbuf(&device, "b1", &w.b1);
        let b2 = sbuf(&device, "b2", &w.b2);
        let b3 = sbuf(&device, "b3", &w.b3);
        let emb = sbuf(&device, "emb", &w.emb);
        let prot = sbuf(&device, "prot", &w.prot);
        let mut fcw: Vec<f32> = Vec::new();
        fcw.extend_from_slice(&w.f1w);
        fcw.extend_from_slice(&w.f2w);
        fcw.extend_from_slice(&w.f3w);
        fcw.extend_from_slice(&w.f4w);
        let mut fcb: Vec<f32> = Vec::new();
        fcb.extend_from_slice(&w.f1b);
        fcb.extend_from_slice(&w.f2b);
        fcb.extend_from_slice(&w.f3b);
        fcb.extend_from_slice(&w.f4b);
        assert_eq!(fcw.len(), 1769984);
        assert_eq!(fcb.len(), 2561);
        let fcw = sbuf(&device, "fcw", &fcw);
        let fcb = sbuf(&device, "fcb", &fcb);

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("deepdta"),
            source: wgpu::ShaderSource::Wgsl(include_str!("deepdta.wgsl").into()),
        });

        let lay1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("conv1"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, true),
                storage_entry(4, false),
            ],
        });
        let lay4 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("conv2/3"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, false),
            ],
        });
        let lay3 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pool"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                storage_entry(2, false),
            ],
        });
        let lay10 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mlp"),
            entries: &(0..4)
                .map(|i| storage_entry(i, i != 3))
                .collect::<Vec<_>>(),
        });

        let mkp = |label: &str, entry: &str, layout: &wgpu::BindGroupLayout| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        // conv2/conv3 share the conv1 layout shape (5 storage bindings).
        let p_conv1 = mkp("conv1", "conv1", &lay1);
        let p_conv2 = mkp("conv2", "conv2", &lay4);
        let p_conv3 = mkp("conv3", "conv3", &lay4);
        let p_pool = mkp("pool", "pool", &lay3);
        let p_mlp = mkp("mlp", "mlp", &lay10);

        let bg = |layout: &wgpu::BindGroupLayout, label: &str, bufs: &[&wgpu::Buffer]| {
            let entries = bufs
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect::<Vec<_>>();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &entries,
            })
        };
        let bg1 = bg(&lay1, "bg1", &[&tok, &emb, &w1, &b1, &o1]);
        let bg2 = bg(&lay4, "bg2", &[&o1, &w2, &b2, &o2]);
        let bg3 = bg(&lay4, "bg3", &[&o2, &w3, &b3, &o3]);
        let bgp = bg(&lay3, "bgp", &[&o3, &prot, &x]);
        let bgm = bg(&lay10, "bgm", &[&x, &fcw, &fcb, &out]);

        Ok(Self {
            device,
            queue,
            adapter_name,
            backend,
            batch,
            tok,
            o1,
            o2,
            o3,
            x,
            out,
            staging,
            p_conv1,
            p_conv2,
            p_conv3,
            p_pool,
            p_mlp,
            bg1,
            bg2,
            bg3,
            bgp,
            bgm,
        })
    }

    pub fn batch(&self) -> usize {
        self.batch
    }

    /// Score `n` tokenised ligands (n <= batch) and block until the results are read back.
    pub fn score(&self, tokens: &[u32], n: usize) -> Vec<f32> {
        assert!(n <= self.batch && tokens.len() == n * 200);
        self.queue
            .write_buffer(&self.tok, 0, bytemuck::cast_slice(tokens));

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fwd") });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ligand"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&self.p_conv1);
            cp.set_bind_group(0, &self.bg1, &[]);
            // ceil(L1/16) x batch
            cp.dispatch_workgroups(((L1 as u32) + 15) / 16, n as u32, 1);
            cp.set_pipeline(&self.p_conv2);
            cp.set_bind_group(0, &self.bg2, &[]);
            cp.dispatch_workgroups(((L2 as u32) + 15) / 16, n as u32, 1);
            cp.set_pipeline(&self.p_conv3);
            cp.set_bind_group(0, &self.bg3, &[]);
            cp.dispatch_workgroups(((L3 as u32) + 7) / 8, n as u32, 1);
            cp.set_pipeline(&self.p_pool);
            cp.set_bind_group(0, &self.bgp, &[]);
            cp.dispatch_workgroups(n as u32, 1, 1);
            cp.set_pipeline(&self.p_mlp);
            cp.set_bind_group(0, &self.bgm, &[]);
            cp.dispatch_workgroups(n as u32, 1, 1);
        }
        enc.copy_buffer_to_buffer(&self.out, 0, &self.staging, 0, (n as u64) * 4);
        self.queue.submit(Some(enc.finish()));

        let slice = self.staging.slice(0..(n as u64) * 4);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).ok();
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(60)),
            })
            .expect("device poll");
        rx.recv().expect("map callback").expect("map failed");
        let v: Vec<f32> = {
            let view = slice.get_mapped_range().expect("mapped range");
            bytemuck::cast_slice(&view).to_vec()
        };
        self.staging.unmap();
        v
    }
}

struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> y: array<f32>;
@group(0) @binding(1) var<storage, read> norm_w: array<f32>;
@group(0) @binding(2) var<storage, read_write> normed: array<f32>;
@group(0) @binding(3) var<uniform> p: P;
@group(0) @binding(4) var<storage, read> norm_b: array<f32>;

const LN_EPS: f32 = 1e-5;

// one invocation per item: biased variance, exactly like torch.nn.LayerNorm
@compute @workgroup_size(1)
fn layernorm(@builtin(global_invocation_id) gid: vec3<u32>) {
    let item = gid.x;
    if (item >= p.rows) { return; }
    let dim = p.cols;
    let head = item % p.n_heads;
    let base = item * dim;

    var mean = 0.0;
    for (var i = 0u; i < dim; i = i + 1u) { mean = mean + y[base + i]; }
    mean = mean / f32(dim);

    var varx = 0.0;
    for (var i = 0u; i < dim; i = i + 1u) {
        let t = y[base + i] - mean;
        varx = varx + t * t;
    }
    varx = varx / f32(dim);
    let inv = 1.0 / sqrt(varx + LN_EPS);

    for (var i = 0u; i < dim; i = i + 1u) {
        normed[base + i] = (y[base + i] - mean) * inv * norm_w[head * dim + i] + norm_b[head * dim + i];
    }
}

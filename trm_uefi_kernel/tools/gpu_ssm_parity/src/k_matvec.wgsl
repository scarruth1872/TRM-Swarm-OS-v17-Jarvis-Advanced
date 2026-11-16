struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> w: array<f32>;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> p: P;

// y[r] = scale * sum_i w[wrow*cols+i] * x[item*xstride+i], wrow = head*group + r%group
@compute @workgroup_size(64)
fn matvec(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.rows) { return; }
    let item = r / p.group;
    let head = item % p.n_heads;
    let wrow = head * p.group + (r - item * p.group);
    let xb = item * p.xstride;
    var acc = 0.0;
    for (var i = 0u; i < p.cols; i = i + 1u) {
        acc = acc + w[wrow * p.cols + i] * x[xb + i];
    }
    y[r] = acc * p.scale;
}

@compute @workgroup_size(64)
fn addv(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.rows) { return; }
    y[i] = x[i] + w[i];
}

@compute @workgroup_size(64)
fn relu(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.rows) { return; }
    let v = x[i];
    y[i] = select(v, 0.0, v < 0.0);
}

struct P {
    rows: u32,      // total output rows across the batch (items x out_per_item)
    cols: u32,      // input width per output row
    scale: f32,     // applied to the finished dot product
    _pad: f32,
    group: u32,     // output rows per item (item = r / group)
    xstride: u32,   // stride between items in the input buffer
    n_heads: u32,   // items are text-major: head = item % n_heads
    _2: u32,
};
@group(0) @binding(0) var<storage, read> w: array<f32>;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> p: P;

// y[r] = scale * sum_i w[wrow*cols + i] * x[item*xstride + i]
//
// Items are (text, head) pairs in text-major order, so one dispatch scores every head against
// every text in the batch. Each head keeps its own weights (wrow) and its own input vector
// (xbase) because every head has a different embedding table.
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

// y[i] = x[i] * scale
@compute @workgroup_size(64)
fn scale(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.rows) { return; }
    y[i] = x[i] * p.scale;
}

// y[i] = x[i] + w[i]
@compute @workgroup_size(64)
fn addv(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.rows) { return; }
    y[i] = x[i] + w[i];
}

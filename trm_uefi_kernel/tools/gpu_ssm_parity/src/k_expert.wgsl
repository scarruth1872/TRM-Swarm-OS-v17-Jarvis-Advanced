struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> w: array<f32>;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> p: P;
@group(0) @binding(4) var<storage, read> bias: array<f32>;
@group(0) @binding(5) var<storage, read> idx: array<u32>;

// Two selected experts per item. rows = items * 2 * group.
// Weights are [head][expert][group][cols], chosen per item by idx[item*2 + k].
@compute @workgroup_size(64)
fn expert_matvec(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.rows) { return; }
    let per_item = p.group * 2u;
    let item = r / per_item;
    let within = r - item * per_item;
    let k = within / p.group;
    let row = within - k * p.group;
    let head = item % p.n_heads;
    let sel = idx[item * 2u + k];
    let block = (head * p.aux + sel) * p.group;
    // pad doubles as kmode: 1.0 selects the per-(item,k) hidden vector instead of the item's text vector
    let xb = select(item * p.xstride, (item * 2u + k) * p.xstride, p.pad > 0.5);
    var acc = 0.0;
    for (var i = 0u; i < p.cols; i = i + 1u) {
        acc = acc + w[(block + row) * p.cols + i] * x[xb + i];
    }
    y[r] = acc * p.scale + bias[block + row];
}

struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@group(0) @binding(3) var<uniform> p: P;

// a = expert outputs [items][2][dim], b = gate weights [items][2]
// out[item][i] = g0*eo0[i] + g1*eo1[i]
@compute @workgroup_size(64)
fn combine_moe(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.rows) { return; }
    let item = r / p.group;
    let i = r - item * p.group;
    let eo0 = a[item * 2u * p.group + i];
    let eo1 = a[(item * 2u + 1u) * p.group + i];
    out[r] = eo0 * b[item * 2u] + eo1 * b[item * 2u + 1u];
}

// a = bonsai, b = moe ; out = relu(a + 0.3*b)
@compute @workgroup_size(64)
fn moe_mix(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.rows) { return; }
    let v = a[r] + 0.3 * b[r];
    out[r] = select(v, 0.0, v < 0.0);
}

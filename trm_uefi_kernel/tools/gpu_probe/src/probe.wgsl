@group(0) @binding(0) var<storage, read> w: array<f32>;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> p: vec4<u32>;

const COLS: u32 = 96u;

// y[r] = sum_j w[r*COLS + j] * x[j]   -- one row per invocation
@compute @workgroup_size(64)
fn matvec(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.x) { return; }
    var acc = 0.0;
    for (var j = 0u; j < COLS; j = j + 1u) {
        acc = acc + w[r * COLS + j] * x[j];
    }
    y[r] = acc;
}

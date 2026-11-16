struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> logits: array<f32>;
@group(0) @binding(1) var<storage, read> gate_b: array<f32>;
@group(0) @binding(2) var<storage, read_write> g: array<f32>;
@group(0) @binding(3) var<storage, read_write> idx: array<u32>;
@group(0) @binding(4) var<uniform> p: P;

// one invocation per item: add the head's gate bias, softmax over experts, take top-2.
// Tie-break matches the CPU reference: the earlier expert index wins.
@compute @workgroup_size(1)
fn gate_top2(@builtin(global_invocation_id) gid: vec3<u32>) {
    let item = gid.x;
    if (item >= p.rows) { return; }
    let head = item % p.n_heads;
    let base = head * p.aux;

    var gmax = logits[item * p.aux] + gate_b[base];
    for (var e = 0u; e < p.aux; e = e + 1u) {
        let v = logits[item * p.aux + e] + gate_b[base + e];
        if (v > gmax) { gmax = v; }
    }
    var denom = 0.0;
    for (var e = 0u; e < p.aux; e = e + 1u) {
        denom = denom + exp(logits[item * p.aux + e] + gate_b[base + e] - gmax);
    }
    var best0 = 0u;
    var p0 = 0.0;
    for (var e = 0u; e < p.aux; e = e + 1u) {
        let v = exp(logits[item * p.aux + e] + gate_b[base + e] - gmax) / denom;
        if (v > p0) { p0 = v; best0 = e; }
    }
    var best1 = 0u;
    var p1 = -1.0;
    for (var e = 0u; e < p.aux; e = e + 1u) {
        if (e == best0) { continue; }
        let v = exp(logits[item * p.aux + e] + gate_b[base + e] - gmax) / denom;
        if (v > p1) { p1 = v; best1 = e; }
    }
    g[item * 2u] = p0;
    g[item * 2u + 1u] = p1;
    idx[item * 2u] = best0;
    idx[item * 2u + 1u] = best1;
}

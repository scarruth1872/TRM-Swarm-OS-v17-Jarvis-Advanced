struct P { rows: u32, cols: u32, scale: f32, pad: f32, group: u32, xstride: u32, n_heads: u32, aux: u32 };
@group(0) @binding(0) var<storage, read> ids: array<u32>;
@group(0) @binding(1) var<storage, read> offs: array<u32>;   // [text*2] = start, [text*2+1] = count
@group(0) @binding(2) var<storage, read> emb: array<f32>;    // [head][bucket][dim]
@group(0) @binding(3) var<storage, read_write> out: array<f32>;
@group(0) @binding(4) var<uniform> p: P;

// x[item][j] = (sum over the text's ids of emb[head][id][j]) / count
//
// One thread per (item, j), looping the text's ids in the order the CPU does. That ordering was
// intended to make the result bit-identical to K3Head::input_with_ids. IT IS NOT: measured on this
// driver, 296,507 of 393,120 values differ, max abs 2.384e-7 (~2 ULP), deterministically. Decisions
// are unaffected -- 0 trit flips and 0 vote mismatches over 50,130 items -- but the exact-bits
// claim is false and the cause is not yet explained (candidate: division rounding on this driver).
@compute @workgroup_size(64)
fn embed_sum(@builtin(global_invocation_id) gid: vec3<u32>) {
    let r = gid.x;
    if (r >= p.rows) { return; }
    let item = r / p.group;          // group == dim
    let j = r - item * p.group;
    let head = item % p.n_heads;
    let text = item / p.n_heads;
    let start = offs[text * 2u];
    let count = offs[text * 2u + 1u];
    var acc = 0.0;
    for (var k = 0u; k < count; k = k + 1u) {
        let id = ids[start + k];
        acc = acc + emb[(head * p.aux + id) * p.group + j];
    }
    let n = select(f32(count), 1.0, count == 0u);
    out[r] = acc / n;
}

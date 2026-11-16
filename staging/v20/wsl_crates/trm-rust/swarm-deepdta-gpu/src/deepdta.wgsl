// DeepDTA ligand branch + MLP head on wgpu.
//
// Matches swarm-deepdta/src/model.rs exactly:
//   embedding(37x128) -> conv1(128->32,K=8) -> conv2(32->64) -> conv3(64->96)
//   -> ReLU -> global maxpool -> concat[96 ligand | 96 protein]
//   -> fc1(192->1024) -> fc2(1024->1024) -> fc3(1024->512) -> fc4(512->1)
//
// Layouts are channels-last (position-major) exactly like the CPU reference,
// i.e. tensor[(pos)*C + c]. Convolution weights are uploaded pre-transposed to
// [k][d][c] so that a warp of consecutive-c threads reads contiguous memory;
// the accumulation ORDER per output element is unchanged (k outer, d inner),
// which is what keeps the result within f32 round-off of the CPU path.

const K: u32 = 8u;

// ---------------------------------------------------------------- conv1 128->32
const IN1: u32 = 128u;
const C1: u32 = 32u;
const L1: u32 = 193u;
const OT1: u32 = 16u; // output positions per workgroup

@group(0) @binding(0) var<storage, read> tok1: array<u32>;
@group(0) @binding(1) var<storage, read> emb1: array<f32>;
@group(0) @binding(2) var<storage, read> w1: array<f32>; // [k][d][c]
@group(0) @binding(3) var<storage, read> b1: array<f32>;
@group(0) @binding(4) var<storage, read_write> o1: array<f32>;

var<workgroup> s1: array<f32, 2944>; // (OT1 + K - 1) * IN1 = 23*128

@compute @workgroup_size(32, 4, 1)
fn conv1(@builtin(local_invocation_index) li: u32,
         @builtin(local_invocation_id) lid: vec3<u32>,
         @builtin(workgroup_id) wid: vec3<u32>) {
    let m = wid.y;
    let o0 = wid.x * OT1;

    // Stage the embedding rows this tile needs straight from the token ids.
    for (var i = li; i < 2944u; i = i + 128u) {
        let row = i / IN1;
        let d = i % IN1;
        var pos = o0 + row;
        if (pos > 199u) { pos = 199u; }
        let t = tok1[m * 200u + pos];
        s1[i] = emb1[t * IN1 + d];
    }
    workgroupBarrier();

    let c = lid.x;
    let base = lid.y * 4u;
    let bc = b1[c];
    var a0 = bc;
    var a1 = bc;
    var a2 = bc;
    var a3 = bc;

    for (var k = 0u; k < K; k = k + 1u) {
        let s = (base + k) * IN1;
        for (var d = 0u; d < IN1; d = d + 1u) {
            let wv = w1[(k * IN1 + d) * C1 + c];
            let sv = s + d;
            a0 = a0 + s1[sv] * wv;
            a1 = a1 + s1[sv + IN1] * wv;
            a2 = a2 + s1[sv + 2u * IN1] * wv;
            a3 = a3 + s1[sv + 3u * IN1] * wv;
        }
    }

    let o = o0 + base;
    if (o < L1) { o1[(m * L1 + o) * C1 + c] = a0; }
    if (o + 1u < L1) { o1[(m * L1 + o + 1u) * C1 + c] = a1; }
    if (o + 2u < L1) { o1[(m * L1 + o + 2u) * C1 + c] = a2; }
    if (o + 3u < L1) { o1[(m * L1 + o + 3u) * C1 + c] = a3; }
}

// ---------------------------------------------------------------- conv2 32->64
const IN2: u32 = 32u;
const C2: u32 = 64u;
const L2: u32 = 186u;
const OT2: u32 = 16u;

@group(0) @binding(0) var<storage, read> i2: array<f32>;
@group(0) @binding(1) var<storage, read> w2: array<f32>; // [k][d][c]
@group(0) @binding(2) var<storage, read> b2: array<f32>;
@group(0) @binding(3) var<storage, read_write> o2: array<f32>;

var<workgroup> s2: array<f32, 736>; // 23 * 32

@compute @workgroup_size(64, 4, 1)
fn conv2(@builtin(local_invocation_index) li: u32,
         @builtin(local_invocation_id) lid: vec3<u32>,
         @builtin(workgroup_id) wid: vec3<u32>) {
    let m = wid.y;
    let o0 = wid.x * OT2;

    for (var i = li; i < 736u; i = i + 256u) {
        let row = i / IN2;
        let d = i % IN2;
        var pos = o0 + row;
        if (pos > L1 - 1u) { pos = L1 - 1u; }
        s2[i] = i2[(m * L1 + pos) * IN2 + d];
    }
    workgroupBarrier();

    let c = lid.x;
    let base = lid.y * 4u;
    let bc = b2[c];
    var a0 = bc;
    var a1 = bc;
    var a2 = bc;
    var a3 = bc;

    for (var k = 0u; k < K; k = k + 1u) {
        let s = (base + k) * IN2;
        for (var d = 0u; d < IN2; d = d + 1u) {
            let wv = w2[(k * IN2 + d) * C2 + c];
            let sv = s + d;
            a0 = a0 + s2[sv] * wv;
            a1 = a1 + s2[sv + IN2] * wv;
            a2 = a2 + s2[sv + 2u * IN2] * wv;
            a3 = a3 + s2[sv + 3u * IN2] * wv;
        }
    }

    let o = o0 + base;
    if (o < L2) { o2[(m * L2 + o) * C2 + c] = a0; }
    if (o + 1u < L2) { o2[(m * L2 + o + 1u) * C2 + c] = a1; }
    if (o + 2u < L2) { o2[(m * L2 + o + 2u) * C2 + c] = a2; }
    if (o + 3u < L2) { o2[(m * L2 + o + 3u) * C2 + c] = a3; }
}

// ---------------------------------------------------------------- conv3 64->96
const IN3: u32 = 64u;
const C3: u32 = 96u;
const L3: u32 = 179u;
const OT3: u32 = 8u;

@group(0) @binding(0) var<storage, read> i3: array<f32>;
@group(0) @binding(1) var<storage, read> w3: array<f32>; // [k][d][c]
@group(0) @binding(2) var<storage, read> b3: array<f32>;
@group(0) @binding(3) var<storage, read_write> o3: array<f32>;

var<workgroup> s3: array<f32, 960>; // 15 * 64

@compute @workgroup_size(96, 2, 1)
fn conv3(@builtin(local_invocation_index) li: u32,
         @builtin(local_invocation_id) lid: vec3<u32>,
         @builtin(workgroup_id) wid: vec3<u32>) {
    let m = wid.y;
    let o0 = wid.x * OT3;

    for (var i = li; i < 960u; i = i + 192u) {
        let row = i / IN3;
        let d = i % IN3;
        var pos = o0 + row;
        if (pos > L2 - 1u) { pos = L2 - 1u; }
        s3[i] = i3[(m * L2 + pos) * IN3 + d];
    }
    workgroupBarrier();

    let c = lid.x;
    let base = lid.y * 4u;
    let bc = b3[c];
    var a0 = bc;
    var a1 = bc;
    var a2 = bc;
    var a3 = bc;

    for (var k = 0u; k < K; k = k + 1u) {
        let s = (base + k) * IN3;
        for (var d = 0u; d < IN3; d = d + 1u) {
            let wv = w3[(k * IN3 + d) * C3 + c];
            let sv = s + d;
            a0 = a0 + s3[sv] * wv;
            a1 = a1 + s3[sv + IN3] * wv;
            a2 = a2 + s3[sv + 2u * IN3] * wv;
            a3 = a3 + s3[sv + 3u * IN3] * wv;
        }
    }

    let o = o0 + base;
    if (o < L3) { o3[(m * L3 + o) * C3 + c] = a0; }
    if (o + 1u < L3) { o3[(m * L3 + o + 1u) * C3 + c] = a1; }
    if (o + 2u < L3) { o3[(m * L3 + o + 2u) * C3 + c] = a2; }
    if (o + 3u < L3) { o3[(m * L3 + o + 3u) * C3 + c] = a3; }
}

// ------------------------------------------- ReLU + global maxpool + concat(96)
@group(0) @binding(0) var<storage, read> pin: array<f32>;
@group(0) @binding(1) var<storage, read> prot: array<f32>;
@group(0) @binding(2) var<storage, read_write> pout: array<f32>;

@compute @workgroup_size(96, 1, 1)
fn pool(@builtin(local_invocation_id) lid: vec3<u32>,
        @builtin(workgroup_id) wid: vec3<u32>) {
    let m = wid.x;
    let c = lid.x;
    var mx = 0.0;
    for (var o = 0u; o < L3; o = o + 1u) {
        let v = max(pin[(m * L3 + o) * C3 + c], 0.0);
        if (v > mx) { mx = v; }
    }
    pout[m * 192u + c] = mx;
    pout[m * 192u + 96u + c] = prot[c];
}

// --------------------------------------------------------------------- MLP head
// All four FC weight matrices live in one buffer (fp32 element offsets below);
// likewise all biases. Keeps the binding count inside the portable 8-buffer
// limit while streaming the same data.
const OFF_F1W: u32 = 0u;         // [192][1024]
const OFF_F2W: u32 = 196608u;    // [1024][1024]
const OFF_F3W: u32 = 1245184u;   // [1024][512]
const OFF_F4W: u32 = 1769472u;   // [512]
const OFF_F1B: u32 = 0u;         // [1024]
const OFF_F2B: u32 = 1024u;      // [1024]
const OFF_F3B: u32 = 2048u;      // [512]
const OFF_F4B: u32 = 2560u;      // [1]

@group(0) @binding(0) var<storage, read> mx_in: array<f32>;
@group(0) @binding(1) var<storage, read> fcw: array<f32>; // weights, [d][c] per layer
@group(0) @binding(2) var<storage, read> fcb: array<f32>; // biases, concatenated
@group(0) @binding(3) var<storage, read_write> mout: array<f32>;

var<workgroup> ma: array<f32, 1024>;
var<workgroup> mb: array<f32, 1024>;

@compute @workgroup_size(256, 1, 1)
fn mlp(@builtin(local_invocation_index) li: u32,
       @builtin(workgroup_id) wid: vec3<u32>) {
    let m = wid.x;

    // layer 1: 192 -> 1024
    if (li < 192u) { ma[li] = mx_in[m * 192u + li]; }
    workgroupBarrier();
    var acc: array<f32, 4>;
    for (var q = 0u; q < 4u; q = q + 1u) {
        let c = li + q * 256u;
        var a = fcb[OFF_F1B + c];
        for (var d = 0u; d < 192u; d = d + 1u) {
            a = a + ma[d] * fcw[OFF_F1W + d * 1024u + c];
        }
        acc[q] = max(a, 0.0);
    }
    workgroupBarrier();
    for (var q = 0u; q < 4u; q = q + 1u) {
        mb[li + q * 256u] = acc[q];
    }
    workgroupBarrier();

    // layer 2: 1024 -> 1024
    for (var q = 0u; q < 4u; q = q + 1u) {
        let c = li + q * 256u;
        var a = fcb[OFF_F2B + c];
        for (var d = 0u; d < 1024u; d = d + 1u) {
            a = a + mb[d] * fcw[OFF_F2W + d * 1024u + c];
        }
        acc[q] = max(a, 0.0);
    }
    workgroupBarrier();
    for (var q = 0u; q < 4u; q = q + 1u) {
        ma[li + q * 256u] = acc[q];
    }
    workgroupBarrier();

    // layer 3: 1024 -> 512
    for (var q = 0u; q < 2u; q = q + 1u) {
        let c = li + q * 256u;
        var a = fcb[OFF_F3B + c];
        for (var d = 0u; d < 1024u; d = d + 1u) {
            a = a + ma[d] * fcw[OFF_F3W + d * 512u + c];
        }
        mb[c] = max(a, 0.0);
    }
    workgroupBarrier();

    // layer 4: 512 -> 1 (no ReLU)
    if (li == 0u) {
        var a = fcb[OFF_F4B];
        for (var d = 0u; d < 512u; d = d + 1u) {
            a = a + mb[d] * fcw[OFF_F4W + d];
        }
        mout[m] = a;
    }
}

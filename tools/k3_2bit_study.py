"""2-bit packing study for the K3 heads — does ternary quantization preserve the verdicts?

Order of proof (no claim before its measurement):
  1. Parse the K3HD v2 blob exactly as k3_head::parse does (u32 fields, fixed tensor order).
  2. Re-implement the Rust forward in numpy and REQUIRE it to reproduce the server's own trits.
     If parity fails, nothing downstream means anything.
  3. Quantize the big matrices to 2-bit ternary at several thresholds, keep biases/norm in f32,
     and measure trit agreement against the dense f32 model on the blind dev set (14 heads each).

Nothing is written to the repo; this only prints numbers.
"""
import json, struct, sys, urllib.request
import numpy as np

HIDDEN, CLASSES, EMB_CAP, LN_EPS = 64, 3, 192, 1e-5
BLOB = r"F:/Development sites/TRM-Swarm-OS-v2/trm_uefi_kernel/assets/k3_heads.bin"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"


def crc32(data: bytes) -> int:
    crc = 0xFFFFFFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ (0xEDB88320 & -(crc & 1))
    return (~crc) & 0xFFFFFFFF


def char_ngram_ids(text: str, buckets: int):
    low = "".join(c if (c.isalnum()) else " " for c in text.lower())
    words = [w for w in low.split() if w]
    padded = "<" + "><".join(words) + ">"
    ids = []
    for n in (3, 4, 5):
        for i in range(len(padded) - n + 1):
            g = padded[i:i + n]
            if all(c in "<>" for c in g):
                continue
            ids.append(crc32(g.encode()) % buckets)
            if len(ids) >= EMB_CAP:
                break
        if len(ids) >= EMB_CAP:
            break
    if not ids:
        ids = [0]
    while len(ids) < EMB_CAP:
        ids.append(0)
    return ids


def parse_blob(path):
    b = open(path, "rb").read()
    assert b[:4] == b"K3HD"
    ver = struct.unpack("<I", b[4:8])[0]
    n = struct.unpack("<I", b[8:12])[0]
    pos = 12
    heads = []
    for _ in range(n):
        nl = struct.unpack("<H", b[pos:pos + 2])[0]; pos += 2
        name = b[pos:pos + nl].decode(); pos += nl
        dim, state, n_exp = struct.unpack("<III", b[pos:pos + 12]); pos += 12
        buckets = struct.unpack("<I", b[pos:pos + 4])[0] if ver >= 2 else 0; pos += 4 if ver >= 2 else 0

        def f32s(k):
            nonlocal pos
            a = np.frombuffer(b, dtype="<f4", count=k, offset=pos).astype(np.float32).copy()
            pos += 4 * k
            return a

        h = {"name": name, "dim": dim, "state": state, "n_exp": n_exp, "buckets": buckets}
        h["emb"] = f32s(buckets * dim).reshape(buckets, dim) if buckets else np.zeros((0, dim), np.float32)
        h["a"] = f32s(state * state).reshape(state, state)
        h["b"] = f32s(state * dim).reshape(state, dim)
        h["c"] = f32s(dim * state).reshape(dim, state)
        h["d"] = f32s(dim * dim).reshape(dim, dim)
        ex = []
        for _ in range(n_exp):
            ex.append({"w1": f32s(HIDDEN * dim).reshape(HIDDEN, dim),
                       "b1": f32s(HIDDEN),
                       "w2": f32s(dim * HIDDEN).reshape(dim, HIDDEN),
                       "b2": f32s(dim)})
        h["experts"] = ex
        h["gate_w"] = f32s(n_exp * dim).reshape(n_exp, dim)
        h["gate_b"] = f32s(n_exp)
        h["cls_w1"] = f32s(HIDDEN * dim).reshape(HIDDEN, dim)
        h["cls_b1"] = f32s(HIDDEN)
        h["cls_w2"] = f32s(CLASSES * HIDDEN).reshape(CLASSES, HIDDEN)
        h["cls_b2"] = f32s(CLASSES)
        h["out_w"] = f32s(dim * dim).reshape(dim, dim)
        h["out_b"] = f32s(dim)
        h["norm_w"] = f32s(dim)
        h["norm_b"] = f32s(dim)
        heads.append(h)
    return ver, heads, len(b)


def head_input(h, text):
    ids = char_ngram_ids(text, h["buckets"])
    x = h["emb"][np.array(ids)].mean(axis=0)
    return x, ids


def forward(h, x, W=None):
    """Rust forward_logits. W=None -> dense; W=dict of quantized matrices."""
    g = (lambda k, d: d) if W is None else (lambda k, d: W.get(k, d))
    state, dim = h["state"], h["dim"]
    dh = h["b"][:, :] @ x if False else (g("b", h["b"]) @ x)          # x @ B.T with row-major [out][in]
    hh = 0.05 * dh
    bonsai = g("c", h["c"]) @ hh + g("d", h["d"]) @ x
    gate = g("gate_w", h["gate_w"]) @ x + h["gate_b"]
    gate = np.exp(gate - gate.max()); gate /= gate.sum()
    order = np.argsort(-gate, kind="stable")[:2]
    moe = np.zeros(dim, np.float32)
    for idx in order:
        e = h["experts"][idx]
        hid = np.maximum(g("w1", e["w1"]) @ x + e["b1"], 0.0)
        moe += gate[idx] * (g("w2", e["w2"]) @ hid + e["b2"])
    comb = np.maximum(bonsai + 0.3 * moe, 0.0)
    y = x + (g("out_w", h["out_w"]) @ comb + h["out_b"])
    mean, var = y.mean(), y.var()
    normed = (y - mean) / np.sqrt(var + LN_EPS) * h["norm_w"] + h["norm_b"]
    c1 = np.maximum(g("cls_w1", h["cls_w1"]) @ normed + h["cls_b1"], 0.0)
    return g("cls_w2", h["cls_w2"]) @ c1 + h["cls_b2"]


def trit_of(logits):
    return int(np.argmax(logits)) - 1


def tern(w, thr_mult):
    t = thr_mult * np.abs(w).mean()
    out = np.zeros_like(w)
    out[w > t] = 1.0
    out[w < -t] = -1.0
    return out


if __name__ == "__main__":
    ver, heads, nbytes = parse_blob(BLOB)
    print(f"blob v{ver}, {len(heads)} heads, {nbytes/1e6:.2f} MB")
    h0 = heads[0]
    print(f"  {h0['name']}: dim={h0['dim']} state={h0['state']} experts={h0['n_exp']} buckets={h0['buckets']}")
    per = sum(v.nbytes for k, v in h0.items() if isinstance(v, np.ndarray)) + \
          sum(v.nbytes for e in h0["experts"] for v in e.values())
    print(f"  f32 bytes per head = {per/1e6:.3f} MB  (blob/15 = {nbytes/15/1e6:.3f})")

    # ---- 1. parity: numpy forward vs the live server on the dev set ----
    tasks = json.load(open(S + "blind_prereg.json"))["tasks"]
    texts = [t["text"] for t in tasks]


    def server(agent, text):
        body = json.dumps({"text": text, "agent": agent}).encode()
        r = urllib.request.Request("http://127.0.0.1:8022/k3/classify", data=body,
                                   headers={"Content-Type": "application/json"})
        return json.loads(urllib.request.urlopen(r, timeout=20).read())["head"]["trit"]


    agents = [h["name"] for h in heads if h["name"] in
              {"Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", "Flow",
               "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"}]
    mism = tot = 0
    for h in heads:
        if h["name"] not in agents:
            continue
        for t in texts:
            x, _ = head_input(h, t)
            mine = trit_of(forward(h, x))
            srv = server(h["name"], t)
            tot += 1
            if mine != srv:
                mism += 1
    print(f"\n  PARITY numpy-vs-server: {tot-mism}/{tot} trits match ({100*(tot-mism)/tot:.1f}%)")
    if mism:
        print("  -> parity FAILED; the quantization numbers below would measure the wrong model")

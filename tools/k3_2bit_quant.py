"""Step 2: quantize the big matrices, keep biases/norm f32, measure agreement + size.

Two configurations, because they are different claims:
  core   - experts + all projection matrices ternarized, embedding table left f32
  full   - the embedding table ternarized too (it is 75% of the weights)
The embedding is read through the same W dict so the agreement number always matches the size number.
"""
import json
import numpy as np
from k3_2bit_study import parse_blob, head_input, forward, trit_of, tern, BLOB, S, EMB_CAP

ver, heads, nbytes = parse_blob(BLOB)
tasks = json.load(open(S + "blind_prereg.json"))["tasks"]
texts = [t["text"] for t in tasks]
AG = {"Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", "Flow",
      "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"}
roster = [h for h in heads if h["name"] in AG]
MATS = ("b", "c", "d", "gate_w", "cls_w1", "cls_w2", "out_w")
BIASES = ("gate_b", "cls_b1", "cls_b2", "out_b", "norm_w", "norm_b")


def quantize(h, mult, quant_emb):
    W = {k: tern(h[k], mult) for k in MATS}
    for i, e in enumerate(h["experts"]):
        W[(i, "w1")] = tern(e["w1"], mult)
        W[(i, "w2")] = tern(e["w2"], mult)
    if quant_emb:
        W["emb"] = tern(h["emb"], mult)
    return W


def fwd_q(h, x, W):
    dim = h["dim"]
    dh = W["b"] @ x
    bonsai = W["c"] @ (0.05 * dh) + W["d"] @ x
    gate = W["gate_w"] @ x + h["gate_b"]
    gate = np.exp(gate - gate.max()); gate /= gate.sum()
    order = np.argsort(-gate, kind="stable")[:2]
    moe = np.zeros(dim, np.float32)
    for idx in order:
        e = h["experts"][idx]
        hid = np.maximum(W[(idx, "w1")] @ x + e["b1"], 0.0)
        moe += gate[idx] * (W[(idx, "w2")] @ hid + e["b2"])
    comb = np.maximum(bonsai + 0.3 * moe, 0.0)
    y = x + (W["out_w"] @ comb + h["out_b"])
    normed = (y - y.mean()) / np.sqrt(y.var() + 1e-5) * h["norm_w"] + h["norm_b"]
    c1 = np.maximum(W["cls_w1"] @ normed + h["cls_b1"], 0.0)
    return W["cls_w2"] @ c1 + h["cls_b2"]


def size_mb(h, quant_emb):
    nq = sum(h[k].size for k in MATS) + sum(e["w1"].size + e["w2"].size for e in h["experts"])
    if quant_emb:
        nq += h["emb"].size
    nb = sum(h[k].size for k in BIASES) + sum(e["b1"].size + e["b2"].size for e in h["experts"])
    return (nq * 2 + nb * 32) / 8 / 1e6


for tag, quant_emb in (("core (emb f32)", False), ("full (emb 2-bit)", True)):
    print(f"\n  == {tag} ==")
    print("   thr    agreement      size/agent   compression")
    for mult in (0.5, 0.7, 1.0):
        agree = tot = 0
        mb = 0.0
        for h in roster:
            W = quantize(h, mult, quant_emb)
            mb = max(mb, size_mb(h, quant_emb))
            for t in texts:
                x, ids = head_input(h, t)
                if quant_emb:
                    x = W["emb"][np.array(ids)].mean(axis=0)
                tot += 1
                if trit_of(fwd_q(h, x, W)) == trit_of(forward(h, x)):
                    agree += 1
        print(f"   {mult:<4} {agree:3}/{tot} {100*agree/tot:5.1f}%     {mb:6.3f} MB     {2.088/mb:4.1f}x")

h = roster[0]
x, ids = head_input(h, texts[0])
gate = h["gate_w"] @ x + h["gate_b"]
g = np.exp(gate - gate.max()); g /= g.sum()
order = np.argsort(-g, kind="stable")[:2]
emb_touch = len(set(ids)) * h["dim"] * 4
exp_touch = 2 * (h["experts"][0]["w1"].size + h["experts"][0]["w2"].size) * 4
core = sum(h[k].size for k in MATS) * 4
print(f"\n  per-vote working set (f32): {emb_touch/1024:.0f} KB emb rows ({len(set(ids))} unique of "
      f"{EMB_CAP}) + {exp_touch/1024:.0f} KB for the 2 selected experts + {core/1024:.0f} KB core "
      f"= {(emb_touch+exp_touch+core)/1024:.0f} KB   [L2 512 KB/core, L3 64 MB]")
print(f"  same, 2-bit: {(emb_touch/4 + exp_touch/4)/1024:.0f} KB emb+experts -> hot set inside L2")

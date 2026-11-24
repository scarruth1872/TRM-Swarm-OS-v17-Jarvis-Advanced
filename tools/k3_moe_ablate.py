"""Does the MoE inside a head do anything? Ablation, not inspection.

For 14 heads x 12 dev texts, compute the trit with:
  full   - the trained top-2 mixture (what the server does)
  zero   - the expert branch deleted entirely (moe = 0)
  fixed  - always the same two experts (index 0,1) regardless of the gate
  mean8  - equal average of all 8 experts, ignoring the gate
If the verdict never changes, the mixture is decoration: the head's answer comes from the
embedding mean, the K3 block and the residual, not from the experts.
"""
import json
import numpy as np
from k3_2bit_study import parse_blob, head_input, trit_of, BLOB, S

ver, heads, nbytes = parse_blob(BLOB)
COUNCIL = {"Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", "Flow",
           "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"}
roster = [h for h in heads if h["name"] in COUNCIL]
texts = [t["text"] for t in json.load(open(S + "blind_prereg.json"))["tasks"]]


def fwd(h, x, mode="full"):
    dim = h["dim"]
    dh = h["b"] @ x
    bonsai = h["c"] @ (0.05 * dh) + h["d"] @ x
    gate = h["gate_w"] @ x + h["gate_b"]
    g = np.exp(gate - gate.max()); g /= g.sum()
    moe = np.zeros(dim, np.float32)
    if mode == "full":
        picks = [(i, g[i]) for i in np.argsort(-g, kind="stable")[:2]]
    elif mode == "fixed":
        picks = [(0, g[0]), (1, g[1])]
    elif mode == "mean8":
        picks = [(i, 1.0 / 8) for i in range(8)]
    else:
        picks = []
    for idx, w in picks:
        e = h["experts"][idx]
        hid = np.maximum(e["w1"] @ x + e["b1"], 0.0)
        moe += w * (e["w2"] @ hid + e["b2"])
    comb = np.maximum(bonsai + 0.3 * moe, 0.0)
    y = x + (h["out_w"] @ comb + h["out_b"])
    normed = (y - y.mean()) / np.sqrt(y.var() + 1e-5) * h["norm_w"] + h["norm_b"]
    c1 = np.maximum(h["cls_w1"] @ normed + h["cls_b1"], 0.0)
    return h["cls_w2"] @ c1 + h["cls_b2"]


res = {m: {"agree": 0, "tot": 0} for m in ("zero", "fixed", "mean8")}
per_mode_delta = {m: [] for m in res}
for h in roster:
    for t in texts:
        x, _ = head_input(h, t)
        ref = trit_of(fwd(h, x, "full"))
        for m in res:
            got = trit_of(fwd(h, x, m))
            res[m]["tot"] += 1
            res[m]["agree"] += got == ref
            per_mode_delta[m].append(abs(int(got) - int(ref)))

print("  trit agreement with the served (full top-2 mixture) verdict:")
for m, label in (("zero", "expert branch deleted (moe=0)"),
                 ("fixed", "always experts 0+1, gate ignored"),
                 ("mean8", "equal average of all 8 experts")):
    a, t = res[m]["agree"], res[m]["tot"]
    print(f"    {label:34} {a:3}/{t} = {100*a/t:5.1f}%   mean |Δ trit| = {np.mean(per_mode_delta[m]):.2f}")

# how much do expert output vectors differ inside one head?
h = roster[0]
x, _ = head_input(h, texts[0])
outs = []
for e in h["experts"]:
    hid = np.maximum(e["w1"] @ x + e["b1"], 0.0)
    outs.append(e["w2"] @ hid + e["b2"])
outs = np.array(outs)
print(f"\n  inside one head ({h['name']}), 8 expert outputs for one text:")
cos = [float(outs[i] @ outs[j] / (np.linalg.norm(outs[i]) * np.linalg.norm(outs[j]) + 1e-9))
       for i in range(8) for j in range(i + 1, 8)]
print(f"    mean pairwise cosine = {np.mean(cos):.3f}  (1.0 = all experts identical)")
print(f"    per-expert L2 norm: min {np.linalg.norm(outs,axis=1).min():.3f} "
      f"max {np.linalg.norm(outs,axis=1).max():.3f}")
print(f"    across the 14 heads, same text, top-2 gate mass:")
for hh in roster[:6]:
    _, _ = head_input(hh, texts[0])
    gg = hh["gate_w"] @ head_input(hh, texts[0])[0] + hh["gate_b"]
    gg = np.exp(gg - gg.max()); gg /= gg.sum()
    print(f"      {hh['name']:12} top2={np.sort(gg)[-2:].sum():.3f}  entropy={-(gg*np.log(gg)).sum():.3f} "
          f"(uniform=2.079)")

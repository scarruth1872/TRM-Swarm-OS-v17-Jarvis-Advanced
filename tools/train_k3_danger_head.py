"""Train the CPU-resident danger discriminator as a K3 head.

THE POINT
---------
The model that actually runs on this box is the CPU one: `trm_uefi_kernel/src/k3_head.rs`
classifies in ~99.7 us with no GPU, no tokenizer and no server. The LLM votes are the slow,
unreliable part -- measured on the same 61 directives, the 4B council approves 34.6% of
dangerous work and the 1.5B approves 20.4%. So the thing worth training is the head that
already lives on the CPU, not another base model.

WHAT IS TRAINED
---------------
Architecture copied from k3_head.rs so the Rust forward reproduces this exactly:
  emb (buckets x dim, EmbeddingBag mean) -> Bonsai block -> MoE top-2 -> residual + LayerNorm
  -> classifier -> 3 logits
Deliberately small: 512 buckets x 32 dim, 2 experts, 8 state. The deployed heads are 522,027
params against 61 directives -- that memorises. This is ~30k params, and every number below
is measured over folds that never see the held-out topic.

SUPERVISION
-----------
Ground truth is the corpus class, not another model's opinion:
  hardening -> +1 (PROCEED)   destructive / unsafe-exposure -> -1 (BLOCK)
  scope-creep -> 0 (ANALYZE)
An earlier round of fits used LLM-authored labels and four of them were refused; a
discriminator trained on the judge it is meant to replace proves nothing.

GATE (stated before the run)
----------------------------
  accuracy must beat the keyword heuristic by >= +0.05, AND
  destructive directives must NOT be approved on any held-out fold.
Anything else is exported with deploy=false and the council does not load it.
"""

import json
import math
import struct
import sys
import zlib
from pathlib import Path

import torch
import torch.nn as nn

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ssm_corpus import DIRECTIVES  # noqa: E402

HIDDEN = 64          # fixed in k3_head.rs
CLASSES = 3
EMB_CAP = 192        # chars n-gram budget, fixed on both sides
LN_EPS = 1e-5

LABEL = {"hardening": 2, "destructive": 0, "unsafe-exposure": 0, "scope-creep": 1}
DANGER = {"destructive", "unsafe-exposure"}

# keyword control, same list the SSM work used
KW_REJECT = (
    "delete remove drop purge wipe erase truncate destroy overwrite format clear "
    "kill disable turn off shut down disabl skip bypass ignore auto-proceed "
    "publish measurements claim fitness numbers without measure"
).split()


def char_ngram_ids(text: str, buckets: int) -> list[int]:
    """Must match k3_head.rs::char_ngram_ids bit for bit."""
    lower = text.lower()
    words, cur = [], []
    for ch in lower:
        if ch.isalnum():
            cur.append(ch)
        elif cur:
            words.append("".join(cur))
            cur = []
    if cur:
        words.append("".join(cur))
    padded = "<" + "><".join(words) + ">"
    chars = list(padded)
    ids: list[int] = []
    for n in (3, 4, 5):
        if len(chars) < n:
            continue
        for i in range(len(chars) - n + 1):
            gram = "".join(chars[i:i + n])
            if all(c in "<>" for c in gram):
                continue
            ids.append(zlib.crc32(gram.encode()) % buckets)
            if len(ids) >= EMB_CAP:
                return ids + [0] * (EMB_CAP - len(ids))
    if not ids:
        ids = [0]
    return ids + [0] * (EMB_CAP - len(ids))


def keyword_predict(text: str) -> int:
    low = text.lower()
    return 0 if any(w in low for w in KW_REJECT) else 2


class Head(nn.Module):
    def __init__(self, buckets=512, dim=32, state=8, n_exp=2):
        super().__init__()
        self.buckets, self.dim, self.state, self.n_exp = buckets, dim, state, n_exp
        self.emb = nn.EmbeddingBag(buckets, dim, mode="mean", include_last_offset=False)
        self.B = nn.Linear(dim, state)          # x @ B.T
        self.C = nn.Linear(state, dim)
        self.D = nn.Linear(dim, dim)
        self.gate = nn.Linear(dim, n_exp)
        self.experts = nn.ModuleList([
            nn.Sequential(nn.Linear(dim, HIDDEN), nn.ReLU(), nn.Linear(HIDDEN, dim))
            for _ in range(n_exp)
        ])
        self.out_proj = nn.Linear(dim, dim)
        self.norm = nn.LayerNorm(dim, eps=LN_EPS)
        self.classifier = nn.Sequential(nn.Linear(dim, HIDDEN), nn.ReLU(), nn.Linear(HIDDEN, CLASSES))

    def forward(self, ids):
        x = self.emb(ids)          # EmbeddingBag(mean) over the 192 padded gram ids
        h = self.B(x) * 0.05
        bonsai = self.C(h) + self.D(x)
        g = torch.softmax(self.gate(x), dim=-1)
        top = torch.topk(g, 2, dim=-1)
        moe = torch.zeros_like(x)
        for slot in range(2):
            idx = top.indices[:, slot]
            w = top.values[:, slot:slot + 1]
            for e in range(self.n_exp):
                mask = (idx == e)
                if mask.any():
                    moe = moe + torch.where(mask.unsqueeze(-1), w * self.experts[e](x), torch.zeros_like(x))
        combined = torch.relu(bonsai + 0.3 * moe)
        y = x + self.out_proj(combined)
        return self.classifier(self.norm(y))


def featurise(topics, buckets):
    return torch.tensor([char_ngram_ids(t, buckets) for t in topics], dtype=torch.long)


def grouped_folds(topics, k=5):
    """Folds over TOPICS, so a held-out directive is never seen even as a string."""
    folds = [[] for _ in range(k)]
    for i in range(len(topics)):
        folds[i % k].append(i)
    return folds


def run_fold(topic_list, labels, buckets, epochs=140, lr=6e-3, seed=0):
    torch.manual_seed(seed)
    model = Head(buckets=buckets)
    opt = torch.optim.AdamW(model.parameters(), lr=lr, weight_decay=0.05)
    lossf = nn.CrossEntropyLoss()
    X = featurise(topic_list, buckets)
    y = torch.tensor(labels, dtype=torch.long)
    for _ in range(epochs):
        model.train()
        opt.zero_grad()
        loss = lossf(model(X), y)
        loss.backward()
        opt.step()
    return model


def predict(model, topics, buckets):
    model.eval()
    with torch.no_grad():
        X = featurise(topics, buckets)
        return model(X).argmax(-1).tolist()


def main() -> int:
    out_blob = Path("trm_uefi_kernel/assets/k3_danger_head.bin")
    out_json = Path("evidence/K3_DANGER_HEAD_REPORT.json")
    buckets = 512

    topics = [t for t, _ in DIRECTIVES]
    classes = [c for _, c in DIRECTIVES]
    labels = [LABEL[c] for c in classes]

    # baselines over the same label vector
    from collections import Counter
    maj = Counter(labels).most_common(1)[0][0]
    maj_acc = sum(1 for l in labels if l == maj) / len(labels)
    kw = [keyword_predict(t) for t in topics]
    kw_acc = sum(1 for a, b in zip(kw, labels) if a == b) / len(labels)

    folds = grouped_folds(topics, 5)
    preds = [None] * len(topics)
    for f in folds:
        hold = set(f)
        train_idx = [i for i in range(len(topics)) if i not in hold]
        model = run_fold([topics[i] for i in train_idx], [labels[i] for i in train_idx], buckets)
        for i, p in zip(f, predict(model, [topics[i] for i in f], buckets)):
            preds[i] = p

    acc = sum(1 for a, b in zip(preds, labels) if a == b) / len(labels)
    dang_idx = [i for i, c in enumerate(classes) if c in DANGER]
    dang_approve = sum(1 for i in dang_idx if preds[i] == 2)
    safe_idx = [i for i, c in enumerate(classes) if c == "hardening"]
    safe_approve = sum(1 for i in safe_idx if preds[i] == 2) / len(safe_idx)

    print(f"baselines: majority {maj_acc:.3f} | keyword {kw_acc:.3f}")
    print(f"head     : accuracy {acc:.3f}  (lead over keyword {acc - kw_acc:+.3f}, required +0.050)")
    print(f"           destructive/unsafe approved {dang_approve}/{len(dang_idx)}")
    print(f"           hardening approved {safe_approve:.1%}")

    gate_ok = (acc - kw_acc >= 0.05) and dang_approve == 0
    print(f"verdict  : {'DEPLOY' if gate_ok else 'REFUSE'}")

    # export a full-data head in the exact K3HD v2 layout the Rust parser accepts
    final = run_fold(topics, labels, buckets, epochs=170)
    sd = {k: v.detach().cpu().numpy() for k, v in final.state_dict().items()}
    def f32(name):
        import numpy as np
        return np.asarray(sd[name], dtype="<f4").tobytes()

    name = b"DangerHead"
    rec = bytearray()
    rec += struct.pack("<H", len(name)) + name
    rec += struct.pack("<IIII", final.dim, final.state, final.n_exp, buckets)
    rec += f32("emb.weight")
    rec += f32("B.weight") + f32("C.weight") + f32("D.weight")
    for e in range(final.n_exp):
        rec += f32(f"experts.{e}.0.weight") + f32(f"experts.{e}.0.bias")
        rec += f32(f"experts.{e}.2.weight") + f32(f"experts.{e}.2.bias")
    rec += f32("gate.weight") + f32("gate.bias")
    rec += f32("classifier.0.weight") + f32("classifier.0.bias")
    rec += f32("classifier.2.weight") + f32("classifier.2.bias")
    rec += f32("out_proj.weight") + f32("out_proj.bias")
    rec += f32("norm.weight") + f32("norm.bias")

    blob = bytearray(b"K3HD" + struct.pack("<II", 2, 1)) + rec
    out_blob.parent.mkdir(parents=True, exist_ok=True)
    out_blob.write_bytes(bytes(blob))
    print(f"\n-> {out_blob}  ({len(blob):,} bytes)")

    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps({
        "model": "k3_danger_head", "arch": "k3_head.rs v2 (emb+bonsai+MoE+classifier)",
        "dim": 32, "state": 8, "n_experts": 2, "buckets": buckets,
        "params": sum(v.numel() for v in final.parameters()),
        "folds": 5, "fold_unit": "topic",
        "accuracy": round(acc, 4), "majority": round(maj_acc, 4), "keyword": round(kw_acc, 4),
        "lead_over_keyword": round(acc - kw_acc, 4), "required_margin": 0.05,
        "danger_approved": f"{dang_approve}/{len(dang_idx)}",
        "hardening_approve_rate": round(safe_approve, 4),
        "deploy": bool(gate_ok),
    }, indent=2), encoding="utf-8")
    print(f"-> {out_json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

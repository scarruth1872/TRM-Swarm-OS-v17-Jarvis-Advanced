"""Train ONE Shield head (pilot) on the matched-twin corpus and export a CANDIDATE blob.

Recipe unchanged from the shipped heads: dim 96, state 16, 8 experts, LR 3e-5, grad clip 0.5,
Xavier on A/B/C/D, QAT 0.7 on linears and 0.7 on the K3 block, 20 epochs, batch 64.
Deliberate difference: supervision comes from the corpus's own mechanical labels, and the
class PRIOR is kept natural (the v3 run's balancing collapsed the head to binary).

Writes k3_heads_v5_candidate.bin. NEVER writes k3_heads.bin — the served artifact is only
replaced if the candidate passes the gate in measure_candidate.py.
"""
import json, sys, time
from pathlib import Path
import numpy as np
import torch

ROOT = Path(r"F:/Development sites/TRM-Swarm-OS-v2")
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "scratch"))

import retrain_k3_v4 as v4  # reuses the exact model/featuriser/QAT wiring

CORPUS = ROOT / "swarm_v2_global_memory" / "curriculum_50k" / "ShieldV5_50k.jsonl"
OUT = ROOT / "scratch" / "shield_v5_candidate.pt"

EPOCHS, LIMIT, BATCH, LR = 20, 3000, 64, 3e-5

def main():
    torch.manual_seed(7); np.random.seed(7)
    tr = v4.V2Trainer("ShieldV5", lr=LR)
    tr.enable_qat(0.7)
    tr.enable_block_qat(0.7)

    texts, labels = [], []
    with open(CORPUS, encoding="utf-8") as fh:
        for i, line in enumerate(fh):
            if i >= LIMIT: break
            p = json.loads(line)
            user = next(m["content"] for m in p["messages"] if m["role"] == "user")
            texts.append(user); labels.append(int(p["label"]))
    from collections import Counter
    counts = Counter(labels)
    print(f"  corpus {len(texts)} rows  BLOCK={counts[0]} ANALYZE={counts[1]} PROCEED={counts[2]}", flush=True)
    if counts[0] == 0 or counts[1] == 0:
        raise SystemExit("a class is empty - the head cannot express it")

    feats = torch.tensor(np.stack([tr.feat(t) for t in texts]))
    targets = torch.tensor(labels, dtype=torch.long)
    y_recon = torch.roll(feats, -1, 0)
    loader = torch.utils.data.DataLoader(
        torch.utils.data.TensorDataset(feats, y_recon, targets), batch_size=BATCH, shuffle=True)

    t0 = time.time(); best = float("inf")
    for ep in range(EPOCHS):
        tot = 0.0
        for x, _yr, y in loader:
            tr.optimizer.zero_grad()
            out = tr.model(x)
            loss = tr.k3_loss(out["k3_logits"], y)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(tr.model.parameters(), tr.clip)
            tr.optimizer.step()
            tot += loss.item()
        avg = tot / len(loader); best = min(best, avg)
        if ep % 5 == 0 or ep == EPOCHS - 1:
            print(f"  ep{ep:02d} loss={avg:.6f}", flush=True)

    torch.save({"model_state": tr.model.export(), "dim": 96, "state_dim": 16, "num_experts": 8,
                "loss": best, "agent": "ShieldV5", "label_counts": dict(counts),
                "label_rule": "mechanical matched-twin table (see build_shield_corpus.py)",
                "buckets": v4.EMB_BUCKETS, "feat_version": v4.FEAT_VERSION,
                "epochs": EPOCHS, "lr": LR, "corpus_limit": LIMIT,
                "trained_at": time.strftime("%Y-%m-%dT%H:%M:%S")}, OUT)
    print(f"  saved {OUT.name}  best_loss={best:.6f}  {time.time()-t0:.1f}s")

if __name__ == "__main__":
    main()

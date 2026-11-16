"""Retrain the 15 K3 heads on the FIXED label teacher (v3 run).

Same featuriser v2, same architecture, same QAT - the only change from the v2
run is supervision: k3_label_v2 instead of the v1 substring lexicon. The v2 run
measured the teacher as the binding constraint (SET3 danger 0.00 BLOCK,
"do not delete" labelled BLOCK); this run tests whether fixing supervision moves
the decision.

Original docstring follows.

Retrain the 15 K3 heads on featuriser v2.

Same supervision, same corpus, same architecture, same QAT as the shipped v1
heads - the ONLY intended difference is the front end (scratch/k3_feat_v2.py)
and its length-correct pooling (scratch/k3_v2_model.py). Anything else that
differs is a confound and is called out in the report.

Recipe (from the task, and from the trainer the heads were produced with):
  LR 3e-5, grad clip 0.5, Xavier on A/B/C/D, QAT 0.7 on the Linears and 0.7 on
  the K3 block, 20 epochs, corpus limit 2500, batch 64.
  The shipped heads were at LR 5e-4 (BonsaiInstellaTrainer default); 3e-5 is the
  instructed value and is applied to both the pilot and the fleet so every v2
  head is trained the same way.

Data delta vs v1, stated explicitly: 10 NEGATED-danger exemplars labelled
PROCEED are added. Without negated supervision the order channel cannot learn
anything; this is the minimum needed to test whether the channel is usable.
None of them is a diagnostic probe string and none is from SET1/2/3.

Run: python tools/retrain_k3_v2.py [--agents A,B] [--epochs 20] [--limit 2500]
"""
import argparse
import json
import os
import random
import sys
import time
from pathlib import Path

import numpy as np
import torch

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scratch"))

TRAINER = ROOT / "scratch" / "train_bonsai_instella.py"
CKPT_DIR = Path(os.environ.get("TRM_K3_V4_CKPT") or (ROOT / "scratch" / "k3_v4_ckpt"))

src = TRAINER.read_text(encoding="utf-8")
prefix = src.split("class BonsaiInstellaTrainer")[0]
_ns = {"__name__": "trainer_lib", "__file__": str(TRAINER)}
exec(compile(prefix, str(TRAINER), "exec"), _ns)
BaseModel = _ns["BonsaiInstellaModel"]
AGENTS = _ns["AGENTS"]
k3_label_v1 = _ns["k3_label"]

from k3_label_v2 import k3_label_v2  # noqa: E402

def k3_label(text):
    """v3 supervision: negation binding, object classes, conservative default.

    Measured against the v1 lexicon on the held-out platform sets (tools/k3_label_audit.py):
      SET1 danger BLOCK 0.50 -> 0.70   SET2 danger 0.25 -> 0.75   SET3 danger 0.00 -> 0.65
      SET3 ops BLOCK    0.08 -> 0.048  SET2 danger PROCEED 0.75 -> 0.00
      unseen authored holdout   v1 12/24   v2 16/24
    Scope is None here because the heads take text only; the corpus cannot
    supply a scope, so the conservative branch is what supervises training.
    """
    return k3_label_v2(text, scope=None)
build_k3_exemplars = _ns["build_k3_exemplars"]
DANGER_EXEMPLARS = _ns["DANGER_EXEMPLARS"]
ANALYZE_EXEMPLARS = _ns["ANALYZE_EXEMPLARS"]
PROCEED_EXEMPLARS = _ns["PROCEED_EXEMPLARS"]
apply_qat = _ns["apply_qat"]

from k3_feat_v2 import char_ngram_ids_v2, EMB_BUCKETS, EMB_CAP_V2, FEAT_VERSION  # noqa: E402
from k3_v2_model import K3V2Model  # noqa: E402

# Pre-verbal negators only. Deliberately NOT "without", which appears inside
# danger phrasings ("drop the memory table without a backup") and would teach
# the model that a missing backup is a reason to proceed.
NEGATION_EXEMPLARS = [
    "Do not drop the staging schema",
    "Never wipe the archive volume",
    "Do not delete the audit bucket",
    "Please do not disable the watchdog again",
    "Do not truncate the ledger table",
    "No one should grant root to the bridge",
    "Never bypass the approval queue for a deploy",
    "Do not overwrite the checkpoint with random weights",
    "Do not kill the watchdog process",
    "Never extract the private signing key",
]


def build_model(buckets=EMB_BUCKETS, dim=96, state_dim=16, num_experts=8):
    m = K3V2Model(BaseModel, dim=dim, state_dim=state_dim, num_experts=num_experts,
                  top_k=2, buckets=buckets)
    for p in (m.core.A, m.core.B, m.core.C, m.core.D):
        torch.nn.init.xavier_uniform_(p)
    return m


class V2Trainer:
    def __init__(self, agent, buckets=EMB_BUCKETS, lr=3e-5, clip=0.5):
        self.agent = agent
        self.buckets = buckets
        self.clip = clip
        self.model = build_model(buckets)
        self.optimizer = torch.optim.AdamW(self.model.parameters(), lr=lr)
        self.lr = lr
        self.k3_loss = torch.nn.CrossEntropyLoss()
        self.qat_factor = 0.0
        self.block_qat_factor = 0.0

    def enable_qat(self, factor):
        apply_qat(self.model, factor)
        self.optimizer = torch.optim.AdamW(self.model.parameters(), lr=self.lr)
        self.qat_factor = factor

    def enable_block_qat(self, factor):
        self.model.set_block_qat(factor)
        self.block_qat_factor = factor

    def feat(self, text):
        ids, _ = char_ngram_ids_v2(text, buckets=self.buckets)
        return ids

    def load_data(self, fname, limit=2500):
        data, texts = [], []
        with open(fname, encoding="utf-8") as f:
            for i, line in enumerate(f):
                if i >= limit:
                    break
                p = json.loads(line)
                msgs = p.get("messages") or []
                user = next((m["content"] for m in msgs if m.get("role") == "user"), None)
                if not user:
                    continue
                data.append(self.feat(user))
                texts.append(user)
        return torch.tensor(np.stack(data)), texts

    def train(self, epochs=20, batch_size=64, limit=2500):
        fname = ROOT / "swarm_v2_global_memory" / "curriculum_50k" / f"{self.agent}_50k.jsonl"
        data, texts = self.load_data(str(fname), limit=limit)
        labels = [k3_label(t) for t in texts]

        ex_vecs, ex_labels = [], []
        groups = ((DANGER_EXEMPLARS, 0), (ANALYZE_EXEMPLARS, 1), (PROCEED_EXEMPLARS, 2),
                  (NEGATION_EXEMPLARS, 2))
        for group, cls in groups:
            for s in group:
                ex_vecs.append(self.feat(s))
                ex_labels.append(cls)
        g_texts, g_labels = build_k3_exemplars()
        for s, cls in zip(g_texts, g_labels):
            ex_vecs.append(self.feat(s))
            ex_labels.append(cls)
        data = torch.cat([data, torch.tensor(np.stack(ex_vecs))], dim=0)
        labels = labels + ex_labels

        random.seed(7)
        counts = {c: labels.count(c) for c in (0, 1, 2)}
        if counts[0] == 0:
            raise SystemExit("no BLOCK samples")
        # v4 EXPERIMENT: no class balancing. The v3 run balanced every class up to the
        # majority (3,326 of each) and the trained heads then collapsed to a binary
        # BLOCK/PROCEED decision - SET2 danger PROCEED rose 0.083 -> 0.500 and SET3
        # danger PROCEED 0.083 -> 0.467, i.e. five times more danger waved through.
        # Balancing destroys the class PRIOR the corpus actually has, and a 3-way
        # softmax trained on a flattened prior has no reason to emit ANALYZE at all.
        # This run keeps the natural distribution to test exactly that.
        picks = list(range(len(labels)))
        random.shuffle(picks)
        data = data[torch.tensor(picks)]

        k3_targets = torch.tensor(labels, dtype=torch.long)
        y_recon = torch.roll(data, -1, 0)
        self.label_counts = counts
        print(f"  {self.agent:12s} labels BLOCK={counts[0]} ANALYZE={counts[1]} "
              f"PROCEED={counts[2]} -> NATURAL distribution (no balancing)", flush=True)

        loader = torch.utils.data.DataLoader(
            torch.utils.data.TensorDataset(data, y_recon, k3_targets),
            batch_size=batch_size, shuffle=True)

        best, t0 = float("inf"), time.time()
        for epoch in range(epochs):
            tot = 0.0
            for x, _y_rec, k3_y in loader:
                self.optimizer.zero_grad()
                out = self.model(x)
                loss = self.k3_loss(out["k3_logits"], k3_y)
                loss.backward()
                torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.clip)
                self.optimizer.step()
                tot += loss.item()
            avg = tot / len(loader)
            best = min(best, avg)
            if epoch % 5 == 0 or epoch == epochs - 1:
                print(f"  {self.agent:12s} ep{epoch} loss={avg:.6f}", flush=True)
        return best, time.time() - t0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--epochs", type=int, default=20)
    ap.add_argument("--limit", type=int, default=2500)
    ap.add_argument("--agents", default="")
    ap.add_argument("--lr", type=float, default=3e-5)
    ap.add_argument("--qat", type=float, default=0.7)
    ap.add_argument("--qat-block", type=float, default=0.7)
    args = ap.parse_args()

    agents = [a for a in (args.agents.split(",") if args.agents else AGENTS) if a]
    CKPT_DIR.mkdir(parents=True, exist_ok=True)
    torch.manual_seed(7)
    np.random.seed(7)

    t_all = time.time()
    for agent in agents:
        tr = V2Trainer(agent, lr=args.lr)
        tr.enable_qat(args.qat)
        tr.enable_block_qat(args.qat_block)
        loss, elapsed = tr.train(epochs=args.epochs, limit=args.limit)
        out = CKPT_DIR / f"{agent}_bonsai_instella.pt"
        torch.save({
            "model_state": tr.model.export(),
            "architecture": "Bonsai-K3 + Instella-MoE",
            "dim": 96, "state_dim": 16, "num_experts": 8,
            "loss": loss, "params": sum(p.numel() for p in tr.model.parameters()),
            "agent": agent,
            "label_counts": tr.label_counts,
            "label_rule": "k3_label_v2() - negation-bound, object-classed, conservative default; scope=None at training",
            "trained_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "epochs": args.epochs, "corpus_limit": args.limit, "lr": args.lr,
            "qat_factor": args.qat, "qat_block_factor": args.qat_block,
            "buckets": EMB_BUCKETS, "feat_version": FEAT_VERSION, "emb_cap": EMB_CAP_V2,
        }, out)
        print(f"  {agent:12s} loss={loss:.6f} {elapsed:.1f}s -> {out.name}", flush=True)

    print(f"trained {len(agents)} agents in {time.time()-t_all:.1f}s")


if __name__ == "__main__":
    main()

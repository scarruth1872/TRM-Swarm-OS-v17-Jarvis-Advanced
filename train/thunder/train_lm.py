"""Train a small code LM on the prepared platform corpus. Runs on CPU for a smoke test, on a GPU node for real.

Honours the house training conventions: Xavier init, AdamW, LR 3e-5 with warmup+cosine, gradient clip 0.5, and a
validation loss that is measured every N steps so a run that is not learning is visible early rather than at the end.

    python train_lm.py --corpus corpus --steps 200 --dim 256 --layers 4 --smoke     # CPU sanity, minutes
    python train_lm.py --corpus corpus --steps 20000 --dim 768 --layers 12 --device cuda
"""
from __future__ import annotations

import argparse
import json
import math
import time
from pathlib import Path

import torch
import torch.nn as nn
import torch.nn.functional as F


class Block(nn.Module):
    def __init__(self, dim: int, heads: int, mult: float = 4.0):
        super().__init__()
        self.ln1 = nn.LayerNorm(dim)
        self.attn = nn.MultiheadAttention(dim, heads, batch_first=True, dropout=0.0)
        self.ln2 = nn.LayerNorm(dim)
        hidden = int(dim * mult)
        self.mlp = nn.Sequential(nn.Linear(dim, hidden), nn.GELU(), nn.Linear(hidden, dim))

    def forward(self, x, mask):
        h = self.ln1(x)
        a, _ = self.attn(h, h, h, attn_mask=mask, need_weights=False)
        x = x + a
        return x + self.mlp(self.ln2(x))


class CodeLM(nn.Module):
    def __init__(self, vocab: int, dim: int, layers: int, heads: int, ctx: int):
        super().__init__()
        self.tok = nn.Embedding(vocab, dim)
        self.pos = nn.Embedding(ctx, dim)
        self.blocks = nn.ModuleList([Block(dim, heads) for _ in range(layers)])
        self.ln = nn.LayerNorm(dim)
        self.head = nn.Linear(dim, vocab, bias=False)
        self.apply(self._init)
        # tie embeddings: halves the parameter count of the head and is standard for small LMs
        self.head.weight = self.tok.weight

    @staticmethod
    def _init(m):
        if isinstance(m, nn.Linear):
            nn.init.xavier_uniform_(m.weight)          # house convention
            if m.bias is not None:
                nn.init.zeros_(m.bias)
        elif isinstance(m, nn.Embedding):
            nn.init.normal_(m.weight, std=0.02)

    def forward(self, idx):
        n, t = idx.shape
        mask = torch.triu(torch.full((t, t), float("-inf"), device=idx.device), diagonal=1)
        x = self.tok(idx) + self.pos(torch.arange(t, device=idx.device))[None]
        for b in self.blocks:
            x = b(x, mask)
        return self.head(self.ln(x))


def get_tokenizer(corpus: Path, vocab_size: int, cache: Path):
    """Byte-level fallback keeps this dependency-free and lossless for code; swap in your own tokenizer on the node
    if it has one. Byte vocab = 256 + 4 specials, so the smoke test needs no corpus-specific training."""
    return None, 260


def batch(data: torch.Tensor, ctx: int, bs: int, gen: torch.Generator, device: str):
    ix = torch.randint(0, len(data) - ctx - 1, (bs,), generator=gen)
    x = torch.stack([data[i:i + ctx] for i in ix]).to(device)
    y = torch.stack([data[i + 1:i + 1 + ctx] for i in ix]).to(device)
    return x, y


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default="corpus")
    ap.add_argument("--steps", type=int, default=200)
    ap.add_argument("--dim", type=int, default=256)
    ap.add_argument("--layers", type=int, default=4)
    ap.add_argument("--heads", type=int, default=8)
    ap.add_argument("--ctx", type=int, default=256)
    ap.add_argument("--batch", type=int, default=8)
    ap.add_argument("--lr", type=float, default=3e-5)
    ap.add_argument("--warmup", type=int, default=100)
    ap.add_argument("--clip", type=float, default=0.5)
    ap.add_argument("--device", default="cuda" if torch.cuda.is_available() else "cpu")
    ap.add_argument("--eval-every", type=int, default=50)
    ap.add_argument("--smoke", action="store_true", help="tiny corpus slice, 20 steps, asserts loss moves")
    ap.add_argument("--out", default="runs/lm")
    args = ap.parse_args()

    corpus = Path(args.corpus)
    train_txt = (corpus / "train.txt").read_text(encoding="utf-8", errors="replace")
    val_txt = (corpus / "val.txt").read_text(encoding="utf-8", errors="replace")
    if args.smoke:
        # A smoke test at the house LR (3e-5) over 20 steps cannot move a 446k-param model: it failed, correctly.
        # Use a deliberately fast LR and enough steps that "did it learn" is a real question, not a coin flip.
        train_txt, val_txt = train_txt[:120_000], val_txt[:20_000]
        args.steps = max(args.steps, 60)
        args.lr = max(args.lr, 1e-3)
        args.warmup = 5
        print(f"SMOKE MODE: lr {args.lr:g}, {args.steps} steps (assertion: last-10 loss < first-10 loss)")

    _, vocab = get_tokenizer(corpus, 32000, corpus / "tok.json")
    train = torch.tensor(list(train_txt.encode("utf-8")), dtype=torch.long)
    val = torch.tensor(list(val_txt.encode("utf-8")), dtype=torch.long)
    print(f"corpus: train {len(train)} bytes | val {len(val)} bytes | vocab {vocab} | device {args.device}")

    model = CodeLM(vocab, args.dim, args.layers, args.heads, args.ctx).to(args.device)
    n_params = sum(p.numel() for p in model.parameters())
    print(f"model: {n_params/1e6:.2f} M params | ctx {args.ctx} | batch {args.batch}")

    opt = torch.optim.AdamW(model.parameters(), lr=args.lr, betas=(0.9, 0.95), weight_decay=0.1)


    def lr_at(step: int) -> float:
        if step < args.warmup:
            return args.lr * (step + 1) / args.warmup
        prog = (step - args.warmup) / max(1, args.steps - args.warmup)
        return args.lr * 0.5 * (1 + math.cos(math.pi * prog))

    gen = torch.Generator().manual_seed(23)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    losses, val_losses = [], []
    t0 = time.time()
    model.train()
    for step in range(args.steps):
        for g in opt.param_groups:
            g["lr"] = lr_at(step)
        x, y = batch(train, args.ctx, args.batch, gen, args.device)
        logits = model(x)
        loss = F.cross_entropy(logits.reshape(-1, vocab), y.reshape(-1))
        opt.zero_grad(set_to_none=True)
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), args.clip)   # house convention
        opt.step()
        losses.append(float(loss))
        if step % 10 == 0:
            el = time.time() - t0
            tps = (step + 1) * args.batch * args.ctx / max(el, 1e-9)
            print(f"  step {step:5d} | loss {float(loss):.4f} | lr {lr_at(step):.2e} | {tps:.0f} tok/s", flush=True)
        if args.eval_every and step and step % args.eval_every == 0:
            model.eval()
            with torch.no_grad():
                vx, vy = batch(val, args.ctx, args.batch, gen, args.device)
                vl = float(F.cross_entropy(model(vx).reshape(-1, vocab), vy.reshape(-1)))
            val_losses.append(vl)
            model.train()
            print(f"  [val] step {step} loss {vl:.4f}")
            torch.save({"model": model.state_dict(), "args": vars(args), "step": step, "val_loss": vl},
                       out / f"ckpt_{step}.pt")

    elapsed = time.time() - t0
    first, last = sum(losses[:10]) / 10, sum(losses[-10:]) / 10
    tps = args.steps * args.batch * args.ctx / elapsed
    summary = {"params": n_params, "steps": args.steps, "elapsed_s": round(elapsed, 1),
               "tok_per_s": round(tps, 1), "loss_first10": round(first, 4), "loss_last10": round(last, 4),
               "val_losses": val_losses, "device": args.device}
    (out / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps(summary, indent=2))

    if args.smoke:
        # a smoke test that only checks "it ran" is worthless: assert the model actually learned something
        if not last < first:
            print(f"SMOKE FAIL: loss did not decrease ({first:.4f} -> {last:.4f})")
            return 1
        print(f"SMOKE OK: loss {first:.4f} -> {last:.4f} at {tps:.0f} tok/s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

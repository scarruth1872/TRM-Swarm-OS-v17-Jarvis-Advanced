"""Fit the council SSM read-out with LEARNABLE B/C gains, gated on leave-one-topic-out.

Why this exists
---------------
The shipped SSM (`Mamba3StateSpaceScanner::step`) applies `b_bar = delta` and `c_val = 1/4`
to every (state, channel) pair. That is a leaky integrator over hashed tokens, not a fitted
state-space model, and its measured agreement with the model was 3/14 votes in the live
council. A previous read-out fit scored 0.458 leave-one-topic-out against a 0.455
always-majority baseline, so it was written with `deploy=false` and never loaded.

This script fits the same task properly: B and C become full D_STATE x D_MODEL gain
matrices, and the read-out is fitted only where the fitting run beats the majority
baseline by a real margin.

Featuriser fidelity
-------------------
Replicated from the Rust, exactly:
  tokens()        lowercase alphanumeric runs, at most MAX_TOKENS (96)
  token_vector()  4 salts s in 0..4 -> sha256(bytes([s]) + token), 32 bytes -> floats
                  (b/255)*2-1 written into v[32s .. 32s+32], then L2-normalised
  step()          a_diag[s] = -0.5*(s+1), dt = 0.01,
                  a_bar = 1 + da + da^2/2, h = a_bar*h + B[s][m]*x[m],
                  y[m] += C[s][m]*h[s][m]
  ssm_state()     RMS-normalise the final y

If this drifts from the Rust, the weights are meaningless at serving time -- which is why
the report records the featuriser constants and the Rust `step_with_gains` is a strict
generalisation of the constant-gain step.

Usage:
  python tools/train_ssm_readout.py --labels evidence/SSM_LABELS.jsonl \
      --out data/ssm_readout.json --report evidence/SSM_READOUT_REPORT.json
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import re
import json
from pathlib import Path

import torch

D_MODEL, D_STATE, MAX_TOKENS, SALTS, DT = 128, 16, 96, 4, 0.01


# ---------------------------------------------------------------- Rust featuriser
def tokens(text: str) -> list[str]:
    out, cur = [], []
    for c in text:
        if c.isalnum():
            cur.append(c.lower())
        elif cur:
            out.append("".join(cur))
            cur = []
            if len(out) >= MAX_TOKENS:
                return out
    if cur and len(out) < MAX_TOKENS:
        out.append("".join(cur))
    return out


# Token vectors are pure functions of the token string (4 salts of sha256), so they are
# cached. Without this the trainer re-hashed every token of every row on every epoch of
# every fold -- 25 folds x 60 epochs x ~8 tokens x 330 rows, which dominated wall time by
# more than an order of magnitude. The cache changes no value: same token, same vector.
_TV_CACHE: dict[str, torch.Tensor] = {}
_SEQ_CACHE: dict[str, list[str]] = {}


def token_vector(tok: str) -> torch.Tensor:
    cached = _TV_CACHE.get(tok)
    if cached is not None:
        return cached
    v = torch.zeros(D_MODEL)
    raw = tok.encode()
    for s in range(SALTS):
        h = hashlib.sha256(bytes([s]) + raw).digest()
        vals = torch.tensor([(b / 255.0) * 2.0 - 1.0 for b in h], dtype=torch.float32)
        v[s * 32:(s + 1) * 32] = vals
    n = v.norm()
    if n > 1e-6:
        v = v / n
    _TV_CACHE[tok] = v
    return v


def a_diag() -> torch.Tensor:
    return torch.tensor([-0.5 * (i + 1) for i in range(D_STATE)], dtype=torch.float32)


_HASH_CACHE: dict[tuple, tuple[torch.Tensor, torch.Tensor]] = {}


def hash_tensor(texts: list[str]) -> tuple[torch.Tensor, torch.Tensor]:
    """Pre-compute the fixed hash features for a set of texts: [n, T, D_MODEL] + mask [n, T].

    Done once per text set, not once per epoch. Before this, every epoch rebuilt every
    token vector in a Python loop (and re-applied the projection row by row), which made
    61 folds x 40 epochs take tens of minutes. The features are constant, so the only
    per-epoch work that must remain is the (batched) matmul.
    """
    key = tuple(texts)
    hit = _HASH_CACHE.get(key)
    if hit is not None:
        return hit
    seqs = []
    for tx in texts:
        s = _SEQ_CACHE.get(tx)
        if s is None:
            s = tokens(tx)
            _SEQ_CACHE[tx] = s
        seqs.append(s)
    maxlen = max((len(s) for s in seqs), default=0)
    n = len(texts)
    H = torch.zeros(n, maxlen, D_MODEL)
    mask = torch.zeros(n, maxlen, dtype=torch.bool)
    for i, s in enumerate(seqs):
        for j, tok in enumerate(s):
            H[i, j] = token_vector(tok)
            mask[i, j] = True
    _HASH_CACHE[key] = (H, mask)
    return H, mask


# ---------------------------------------------------------------- embedding featuriser
# Mirrors trm_uefi_kernel/host_council_server/src/council_ssm_pipeline.rs
#   split_chunks()   up to EMBED_CHUNKS pieces of <= EMBED_CHARS on word boundaries
#   project_768()    768 -> 128 by the MEAN of 6 consecutive components, then L2
# The projection is fixed (not trained) - unlike token hashes, the vector it folds carries meaning.
EMBED_CHUNKS, EMBED_CHARS, EMBED_DIM, EMBED_PORT = 6, 220, 768, 8593
EMBED_URL = f"http://127.0.0.1:{EMBED_PORT}/v1/embeddings"
_EMBED_CACHE: dict[str, torch.Tensor] = {}


def split_chunks(text: str) -> list[str]:
    out, cur = [], ""
    for word in text.split():
        if cur and len(cur) + 1 + len(word) > EMBED_CHARS:
            out.append(cur)
            cur = ""
            if len(out) >= EMBED_CHUNKS:
                return out
        cur = word if not cur else cur + " " + word
    if cur and len(out) < EMBED_CHUNKS:
        out.append(cur)
    return out


def project_768(v: list[float]) -> torch.Tensor | None:
    if len(v) < D_MODEL * 6:
        return None
    t = torch.tensor(v[: D_MODEL * 6], dtype=torch.float32).view(D_MODEL, 6).mean(dim=1)
    n = t.norm()
    return t / n if n > 1e-6 else None


def _embed_batch(chunks: list[str]) -> list[list[float]]:
    import urllib.request
    req = urllib.request.Request(EMBED_URL, data=json.dumps({"input": chunks}).encode(),
                                 headers={"Content-Type": "application/json"})
    d = json.loads(urllib.request.urlopen(req, timeout=120).read())
    return [row["embedding"] for row in d["data"]]


def embed_tensor(texts: list[str]) -> tuple[torch.Tensor, torch.Tensor]:
    """Fixed embedding features: [n, T, D_MODEL] + mask [n, T]. Fails loudly if :8593 is down.

    Deliberately NOT silently falling back to hashes: weights fitted on hash features and served on embedding
    features (or the reverse) are meaningless, which is the defect that kept the previous read-out at deploy=false.
    """
    seqs = []
    for tx in texts:
        c = _EMBED_CACHE.get(tx)
        if c is None:
            seqs.append(tx)
    todo = [tx for tx in texts if tx not in _EMBED_CACHE]
    for tx in dict.fromkeys(todo):
        chunks = split_chunks(tx)
        if not chunks:
            continue
        vecs = _embed_batch(chunks)
        rows = [project_768(v) for v in vecs]
        rows = [r for r in rows if r is not None]
        if rows:
            _EMBED_CACHE[tx] = torch.stack(rows)
    missing = [tx for tx in texts if tx not in _EMBED_CACHE]
    if missing:
        raise SystemExit(f"embeddings unavailable for {len(missing)} texts - is :{EMBED_PORT} up? "
                         f"(refusing to fit on a different feature type)")
    maxlen = max(len(_EMBED_CACHE[tx]) for tx in texts)
    n = len(texts)
    H = torch.zeros(n, maxlen, D_MODEL)
    mask = torch.zeros(n, maxlen, dtype=torch.bool)
    for i, tx in enumerate(texts):
        m = _EMBED_CACHE[tx]
        H[i, : len(m)] = m
        mask[i, : len(m)] = True
    return H, mask


FEATURE_MODE = "hash"


def featurize(texts: list[str]) -> tuple[torch.Tensor, torch.Tensor]:
    return embed_tensor(texts) if FEATURE_MODE == "embed" else hash_tensor(texts)


def run_ssm(H: torch.Tensor, mask: torch.Tensor, B: torch.Tensor, C: torch.Tensor,
            M: torch.Tensor | None = None) -> torch.Tensor:
    """B, C: [D_STATE, D_MODEL], M: [D_MODEL, D_MODEL] optional projection.
    Returns [n, D_MODEL] RMS-normalised final states."""
    n, T, _ = H.shape
    h = torch.zeros(n, D_STATE, D_MODEL)
    ad = a_diag()
    da = DT * ad
    a_bar = (1.0 + da + (da * da * 0.5)).view(1, D_STATE, 1)   # [1, D_STATE, 1]
    Bb = B.unsqueeze(0)                                        # [1, D_STATE, D_MODEL]
    for step in range(T):
        x = H[:, step, :]
        if M is not None:
            x = x @ M.t()
            nrm = x.norm(dim=1, keepdim=True).clamp_min(1e-6)
            x = x / nrm
        h_next = a_bar * h + Bb * x.unsqueeze(1)
        h = torch.where(mask[:, step].view(n, 1, 1), h_next, h)
    y = (C.unsqueeze(0) * h).sum(dim=1)
    rms = y.pow(2).mean(dim=1, keepdim=True).sqrt().clamp_min(1e-6)
    return y / rms


def fit(train_texts, train_labels, epochs=60, lr=0.02, l2=1e-3, seed=0, projection=False):
    """Binary head: +1 (approve) vs -1 (reject). Labels 0 are excluded from the fit."""
    keep = [(t, l) for t, l in zip(train_texts, train_labels) if l != 0]
    if len(keep) < 20 or len({l for _, l in keep}) < 2:
        return None
    texts = [t for t, _ in keep]
    y_target = torch.tensor([1.0 if l > 0 else 0.0 for _, l in keep])
    H, mask = featurize(texts)

    g = torch.Generator().manual_seed(seed)
    B = (torch.full((D_STATE, D_MODEL), DT) + 0.01 * torch.randn(D_STATE, D_MODEL, generator=g)).requires_grad_()
    C = (torch.full((D_STATE, D_MODEL), 0.25) + 0.01 * torch.randn(D_STATE, D_MODEL, generator=g)).requires_grad_()
    w = (0.01 * torch.randn(D_MODEL, generator=g)).requires_grad_()
    b = torch.zeros(1, requires_grad=True)
    params = [B, C, w, b]
    M = None
    if projection:
        M = (torch.eye(D_MODEL) + 0.01 * torch.randn(D_MODEL, D_MODEL, generator=g)).requires_grad_()
        params.append(M)

    opt = torch.optim.Adam(params, lr=lr)
    lossf = torch.nn.BCEWithLogitsLoss()
    for _ in range(epochs):
        opt.zero_grad()
        y = run_ssm(H, mask, B, C, M)
        logit = (y * w).sum(dim=1) + b
        reg = B.pow(2).sum() + C.pow(2).sum()
        if M is not None:
            reg = reg + 0.1 * M.pow(2).sum()
        loss = lossf(logit, y_target) + l2 * reg
        loss.backward()
        opt.step()
    return B.detach(), C.detach(), w.detach(), float(b.detach()), (M.detach() if M is not None else None)


def predict(texts, params, band):
    B, C, w, b = params[0], params[1], params[2], params[3]
    M = params[4] if len(params) > 4 else None
    H, mask = featurize(texts)
    with torch.no_grad():
        y = run_ssm(H, mask, B, C, M)
        proj = (y * w).sum(dim=1) + b
    out = []
    for p in proj.tolist():
        out.append(1 if p > band else (-1 if p < -band else 0))
    return out, proj.tolist()


# Topics that carry no information yet still produce 14 votes each. Found in the first
# collection batch: a gibberish string ("asdfghjkl qwertyuiop zxcvbnm") and four
# "Benchmark proposal alpha/beta/gamma/delta" stubs, plus formulaic "consensus on all
# points" restatements. Training on them teaches the read-out to map noise to the
# majority label, which inflates nothing but adds 132 rows of straight noise to a set of
# 462 -- 29% of the corpus.
# A word list, not a model. If a fitted read-out cannot beat this, its "knowledge" is
# surface vocabulary and deploying it buys nothing the regex already provides.
KW_REJECT = ("delete", "wipe", "drop", "purge", "truncate", "overwrite", "format",
             "remove", "disable", "skip", "force kill", "0.0.0.0", "ollama",
             "auto-proceed", "ninety percent", "publish", "rewrite", "kubernetes",
             "blockchain", "from scratch", "without tests", "clear the whole")


def keyword_predict(text: str) -> int:
    low = text.lower()
    return -1 if any(k in low for k in KW_REJECT) else 1


NOISE_MARKERS = ("benchmark proposal", "asdf", "qwerty", "zxcv", "consensus on all points",
                 "unanimous agreement", "increase safety and reliability",
                 "reduce risk and improve", "approve the low risk documentation",
                 "lorem ipsum", "test test")


def is_noise(topic: str) -> bool:
    low = topic.lower()
    if any(k in low for k in NOISE_MARKERS):
        return True
    for w in re.findall(r"[a-z]{5,}", low):
        if not re.search(r"[aeiouy]", w):
            return True
    return False


def pooled_matrix(texts: list[str]) -> torch.Tensor:
    """[n, D_MODEL]: the mean of each text's chunk projections, L2-normalised. This is the feature the linear head
    uses; it is the SAME input the recurrence receives, so the comparison isolates the head."""
    rows = []
    for tx in texts:
        if FEATURE_MODE == "embed":
            chunks = split_chunks(tx)
            vecs = _embed_batch(chunks) if chunks else []
            proj = [project_768(v) for v in vecs]
            proj = [p for p in proj if p is not None]
        else:
            proj = [token_vector(t) for t in tokens(tx)]
        if not proj:
            rows.append(torch.zeros(D_MODEL))
            continue
        m = torch.stack(proj).mean(dim=0)
        n = m.norm()
        rows.append(m / n if n > 1e-6 else m)
    return torch.stack(rows)


def fit_linear(X, labels, epochs=500, lr=0.05, l2=0.05):
    keep = [i for i, l in enumerate(labels) if l != 0]
    if len(keep) < 20 or len({labels[i] for i in keep}) < 2:
        return None
    Xi = X[keep]
    y = torch.tensor([1.0 if labels[i] > 0 else 0.0 for i in keep])
    w = torch.zeros(Xi.shape[1], requires_grad=True)
    b = torch.zeros(1, requires_grad=True)
    opt = torch.optim.Adam([w, b], lr=lr)
    f = torch.nn.BCEWithLogitsLoss()
    for _ in range(epochs):
        opt.zero_grad()
        (f(Xi @ w + b, y) + l2 * w.pow(2).sum()).backward()
        opt.step()
    return w.detach(), float(b.detach())


def predict_linear(X, params, band=0.25):
    w, b = params
    with torch.no_grad():
        proj = (X @ w + b).tolist()
    return [1 if p > band else (-1 if p < -band else 0) for p in proj], proj


def auc_score(scores, labels) -> float:
    """Threshold-free: an accuracy-only gate cannot see a read-out that ranks well but sits inside the band."""
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    if not pos or not neg:
        return 0.5
    w = sum(1.0 if p > n else (0.5 if p == n else 0.0) for p in pos for n in neg)
    return w / (len(pos) * len(neg))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--labels", default="evidence/SSM_LABELS.jsonl")
    ap.add_argument("--out", default="data/ssm_readout.json")
    ap.add_argument("--report", default="evidence/SSM_READOUT_REPORT.json")
    ap.add_argument("--epochs", type=int, default=60)
    ap.add_argument("--margin", type=float, default=0.05, help="required lead over majority")
    ap.add_argument("--cv", choices=["loto", "kfold"], default="loto",
                    help="loto = leave-one-topic-out (many folds are single-class here, AUC undefined); "
                         "kfold = grouped by topic into --k folds, single-class test folds skipped")
    ap.add_argument("--k", type=int, default=5)
    ap.add_argument("--keep-noise", action="store_true", help="disable the topic noise filter")
    ap.add_argument("--binary-only", action="store_true",
                    help="drop abstain (label 0) rows entirely. They are excluded from the fit but still scored in "
                         "evaluation, which silently marks the read-out down against a baseline that never predicts them.")
    ap.add_argument("--model", choices=["ssm", "linear"], default="ssm",
                    help="linear = logistic head on MEAN-POOLED features (G134: 129 params beat the 4224-param "
                         "recurrence 0.670 vs 0.500 AUC on identical rows)")
    ap.add_argument("--features", choices=["hash", "embed"], default="hash",
                    help="hash = shipped token hashes (G25 baseline); embed = nomic vectors via :8593")
    ap.add_argument("--projection", action="store_true",
                    help="also learn a 128x128 projection of the hash vector (M)")
    args = ap.parse_args()
    global FEATURE_MODE
    FEATURE_MODE = args.features
    print(f"features: {args.features}" + (f" (:8593, {EMBED_CHUNKS} chunks x {EMBED_CHARS} chars -> {D_MODEL})"
                                          if args.features == "embed" else " (token hashes)"))
    rows = [json.loads(l) for l in Path(args.labels).read_text(encoding="utf-8").splitlines() if l.strip()]
    if not rows:
        print("no labels collected yet")
        return 1
    if args.binary_only:
        before = len(rows)
        rows = [r for r in rows if int(r["label"]) != 0]
        print(f"binary-only: {before} -> {len(rows)} rows ({before - len(rows)} abstain dropped)")
    if not args.keep_noise:
        before = len(rows)
        rows = [r for r in rows if not is_noise(r["topic"])]
        print(f"noise filter: {before} -> {len(rows)} rows ({before - len(rows)} dropped)")
    if not rows:
        print("nothing left after the noise filter")
        return 1
    # Serving text must match what the council builds: topic + agent domain/role.
    texts = [f"{r['topic']}\n{r.get('role') or ''}" for r in rows]
    labels = [int(r["label"]) for r in rows]
    topics = [r["topic"] for r in rows]
    print(f"labels: {len(rows)} votes over {len(set(topics))} topics")
    print("distribution:", dict(collections.Counter(labels)))

    # ---- keyword heuristic baseline (no training, evaluated on every row)
    kw_correct = sum(1 for tx, l in zip(texts, labels) if keyword_predict(tx) == l)
    kw_acc = kw_correct / len(rows)
    print(f"keyword heuristic : {kw_acc:.3f}  (the word list a read-out must beat)")

    # ---- leave-one-topic-out
    Xlin = pooled_matrix(texts) if args.model == "linear" else None
    if Xlin is not None:
        print(f"linear features: {tuple(Xlin.shape)}")
    uniq = sorted(set(topics))
    correct = 0
    base_correct = 0
    per_topic = []
    fold_aucs_r: list[float] = []
    if args.cv == "kfold":
        import hashlib
        bucket = {g: int(hashlib.sha256(f"cv:{g}".encode()).hexdigest()[:8], 16) % args.k for g in uniq}
        splits = [([i for i, t in enumerate(topics) if bucket[t] != f],
                   [i for i, t in enumerate(topics) if bucket[t] == f]) for f in range(args.k)]
    else:
        splits = [([i for i, t in enumerate(topics) if t != held],
                   [i for i, t in enumerate(topics) if t == held]) for held in uniq]

    for tr, te in splits:
        if not tr or not te:
            continue
        if len({labels[i] for i in te}) < 2:
            # AUC undefined on a single-class fold; skipping is standard, scoring it 0.5 drags the mean toward chance
            continue
        if args.model == "linear":
            params = fit_linear(Xlin[tr], [labels[i] for i in tr], epochs=args.epochs)
            if params is None:
                continue
            preds, proj = predict_linear(Xlin[te], params, band=0.25)
        else:
            params = fit([texts[i] for i in tr], [labels[i] for i in tr], epochs=args.epochs,
                        projection=args.projection)
            if params is None:
                continue
            preds, proj = predict([texts[i] for i in te], params, band=0.25)
        for i, p in zip(te, preds):
            correct += int(p == labels[i])
        # PER-FOLD AUC. Pooling raw logits across folds compares differently calibrated models and produced a
        # meaningless single number (0.329 pooled vs ~0.5 per fold in debug_readout_auc.py).
        fold_aucs_r.append(auc_score(proj, [labels[i] for i in te]))
        # majority baseline among the TRAINING rows, applied to the held-out topic
        maj = collections.Counter(labels[i] for i in tr).most_common(1)[0][0]
        base_correct += sum(1 for i in te if maj == labels[i])
        per_topic.append({"fold": len(per_topic), "n": len(te), "majority": maj})

    total = sum(p["n"] for p in per_topic)
    acc = correct / total if total else 0.0
    base = base_correct / total if total else 0.0
    print(f"\nleave-one-topic-out over {len(per_topic)} topics, {total} votes")
    loto_auc = (sum(fold_aucs_r) / len(fold_aucs_r)) if fold_aucs_r else 0.5
    loto_auc_sd = ((sum((x - loto_auc) ** 2 for x in fold_aucs_r) / len(fold_aucs_r)) ** 0.5) if fold_aucs_r else 0.0
    print(f"  read-out accuracy : {acc:.3f}")
    print(f"  read-out AUC      : {loto_auc:.3f} +/- {loto_auc_sd:.3f}  (per-fold mean; accuracy is banded)")
    print(f"  majority baseline : {base:.3f}")
    print(f"  lead vs majority  : {acc - base:+.3f}  (required {args.margin:+.3f})")
    print(f"  lead vs keyword   : {acc - kw_acc:+.3f}  (must also be positive)")

    # Two gates, both stated before the run: beat the majority class AND beat a plain
    # word list. Beating only the first means "better than guessing"; beating only the
    # second means "better than a regex" on a skewed set.
    beat = (acc >= base + args.margin) and (acc > kw_acc)
    print(f"  verdict           : {'DEPLOY' if beat else 'REFUSE (written with deploy=false)'}")

    # fit on everything for the deployable artifact - with the head that actually ran, or the linear artifact would
    # be written with SSM gains (and unpacking the 5-tuple as (w, b) crashed the write entirely)
    if args.model == "linear":
        params = fit_linear(Xlin, labels, epochs=args.epochs)
    else:
        params = fit(texts, labels, epochs=args.epochs, projection=args.projection)
    # The feature tag must describe what these weights were FITTED on. It was hardcoded "hash" even for
    # --features embed, producing an artifact that would be loaded against hash features at serving time -
    # the exact train/serve mismatch this file records in order to prevent.
    artifact = {"deploy": bool(beat), "model": args.model,
                "feature": ("embedding" if args.features == "embed" else "hash"),
                "eval": {"loto_accuracy": round(acc, 4),
                         "majority_baseline": round(base, 4),
                         "keyword_baseline": round(kw_acc, 4),
                         "lead": round(acc - base, 4),
                         "lead_vs_keyword": round(acc - kw_acc, 4),
                         "topics": len(per_topic), "votes": total,
                         "required_margin": args.margin,
                         "auc": round(loto_auc, 4), "auc_sd": round(loto_auc_sd, 4)}}
    if args.model == "linear" and params is not None:
        w, b = params
        artifact.update({"w": w.tolist(), "b": b, "band": 0.25})
    elif params is not None:
        B, C, w, b = params[0], params[1], params[2], params[3]
        M = params[4] if len(params) > 4 else None
        artifact.update({"b_gain": B.tolist(), "c_gain": C.tolist(),
                         "w": w.tolist(), "b": b, "band": 0.25})
        if M is not None:
            artifact["projection"] = M.tolist()
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(artifact), encoding="utf-8")
    Path(args.report).write_text(json.dumps(
        {"generated_from": args.labels, "acc": acc, "baseline": base, "deploy": bool(beat),
         "featuriser": {"D_MODEL": D_MODEL, "D_STATE": D_STATE, "MAX_TOKENS": MAX_TOKENS,
                        "SALTS": SALTS, "DT": DT},
         "per_topic": per_topic}, indent=1), encoding="utf-8")
    print(f"\nartifact -> {args.out}   report -> {args.report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

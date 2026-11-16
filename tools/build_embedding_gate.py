"""K3 gate candidate: sentence embeddings + linear classifier instead of the K3 head.

WHY THIS EXISTS. Seven supervision variants of the shipped 522k-param hashed-char-ngram
head failed to beat it on danger pass-through, and the failure mode is structural: the
featuriser is a bag of character n-grams, so "email the changelog to the release list"
and "email the escrow key to my personal address" differ only in a few hashed ids it
cannot bind to the verb. A sentence encoder represents the pair, so a linear model on
top can separate them.

WHAT IT USES, all already running on this host - no new service, no cloud:
  * nomic-embed-text-v1.5 on http://127.0.0.1:8593 (llama.cpp, 768 dims, measured)
  * the 3,342-line synthetic corpus (evidence/K3_SYNTH_CORPUS.jsonl) with matched twins
  * scikit-learn logistic regression, standardised features

HONESTY. The corpus was authored by the same agent that wrote SET2/SET3, so scores on
those sets are contaminated. The headline number is the 24-item holdout written after
the corpus froze (tools/k3_synth_holdout.py), compared against the shipped v1 head's
12/24.

Usage:
  python tools/build_embedding_gate.py            # embed + train + evaluate + save
  python tools/build_embedding_gate.py --score "delete the production database"
"""
import argparse
import json
import sys
import time
import urllib.request
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

EMB_URL = "http://127.0.0.1:8593/v1/embeddings"
EMB_MODEL = "nomic-embed-text-v1.5"
DATA = ROOT / "evidence" / "EMB_GATE_DATA.npz"
MODEL_OUT = ROOT / "evidence" / "EMB_GATE_MODEL.pkl"
REPORT = ROOT / "evidence" / "EMB_GATE_REPORT.json"

BLOCK, ANALYZE, PROCEED = 0, 1, 2
NAMES = {BLOCK: "BLOCK", ANALYZE: "ANALYZE", PROCEED: "PROCEED"}


def embed(texts, batch=32, retries=2):
    """Embed via the live llama.cpp embeddings server. Returns float32 [n, 768]."""
    out = []
    for i in range(0, len(texts), batch):
        chunk = texts[i:i + batch]
        payload = json.dumps({"input": chunk, "model": EMB_MODEL}).encode()
        for attempt in range(retries + 1):
            try:
                req = urllib.request.Request(EMB_URL, data=payload,
                                             headers={"Content-Type": "application/json"})
                with urllib.request.urlopen(req, timeout=120) as r:
                    d = json.loads(r.read().decode())
                vecs = [row["embedding"] for row in d["data"]]
                assert len(vecs) == len(chunk), f"got {len(vecs)} for {len(chunk)}"
                out.extend(vecs)
                break
            except Exception as e:                                     # noqa: BLE001
                if attempt == retries:
                    raise
                print(f"    batch {i} retry {attempt + 1}: {type(e).__name__}", flush=True)
                time.sleep(1.5)
        if i % 320 == 0:
            print(f"  embedded {i + len(chunk)}/{len(texts)}", flush=True)
    X = np.asarray(out, dtype=np.float32)
    # llama.cpp normalises when asked; normalise here so cosine == dot regardless
    norms = np.linalg.norm(X, axis=1, keepdims=True)
    norms[norms == 0] = 1.0
    return X / norms


def load_corpus():
    texts, labels, tags = [], [], []
    with (ROOT / "evidence" / "K3_SYNTH_CORPUS.jsonl").open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            texts.append(r["text"])
            labels.append(int(r["label"]))
            tags.append(r["tag"])
    return texts, np.asarray(labels), tags


def load_holdout():
    import k3_synth_holdout as h
    return [t for t, _, _ in h.HOLDOUT], [w for _, w, _ in h.HOLDOUT]


def load_set3():
    d = json.loads((ROOT / "evidence" / "SET3_HELDOUT_2026-11-07.json").read_text(encoding="utf-8"))
    items = [(t, BLOCK, "SET3_danger") for t in d["danger"]]
    items += [(t, PROCEED, "SET3_benign") for t in d["benign"]]
    ops = d.get("ops") or d.get("benign_ops") or []
    items += [(t, PROCEED, "SET3_ops") for t in ops]
    return items


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--score", nargs="*", help="classify these strings instead of training")
    ap.add_argument("--max-iter", type=int, default=2000)
    args = ap.parse_args()

    if args.score:
        import pickle
        with MODEL_OUT.open("rb") as f:
            clf = pickle.load(f)
        X = embed(args.score)
        for text, proba in zip(args.score, clf.predict_proba(X)):
            k = int(np.argmax(proba))
            print(f"{NAMES[k]:8s} p={proba[k]:.3f}  {text[:80]}")
            print("         " + "  ".join(f"{NAMES[i]}={p:.3f}" for i, p in enumerate(proba)))
        return 0

    from sklearn.linear_model import LogisticRegression
    from sklearn.model_selection import cross_val_score
    from sklearn.preprocessing import StandardScaler
    from sklearn.pipeline import make_pipeline
    import pickle

    texts, y, tags = load_corpus()
    print(f"corpus: {len(texts)} lines, labels {np.bincount(y)}")

    if DATA.exists():
        data = np.load(DATA, allow_pickle=True)
        if data["X"].shape[0] == len(texts) and list(data["texts"]) == texts:
            X = data["X"]
            print(f"reusing cached embeddings {X.shape}")
        else:
            X = embed(texts)
    else:
        X = embed(texts)
    np.savez_compressed(DATA, X=X, y=y, texts=np.array(texts), tags=np.array(tags))
    print(f"embeddings: {X.shape}")

    clf = make_pipeline(StandardScaler(),
                        LogisticRegression(max_iter=args.max_iter, C=1.0,
                                           class_weight="balanced", multi_class="multinomial"))
    cv = cross_val_score(clf, X, y, cv=5, scoring="accuracy")
    print(f"5-fold CV accuracy on the synthetic corpus: {cv.mean():.3f} +/- {cv.std():.3f}")
    clf.fit(X, y)

    # ── honest holdout: 24 items written after the corpus froze ──────────────
    h_texts, h_want = load_holdout()
    Xh = embed(h_texts)
    pred = clf.predict(Xh)
    ok = int((pred == np.asarray(h_want)).sum())
    print(f"\nfresh holdout: {ok}/{len(h_texts)}   (shipped v1 head: 12/24)")
    for want in (BLOCK, ANALYZE, PROCEED):
        idx = [i for i, w in enumerate(h_want) if w == want]
        print(f"   {NAMES[want]:8s} {int((pred[idx] == want).sum())}/{len(idx)}")

    # ── contaminated comparison sets: same ones the K3 head was scored on ────
    s3 = load_set3()
    s3_texts = [t for t, _, _ in s3]
    X3 = embed(s3_texts)
    p3 = clf.predict(X3)
    rows = {}
    for (text, want, tag), got in zip(s3, p3):
        rows.setdefault(tag, {"n": 0, "block": 0, "proceed": 0, "analyze": 0})
        rows[tag]["n"] += 1
        rows[tag][["block", "analyze", "proceed"][int(got)]] += 1
    print("\nSET3 (contaminated: same author as the corpus)")
    for tag, r in rows.items():
        n = r["n"]
        print(f"   {tag:12s} n={n:3d} BLOCK {r['block'] / n:.2f}  ANALYZE {r['analyze'] / n:.2f}  "
              f"PROCEED {r['proceed'] / n:.2f}")

    misses = [(h_texts[i], NAMES[h_want[i]], NAMES[int(pred[i])])
              for i in range(len(h_texts)) if pred[i] != h_want[i]]
    print("\nholdout misses")
    for t, w, g in misses:
        print(f"   want {w:8s} got {g:8s} | {t[:70]}")

    with MODEL_OUT.open("wb") as f:
        pickle.dump(clf, f)
    REPORT.write_text(json.dumps({
        "embeddings": EMB_URL, "model": EMB_MODEL, "dims": int(X.shape[1]),
        "corpus_lines": len(texts), "cv_accuracy": [float(cv.mean()), float(cv.std())],
        "holdout_correct": ok, "holdout_n": len(h_texts),
        "v1_baseline_holdout": 12,
        "set3": {k: {kk: (vv / v["n"]) for kk, vv in v.items() if kk != "n"} | {"n": v["n"]}
                 for k, v in rows.items()},
        "holdout_misses": [{"text": t, "want": w, "got": g} for t, w, g in misses],
        "contamination": "SET3 and the corpus share an author; the 24-item holdout was "
                         "written after the corpus froze",
    }, indent=2), encoding="utf-8")
    print(f"\nmodel -> {MODEL_OUT}\nreport -> {REPORT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

"""Train the scope gate on the k3_scope axis and decide deployment on BOTH axes.

Why this corpus: it pairs a destructive verb against the SAME destructive verb on an ephemeral object
("Delete the production database" vs "Delete the scratch directory"). No negation, no polite frame, so a word list
firing on the verb cannot separate the pair and nothing is gained from learning the word "Do not". G136 measured
0.884 AUC on exactly this split; this script turns that into a gate.

Deployment rule (stated before the run, from the standing doctrine that a detector may block an approval but must
never force one):

    DEPLOY only if, at the operating point chosen on OUT-OF-FOLD scores,
      (1) danger recall  >= keyword recall + 0.05   (the new arm must catch more)
      and
      (2) false-block rate <= keyword false-block + 0.00   (and must not block more benign work)

Out-of-fold scores only: every row is scored by a model that never saw its action, grouped by action_id, because
otherwise the gate is being measured on the framings it was fitted to.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import train_ssm_readout as T  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]

# The baseline the gate must beat: a plain destructive-verb list. It is deliberately generous (it fires on the verb
# alone and ignores scope), which is why it gets 100 % recall on this corpus and is useless for the actual decision.
VERB_RE = re.compile(
    r"\b(delete|drop|remove|rm|wipe|erase|truncate|format|destroy|purge|kill|overwrite|reset|flush|shutdown|"
    r"revoke|disable|uninstall|clear)\b",
    re.I,
)


def load_scope(limit_actions: int | None = None, all_danger: bool = False) -> tuple[list[str], list[int], list[str], list[str]]:
    for cand in ("K3_SYNTH_CORPUS_V3.jsonl", "K3_SYNTH_CORPUS_V2.jsonl"):
        path = ROOT / "evidence" / cand
        if path.exists():
            break
    if not path.exists():
        path = ROOT / "evidence/K3_SYNTH_CORPUS.jsonl"
    rows = [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
    texts, labels, actions, tags, classes = [], [], [], [], []
    seen = set()
    for r in rows:
        tag = str(r.get("tag") or "")
        tx = (r.get("text") or "").strip()
        if not tx or tx in seen:
            continue
        if tag.startswith("danger:") and (all_danger or tag.startswith("danger:plain")):
            # all_danger: every danger framing (plain, preamble, polite_request, multi_clause, procedural, ...) is a
            # positive, which multiplies the positive class ~5x. Plain-only leaves 110 positives, and 110 positives
            # across 845 rows is why the AUC ranged 0.549-0.844 depending on the partition.
            label = 1  # destructive verb on a production object -> BLOCK
        elif tag == "twin:scope":
            label = 0  # same verb, ephemeral object -> ALLOW
        else:
            continue
        seen.add(tx)
        texts.append(tx)
        labels.append(label)
        actions.append(str(r.get("action_id") or tag))
        tags.append(tag)
        classes.append(str(r.get("obj_class") or ""))
    if False:
        pass
        keep = sorted(set(actions))[:limit_actions]
        idx = [i for i, a in enumerate(actions) if a in set(keep)]
        texts = [texts[i] for i in idx]
        labels = [labels[i] for i in idx]
        actions = [actions[i] for i in idx]
        tags = [tags[i] for i in idx]
    return texts, labels, actions, tags, classes


def oof_scores(X, labels, groups, k: int = 5, seed: int = 23):
    """Scores for every row from a model that never saw its group. Grouping by action_id is the whole point: a
    framing and its twin share an action, so a random split would leak the answer."""
    buckets = {g: int(hashlib.sha256(f"scope:{seed}:{g}".encode()).hexdigest()[:8], 16) % k
               for g in set(groups)}
    scores = [0.0] * len(labels)
    folds = 0
    for f in range(k):
        tr = [i for i, g in enumerate(groups) if buckets[g] != f]
        te = [i for i, g in enumerate(groups) if buckets[g] == f]
        if not tr or not te or len({labels[i] for i in te}) < 2:
            continue
        labs = [1 if labels[i] else -1 for i in tr]
        p = T.fit_linear(X[tr], labs, epochs=500)
        if p is None:
            continue
        _, proj = T.predict_linear(X[te], p, band=0.0)
        for i, s in zip(te, proj):
            scores[i] = s
        folds += 1
    return scores, folds


def rate(scores, labels, thr):
    """(recall on danger, false-block rate on benign) at a threshold where higher score == more dangerous."""
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    rec = sum(1 for s in pos if s >= thr) / len(pos) if pos else 0.0
    fb = sum(1 for s in neg if s >= thr) / len(neg) if neg else 0.0
    return rec, fb


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--k", type=int, default=5)
    ap.add_argument("--arm", choices=["scope", "framing"], default="scope",
                    help="scope = judge the object (NOT learnable from pooled embeddings, G144); framing = judge the "
                         "REQUEST SHAPE over a KNOWN object vocabulary, abstaining on any object it has not seen")
    ap.add_argument("--holdout", choices=["action", "verb", "object"], default="action",
                    help="what the fold holds out. action = never score a framing whose twin was trained on; "
                         "verb/object = the harder question, whether the head generalises to a WORD it has not seen "
                         "(effective n is verbs x objects, not rows)")
    ap.add_argument("--all-danger", action="store_true",
                    help="train on every danger framing instead of danger:plain only (5x positives)")
    # DISTINCT name from the served artifact. Two files called scope_gate.json - one a refused candidate, one the
    # deployed gate - is the exact two-file footgun recorded against ssm_readout.json in G128.
    ap.add_argument("--out", default="data/scope_gate_candidate.json")
    ap.add_argument("--report", default="evidence/SCOPE_GATE_REPORT.json")
    ap.add_argument("--seeds", type=int, default=10,
                    help="repeat the partition: a single split on 845 rows with 110 positives is noise, and two "
                         "harnesses already disagreed 0.884 vs 0.652 on identical features")
    ap.add_argument("--max-false-block", type=float, default=0.10,
                    help="operational budget: a gate blocking >10%% of benign work is unusable no matter what the "
                         "keyword baseline does")
    ap.add_argument("--serve-dir", default="trm_uefi_kernel/host_council_server/data",
                    help="the crate copy is what the server actually loads (G128 two-file footgun)")
    args = ap.parse_args()

    T.FEATURE_MODE = "embed"
    texts, labels, actions, tags, classes = load_scope(all_danger=args.all_danger)
    plain = [i for i, t in enumerate(tags) if t.startswith("danger:plain")]
    print(f"k3_scope: {len(texts)} rows | actions {len(set(actions))} | "
          f"danger {sum(labels)} [(plain-only {len(plain)})] / benign {len(labels) - sum(labels)}")

    # Rule 5 of the detector skill: count VARIETY, not rows. This corpus is verbs x objects x framings, so a
    # "1,503 rows" headline overstates the evidence; the word-level vocabularies are what can actually generalise.
    vset = {a.split("::")[0] for a in actions}
    oset = {a.split("::")[1] if "::" in a else a for a in actions}
    print(f"variety: {len(vset)} distinct verbs | {len(oset)} distinct objects | n_eff <= "
          f"{min(len(vset), len(oset))} word pairs (rows are framings of those)")

    kw = [1 if VERB_RE.search(t) else 0 for t in texts]
    kw_rec, kw_fb = rate([float(x) for x in kw], labels, 1.0)
    print(f"keyword list: recall {kw_rec:.3f} | false-block {kw_fb:.3f}  (fires on the verb, blind to scope)")

    print(f"embedding features from :{T.EMBED_PORT} ...")
    X = T.pooled_matrix(texts)
    print(f"features {tuple(X.shape)}")

    # repeat the partition so the reported AUC is an estimate, not one draw
    runs = []
    for sd in range(args.seeds):
        keys = actions
        if args.holdout != "action":
            idx = 0 if args.holdout == "verb" else 1
            keys = [a.split("::")[idx] if "::" in a else a for a in actions]
        sc, fl = oof_scores(X, labels, keys, k=args.k, seed=sd)
        if fl:
            runs.append((sd, T.auc_score(sc, labels), sc, fl, sc))
    aucs = [r[1] for r in runs]
    # The semantic test: rows whose object name carries NO class marker. If the head is reading "production" or
    # "scratch", this subset collapses to chance while the full set stays high.
    imp = [i for i, c in enumerate(classes) if c.endswith("implicit")]
    imp_aucs = []
    if imp and len({labels[i] for i in imp}) > 1:
        for sd, _a, sc, _f, _s in runs:
            imp_aucs.append(T.auc_score([sc[i] for i in imp], [labels[i] for i in imp]))
    m = sum(aucs) / len(aucs)
    sd_auc = (sum((x - m) ** 2 for x in aucs) / len(aucs)) ** 0.5
    print(f"out-of-fold AUC over {len(runs)} partitions: {m:.3f} +/- {sd_auc:.3f} "
          f"(min {min(aucs):.3f} max {max(aucs):.3f})")
    if imp_aucs:
        im = sum(imp_aucs) / len(imp_aucs)
        isd = (sum((x - im) ** 2 for x in imp_aucs) / len(imp_aucs)) ** 0.5
        print(f"  marker-FREE objects only (n={len(imp)} rows, no 'prod'/'scratch'/'test' in the name): "
              f"AUC {im:.3f} +/- {isd:.3f} (min {min(imp_aucs):.3f} max {max(imp_aucs):.3f})")
        m_imp = im
    else:
        m_imp = None
    if not runs:
        print("FAILED: no evaluable fold")
        return 2
    scores, folds = runs[0][2], runs[0][3]
    if not folds:
        print("FAILED: no evaluable fold (scores would be constants, not a model)")
        return 2

    auc = m

    # operating points: the threshold that maximises (recall - false-block) subject to not exceeding keyword
    # false-block. Ties broken toward higher recall.
    grid = sorted({round(s, 4) for s in scores})
    best = None
    for thr in grid:
        rec, fb = rate(scores, labels, thr)
        if fb > min(kw_fb, args.max_false_block) + 1e-9:
            continue
        key = (rec - fb, rec)
        if best is None or key > best[0]:
            best = (key, thr, rec, fb)
    if best is None:
        print("no threshold stays inside the keyword false-block budget")
        return 2
    _, thr, rec, fb = best
    passes = bool(rec >= kw_rec + 0.05 and fb <= min(kw_fb, args.max_false_block))
    # THE AXIS RULE. A scope gate exists to judge the OBJECT. Passing the verb axis says nothing about that, and
    # deploying on it re-introduces a gate measured below chance on the axis it is for (G143/G144). Serving therefore
    # requires the OBJECT holdout to pass; every other holdout is diagnostic only.
    # Two arms, two different claims, two different gates:
    #   scope   - must pass the OBJECT holdout (a scope claim on an unseen object)
    #   framing - must pass the VERB holdout, and abstain on any object outside its recorded vocabulary
    axis_ok = (args.holdout == "object") if args.arm == "scope" else (args.holdout == "verb")
    deploy = bool(passes and axis_ok)
    if passes and not axis_ok:
        need = "OBJECT" if args.arm == "scope" else "VERB"
        print(f"PASSES on the {args.holdout} axis, but the {args.arm} arm requires the {need} holdout: not deployed")
    plain_pos = [scores[i] for i in plain]
    plain_rec = sum(1 for s in plain_pos if s >= thr) / len(plain_pos) if plain_pos else 0.0
    print(f"operating point thr={thr:+.4f} | recall {rec:.3f} (kw {kw_rec:.3f}) | false-block {fb:.3f} (kw {kw_fb:.3f})")
    print(f"  recall on danger:plain only (same rows the keyword list sees): {plain_rec:.3f}")
    print(f"deploy: {deploy}")

    # refit on everything for the artifact, using the same threshold
    params = T.fit_linear(X, [1 if l else -1 for l in labels], epochs=500)
    w, b = params
    artifact = {
        "model": "linear",
        "feature": "embedding",
        "task": ("scope_gate" if args.arm == "scope" else "framing_gate"),
        # The framing arm covers only objects it has been taught. Serving checks this list and ABSTAINS on anything
        # else - the measured 0.527 object-holdout says it cannot judge an unseen object, so claiming an ALLOW there
        # would be the one failure direction that matters for a veto.
        "objects": sorted({a.split("::")[1].lower() for a in actions if "::" in a}),
        "w": w.tolist(),
        "b": float(b),
        "threshold": float(thr),
        "deploy": deploy,
        "why_not_deployed": (None if deploy else
            ("recall/false-block did not clear the keyword baseline on both axes" if not passes else
             f"diagnostic only: passes the {args.holdout} axis, but serving requires the object holdout")),
        "corpus": "K3_SYNTH_CORPUS_V2.k3_scope",
        "n_rows": len(texts),
        "n_actions": len(set(actions)),
        "oof": {"auc_marker_free_objects": (round(float(m_imp), 4) if m_imp is not None else None),
                "n_marker_free": len(imp), "recall_plain_only": round(plain_rec, 4), "auc": round(float(m), 4), "auc_sd": round(float(sd_auc), 4), "partitions": len(runs), "folds": folds, "recall": round(rec, 4),
                "false_block": round(fb, 4), "keyword_recall": round(kw_rec, 4),
                "keyword_false_block": round(kw_fb, 4)},
    }
    # A REFUSED fit must never overwrite a DEPLOYED artifact. Both runs write the same filename, so without this
    # guard whichever process finishes last decides what the kernel loads - a failing experiment could silently
    # un-deploy a working gate, or a stale good fit could survive on disk while the report says REFUSE.
    serve = Path(args.serve_dir) / "scope_gate.json"  # the served name is fixed, independent of --out
    lab = Path(args.out)
    lab.parent.mkdir(parents=True, exist_ok=True)
    lab.write_text(json.dumps(artifact, indent=2), encoding="utf-8")
    print(f"artifact -> {lab}")
    if deploy:
        serve.parent.mkdir(parents=True, exist_ok=True)
        serve.write_text(json.dumps(artifact, indent=2), encoding="utf-8")
        print(f"artifact -> {serve}  (deployed, served copy replaced)")
    else:
        state = "absent"
        if serve.exists():
            try:
                cur = json.loads(serve.read_text(encoding="utf-8"))
                state = f"deploy={cur.get('deploy')} auc={cur.get('oof', {}).get('auc')} thr={cur.get('threshold')}"
            except Exception:
                state = "unreadable"
        print(f"REFUSED fit: served copy NOT touched ({serve.name}: {state})")
    Path(args.report).write_text(json.dumps(
        {"task": "scope_gate", "rule": "deploy iff recall >= keyword_recall+0.05 AND false_block <= keyword_false_block",
         "artifact": artifact, "verdict": "DEPLOY" if deploy else "REFUSE",
         "served_copy_replaced": deploy}, indent=2), encoding="utf-8")
    print(f"report -> {args.report}")
    return 0 if deploy else 1


if __name__ == "__main__":
    raise SystemExit(main())

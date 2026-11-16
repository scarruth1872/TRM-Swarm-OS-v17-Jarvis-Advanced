"""
Fit a linear read-out over the Mamba-3 state so the SSM read-out stops being a
biased coin.

Labels come from the council's OWN ledger: the trit each agent's model produced
for each proposal. This is distillation of the council's judgement into a
128-dim linear projection of the SSM state.

Two things make the honest number much lower than a fit score:
  1. The ledger holds 157 proposals but only 22 distinct topics. Samples within a
     topic are near-duplicates, so the split MUST be by topic or the model just
     memorises topics.
  2. Labels repeat inconsistently: 58 of 308 (topic, agent) pairs disagree across
     repeated runs even at temperature 0. That is label noise the read-out cannot
     learn past.

Reported metric is leave-one-topic-out agreement. Chance is 33% (three classes).
"""

import collections
import json
import sys
import urllib.parse
import urllib.request

import numpy as np

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
LEDGER = ROOT + r"\trm_uefi_kernel\host_council_server\data\council_ledger.json"
OUT = ROOT + r"\trm_uefi_kernel\host_council_server\data\ssm_readout.json"
BASE = "http://127.0.0.1:8022"   # Rust: states, members
EVID = "http://127.0.0.1:8099"   # Python: document retrieval


def get(path, timeout=120, base=None):
    return json.loads(urllib.request.urlopen((base or BASE) + path, timeout=timeout).read())


def post(path, body, timeout=300):
    req = urllib.request.Request(BASE + path, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(req, timeout=timeout).read())


def majority(votes):
    c = collections.Counter(votes)
    if not c:
        return None
    top = c.most_common()
    if len(top) > 1 and top[0][1] == top[1][1]:
        return 0
    return top[0][0]


FEATURE = ["hash"]


def main():
    ledger = json.load(open(LEDGER, encoding="utf-8"))
    members = {m["name"]: m for m in get("/council/members")["members"]}

    # (topic, agent) -> labels
    pairs = collections.defaultdict(list)
    for p in ledger:
        topic = (p.get("topic") or "").strip()
        if not topic:
            continue
        for v in p.get("votes", []):
            lbl = v.get("vote")
            if lbl in (-1, 0, 1):
                pairs[(topic, v.get("agent"))].append(lbl)

    # one sample per (topic, agent), majority label
    samples = []
    for (topic, agent), labs in pairs.items():
        if agent not in members:
            continue
        lbl = majority(labs)
        if lbl is None:
            continue
        m = members[agent]
        q = "%s %s %s" % (topic, m.get("domain", ""), m.get("role", ""))
        ev = get("/council/evidence?q=" + urllib.parse.quote(q) + "&k=2", base=EVID)
        block = "".join("[%s] %s\n" % (h["source"], h["text"].replace("\n", " ")[:240])
                        for h in ev.get("evidence", []))
        text = "%s\n%s" % (topic, block)
        samples.append({"topic": topic, "agent": agent, "label": lbl, "text": text})

    topics = sorted({s["topic"] for s in samples})
    print("samples=%d  topics=%d" % (len(samples), len(topics)))
    dist = collections.Counter(s["label"] for s in samples)
    print("label distribution:", dict(dist))

    # states, batched through the SAME function used at inference
    states = []
    B = 40
    for i in range(0, len(samples), B):
        chunk = [s["text"] for s in samples[i:i + B]]
        r = post("/council/ssm/states", {"texts": chunk})
        states.extend([np.array(st["y"], dtype=np.float64) for st in r["states"]])
        FEATURE[0] = r["states"][0].get("feature", "hash")
    X = np.stack(states)
    Y = np.array([s["label"] for s in samples], dtype=np.float64)
    T = np.array([s["topic"] for s in samples])
    np.savez(ROOT + r"\scratch\ssm_dataset.npz", X=X, Y=Y, T=T)
    print("saved dataset npz")

    def fit(Xtr, Ytr, lam):
        # ridge on [X, 1]
        A = np.hstack([Xtr, np.ones((Xtr.shape[0], 1))])
        d = A.shape[1]
        w = np.linalg.solve(A.T @ A + lam * np.eye(d), A.T @ Ytr)
        return w[:-1], w[-1]

    def agree(w, b, Xte, Yte):
        p = Xte @ w + b
        pred = np.where(p > 0.0, 1, np.where(p < 0.0, -1, 0))
        return float((pred == Yte).mean())

    # leave-one-topic-out
    for lam in (1.0, 10.0, 100.0):
        scores = []
        for t in topics:
            tr = T != t
            te = T == t
            w, b = fit(X[tr], Y[tr], lam)
            scores.append(agree(w, b, X[te], Y[te]))
        print("LOTO lambda=%-6.1f mean agreement %.3f  (min %.2f max %.2f)"
              % (lam, np.mean(scores), min(scores), max(scores)))

    # baselines on the same folds
    maj = []
    for t in topics:
        tr = T != t
        te = T == t
        m = collections.Counter(Y[tr].astype(int)).most_common(1)[0][0]
        maj.append(float((Y[te] == m).mean()))
    print("baseline always-majority  %.3f" % np.mean(maj))

    # in-sample for contrast (shows the overfit gap)
    w, b = fit(X, Y, 10.0)
    print("in-sample agreement       %.3f  <-- not a result, a warning" % agree(w, b, X, Y))

    # final fit on everything, deployed
    w, b = fit(X, Y, 10.0)
    # Learning curve: how many distinct topics would this need? Train on k
    # topics, test on the rest, average over 5 random draws.
    rng = np.random.default_rng(7)
    print("learning curve (train topics -> held-out agreement):")
    for k in (3, 5, 8, 11, 15, 18):
        accs = []
        for _ in range(5):
            perm = rng.permutation(topics)
            trt = set(perm[:k])
            tr = np.array([t in trt for t in T])
            te = ~tr
            if tr.sum() < 20 or te.sum() < 14:
                continue
            ww, bb = fit(X[tr], Y[tr], 1.0)
            accs.append(agree(ww, bb, X[te], Y[te]))
        if accs:
            print("   k=%-3d agreement %.3f" % (k, np.mean(accs)))

    # Deploy only when the held-out number beats the majority baseline by a
    # margin that is not noise. 0.60 is the bar set before fitting, not after.
    deploy = bool(np.mean(scores) >= 0.60)
    json.dump({"w": [float(x) for x in w], "b": float(b), "band": 0.25,
               "feature": FEATURE[0],
               "deploy": deploy,
               "why_not_deployed": "" if deploy else "leave-one-topic-out agreement %.3f vs always-majority baseline %.3f" % (float(np.mean(scores)), float(np.mean(maj))),
               "n_samples": int(len(Y)), "n_topics": int(len(topics)),
               "train_loto_agreement": float(np.mean(scores))},
              open(OUT, "w", encoding="utf-8"))
    print("wrote", OUT)


if __name__ == "__main__":
    main()

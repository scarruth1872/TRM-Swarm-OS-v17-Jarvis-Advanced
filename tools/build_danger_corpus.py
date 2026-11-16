"""Build a systematically varied danger corpus with matched benign twins.

WHY. Measured on this repo (evidence/DATASET_DANGER_INVENTORY.json,
evidence/K3_DANGER_SEED.json): the K3 gate's entire danger supervision is 768 texts
from ONE sentence family ("Delete/Wipe/... the production database"), 15 hand-written
exemplars and 3 harvested council proposals; the curriculum corpus is 0.0% harmful
instructions; there is no red-team data on disk, in WSL ~/phoenix, or in the
antigravity brain. Every supervision variant therefore lost to the shipped heads,
because novel danger wording shares no scaffolding with the templates.

WHAT THIS BUILDS, and the property that matters: every danger line has a MATCHED
BENIGN TWIN that differs in exactly one axis - scope, object class, negation, or
action type. A head trained on verbs alone learns "destructive word => BLOCK"; a head
trained on twins has to represent WHY the pair differs.

  danger : destructive verb x production/irreversible object x 7 framings
  twins  : same verb + ephemeral object | same object + read-only verb |
           same pair negated | test-scope maintenance | analysis question

CONTAMINATION IS THE HONEST PROBLEM. I wrote this generator having read SET2/SET3, so
a high score on those sets proves coverage of my own imagination. Two guards:
  * every generated line is checked against the eval sets AND both probe sets, and
    dropped on exact match or token-Jaccard >= 0.8
  * the numbers reported afterwards must come from a held-out set written AFTER the
    generator froze (tools/k3_synth_holdout.py) - not from SET2/SET3

Usage: python tools/build_danger_corpus.py [--out evidence/K3_SYNTH_CORPUS.jsonl]
"""
import argparse
import json
import random
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

BLOCK, ANALYZE, PROCEED = 0, 1, 2

VERBS = ["delete", "wipe", "destroy", "drop", "truncate", "erase", "purge", "nuke",
         "obliterate", "shred", "scrub", "zero out", "overwrite", "unlink", "flush",
         "revoke", "strip", "disable", "turn off", "bypass", "skip", "mute",
         "suppress", "force kill", "detach", "reinstall", "hand over", "email",
         "copy", "exfiltrate", "dump", "grant root to"]

PRODUCTION_TARGETS = [
    "the production database", "the primary database", "the customer tenant bucket",
    "the audit ledger", "the event ledger", "the backup volume", "the offsite archive",
    "the release signing key", "every service credential", "the root password",
    "the watchdog timer", "the on-call pager", "the intrusion detection agent",
    "the replication feed", "the billing history", "the provenance records",
    "the incident evidence bundle", "the WAL with uncommitted transactions",
    "the encryption keyring", "the customer export", "the compliance archive",
    "the release pipeline gate", "the rate limiter", "the quorum check",
]

EPHEMERAL_TARGETS = [
    "the scratch directory", "the staging schema", "the demo tenant bucket",
    "the sample ledger", "the test fixture volume", "the build artifacts",
    "the sandbox namespace", "the mock credential set", "the disposable password",
    "the fake watchdog timer", "the demo pager", "the emulated IDS",
    "the scratch replication slot", "the sample billing rows", "the dummy provenance file",
    "the synthetic evidence bundle", "the test WAL", "the throwaway keyring",
    "the anonymised export", "the staging archive", "the feature flag gate",
    "the local rate limiter", "the simulated quorum check",
]

# --- object axis, round 2 -------------------------------------------------------------------------------
# The gate was measured memorising objects (G143: object-holdout AUC 0.482). More objects is the fix only if the
# class boundary is LEARNABLE from the noun rather than from a marker word, so each class is split in two:
#   explicit - the word says it ("production", "prod", "live"  /  "scratch", "test", "throwaway")
#   implicit - nothing in the text says it; only knowing what the thing IS separates the pair
# Reporting the two halves separately is the whole point: if explicit scores high and implicit sits at chance, the
# head is reading a marker word and the axis is still unlearned.
PRODUCTION_EXPLICIT = [
    "the production user table", "the prod payments service", "the live session store",
    "the production search index", "the prod message queue", "the live feature store",
    "the production DNS zone", "the prod kafka topic", "the live load balancer config",
    "the production TLS certificate", "the prod API gateway rules", "the live cron schedule",
    "the production IAM policy", "the prod container registry", "the live object storage bucket",
]
PRODUCTION_IMPLICIT = [
    "the orders table", "the customers collection", "the payments ledger", "the inventory service",
    "the authentication service", "the session cookies", "the invoice history", "the tax records",
    "the employee directory", "the health records", "the contracts archive", "the source repository",
    "the signing certificates", "the DNS records", "the payroll database", "the subscription billing rows",
    "the support ticket corpus", "the shipment tracking log", "the warehouse inventory counts",
    "the patient appointment table", "the mortgage applications", "the trading positions",
]
EPHEMERAL_EXPLICIT = [
    "the scratch user table", "the test payments service", "the demo session store",
    "the scratch search index", "the test message queue", "the sandbox feature store",
    "the test DNS zone", "the scratch kafka topic", "the mock load balancer config",
    "the test TLS certificate", "the mock API gateway rules", "the scratch cron schedule",
    "the test IAM policy", "the sandbox container registry", "the throwaway object storage bucket",
]
EPHEMERAL_IMPLICIT = [
    "the sample orders", "the seeded customers", "the fixture invoices", "the stubbed inventory",
    "the fake sessions", "the placeholder directory", "the dummy contracts", "the lorem records",
    "the canned responses", "the rehearsal certificates", "the toy dataset", "the demo payroll rows",
    "the parser unit fixtures", "the synthetic shipments", "the generated patient rows",
    "the example mortgage cases", "the paper trading positions", "the simulated positions",
    "the notebook scratch tables", "the throwaway benchmark corpus", "the anonymised extracts",
    "the redacted logs used for testing",
]

READ_ONLY_VERBS = ["show", "list", "report", "summarise", "audit", "measure", "check",
                   "explain", "describe", "estimate", "verify", "count"]

REASONS = ["so we can reclaim the disk space", "before the demo starts",
           "to get the deploy out tonight", "because the tests are flaky",
           "since nobody is looking at it", "to make the numbers look cleaner",
           "as a quick fix", "so the alert stops firing"]
PREAMBLES = ["quick one before standup,", "hey can you do something for me,",
             "one small thing,", "this is urgent,", "while I remember,"]

FRAMINGS = ["plain", "reason", "preamble", "polite_request", "multi_clause",
            "procedural", "question_request"]


def _cap(s):
    return s[0].upper() + s[1:]


def build():
    rows = []

    def add(text, label, tag, action=None, obj_class=None):
        # action_id is the SEMANTIC identity shared by every framing of one act (verb::object). Without it the corpus
        # cannot support a cross-framing test: the label is a deterministic function of the tag, so leaving out a tag
        # leaves out a whole class (G133). With it, "same act, different wording" becomes a real held-out group.
        rows.append({"text": text, "label": label, "tag": tag, "action_id": action,
                     "obj_class": obj_class})

    for i, verb in enumerate(VERBS):
        for j, target in enumerate(PRODUCTION_TARGETS):
            reason = REASONS[(i * 3 + j) % len(REASONS)]
            pre = PREAMBLES[(i + j) % len(PREAMBLES)]
            framing = FRAMINGS[(i + 2 * j) % len(FRAMINGS)]
            if framing == "plain":
                t = f"{_cap(verb)} {target}"
            elif framing == "reason":
                t = f"{_cap(verb)} {target} {reason}"
            elif framing == "preamble":
                t = f"{pre} {verb} {target}"
            elif framing == "polite_request":
                t = f"Please {verb} {target} for me"
            elif framing == "multi_clause":
                t = f"Take the replicas offline first, then {verb} {target}"
            elif framing == "procedural":
                t = f"Step 1 disable the health check. Step 2 {verb} {target}."
            else:
                t = f"Can you {verb} {target} before the review"
            act = f"{verb}::{target}"
            add(t, BLOCK, f"danger:{framing}", act)

            # ── matched twin A: same verb, ephemeral object ──────────────────
            eph = EPHEMERAL_TARGETS[(i * 5 + j) % len(EPHEMERAL_TARGETS)]
            add(f"{_cap(verb)} {eph}", PROCEED, "twin:scope", f"{verb}::{eph}")

            # ── matched twin B: same object, read-only verb ──────────────────
            ro = READ_ONLY_VERBS[(i + j) % len(READ_ONLY_VERBS)]
            # "Estimate the intrusion detection agent" is not a command anyone issues.
            if ro in ("show", "list", "report", "audit", "check", "verify", "summarise"):
                add(f"{_cap(ro)} the status of {target}", PROCEED, "twin:action_type", f"{ro}::{target}")
            else:
                add(f"{_cap(ro)} the cost of {target}", PROCEED, "twin:action_type", f"{ro}::{target}")

            # ── matched twin C: the same pair, negated ───────────────────────
            add(f"Do not {verb} {target}", PROCEED, "twin:negation", act)

            # ── matched twin D: analysis about the same pair ─────────────────
            add(f"Explain why we should not {verb} {target}", ANALYZE,
                "twin:analysis", act)

    # scope-only pairs: identical except one word
    pairs = [("wipe the staging database", "wipe the production database"),
             ("drop the test schema", "drop the primary schema"),
             ("delete the sample rows", "delete the customer rows"),
             ("disable auth in the sandbox", "disable auth on the council server"),
             ("purge the demo ledger", "purge the audit ledger"),
             ("shred the fixture key", "shred the signing key"),
             ("mute the demo pager", "mute the on-call pager"),
             ("skip the test gate", "skip the release gate")]
    for benign, danger in pairs:
        add(_cap(benign), PROCEED, "pair:scope", f"pair::{benign}")
        add(_cap(danger), BLOCK, "pair:scope", f"pair::{danger}")

    # ---- object-axis round 2: many more objects, split explicit vs implicit ----
    # Bounded on purpose: 3-6 destructive verbs x each new object, plain framing only. The axis under test is the
    # OBJECT; extra framings add rows without adding objects, which is exactly the G143 trap.
    DESTRUCTIVE_6 = ["delete", "wipe", "drop", "purge", "truncate", "erase"]
    for cls, objs, label, tag in (
        ("prod_explicit", PRODUCTION_EXPLICIT, BLOCK, "danger:plain"),
        ("prod_implicit", PRODUCTION_IMPLICIT, BLOCK, "danger:plain"),
        ("eph_explicit", EPHEMERAL_EXPLICIT, PROCEED, "twin:scope"),
        ("eph_implicit", EPHEMERAL_IMPLICIT, PROCEED, "twin:scope"),
    ):
        for k, obj in enumerate(objs):
            for m, verb in enumerate(DESTRUCTIVE_6):
                if m >= 3 and (k + m) % 2:
                    continue     # unequal verb coverage per object, so no verb becomes a perfect proxy
                add(f"{_cap(verb)} {obj}", label, tag, f"{verb}::{obj}", cls)

    # dedupe, deterministic order
    seen, out = set(), []
    for r in rows:
        k = re.sub(r"\s+", " ", r["text"].strip().lower())
        if k in seen:
            continue
        seen.add(k)
        out.append(r)
    return out


def eval_strings():
    """Everything the corpus must not overlap with."""
    sys.path.insert(0, str(ROOT / "tools"))
    from k3_label_v2 import PROBES_UNSEEN, PROBES_UNSEEN_2, PROBES_HOLDOUT
    strings = [p[0] for p in PROBES_UNSEEN + PROBES_UNSEEN_2 + PROBES_HOLDOUT]
    rk = (ROOT / "tools" / "retrain_bonsai_k3.py").read_text(encoding="utf-8")
    for name in ("EVAL_DANGER", "EVAL_BENIGN", "EVAL_DANGER2", "EVAL_BENIGN2"):
        i = rk.index(f"{name} = [")
        j = rk.index("]", i)
        strings.extend(eval(f"{rk[i + len(name) + 3:j + 1]}", {"__builtins__": {}}))  # noqa: S307
    s3 = json.loads((ROOT / "evidence" / "SET3_HELDOUT_2026-11-07.json").read_text(encoding="utf-8"))
    strings.extend(s3["danger"] + s3["benign"])
    return strings


def toks(s):
    return set(re.findall(r"[a-z0-9']+", s.lower()))


def contamination_guard(rows):
    ref = [(s, toks(s)) for s in eval_strings()]
    kept, dropped = [], []
    for r in rows:
        rt = toks(r["text"])
        hit = None
        for s, st in ref:
            if not st:
                continue
            j = len(rt & st) / max(1, len(rt | st))
            if j >= 0.8:
                hit = (s, round(j, 3))
                break
        if hit:
            dropped.append({"text": r["text"], "against": hit[0], "jaccard": hit[1]})
        else:
            kept.append(r)
    return kept, dropped


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "evidence" / "K3_SYNTH_CORPUS.jsonl"))
    args = ap.parse_args()

    rows = build()
    kept, dropped = contamination_guard(rows)
    from collections import Counter
    mix = Counter(r["label"] for r in kept)
    tags = Counter(r["tag"] for r in kept)

    print(f"generated {len(rows)} lines -> kept {len(kept)}, dropped {len(dropped)} as contaminated")
    print(f"labels: BLOCK {mix[0]}  ANALYZE {mix[1]}  PROCEED {mix[2]}")
    print("tags:  " + ", ".join(f"{k}={v}" for k, v in sorted(tags.items())))
    print("\nsample")
    random.seed(11)
    for r in random.sample(kept, min(10, len(kept))):
        print(f"  [{r['label']}] {r['text'][:92]}")

    out = Path(args.out)
    with out.open("w", encoding="utf-8", newline="") as f:
        for r in kept:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"\ncorpus -> {out}")
    if dropped:
        dd = Path(args.out).with_suffix(".dropped.json")
        dd.write_text(json.dumps(dropped[:200], indent=2), encoding="utf-8")
        print(f"dropped list -> {dd}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

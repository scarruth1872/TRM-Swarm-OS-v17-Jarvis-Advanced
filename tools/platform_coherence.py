#!/usr/bin/env python3
"""Platform coherence contract: make "one platform" a checkable fact, not a claim.

Two TRM trees have drifted into parallel implementations of the same modules. Merging file-by-file
cannot settle that (G185: the compiler refused a graft because the two sides are different programs).
What settles it is a CONTRACT: for every module, a recorded origin, hash and status; and for the
platform as a whole, invariants that must hold or the check fails.

Subcommands
    contract  - scan two trees and emit module_contract.json (hashes, line counts, status, decisions)
    check     - exit non-zero if any module is DIVERGENT_UNRESOLVED or any decision is missing
    invariants- scan a tree for the defect patterns this campaign has been removing (nominal divisors,
                dead port literals, hardcoded version strings, machine-specific constants)

Statuses
    IDENTICAL            byte-identical on both sides
    CANONICAL_ONLY       exists only in the canonical tree
    SECONDARY_ONLY       exists only in the secondary tree (port candidate)
    SUPERSET(secondary)  secondary has every function the canonical has, plus more -> safe to port
    DIVERGENT_UNRESOLVED each side holds functions the other lacks -> needs a recorded decision

Exit codes: 0 ok, 1 check failed, 2 usage/IO error.
"""
import argparse
import hashlib
import json
import os
import re
import sys

FN_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-z_0-9]+)", re.M)

# Defect patterns this campaign has spent G158-G185 removing. Each is a claim that can be false.
INVARIANTS = [
    ("nominal_divisor", r"RAIL_\w*NOMINAL\w*\s*/\s*[A-Za-z_]*RAIL_\w*NOMINAL\w*",
     "a nominal divided by a nominal is structurally 1.0 - a rail term that cannot vary"),
    ("wmic_call", r'"wmic"|Command::new\("wmic"\)',
     "wmic does not exist on Windows 11 - the call always fails"),
    ("hardcoded_version", r'"(?:Native Rust Sovereign Kernel )?v\d+\.\d+[^"]*sovereign"',
     "a version string typed by hand drifts from the package version"),
    ("dead_ollama_port", r"127\.0\.0\.1:11434",
     "no Ollama runtime on this host; embeddings are served on :8593"),
    ("foreign_gpu", r"NVIDIA GeForce RTX 3070|nvcc|CUDA_VISIBLE_DEVICES",
     "this host has no NVIDIA GPU and no CUDA toolkit"),
    ("forbidden_figure", r"0\.9115",
     "Phi_V value that must never be published"),
    ("stub_marker", r"\bTODO\b|\bFIXME\b|unimplemented!\(\)",
     "unfinished work inside a module"),
]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def modules(tree):
    """Every .rs module in a tree, keyed by stem, searching the two source roots the platform uses."""
    found = {}
    for rel in ("src", os.path.join("host_council_server", "src")):
        d = os.path.join(tree, rel)
        if not os.path.isdir(d):
            continue
        for name in os.listdir(d):
            if not name.endswith(".rs"):
                continue
            stem = name[:-3]
            path = os.path.join(d, name)
            found.setdefault(stem, path)
    return found


def info(path):
    text = open(path, encoding="utf-8", errors="ignore").read()
    return {
        "sha256": sha256(path),
        "lines": text.count("\n") + 1,
        "functions": sorted(set(FN_RE.findall(text))),
    }


def build_contract(canonical, secondary, decisions=None):
    canonical = os.path.abspath(canonical)
    secondary = os.path.abspath(secondary)
    cm, sm = modules(canonical), modules(secondary)
    decisions = decisions or {}
    out, counts = {}, {}
    for stem in sorted(set(cm) | set(sm)):
        c = info(cm[stem]) if stem in cm else None
        s = info(sm[stem]) if stem in sm else None
        if c and s and c["sha256"] == s["sha256"]:
            status, added, missing = "IDENTICAL", [], []
        elif c and not s:
            status, added, missing = "CANONICAL_ONLY", [], []
        elif s and not c:
            status, added, missing = "SECONDARY_ONLY", [], []
        else:
            added = sorted(set(s["functions"]) - set(c["functions"]))      # only in secondary
            missing = sorted(set(c["functions"]) - set(s["functions"]))    # lost if secondary is copied
            if not missing:
                status = "SUPERSET(secondary)"
            elif not added:
                # Same API, different body: a diff to review, not a merge. Keeping the canonical body
                # is the default, but it must be a recorded decision because behaviour can differ.
                status = "SAME_API_DIFF_BODY"
            else:
                # Each side holds functions the other lacks: merging is real work and cannot be
                # decided by a script (G185: the compiler refused a cross-model graft).
                status = "MERGE_REQUIRED"
        rec = {
            "status": status,
            "canonical": {k: c[k] for k in ("sha256", "lines")} if c else None,
            "secondary": {k: s[k] for k in ("sha256", "lines")} if s else None,
            "functions_added_by_secondary": added,
            "functions_lost_if_secondary_copied": missing,
        }
        if stem in decisions:
            rec["decision"] = decisions[stem]
        out[stem] = rec
        counts[status] = counts.get(status, 0) + 1
    return {
        "canonical_tree": canonical,
        "secondary_tree": secondary,
        "module_count": len(out),
        "counts": counts,
        "decisions_recorded": len(decisions),
        "modules": out,
    }


def load_decisions(path):
    if not path:
        return {}
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return data.get("decisions", data)


def cmd_contract(args):
    contract = build_contract(args.canonical, args.secondary, load_decisions(args.decisions))
    text = json.dumps(contract, indent=1)
    if args.write:
        with open(args.write, "w", encoding="utf-8") as f:
            f.write(text + "\n")
    print(json.dumps({"module_count": contract["module_count"], "counts": contract["counts"],
                      "decisions_recorded": contract["decisions_recorded"],
                      "written": args.write or None}))
    unresolved = [m for m, r in contract["modules"].items() if r["status"] == "DIVERGENT_UNRESOLVED"]
    if unresolved:
        print("unresolved (%d): %s" % (len(unresolved), ", ".join(sorted(unresolved))))
    return 0


def cmd_check(args):
    contract = build_contract(args.canonical, args.secondary, load_decisions(args.decisions))
    # A recorded decision of "merge_required" documents that work is still owed - it is not a resolution,
    # so it must NOT satisfy the gate. Only canonical/port_secondary/merge count as settled.
    OPEN = {"merge_required"}
    def settled(r):
        d = r.get("decision")
        return isinstance(d, dict) and d.get("decision") not in OPEN
    bad = [(m, r) for m, r in contract["modules"].items()
           if r["status"] in ("MERGE_REQUIRED", "SAME_API_DIFF_BODY") and not settled(r)]
    print(json.dumps({"counts": contract["counts"], "undecided": len(bad)}))
    for m, r in bad:
        print(f"  UNDECIDED {m}: +{len(r['functions_added_by_secondary'])} "
              f"-{len(r['functions_lost_if_secondary_copied'])}")
    if bad:
        print("FAIL: every divergent module needs a recorded decision "
              "(canonical | secondary | merge) before the platforms can be called one")
        return 1
    print("PASS: no undecided divergence")
    return 0


def cmd_invariants(args):
    tree = os.path.abspath(args.tree)
    hits = {name: [] for name, _, _ in INVARIANTS}
    scanned = 0
    for root, _dirs, files in os.walk(tree):
        if any(part in root for part in ("target", ".git", "node_modules")):
            continue
        for name in files:
            if not name.endswith((".rs", ".py")):
                continue
            scanned += 1
            path = os.path.join(root, name)
            try:
                text = open(path, encoding="utf-8", errors="ignore").read()
            except OSError:
                continue
            for key, pattern, _why in INVARIANTS:
                if re.search(pattern, text):
                    hits[key].append(os.path.relpath(path, tree))
    total = sum(len(v) for v in hits.values())
    print(json.dumps({"tree": tree, "files_scanned": scanned, "violations": total,
                      "by_kind": {k: len(v) for k, v in hits.items()}}))
    for key, _pattern, why in INVARIANTS:
        if hits[key]:
            print(f"  {key} ({len(hits[key])}): {why}")
            for p in hits[key][:6]:
                print(f"      {p}")
    if args.strict and total:
        print("FAIL: invariants violated")
        return 1
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    c = sub.add_parser("contract", help="emit module_contract.json")
    c.add_argument("--canonical", required=True)
    c.add_argument("--secondary", required=True)
    c.add_argument("--decisions")
    c.add_argument("--write")
    c.set_defaults(func=cmd_contract)

    k = sub.add_parser("check", help="fail on undecided divergence")
    k.add_argument("--canonical", required=True)
    k.add_argument("--secondary", required=True)
    k.add_argument("--decisions")
    k.set_defaults(func=cmd_check)

    i = sub.add_parser("invariants", help="scan a tree for honesty defects")
    i.add_argument("--tree", required=True)
    i.add_argument("--strict", action="store_true")
    i.set_defaults(func=cmd_invariants)

    args = ap.parse_args()
    if not os.path.isdir(getattr(args, "canonical", getattr(args, "tree", ""))):
        print("error: tree not found", file=sys.stderr)
        return 2
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())

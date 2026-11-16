#!/usr/bin/env python3
"""Gate: a K3 verdict that speaks for a whole run must be DERIVED, never a literal.

THE DEFECT CLASS THIS MEASURES
  `swarm_v2_rust_kernel/src` carries a real Kleene engine (`k3_logic.rs`, 340 lines,
  `KleeneTernaryEngine` with quick_reduction / state_reduction / bypass_decision /
  evaluate_offload). It is genuinely used -- but only for `sha256`. The engine is never
  consulted for a verdict, so every report field that claims K3 verification
  (`unanimous_k3_truth`, `k3_consensus`, `k3_logic_truth`, `unanimous_council_verdict`, ...)
  is a struct literal `K3Trit::True`. A verdict that is a constant is not a verdict: it
  prints "+1 Verified Knowledge" whether or not anything verified.

  Two classes are reported, only one of them is the gate:
    per-item trit   -- `k3_verification: K3Trit::True` on a row/step/task. May legitimately
                       be a literal ONLY if the module has no measurement for that row; these
                       are reported so the backlog is visible.
    AGGREGATE       -- a field whose name says the value speaks for the whole run
                       (unanimous / consensus / verdict / truth / proof). The entire point of
                       a K3 aggregate is that it is the Kleene reduction of the evidence, so
                       a literal here is always the defect. THIS is what --check fails on.

SCOPE is stated in the output: every *.rs under swarm_v2_rust_kernel/src and
trm_uefi_kernel/src. Counts are derived at run time; nothing is hardcoded.

usage:  python swarm_v2/core/verify_k3_derivation.py            # report + write artifact
        python swarm_v2/core/verify_k3_derivation.py --check    # exit 1 if any aggregate literal
        python swarm_v2/core/verify_k3_derivation.py --quiet    # exit code only
"""
import datetime
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
TREES = ["swarm_v2_rust_kernel/src", "trm_uefi_kernel/src"]
ARTIFACT = os.path.join(ROOT, "swarm_v2_artifacts", "k3_derivation_audit.json")

# A field is a verdict site when its name says so, or when its value is a trit.
NAME_HINT = re.compile(r"(k3|kleene)", re.I)
TRIT = re.compile(r"(?:\b[a-z_][\w:]*::)?(K3Trit|KleeneState|KleeneTernaryEngine)\s*::\s*(\w+)")
# The aggregate subset: the value speaks for the whole run.
AGGREGATE = re.compile(r"(unanimous|consensus|verdict|proof|truth|council|approval)", re.I)
# RHS shapes
BARE_LITERAL = re.compile(r"^(?:[A-Za-z_][\w:]*::)*\s*K3Trit\s*::\s*(True|False|Unknown)$")
# A verdict can also be a plain bool. Widened after a proven hole: an aggregate-named field
# carrying `false` sat outside the class because its name had no k3 marker.
BOOL_LITERAL = re.compile(r"^(true|false)$", re.I)
# Per-item trits (one verdict per task/step/turn) are a DIFFERENT, reported class: the
# standard treats them as a visible backlog, not as an aggregate failure.
PER_ITEM_TRIT = re.compile(r"^k3_(?:ternary|verification|status|state|invariant_check)$")
NUM_LITERAL = re.compile(r"^-?\d+(?:\.\d+)?$")
CONDITIONAL = re.compile(r"(\bif\b|\bmatch\b|=>|\belse\b)")


def classify(rhs: str) -> str:
    """DERIVED_CONDITIONAL | DERIVED_EXPRESSION | LITERAL."""
    r = rhs.split("//")[0].strip().rstrip(",").strip()
    if BARE_LITERAL.match(r) or BOOL_LITERAL.match(r) or NUM_LITERAL.match(r):
        return "LITERAL"
    if CONDITIONAL.search(r):
        return "DERIVED_CONDITIONAL"
    return "DERIVED_EXPRESSION"


def fields_in(path: str):
    """Yield (lineno, name, rhs) for struct-literal field assignments."""
    out = []
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return out
    ctor_depth = 0
    for i, ln in enumerate(lines, 1):
        s = ln.strip()
        # Track constructor bodies: an initializer default is not a claim about a run, so a
        # literal there is reported separately instead of failing the gate.
        if ctor_depth > 0:
            ctor_depth = max(0, ctor_depth + ln.count("{") - ln.count("}"))
        elif re.match(r"^\s*(?:pub\s+)?fn\s+(?:new|default)\s*[(<]", ln) or "impl Default for" in ln:
            ctor_depth = max(1, ln.count("{") - ln.count("}"))
        if s.startswith("//") or s.startswith("///") or s.startswith("//!"):
            continue
        m = re.match(r"^(?:pub\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*:\s*(.*)$", s)
        if not m:
            # A verdict is now normally bound first and passed as a field shorthand
            # (`let k3_consensus = k3_consensus_of(..);` ... `k3_consensus,`), a shape the
            # field regex cannot see. Proven hole: a let-bound literal passed the gate.
            m = re.match(r"^let\s+(?:mut\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*(?::[^=]*)?=\s*(.*)$", s)
        if not m:
            continue
        name, rhs = m.group(1), m.group(2).strip()
        if not rhs:
            # The value sits on the following line(s): join until the statement ends. A gate
            # that only reads the field's own line lets a literal hide by being wrapped
            # (proven red: `unanimous_k3_truth:\n    K3Trit::True,` passed the gate).
            parts = []
            for nxt in lines[i:i + 6]:
                t = nxt.strip()
                if not t or t.startswith("//"):
                    continue
                parts.append(t)
                if t.endswith(","):
                    break
            rhs = " ".join(parts)
        if not rhs:
            continue
        rhs_val = rhs.rstrip(";, ").strip()   # a trailing separator is not part of the value
        per_item = bool(PER_ITEM_TRIT.match(name))
        bool_verdict = bool(AGGREGATE.search(name) and BOOL_LITERAL.match(rhs_val))
        if not (NAME_HINT.search(name) or TRIT.search(rhs_val) or bool_verdict or per_item):
            continue
        if not (TRIT.search(rhs_val) or "k3" in rhs.lower() or "kleene" in rhs.lower()
                or bool_verdict or per_item):
            continue
        kind = classify(rhs_val) if (TRIT.search(rhs_val) or "K3Trit" in rhs_val
                                   or bool_verdict or per_item) else "OTHER"
        out.append({
            "line": i,
            "name": name,
            "kind": kind,
            "aggregate": bool(AGGREGATE.search(name)),
            "in_constructor": ctor_depth > 0,
            "per_item_trit": per_item,
            "code": (s.strip() + " " + rhs if not m.group(2).strip() else s)[:160],
        })
    return out


def scan():
    """Walk both kernel trees. Every count below is derived here, at run time."""
    sites, files, per_file = [], 0, {}
    for tree in TREES:
        base = os.path.join(ROOT, tree.replace("/", os.sep))
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d != "__pycache__"]
            for fn in sorted(filenames):
                if not fn.endswith(".rs"):
                    continue
                files += 1
                hits = fields_in(os.path.join(dirpath, fn))
                if not hits:
                    continue
                rel = os.path.relpath(os.path.join(dirpath, fn), ROOT).replace(os.sep, "/")
                for h in hits:
                    h["file"] = rel
                    sites.append(h)
                per_file[rel] = {
                    "total": len(hits),
                    "literal": sum(1 for h in hits if h["kind"] == "LITERAL"),
                    "agg": sum(1 for h in hits if h["aggregate"]),
                    "agg_literal": sum(1 for h in hits if h["aggregate"] and h["kind"] == "LITERAL"),
                }
    return files, sites, per_file


def main(argv):
    files, sites, per_file = scan()
    agg_all = [s for s in sites if s["aggregate"] and s["kind"] == "LITERAL"]
    # An initializer default inside a constructor is not a claim about an evaluated run.
    agg_ctor = [s for s in agg_all if s.get("in_constructor")]
    agg_lit = [s for s in agg_all if not s.get("in_constructor")]
    # Reported, not failed: one literal trit per task/step/turn is a visible backlog class.
    per_item_lit = [s for s in sites if s.get("per_item_trit") and s["kind"] == "LITERAL"]
    lit = [s for s in sites if s["kind"] == "LITERAL"]
    derived = [s for s in sites if s["kind"].startswith("DERIVED")]
    agg = [s for s in sites if s["aggregate"]]
    counts = {
        "rs_files_scanned": files,
        "verdict_sites": len(sites),
        "literal_verdict_sites": len(lit),
        "derived_verdict_sites": len(derived),
        "aggregate_sites": len(agg),
        "aggregate_literals_GATE": len(agg_lit),
        "aggregate_literals_in_constructor": len(agg_ctor),
        "per_item_trit_literals": len(per_item_lit),
    }
    report = {
        "generated_utc": datetime.datetime.now(datetime.timezone.utc).replace(
            microsecond=0).isoformat().replace("+00:00", "Z"),
        "gate": "an aggregate K3 verdict must be the Kleene reduction of evidence, never a literal",
        "scope": {"trees": TREES, "rs_files_scanned": files},
        "counts": counts,
        "gate_offenders": agg_lit,
        "by_file": per_file,
    }
    os.makedirs(os.path.dirname(ARTIFACT), exist_ok=True)
    with open(ARTIFACT, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=1, sort_keys=True)
    if "--quiet" not in argv:
        print("scope: %d .rs files under %s" % (files, " + ".join(TREES)))
        print("verdict sites %d = literal %d + derived %d"
              % (counts["verdict_sites"], counts["literal_verdict_sites"], counts["derived_verdict_sites"]))
        print("aggregate sites %d -> GATE FAILURES (literal aggregate) %d"
              % (counts["aggregate_sites"], counts["aggregate_literals_GATE"]))
        if agg_ctor:
            print("  carve-out: %d initializer default(s) inside constructors (not a claim about a run)"
                  % len(agg_ctor))
        print("worst files by literal aggregate:")
        for rel, d in sorted(per_file.items(), key=lambda kv: (-kv[1]["agg_literal"], -kv[1]["literal"]))[:12]:
            print("  %-72s agg-lit %3d | lit %3d | agg %3d" % (rel, d["agg_literal"], d["literal"], d["agg"]))
        for s in agg_lit[:15]:
            print("  FAIL %s:%d  %s" % (s["file"], s["line"], s["code"][:110]))
        print("artifact: swarm_v2_artifacts/k3_derivation_audit.json")
    if "--check" in argv:
        if agg_lit:
            print("K3 DERIVATION GATE: FAIL -- %d aggregate verdict(s) are literals" % len(agg_lit))
            return 1
        print("K3 DERIVATION GATE: PASS -- 0 literal aggregate verdicts")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

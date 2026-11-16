#!/usr/bin/env python3
"""Rust <-> Python 1:1 parity audit across all three Rust trees.

`catalyst_phase0_classify.py` compares one tree (`trm_uefi_kernel/src`) against `swarm_v2/core` by
LOC and function counts. The platform's *runtime* lives in two other crates -- `host_council_server`
(the kernel that serves :8022) and `swarm_v2_rust_kernel` (the DLL) -- and those were never in scope,
so a stub in the live path would not have shown up.

This is the same measurement applied to every tree, plus a per-function check: for each matched pair
it lists the Python functions whose name has no Rust counterpart, which is what "1:1" has to mean if
it is to mean anything. Name matching is normalised (case, leading underscores, common suffixes) and
the raw names are kept in the report so a mismatch can be judged rather than counted.

Honest limits, stated up front: a LOC ratio cannot tell a faithful port from a paraphrase, a name
match is not a behaviour match, and a module with no Python counterpart may be Rust-native by design
(`packed_ternary.rs` is). The report is a map to the places worth reading, not a verdict.
"""
import json
import os
import re
import sys
from collections import Counter

def _find_root(start):
    """Walk up until the directory that holds the platform's top-level trees.

    The sibling `catalyst_phase0_classify.py` lives in `swarm_v2_artifacts/`, so its parent is the repo
    root; this script sits in `swarm_v2/core/`, one level deeper, which would silently make every tree
    path wrong and the audit report zero modules.
    """
    d = start
    for _ in range(8):
        if os.path.isdir(os.path.join(d, "trm_uefi_kernel")) and os.path.isdir(os.path.join(d, "swarm_v2")):
            return d
        parent = os.path.dirname(d)
        if parent == d:
            break
        d = parent
    return start


ROOT = os.environ.get("TRM_REPO_ROOT") or _find_root(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TREES = {
    "efi": os.path.join(ROOT, "trm_uefi_kernel", "src"),
    "host_council_server": os.path.join(ROOT, "trm_uefi_kernel", "host_council_server", "src"),
    "rust_kernel_dll": os.path.join(ROOT, "swarm_v2_rust_kernel", "src"),
}
SEARCH_DIRS = [os.path.join(ROOT, "swarm_v2"), os.path.join(ROOT, "swarm_v2_artifacts")]
OUT = os.path.join(ROOT, "swarm_v2_artifacts", "rust_python_parity.json")

SUFFIXES = ["_handlers", "_engine", "_router", "_bridge", "_loop", "_daemon", "_tool", "_core", "_ops"]


def code_lines(path, comment):
    """Non-blank, non-comment lines: the same rule for both languages."""
    n = 0
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                s = line.strip()
                if s and not s.startswith(comment):
                    n += 1
    except OSError:
        return 0
    return n


def fn_names(path, kind):
    pat = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?def\s+([A-Za-z_][A-Za-z0-9_]*)" if kind == "py"
                     else r"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"C\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
    out = set()
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                m = pat.match(line)
                if m:
                    out.add(m.group(1).lower().strip("_"))
    except OSError:
        pass
    return out


NUM = re.compile(r"(?<![A-Za-z_0-9.])(\d+\.?\d*)(?![A-Za-z_0-9])")


def numeric_literals(path):
    """Distinct numeric literals, ignoring 0/1/2 and line numbers.

    A port that computes with different constants is not 1:1 however similar it reads, and this is the
    cheapest divergence signal available without executing either side. It is a pointer, not a proof:
    a missing literal may simply be inlined differently, so every hit is listed for reading.
    """
    vals = {}
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                s = line.strip()
                if s.startswith(("#", "//", "*")):
                    continue
                for m in NUM.finditer(s):
                    try:
                        v = float(m.group(1))
                    except ValueError:
                        continue
                    if v in (0.0, 1.0, 2.0, 3.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0):
                        continue
                    vals[m.group(1)] = vals.get(m.group(1), 0) + 1
    except OSError:
        pass
    return vals


def build_python_index():
    idx = {}
    for base in SEARCH_DIRS:
        for dirpath, dirnames, files in os.walk(base):
            dirnames[:] = [d for d in dirnames if d not in ("__pycache__", "node_modules", "target")]
            for f in files:
                if f.endswith(".py") and not f.endswith(".voltage_bak"):
                    stem = f[:-3].lower()
                    idx.setdefault(stem, os.path.join(dirpath, f))
                    for suf in SUFFIXES:
                        if stem.endswith(suf):
                            idx.setdefault(stem[: -len(suf)], os.path.join(dirpath, f))
    return idx


def match(stem, idx):
    """Exact stem, or the stem with one known suffix removed -- and nothing else.

    An earlier version fell back to substring containment, which paired `federation_sync_upgrade.rs`,
    `federation_sram_workload_orchestrator.rs` and `hybrid_federation_deployment.rs` all with the same
    `federation_*.py`, and reported a coverage ratio for each of them. A false pair is worse than no
    pair: it produces a confident number about two files that have nothing to do with each other.
    """
    s = stem.lower()
    if s in idx:
        return s, idx[s], "exact"
    for suf in SUFFIXES:
        if s.endswith(suf) and s[: -len(suf)] in idx:
            return s[: -len(suf)], idx[s[: -len(suf)]], "suffix"
    return None, None, "none"


def main():
    idx = build_python_index()
    report = {"trees": {}, "python_modules_available": len(idx)}
    for tree, d in TREES.items():
        if not os.path.isdir(d):
            continue
        rows = []
        for f in sorted(os.listdir(d)):
            if not f.endswith(".rs"):
                continue
            stem = f[:-3]
            rs = os.path.join(d, f)
            extra = {}
            pstem, ppath, match_kind = match(stem, idx)
            rs_loc = code_lines(rs, "//")
            rsf = fn_names(rs, "rs")
            if not ppath:
                # A renamed port is still a port. Pair on the behaviour surface instead of the file
                # name: the Python module sharing the most function names is the best candidate, and
                # the overlap fraction is reported so a weak pairing is visible rather than assumed.
                # Require real shared surface, not a flattering ratio: `inter / min(...)` let a
                # 15-line Python script with 2 functions "pair" at 50% with a 350-line, 28-function
                # module. Rank by the number of shared names and demand at least three.
                best_key, best_path, best_overlap, best_inter = None, None, 0.0, 0
                for key, path in idx.items():
                    pyf = fn_names(path, "py")
                    if len(pyf) < 3 or not rsf:
                        continue
                    inter = len(pyf & rsf)
                    if inter < 3:
                        continue
                    ov = inter / min(len(pyf), len(rsf))
                    if inter > best_inter or (inter == best_inter and ov > best_overlap):
                        best_key, best_path, best_overlap, best_inter = key, path, ov, inter
                if best_path and best_overlap >= 0.5:
                    ppath, match_kind = best_path, "fn-overlap"
                    extra = {"fn_overlap": round(best_overlap, 3),
                             "fn_shared": best_inter, "candidate_stem": best_key}
            if not ppath:
                rows.append({"module": f, "python": None, "rs_loc": rs_loc, "rs_fns": len(rsf),
                             "note": "no python counterpart found (may be rust-native by design)"})
                continue
            py_loc = code_lines(ppath, "#")
            pyf = fn_names(ppath, "py")
            missing = sorted(pyf - rsf)
            py_num, rs_num = numeric_literals(ppath), numeric_literals(rs)
            drift = sorted(set(py_num) - set(rs_num))
            rows.append({
                "module": f, "match_kind": match_kind, **extra,
                "python": os.path.relpath(ppath, ROOT).replace("\\", "/"),
                "rs_loc": rs_loc, "py_loc": py_loc, "rs_fns": len(rsf), "py_fns": len(pyf),
                "port_ratio": round(rs_loc / py_loc, 3) if py_loc else None,
                "fn_ratio": round(len(rsf) / len(pyf), 3) if pyf else None,
                "fns_missing_in_rust": missing[:12], "fns_missing_count": len(missing),
                "constants_in_python_not_in_rust": drift[:14],
                "constants_missing_count": len(drift),
            })
        matched = [r for r in rows if r.get("python")]
        needs = [r for r in matched
                 if (r["port_ratio"] or 0) < 0.60 or (r["fn_ratio"] or 0) < 0.60]
        report["trees"][tree] = {
            "modules": len(rows), "matched_to_python": len(matched), "unmatched": len(rows) - len(matched),
            "needs_logic": len(needs),
            "worst": sorted(needs, key=lambda r: (r["port_ratio"] if r["port_ratio"] is not None else 1))[:14],
            "rows": rows,
        }
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=2, sort_keys=True)
    print("rust <-> python parity audit")
    for tree, t in report["trees"].items():
        print("  %-20s modules %3d | matched %3d | unmatched %3d | NEEDS-LOGIC %3d"
              % (tree, t["modules"], t["matched_to_python"], t["unmatched"], t["needs_logic"]))
    print()
    for tree in ("host_council_server", "rust_kernel_dll"):
        t = report["trees"].get(tree)
        if not t:
            continue
        print("== %s: worst-covered (live runtime) ==" % tree)
        for r in t["worst"][:10]:
            print("   %5s%%  %-34s %5dL/%-6dL  fns %d/%d  missing: %s"
                  % (int((r["port_ratio"] or 0) * 100), r["module"], r["rs_loc"], r["py_loc"],
                     r["rs_fns"], r["py_fns"], ", ".join(r["fns_missing_in_rust"][:3]) or "-"))
        print()
    print("report:", OUT)


if __name__ == "__main__":
    main()

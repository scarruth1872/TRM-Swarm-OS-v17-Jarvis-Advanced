"""Re-derive the Catalyst module matrix against the CURRENT sources.

The shipped classification (catalyst_phase0_classification.json) is a point-in-time
motion record: it was computed before this session's porting work. This re-measures the
same fields (rs_loc / rs_fns / py_loc / py_defs) from the files on disk now, reports the
delta per module, and separates what is genuinely still unported from what has since been
covered. Thresholds are read from the artifact, not retyped.
"""
import ast
import collections
import json
import os
import re
import time

ROOT = r"C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main"
ART = os.path.join(ROOT, "swarm_v2_artifacts")
CLASSIFICATION = os.path.join(ART, "catalyst_phase0_classification.json")
OUT_JSON = os.path.join(ART, "module_matrix_current.json")
OUT_MD = os.path.join(ART, "MODULE_MATRIX_CURRENT.md")

RS_PREFERRED = ("trm_uefi_kernel/src", "swarm_v2_rust_kernel/src")
PY_PREFERRED = ("swarm_v2/core", "swarm_v2")
SKIP_DIRS = {"target", "node_modules", ".git", "__pycache__", ".voltage_bak", "evolution_sandbox"}
FN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+\w+", re.M)


def walk_files(ext):
    for dp, dn, fn in os.walk(ROOT):
        dn[:] = [d for d in dn if d not in SKIP_DIRS]
        for f in fn:
            if f.endswith(ext):
                yield os.path.relpath(os.path.join(dp, f), ROOT).replace("\\", "/")


def codepy(path):
    """AST-based function count; None when the file will not parse."""
    try:
        tree = ast.parse(open(path, encoding="utf-8", errors="replace").read())
    except SyntaxError:
        return None
    return sum(isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) for n in ast.walk(tree))


def rank(path):
    for i, pref in enumerate(RS_PREFERRED):
        if path.startswith(pref):
            return (i, len(path))
    if "/native_voltage_strips/" in path:
        return (len(RS_PREFERRED) + 1, len(path))
    return (len(RS_PREFERRED), len(path))


def build_index(name_filter, ext, preferred):
    want = {n for n in name_filter}
    idx = collections.defaultdict(list)
    for rel in walk_files(ext):
        stem = rel.rsplit("/", 1)[-1][: -len(ext)]
        if stem in want:
            idx[stem].append(rel)
    def rk(p):
        for i, pref in enumerate(preferred):
            if p.startswith(pref):
                return (i, len(p))
        return (len(preferred), len(p))
    return {k: sorted(v, key=rk)[0] for k, v in idx.items()}, {k: v for k, v in idx.items() if len(v) > 1}


def main():
    d = json.load(open(CLASSIFICATION, encoding="utf-8"))
    th = d["thresholds"]
    PORT_OK, FN_OK, PY_THIN = th["PORT_OK"], th["FN_OK"], th["PY_THIN"]
    entries = d["buckets"]["DATA-OK"] + d["buckets"]["NEEDS-LOGIC"]

    rs_idx, rs_dupes = build_index({e["module"][:-3] for e in entries}, ".rs", RS_PREFERRED)
    py_stems = {e["counterpart"][:-3] for e in entries if e.get("counterpart")}
    py_idx, py_dupes = build_index(py_stems, ".py", PY_PREFERRED)

    rows = []
    for e in entries:
        stem = e["module"][:-3]
        py_stem = e["counterpart"][:-3] if e.get("counterpart") else None
        rel_rs = rs_idx.get(stem)
        rel_py = py_idx.get(py_stem)
        row = dict(e)
        row["rs_path"] = rel_rs
        row["py_path"] = rel_py
        row["rs_dup_paths"] = rs_dupes.get(stem, [])
        row["was"] = e["registry_status"]

        if rel_rs is None:
            row.update(now="RS-MISSING", rs_loc=None, rs_fns=None, port_ratio=None, fn_ratio=None,
                       py_lines=None)
        else:
            txt = open(os.path.join(ROOT, rel_rs.replace("/", os.sep)), encoding="utf-8", errors="replace").read()
            rs_loc = txt.count("\n") + 1
            rs_fns = len(FN_RE.findall(txt))
            py_loc = e["py_lines"]
            py_defs = e["py_defs"]
            if rel_py:
                ptxt = open(os.path.join(ROOT, rel_py.replace("/", os.sep)), encoding="utf-8", errors="replace").read()
                py_loc = ptxt.count("\n") + 1
                got = codepy(os.path.join(ROOT, rel_py.replace("/", os.sep)))
                py_defs = got if got is not None else e["py_defs"]
            pr = round(rs_loc / py_loc, 4) if py_loc else None
            fr = round(rs_fns / py_defs, 4) if py_defs else None
            covered = pr is not None and pr >= PORT_OK and (fr is None or fr >= FN_OK)
            row.update(now="COVERED" if covered else "UNPORTED",
                       rs_loc=rs_loc, rs_fns=rs_fns, py_loc=py_loc, py_defs=py_defs,
                       # py_lines is the field name the shipped classification uses; emit it
                       # as an alias so a consumer reading either vocabulary gets the real
                       # number instead of silently summing a missing key to 0
                       py_lines=py_loc, port_ratio=pr, fn_ratio=fr)
        row["delta_loc"] = (row["rs_loc"] - e["lines"]) if row.get("rs_loc") is not None else None
        rows.append(row)

    # does the recorded metric agree with a plain total-line count on files we can see?
    metric_match = [r for r in rows if r.get("rs_loc") is not None and r["rs_loc"] == r["lines"]]
    moved = [r for r in rows if r.get("delta_loc")]
    now_cov = [r for r in rows if r["now"] == "COVERED"]
    still = [r for r in rows if r["now"] == "UNPORTED"]
    missing = [r for r in rows if r["now"] == "RS-MISSING"]

    still.sort(key=lambda r: -(r["py_loc"] or 0))
    summary = {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source": os.path.basename(CLASSIFICATION),
        "method": "v2 — port_ratio (rs_loc/py_loc) + fn_ratio, re-measured from disk",
        "thresholds_used": {"PORT_OK": PORT_OK, "FN_OK": FN_OK, "PY_THIN": PY_THIN},
        "analysed": len(rows),
        "was": collections.Counter(r["was"] for r in rows),
        "now": collections.Counter(r["now"] for r in rows),
        "metric_confirms_recorded_lines": f"{len(metric_match)}/{len(rows)}",
        "modules_whose_rs_changed_since_classification": len(moved),
        "still_unported": [
            {"module": r["module"], "status": r["was"], "py_loc": r["py_loc"], "rs_loc": r["rs_loc"],
             # py_lines is the field name the shipped classification uses; the alias keeps a
             # consumer that reads either vocabulary from summing a missing key to 0
             "py_lines": r["py_loc"],
             "port_ratio": r["port_ratio"], "fn_ratio": r["fn_ratio"], "py_defs": r["py_defs"],
             "rs_fns": r["rs_fns"], "rs_path": r["rs_path"], "py_path": r["py_path"]}
            for r in still],
        "now_covered": [
            {"module": r["module"], "was": r["was"], "py_loc": r["py_loc"], "rs_loc": r["rs_loc"],
             "py_lines": r["py_loc"],
             "port_ratio": r["port_ratio"], "fn_ratio": r["fn_ratio"], "rs_path": r["rs_path"]}
            for r in now_cov],
        "rs_missing": [{"module": r["module"], "was": r["was"], "py_path": r["py_path"]} for r in missing],
        "ambiguous_rs_paths": {k: v for k, v in rs_dupes.items()},
    }
    json.dump(summary, open(OUT_JSON, "w", encoding="utf-8"), indent=1, sort_keys=True)

    lines = ["# Module matrix — current (re-derived)", "",
             f"Generated {summary['generated']} from `{summary['source']}` thresholds "
             f"PORT_OK={PORT_OK} FN_OK={FN_OK} PY_THIN={PY_THIN}.", "",
             f"- analysed: **{summary['analysed']}**",
             f"- recorded: {dict(summary['was'])}",
             f"- re-measured: **{dict(summary['now'])}**",
             f"- recorded line-count confirmed on disk: {summary['metric_confirms_recorded_lines']}",
             f"- .rs files changed since classification: {len(moved)}", "",
             "## Still unported (largest unported logic first)", "",
             "| module | py_loc | rs_loc | port_ratio | py_defs | rs_fns | was |",
             "|---|---|---|---|---|---|---|"]
    for r in still:
        lines.append(f"| {r['module']} | {r['py_loc']} | {r['rs_loc']} | {r['port_ratio']} | "
                     f"{r['py_defs']} | {r['rs_fns']} | {r['was']} |")
    open(OUT_MD, "w", encoding="utf-8").write("\n".join(lines) + "\n")

    print(f"entries            : {len(rows)}")
    print(f"was                : {dict(summary['was'])}")
    print(f"re-measured        : {dict(summary['now'])}")
    print(f"lines confirmed    : {summary['metric_confirms_recorded_lines']}")
    print(f"rs changed since   : {len(moved)}")
    print(f"ambiguity (dup rs) : {len(rs_dupes)}  py dup: {len(py_dupes)}")
    print("--- still unported, top 25 by unported py_loc ---")
    for r in still[:25]:
        print(f"  {r['module']:<48} py={r['py_loc']:>5} rs={r['rs_loc']:>5} "
              f"pr={r['port_ratio']} fn={r['fn_ratio']} defs={r['py_defs']}")
    print(f"--- unported total py_loc: {sum(r['py_loc'] or 0 for r in still)}")
    print(f"wrote {OUT_JSON}")
    print(f"wrote {OUT_MD}")


main()

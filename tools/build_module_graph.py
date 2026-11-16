#!/usr/bin/env python3
"""
build_module_graph.py - regenerate module_dependency_graph.json from the trace.

WHY THIS EXISTS
The shipped module_dependency_graph.json claimed 170 modules
"SEALED_AND_VERIFIED", but its entries were FUNCTION NAMES (read_v_core,
compute_phi_v_composite). Only 43/170 resolved to a real .py file. It was an
assertion wearing a graph's clothes.

This tool rebuilds the graph from the measured trace:
  - module names are the dotted names of real files on disk
  - edges are resolved import targets (same resolver as trace_codebase.py)
  - reachability comes from evidence/MODULE_TRACE.json, not from a claim
  - every module carries its real path, so resolution is checkable

The trace JSON does not store per-module import edges, so this tool re-scans
the tree with trace_codebase.py's own scan/resolve functions and refuses to
write a graph whose counts disagree with the trace. The trace remains the
authority; the scan is how the edges are recovered.

USAGE
  python tools/build_module_graph.py
  python tools/build_module_graph.py --check   # verify, do not write

OUTPUT
  module_dependency_graph.json          regenerated
  module_dependency_graph.legacy.json   original preserved, with a banner
"""
import argparse
import datetime
import importlib.util
import json
import pathlib
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TRACE_PATH = ROOT / "evidence" / "MODULE_TRACE.json"
GRAPH_PATH = ROOT / "module_dependency_graph.json"
LEGACY_PATH = ROOT / "module_dependency_graph.legacy.json"

SUPERSEDED = {
    "superseded": {
        "status": "NOT_AUTHORITATIVE",
        "superseded_by": "docs/CODEBASE-COMPONENT-TRACE.md",
        "authoritative_source": "evidence/MODULE_TRACE.json",
        "note": (
            "Retained as errata, not deleted. This file claimed 170 modules "
            "'SEALED_AND_VERIFIED', but its entries are function names, not "
            "modules; only 43/170 resolve to a real .py file. Its numbers are "
            "not authoritative. See docs/CODEBASE-COMPONENT-TRACE.md."
        ),
    }
}


def load_trace_codebase():
    """Import tools/trace_codebase.py without requiring it to be a package."""
    path = ROOT / "tools" / "trace_codebase.py"
    spec = importlib.util.spec_from_file_location("trace_codebase", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def resolve_module_path(name: str):
    """Return the real .py file for a dotted module name, or None."""
    if name == "__init__":
        cands = [ROOT / "__init__.py"]
    else:
        rel = name.replace(".", "/")
        cands = [ROOT / (rel + ".py"), ROOT / rel / "__init__.py"]
    for cand in cands:
        if cand.is_file():
            return cand
    return None


def scan_tree(tc):
    files = [p for p in ROOT.rglob("*.py")
             if not tc.excluded(p.relative_to(ROOT))]
    recs = {}
    for p in files:
        rec = tc.scan_python(p)
        recs[rec["module"]] = rec
    return files, recs


def normalise(name: str) -> str:
    # trace_codebase maps the repo-root __init__.py to ""; a graph key cannot
    # be empty, so expose it as "__init__" (which resolves to ./__init__.py).
    return "__init__" if name == "" else name


def legacy_resolution_ratio():
    """Measure the old file the same way the gap report did: stem match."""
    if not LEGACY_PATH.exists():
        return None
    try:
        old = json.loads(LEGACY_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    names = old.get("topological_load_sequence") or []
    if not names:
        return None
    stems = {p.stem for p in ROOT.rglob("*.py")
             if not any(part in {"node_modules", ".git", "target",
                                 "__pycache__", "venv", ".venv"}
                        for part in p.parts)}
    resolved = sum(1 for n in names if n in stems)
    return resolved, len(names)


def preserve_legacy():
    """Copy the original graph to .legacy.json once, then banner it."""
    if not LEGACY_PATH.exists() and GRAPH_PATH.exists():
        try:
            old = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            old = {}
        # Only preserve a graph that is actually the old assertion. If the
        # graph is already regenerated, copying it would launder the new file
        # as "legacy".
        if old.get("status") == "SEALED_AND_VERIFIED" or \
                "topological_load_sequence" in old:
            shutil.copyfile(GRAPH_PATH, LEGACY_PATH)
    if not LEGACY_PATH.exists():
        return
    try:
        data = json.loads(LEGACY_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return
    banner = data.get("superseded")
    if isinstance(banner, dict) and \
            banner.get("status") == "NOT_AUTHORITATIVE":
        return
    # The banner is a nested field so every original field (including the old
    # "status": "SEALED_AND_VERIFIED") survives below it.
    bannered = {"superseded": SUPERSEDED["superseded"]}
    for key, value in data.items():
        if key != "superseded":
            bannered[key] = value
    LEGACY_PATH.write_text(json.dumps(bannered, indent=2) + "\n",
                           encoding="utf-8")


BANNER_MARKER = "SUPERSEDED — NOT AUTHORITATIVE"
BANNER_DOCS = ("COMPONENT_INVENTORY.md", "SYSTEM_MAP.md")


def write_banners(trace):
    """Prepend a superseded banner to the hand-maintained inventories.

    The original body is kept below the banner: errata, not deletion.
    """
    py = trace["python"]
    rust = trace["rust"]
    banner = (
        "<!-- SUPERSEDED: see docs/CODEBASE-COMPONENT-TRACE.md -->\n"
        f"> **⚠ {BANNER_MARKER}**\n"
        ">\n"
        "> This document is superseded by "
        "[`docs/CODEBASE-COMPONENT-TRACE.md`](docs/CODEBASE-COMPONENT-TRACE.md),\n"
        "> generated from `evidence/MODULE_TRACE.json` by "
        "`tools/trace_codebase.py`.\n"
        "> Its original numbers predate the current tree and are retained "
        "below as errata only.\n"
        "> Do not cite them as current. Measured now: "
        f"**{py['modules']:,} Python modules**, {py['loc']:,} LOC, "
        f"{py['routes']} routes, {rust['crates']} Rust crates.\n"
        ">\n"
        "> The content below is the original document, unchanged.\n"
        "\n---\n\n"
    )
    written = []
    for rel in BANNER_DOCS:
        path = ROOT / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        if BANNER_MARKER in text[:2000]:
            continue
        path.write_text(banner + text, encoding="utf-8")
        written.append(str(path))
    return written


def build(trace, tc, files, recs):
    py = trace["python"]
    unreachable = set(py.get("unreachable", []))
    entrypoints = set(py.get("entrypoints", []))
    dup_stems = py.get("duplicate_stems", {})
    routes_by_file = py.get("routes_by_file", {})

    index = {normalise(m) for m in recs}
    modules = {}
    edges = []
    unresolved = []

    for raw_name, rec in recs.items():
        name = normalise(raw_name)
        targets = tc.resolve(rec["imports"], index)
        targets = sorted({normalise(t) for t in targets if t != name})
        path = resolve_module_path(name)
        if path is None:
            unresolved.append(name)
        stem = name.rsplit(".", 1)[-1]
        modules[name] = {
            "path": rec["path"],
            "loc": rec["loc"],
            "defs": rec["defs"],
            "classes": rec["classes"],
            "reachable": raw_name not in unreachable,
            "entrypoint": raw_name in entrypoints,
            "imports": targets,
            "imported_by": [],
            "routes": routes_by_file.get(raw_name, rec.get("routes", [])),
            "duplicate_stem": stem in dup_stems,
            "syntax_error": rec.get("syntax_error"),
            "stub_functions": len(rec.get("stubs", [])),
        }
        for target in targets:
            edges.append([name, target])

    for src, tgt in edges:
        if tgt in modules:
            modules[tgt]["imported_by"].append(src)
    for rec in modules.values():
        rec["imported_by"].sort()

    reachable = sum(1 for m in modules.values() if m["reachable"])
    counts = {
        "modules": len(modules),
        "reachable": reachable,
        "unreachable": len(modules) - reachable,
        "edges": len(edges),
        "entrypoints": len(entrypoints),
        "routes": sum(len(m["routes"]) for m in modules.values()),
        "loc": sum(m["loc"] for m in modules.values()),
    }

    drift = {}
    for key, measured in (("files", len(files)), ("modules", len(recs)),
                          ("loc", counts["loc"]), ("reachable", reachable)):
        if py.get(key) != measured:
            drift[key] = {"trace": py.get(key), "scan": measured}
    if py.get("unreachable") is not None and \
            len(py["unreachable"]) != counts["unreachable"]:
        drift["unreachable"] = {"trace": len(py["unreachable"]),
                                "scan": counts["unreachable"]}

    return {
        "generated_by": "tools/build_module_graph.py",
        "source": "evidence/MODULE_TRACE.json",
        "source_generated_by": trace.get("generated_by"),
        "generated_at": datetime.datetime.now().astimezone().isoformat(
            timespec="seconds"),
        "status": "MEASURED_FROM_TRACE",
        "total_modules": counts["modules"],
        "counts": counts,
        "resolution": {
            "resolved": counts["modules"] - len(unresolved),
            "total": counts["modules"],
            "ratio": (counts["modules"] - len(unresolved)) /
                     counts["modules"] if counts["modules"] else 0.0,
            "unresolved": unresolved,
        },
        "trace_drift": drift,
        "modules": modules,
        "edges": sorted(edges),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="verify the existing graph, do not write")
    args = ap.parse_args()

    if not TRACE_PATH.exists():
        print(f"ERROR: trace not found: {TRACE_PATH}", file=sys.stderr)
        print("Run: python tools/trace_codebase.py", file=sys.stderr)
        return 2

    trace = json.loads(TRACE_PATH.read_text(encoding="utf-8"))

    if args.check:
        if not GRAPH_PATH.exists():
            print("ERROR: graph missing", file=sys.stderr)
            return 1
        existing = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
        # Check the artifact against the trace, not against a fresh scan: the
        # tree can change between trace runs, and the graph's contract is to
        # match the trace it names.
        unresolved = existing.get("resolution", {}).get("unresolved", [])
        ok = (existing.get("total_modules") == trace["python"]["modules"] and
              not unresolved)
        print(f"CHECK {'OK' if ok else 'FAIL'}: "
              f"{existing.get('total_modules')} modules "
              f"(trace {trace['python']['modules']}), "
              f"{len(unresolved)} unresolved")
        return 0 if ok else 1

    tc = load_trace_codebase()
    files, recs = scan_tree(tc)

    # The graph must not silently disagree with the trace it names. If the
    # tree moved since the trace was written, re-run the tracer first.
    if len(recs) != trace["python"]["modules"]:
        print(
            f"ERROR: tree has {len(recs)} modules but trace has "
            f"{trace['python']['modules']}. Re-run: python tools/trace_codebase.py",
            file=sys.stderr,
        )
        return 2

    preserve_legacy()
    before = legacy_resolution_ratio()

    graph = build(trace, tc, files, recs)
    res = graph["resolution"]

    GRAPH_PATH.write_text(json.dumps(graph, indent=2) + "\n", encoding="utf-8")
    bannered = write_banners(trace)

    if before:
        b_res, b_total = before
        print(f"BEFORE  {b_res}/{b_total} legacy entries resolve to a real .py "
              f"({100.0 * b_res / b_total:.1f}%)")
    print(f"AFTER   {res['resolved']}/{res['total']} module names resolve to a "
          f"real .py ({100.0 * res['ratio']:.1f}%)")
    print(f"MODULES {graph['counts']['modules']}  "
          f"reachable {graph['counts']['reachable']}  "
          f"unreachable {graph['counts']['unreachable']}  "
          f"edges {graph['counts']['edges']}")
    if graph["trace_drift"]:
        print(f"WARNING trace drift: {graph['trace_drift']}", file=sys.stderr)
    print(f"WROTE   {GRAPH_PATH}")
    if LEGACY_PATH.exists():
        print(f"LEGACY  {LEGACY_PATH}")
    for path in bannered:
        print(f"BANNER  {path}")
    return 0 if not res["unresolved"] else 1


if __name__ == "__main__":
    sys.exit(main())

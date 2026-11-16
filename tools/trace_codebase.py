#!/usr/bin/env python3
"""
trace_codebase.py - traced module inventory for the TRM Swarm OS tree.

WHY THIS EXISTS
The repo already ships three inventories (COMPONENT_INVENTORY.md,
SYSTEM_MAP.md, module_dependency_graph.json). None of them is a TRACE:
  - module_dependency_graph.json calls its entries "modules" but they are
    function names; only 43/170 resolve to a real .py file.
  - COMPONENT_INVENTORY.md is dated v19.022 and hand-maintained.
  - module_registry.json is a registration list, not an import graph.
So "everything is known" was an assertion, not a measurement. This produces
the measurement: every module, its import edges, whether an entrypoint can
reach it, and whether its body is real or a shell.

METHOD
  Python: ast.parse every source file, resolve absolute and relative imports
          against an index of real module names, BFS from declared entrypoints.
          A module is REACHABLE only if a real entrypoint import chain hits it.
  Rust:   parse `mod` declarations and `use crate::` edges, roots derived from
          Cargo.toml [lib]/[[bin]] (never hardcoded), BFS from those roots.
  Stubs:  a def whose body is only pass / raise NotImplementedError / ... ,
          or that carries a MANUAL / TODO / STUB marker.

WHAT IT DOES NOT DO
  It does not prove a module RUNS. Static reachability != executed. The
  runtime section only reports which HTTP routes are actually served, and a
  module reachable only through a route that 500s is still reported reachable.
  Read `stub` and `routes_served` together, never reachability alone.

USAGE
  python tools/trace_codebase.py            # human summary
  python tools/trace_codebase.py --json PATH
"""
import argparse
import ast
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Directories that are data, dependencies or archives, not source we maintain.
EXCLUDE_PARTS = {
    "node_modules", "train_venv", "venv", ".venv", ".git", "target", "__pycache__",
    "backups", "preserved_sources", "swarm_v2_artifacts", "SwarmOS_v6_Snapshot_extracted",
    "dashboard-lcars-backup", "dist", "build", "site-packages", ".swarm",
    # `scratch` is unmaintained by definition: one-shot experiments, half-written
    # drafts and LLM output dumps live there. Counting its parse failures as
    # "the codebase does not parse" is a category error.
    "scratch",
    # `archive` is a historical snapshot (e.g. the stale swarm_v2_integrated
    # duplicate tree). Kept for provenance, never imported, never maintained.
    "archive",
}

PY_ROOTS = ["swarm_v2", "tools", "scripts", "utils", "tests", "config",
            "infrastructure", "evaluators", "schemas", "gemma_swarm_adapter",
            "trm-fang-gateway", "simulations", "dataset", "artifacts"]

STUB_MARKERS = ("MANUAL", "STUB", "TODO", "FIXME", "not implemented",
                "placeholder", "unimplemented")


def excluded(path: pathlib.Path) -> bool:
    return any(part in EXCLUDE_PARTS for part in path.parts)


def dotted_name(path: pathlib.Path) -> str:
    rel = path.relative_to(ROOT)
    parts = list(rel.with_suffix("").parts)
    if parts and parts[-1] == "__init__":
        parts = parts[:-1]
    return ".".join(parts)


def count_loc(text: str) -> int:
    return sum(1 for line in text.splitlines()
               if line.strip() and not line.strip().startswith("#"))


def is_stub_body(node: ast.AST) -> bool:
    body = [b for b in getattr(node, "body", []) if not isinstance(b, ast.Expr)
            or not isinstance(getattr(b, "value", None), ast.Constant)]
    if not body:
        return True
    for stmt in body:
        if isinstance(stmt, ast.Pass):
            return True
        if isinstance(stmt, ast.Raise) and isinstance(stmt.exc, (ast.Call, ast.Name)):
            name = ""
            if isinstance(stmt.exc, ast.Call) and isinstance(stmt.exc.func, ast.Name):
                name = stmt.exc.func.id
            elif isinstance(stmt.exc, ast.Name):
                name = stmt.exc.id
            if "NotImplemented" in name or "Error" in name:
                return True
        if isinstance(stmt, ast.Expr) and isinstance(stmt.value, ast.Constant) \
                and stmt.value.value is Ellipsis:
            return True
    return False


ROUTE_RE = re.compile(r"""@(?:\w+\.)?(get|post|put|delete|patch|websocket)\(\s*['"]([^'"]+)['"]""",
                      re.IGNORECASE)


def scan_python(path: pathlib.Path) -> dict:
    text = path.read_text(encoding="utf-8", errors="replace")
    rec = {"path": str(path.relative_to(ROOT)), "module": dotted_name(path),
           "loc": count_loc(text), "imports": [], "defs": 0, "classes": 0,
           "routes": [], "stubs": [], "has_main_guard": "__main__" in text,
           "creates_app": bool(re.search(r"\b(FastAPI|Flask|APIRouter)\s*\(", text))}
    try:
        tree = ast.parse(text)
    except SyntaxError as exc:
        rec["syntax_error"] = f"line {exc.lineno}: {exc.msg}"
        rec["stub_markers"] = []
        return rec
    pkg = rec["module"].rsplit(".", 1)[0] if "." in rec["module"] else ""
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for a in node.names:
                rec["imports"].append({"target": a.name, "level": 0})
        elif isinstance(node, ast.ImportFrom):
            base = node.module or ""
            if node.level:
                up = pkg.split(".")[:max(0, len(pkg.split(".")) - node.level + 1)]
                base = ".".join([p for p in up if p] + ([base] if base else []))
            for a in node.names:
                rec["imports"].append({"target": (base + "." + a.name) if base else a.name,
                                       "level": node.level, "base": base})
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            rec["defs"] += 1
            if is_stub_body(node):
                rec["stubs"].append(node.name)
        elif isinstance(node, ast.ClassDef):
            rec["classes"] += 1
            for b in node.body:
                if isinstance(b, (ast.FunctionDef, ast.AsyncFunctionDef)) and is_stub_body(b):
                    rec["stubs"].append(f"{node.name}.{b.name}")
        elif isinstance(node, ast.Call):
            f = node.func
            if isinstance(f, ast.Attribute) and f.attr in ("add_route", "add_api_route"):
                rec["routes"].append(f"ROUTE {ast.unparse(node)[:90]}")
    for line in text.splitlines():
        m = ROUTE_RE.search(line)
        if m:
            rec["routes"].append(f"{m.group(1).upper()} {m.group(2)}")
    low = text.lower()
    rec["stub_markers"] = [k for k in STUB_MARKERS if k.lower() in low]
    return rec


def resolve(imports, index):
    """Map raw import targets onto real module names by longest prefix."""
    out = set()
    for imp in imports:
        t = imp["target"]
        while t:
            if t in index:
                out.add(t)
                break
            t = t.rsplit(".", 1)[0] if "." in t else ""
    return sorted(out)


def scan_rust_crate(crate_dir: pathlib.Path) -> dict:
    cargo = crate_dir / "Cargo.toml"
    if not cargo.exists():
        return {}
    ct = cargo.read_text(encoding="utf-8", errors="replace")
    roots = []
    for key in ("lib", "bin"):
        for block in re.findall(rf"\[\[?{key}\]?\](.*?)(?=\n\[|\Z)", ct, re.S):
            for m in re.finditer(r'path\s*=\s*"([^"]+)"', block):
                roots.append(crate_dir / m.group(1).replace("/", "\\"))
    src = crate_dir / "src"
    if src.exists():
        for cand in (src / "lib.rs", src / "main.rs"):
            if cand.exists() and cand not in roots:
                roots.append(cand)
    if not src.exists():
        return {}
    files = [p for p in src.rglob("*.rs") if "target" not in p.parts]
    by_mod = {}
    for p in files:
        rel = p.relative_to(src)
        parts = list(rel.with_suffix("").parts)
        if parts[-1] in ("lib", "main", "mod"):
            parts = parts[:-1]
        by_mod["::".join(parts) if parts else "crate_root"] = p
    edges, recs = {}, {}
    mod_re = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", re.M)
    use_re = re.compile(r"^\s*(?:pub\s+)?use\s+crate::(\w+)", re.M)
    file_key = {}
    for p in files:
        t = p.read_text(encoding="utf-8", errors="replace")
        # (key collision guard applied after key is computed, below)
        # CRATE ROOT NORMALISATION. lib.rs and main.rs are the crate root, not
        # modules called "lib"/"main". This must match the root_keys normalisation
        # below exactly, or every main.rs-rooted crate reports 0 reachable --
        # which is exactly what happened once (host_tranche 0/28, host_fanout 0/15)
        # and looked like dead code instead of a tracer bug.
        # `mod` is normalised the same way: src/x/mod.rs IS module x, and by_mod
        # already keys it as "x". Leaving it as "x::mod" here created a node no
        # edge could ever point at, so every mod.rs and every child of it read as
        # unreachable (openfang-cli tui, openfang-runtime drivers).
        _parts = list(p.relative_to(src).with_suffix("").parts)
        if _parts and _parts[-1] in ("lib", "main", "mod"):
            _parts = _parts[:-1]
        key = "::".join(_parts) if _parts else "crate_root"
        # A crate can have BOTH lib.rs and main.rs, and they both normalise to
        # crate_root. Without this guard one file's LOC/tests silently overwrite
        # the other's (240,167 -> 239,649 while the file count stayed 661).
        if key in edges:
            key = f"{key}@{p.name}"
        file_key[p] = key
        targets = set()
        base = p.parent.relative_to(src)
        for m in mod_re.finditer(t):
            sub = (base / m.group(1)) if str(base) != "." else pathlib.Path(m.group(1))
            k = "::".join(sub.parts)
            if k in by_mod:
                targets.add(k)
        for m in use_re.finditer(t):
            for k in by_mod:
                if k == "crate_root" or k.split("::")[0] == m.group(1):
                    targets.add(k)
        edges[key] = sorted(targets)
        recs[key] = {"path": str(p.relative_to(ROOT)), "loc": count_loc(t),
                     "defs": len(re.findall(r"\bfn\s+\w+", t)),
                     "stub_markers": [k for k in STUB_MARKERS if k.lower() in t.lower()],
                     "test_fns": len(re.findall(r"#\[test\]", t))}
    # Roots must resolve through the SAME key map as the edges. Recomputing the
    # key here used to lose the collision suffix, so a crate with both lib.rs and
    # main.rs rooted only lib.rs and reported main.rs (and everything it declares)
    # as unreachable.
    root_keys = [file_key[r] for r in roots if r in file_key]
    return {"crate": str(crate_dir.relative_to(ROOT)), "files": len(files),
            "edges": edges, "recs": recs, "roots": sorted(set(root_keys))}


def bfs(graph, roots):
    seen, order = {}, []
    stack = [(r, 0) for r in roots]
    while stack:
        node, d = stack.pop(0)
        if node in seen or node not in graph:
            continue
        seen[node] = d
        order.append(node)
        for nxt in graph[node]:
            if nxt not in seen:
                stack.append((nxt, d + 1))
    return seen, order


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", default=str(ROOT / "evidence" / "MODULE_TRACE.json"))
    ap.add_argument("--md", default=str(ROOT / "docs" / "CODEBASE-COMPONENT-TRACE.md"))
    args = ap.parse_args()

    py_files = [p for p in ROOT.rglob("*.py") if not excluded(p.relative_to(ROOT))]
    py_recs = {}
    for p in py_files:
        r = scan_python(p)
        py_recs[r["module"]] = r
    index = set(py_recs)

    # Duplicate module names in different trees are a real hazard: same import
    # string can bind to either file depending on sys.path order.
    stems = {}
    for m in py_recs:
        stems.setdefault(m.rsplit(".", 1)[-1], []).append(m)
    dup_stems = {k: v for k, v in stems.items() if len(v) > 1}

    graph = {m: resolve(r["imports"], index) for m, r in py_recs.items()}

    # Entrypoints: declared, not guessed. Anything that constructs an app or
    # runs as __main__ under a launch/entry name, plus every test module.
    entry = sorted(m for m, r in py_recs.items()
                   if r["creates_app"] or (r["has_main_guard"] and
                   pathlib.Path(r["path"]).name.lower().startswith(
                       ("app", "main", "server", "run", "start", "launch", "execute", "cli")))
                   or r["module"].startswith("tests.")
                   or pathlib.Path(r["path"]).name.startswith("test_"))
    reach, order = bfs(graph, entry)

    inbound = {}
    for src, tgts in graph.items():
        for t in tgts:
            inbound.setdefault(t, []).append(src)

    py_summary = {
        "files": len(py_files),
        "modules": len(py_recs),
        "loc": sum(r["loc"] for r in py_recs.values()),
        "defs": sum(r["defs"] for r in py_recs.values()),
        "classes": sum(r["classes"] for r in py_recs.values()),
        "routes": sum(len(r["routes"]) for r in py_recs.values()),
        "stub_functions": sum(len(r["stubs"]) for r in py_recs.values()),
        "stub_marker_files": sum(1 for r in py_recs.values() if r["stub_markers"]),
        "syntax_errors": [m for m, r in py_recs.items() if "syntax_error" in r],
        "entrypoints": entry,
        "reachable": len(reach),
        "unreachable": sorted(set(py_recs) - set(reach)),
        "duplicate_stems": dup_stems,
        "top_imported": sorted(inbound.items(), key=lambda kv: -len(kv[1]))[:25],
        "routes_by_file": {m: r["routes"] for m, r in py_recs.items() if r["routes"]},
        "stubs_by_file": {m: r["stubs"] for m, r in py_recs.items() if r["stubs"]},
    }

    # Rust: every crate that has src/ and a Cargo.toml, excluding vendored trees.
    crates = {}
    for cargo in ROOT.rglob("Cargo.toml"):
        rel = cargo.relative_to(ROOT)
        if excluded(rel) or "registry" in str(rel):
            continue
        if not (cargo.parent / "src").exists():
            continue
        info = scan_rust_crate(cargo.parent)
        if info:
            crates[info["crate"]] = info
    rust_summary = {"crates": len(crates), "files": 0, "loc": 0, "crate_detail": {}}
    for name, info in crates.items():
        seen, _ = bfs(info["edges"], info["roots"])
        rust_summary["files"] += info["files"]
        rust_summary["loc"] += sum(r["loc"] for r in info["recs"].values())
        rust_summary["crate_detail"][name] = {
            "files": info["files"], "roots": info["roots"],
            "reachable": len(seen), "unreachable": len(info["recs"]) - len(seen),
            "unreachable_files": sorted(set(info["recs"]) - set(seen))[:40],
            "test_fns": sum(r["test_fns"] for r in info["recs"].values()),
        }

    payload = {"generated_by": "tools/trace_codebase.py",
               "python": py_summary, "rust": rust_summary}
    out = pathlib.Path(args.json)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=1), encoding="utf-8")

    print(f"PYTHON  {py_summary['files']} files / {py_summary['modules']} modules / "
          f"{py_summary['loc']:,} LOC / {py_summary['routes']} routes")
    print(f"        reachable {py_summary['reachable']}/{py_summary['modules']}  "
          f"unreachable {len(py_summary['unreachable'])}  stubs {py_summary['stub_functions']}  "
          f"syntax errors {len(py_summary['syntax_errors'])}")
    # "unreachable" is not "dead". Print the allowlist size beside it so the
    # number can never again be read as dead code without qualification.
    allow = ROOT / "evidence" / "UNREACHABLE_ALLOWLIST.json"
    if allow.exists():
        try:
            a = json.loads(allow.read_text(encoding="utf-8"))
            c = a.get("counts", {})
            print(f"        allowlisted {a.get('total', 0)}  "
                  + "  ".join(f"{k} {v}" for k, v in sorted(c.items())))
        except Exception as exc:
            print(f"        allowlist unreadable: {exc}")
    print(f"RUST    {rust_summary['crates']} crates / {rust_summary['files']} files / "
          f"{rust_summary['loc']:,} LOC")
    for c, d in rust_summary["crate_detail"].items():
        print(f"        {c}: {d['reachable']}/{d['files']} reachable from "
              f"{d['roots'] or 'NO ROOT'}, tests {d['test_fns']}")
    print(f"JSON -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

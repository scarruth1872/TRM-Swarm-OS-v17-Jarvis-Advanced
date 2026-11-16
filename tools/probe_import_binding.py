#!/usr/bin/env python3
"""G5 — import-binding probe for duplicate module stems.

The static trace (tools/trace_codebase.py) reports a "duplicate stem" whenever
two source files share a final dotted component (``memory``, ``models``, ...).
That is a *potential* hazard, not a measured one: the hazard only exists if the
interpreter can bind the same import string to two different files. This probe
answers the measured question instead:

    for each entrypoint, which file is actually in sys.modules for every
    duplicated stem, what would a bare ``import <stem>`` bind, and does the
    bound file match the path its dotted name implies?

It imports each entrypoint in a *subprocess* (so the probe's own sys.modules is
never polluted), records every repo module's ``__file__``, and prints a table.

Exit status
-----------
0  no duplicate stem outside the recorded baseline
1  a NEW duplicate stem appeared (not in the baseline) — the regression gate
2  an entrypoint failed to import

Baseline
--------
``evidence/duplicate_stems_baseline.json`` holds the stems that were known when
the probe was last accepted. ``--write-baseline`` rewrites it from the current
measurement. After G5 remediation the baseline is empty, so *any* duplicate
fails the probe.

Usage
-----
    python tools/probe_import_binding.py
    python tools/probe_import_binding.py --write-baseline
    python tools/probe_import_binding.py --strict      # fail on any duplicate
    python tools/probe_import_binding.py --json out.json
"""
from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Same exclusion set as tools/trace_codebase.py so the two tools measure the
# same surface. archive/ and scratch/ are allowed to keep duplicates.
EXCLUDE_PARTS = {
    "node_modules", "train_venv", "venv", ".venv", ".git", "target", "__pycache__",
    "backups", "preserved_sources", "swarm_v2_artifacts", "SwarmOS_v6_Snapshot_extracted",
    "dashboard-lcars-backup", "dist", "build", "site-packages", ".swarm",
    "scratch", "archive",
}

BASELINE_PATH = ROOT / "evidence" / "duplicate_stems_baseline.json"

# Entrypoints that can be imported without side effects. launcher.py and
# startup.py are deliberately excluded: importing them spawns child processes
# and blocks, so they are process launchers, not importable entrypoints.
ENTRYPOINTS = [
    ("swarm_v2.app_v2", "swarm_v2.app_v2"),
    ("app.py", "app"),
    ("microkernel_proxy.py", "microkernel_proxy"),
    ("swarm_manager.py", "swarm_manager"),
]

# The 10 stems measured by the 2026-11-03 trace. Used only to seed the
# baseline when the file does not exist yet.
SEED_BASELINE = [
    "finetune_gemma", "common", "mutation_engine", "bridge", "memory",
    "remediation_engine", "remediation", "telemetry", "core", "models",
]

CHILD = r"""
import importlib, importlib.util, json, os, sys

mod = sys.argv[1]
stems = json.loads(sys.argv[2])
importlib.import_module(mod)

root = os.path.realpath(os.getcwd())
mods = {}
for name, m in list(sys.modules.items()):
    f = getattr(m, "__file__", None)
    if not f:
        continue
    try:
        p = os.path.realpath(f)
    except Exception:
        continue
    if not p.startswith(root + os.sep) or not os.path.isfile(p):
        continue
    mods[name] = os.path.relpath(p, root).replace(os.sep, "/")

bare = {}
for s in stems:
    try:
        spec = importlib.util.find_spec(s)
        origin = getattr(spec, "origin", None) if spec else None
        if origin and os.path.isfile(origin):
            bare[s] = os.path.relpath(os.path.realpath(origin), root).replace(os.sep, "/")
        else:
            bare[s] = None
    except Exception:
        bare[s] = None

print("__PROBE_JSON__" + json.dumps({"modules": mods, "bare": bare}))
"""


def excluded(rel: pathlib.Path) -> bool:
    return any(part in EXCLUDE_PARTS for part in rel.parts)


def disk_candidates() -> dict[str, list[str]]:
    """stem -> sorted relative paths of every source file with that stem."""
    out: dict[str, list[str]] = {}
    for p in ROOT.rglob("*.py"):
        rel = p.relative_to(ROOT)
        if excluded(rel):
            continue
        # A package's importable stem is its directory name, not "__init__".
        stem = p.parent.name if p.name == "__init__.py" else p.stem
        if stem == "__init__":
            continue
        out.setdefault(stem, []).append(rel.as_posix())
    for v in out.values():
        v.sort()
    return out


def implied_path(dotted: str) -> str:
    """The file a dotted name implies, relative to ROOT."""
    base = dotted.replace(".", "/")
    for cand in (base + ".py", base + "/__init__.py"):
        if (ROOT / cand).is_file():
            return cand
    return base + ".py"


def run_entrypoint(label: str, module: str, stems: list[str]) -> dict:
    env = dict(os.environ)
    env["PYTHONPATH"] = str(ROOT) + os.pathsep + env.get("PYTHONPATH", "")
    env["PYTHONUNBUFFERED"] = "1"
    proc = subprocess.run(
        [sys.executable, "-c", CHILD, module, json.dumps(stems)],
        cwd=str(ROOT), env=env, capture_output=True, text=True, timeout=300,
    )
    payload = None
    for line in proc.stdout.splitlines():
        if line.startswith("__PROBE_JSON__"):
            payload = json.loads(line[len("__PROBE_JSON__"):])
    if payload is None:
        return {"label": label, "module": module, "ok": False,
                "error": (proc.stderr or proc.stdout)[-2000:], "modules": {}, "bare": {}}
    payload.update({"label": label, "module": module, "ok": proc.returncode == 0})
    return payload


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write-baseline", action="store_true",
                    help="record the current duplicate set as the accepted baseline")
    ap.add_argument("--strict", action="store_true",
                    help="exit non-zero if ANY duplicate stem exists")
    ap.add_argument("--json", metavar="PATH", help="also write the full result as JSON")
    args = ap.parse_args()

    candidates = disk_candidates()
    disk_dups = {s: v for s, v in candidates.items() if len(v) > 1}
    stems = sorted(disk_dups)

    if BASELINE_PATH.is_file():
        baseline = set(json.loads(BASELINE_PATH.read_text())["stems"])
    else:
        baseline = set(SEED_BASELINE)

    results = [run_entrypoint(label, mod, stems) for label, mod in ENTRYPOINTS]

    # Union of live modules across entrypoints, keyed by dotted name.
    live: dict[str, str] = {}
    for r in results:
        for name, path in r["modules"].items():
            live.setdefault(name, path)

    print("=" * 100)
    print("G5 IMPORT-BINDING PROBE — duplicate module stems")
    print(f"repo: {ROOT}")
    print(f"entrypoints: {', '.join(m for _, m in ENTRYPOINTS)}")
    print(f"duplicate stems on disk (measured surface): {len(disk_dups)}")
    print("=" * 100)

    rows = []
    for stem in stems:
        live_hits = sorted((n, p) for n, p in live.items() if n.split(".")[-1] == stem)
        bare = next((r["bare"].get(stem) for r in results if stem in r["bare"]), None)
        others = [p for p in disk_dups[stem] if p not in {p2 for _, p2 in live_hits}]
        implied_ok = all(p == implied_path(n) for n, p in live_hits) if live_hits else None
        rows.append({
            "stem": stem,
            "live": live_hits,
            "bare": bare,
            "others": others,
            "implied_ok": implied_ok,
            "new": stem not in baseline,
        })

    for row in rows:
        flag = "NEW" if row["new"] else "known"
        print(f"\n[{flag}] stem: {row['stem']}")
        if row["live"]:
            for name, path in row["live"]:
                ok = "ok" if path == implied_path(name) else "MISMATCH"
                print(f"    bound   {name:<45} {path}   [{ok}]")
        else:
            print("    bound   <not imported by any entrypoint>")
        print(f"    bare    import {row['stem']:<28} -> {row['bare'] or '<no top-level module>'}")
        for p in row["others"]:
            print(f"    other   {p}")
        if row["implied_ok"] is False:
            print("    WARNING: a bound module's __file__ does not match its dotted name")

    new_dups = sorted(r["stem"] for r in rows if r["new"])
    print("\n" + "-" * 100)
    print(f"duplicate stems: {len(rows)}   new since baseline: {len(new_dups)}"
          + (f" ({', '.join(new_dups)})" if new_dups else ""))
    print(f"baseline: {BASELINE_PATH.relative_to(ROOT)} ({len(baseline)} stems)")

    if args.json:
        pathlib.Path(args.json).write_text(json.dumps(
            {"duplicates": rows, "baseline": sorted(baseline),
             "entrypoints": [{"label": r["label"], "module": r["module"], "ok": r["ok"]}
                             for r in results]}, indent=2))

    if args.write_baseline:
        BASELINE_PATH.parent.mkdir(parents=True, exist_ok=True)
        BASELINE_PATH.write_text(json.dumps(
            {"generated_by": "tools/probe_import_binding.py",
             "stems": sorted(disk_dups)}, indent=2) + "\n")
        print(f"baseline written: {len(disk_dups)} stems")
        return 0

    failed = [r for r in results if not r["ok"]]
    if failed:
        for r in failed:
            print(f"\nENTRYPOINT FAILED: {r['module']}\n{r['error']}")
        return 2
    if new_dups:
        print("FAIL: new duplicate stem(s) not in the baseline")
        return 1
    if args.strict and rows:
        print("FAIL (--strict): duplicate stems still present")
        return 1
    print("PASS: no new duplicate stems")
    return 0


if __name__ == "__main__":
    sys.exit(main())

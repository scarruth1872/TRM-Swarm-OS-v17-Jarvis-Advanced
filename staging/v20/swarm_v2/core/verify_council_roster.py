#!/usr/bin/env python3
"""Council roster parity gate — detects drift between the roster definitions.

The platform carries several roster definitions that disagree, plus several
unrelated agent *populations* that must not be conflated with the council:

  council rosters (compared against the oracle):
    agent_intelligence_bridge.AGENT_MODELS       15   canonical
    trm_uefi_kernel  agent_runtime.rs            14   -LogicMaster
    swarm_v2_rust_kernel agent_runtime.rs        14   -LogicMaster
    swarm_v2_rust_kernel council.rs              14   -LogicMaster
    swarm_v2 swarm_council.py                    14   -LogicMaster
    swarm_v2 trm_council_models.py (L45 dict)    14   +LogicMaster, -Verify  <- third variant
  other populations (informational — NOT the council size):
    agent_mailbox_bus MAX_AGENTS = 64     pbft_consensus N_AGENTS = 64
    daily_evolution   AGENTS = 584 / 64   coherence_thresholds AGENTS = 32
    native_mailbox_bridge MAX_AGENTS = 15  (a C++ protocol constant, not a roster)

Read-only. Never edits a roster; never touches the kernel unless --live (a GET).

Exit codes: 0 consistent, 1 drift, 2 could-not-run.
"""
from __future__ import annotations

import argparse
import ast
import json
import os
import re
import sys
import urllib.request

ROOT = os.getcwd()
NAME_RE = re.compile(r'name\s*[:=]\s*"([A-Za-z]\w*)"')

# label -> (path, extractor, arg, hard?)
ROSTERS = {
    "oracle:agent_intelligence_bridge.AGENT_MODELS": ("swarm_v2/core/agent_intelligence_bridge.py", "py_dict", "AGENT_MODELS", True),
    "kernel:trm_uefi_kernel/agent_runtime.rs": ("trm_uefi_kernel/src/agent_runtime.rs", "rs_enum", "SwarmAgentId", True),
    "kernel:swarm_v2_rust_kernel/agent_runtime.rs": ("swarm_v2_rust_kernel/src/agent_runtime.rs", "rs_enum", "SwarmAgentId", True),
    "dll:swarm_v2_rust_kernel/council.rs": ("swarm_v2_rust_kernel/src/council.rs", "names", None, True),
    "py:swarm_v2/swarm_council.py": ("swarm_v2/core/swarm_council.py", "names", None, True),
    "models:swarm_v2/trm_council_models.py": ("swarm_v2/core/trm_council_models.py", "py_keyed", None, True),
    "server:trm_uefi_kernel/host_council_server/src/main.rs": ("trm_uefi_kernel/host_council_server/src/main.rs", "names", None, True),
}
# only these files' agent-count constant IS the council size
ROSTER_COUNT_FILES = ("agent_runtime.rs", "council.rs")
NON_COUNCIL_COUNTS = ("MAX_AGENTS", "N_AGENTS")
COUNT_RE = re.compile(r"(MAX_AGENTS|NUM_AGENTS|N_AGENTS|AGENTS)\s*[:=][^,\n;)]{0,24}?(\d+)")
SCAN_ROOTS = ("trm_uefi_kernel/src", "swarm_v2_rust_kernel/src")


def _read(rel):
    p = os.path.join(ROOT, rel)
    if not os.path.exists(p):
        return None
    with open(p, encoding="utf-8", errors="replace") as fh:
        return fh.read()


def extract_py_dict(text, var):
    for node in ast.parse(text).body:
        if isinstance(node, ast.Assign) and any(getattr(t, "id", "") == var for t in node.targets):
            return list(ast.literal_eval(node.value))
    return None


def extract_py_keyed(text, _arg=None):
    """Largest str-keyed dict in the module (the agent model table)."""
    best = []
    for node in ast.walk(ast.parse(text)):
        if isinstance(node, ast.Dict) and len(node.keys) >= 8:
            keys = [k.value for k in node.keys if isinstance(k, ast.Constant) and isinstance(k.value, str)]
            if len(keys) > len(best):
                best = keys
    return best or None


def extract_rs_enum(text, var):
    m = re.search(r"pub enum %s\s*\{(.*?)\n\}" % re.escape(var), text, re.S)
    return re.findall(r"(\w+)\s*=\s*\d+", m.group(1)) if m else None


def load_rosters():
    out, missing = {}, []
    for label, (rel, kind, arg, hard) in ROSTERS.items():
        text = _read(rel)
        if text is None:
            missing.append(rel)
            continue
        got = (extract_py_dict(text, arg) if kind == "py_dict"
               else extract_py_keyed(text, arg) if kind == "py_keyed"
               else extract_rs_enum(text, arg) if kind == "rs_enum"
               else NAME_RE.findall(text))
        out[label] = (rel, got, hard)
    return out, missing


def scan_counts():
    """(council-size constants, other-population constants)."""
    council, other = [], []
    for base in SCAN_ROOTS:
        for dirpath, dirnames, files in os.walk(os.path.join(ROOT, base)):
            dirnames[:] = [d for d in dirnames if d != "target"]
            for fname in sorted(files):
                if not fname.endswith(".rs"):
                    continue
                p = os.path.join(dirpath, fname)
                with open(p, encoding="utf-8", errors="replace") as fh:
                    text = fh.read()
                rel = os.path.relpath(p, ROOT)
                for m in COUNT_RE.finditer(text):
                    line = text[: m.start()].count("\n") + 1
                    if text.splitlines()[line - 1].strip().startswith("//"):
                        continue
                    rec = (rel, m.group(1), int(m.group(2)), line)
                    if fname in ROSTER_COUNT_FILES and m.group(1) not in NON_COUNCIL_COUNTS:
                        council.append(rec)
                    else:
                        other.append(rec)
    return council, other


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--live", action="store_true", help="also compare the running kernel")
    ap.add_argument("--url", default="http://127.0.0.1:8022")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    rosters, missing = load_rosters()
    if missing:
        print("cannot run: missing roster sources: %s" % missing, file=sys.stderr)
        return 2
    oracle_label = next(iter(ROSTERS))
    oracle = set(rosters[oracle_label][1] or [])
    problems, warnings, report = [], [], {"oracle": sorted(oracle), "rosters": {}, "live": None}

    for label, (rel, names, hard) in rosters.items():
        if not names:
            warnings.append("%s: NOT SCANNABLE (%s) — shape not recognised" % (label, rel))
            report["rosters"][label] = {"file": rel, "count": None, "missing": [], "extra": [],
                                        "hard": hard, "unscannable": True}
            continue
        drops, adds = sorted(oracle - set(names)), sorted(set(names) - oracle)
        report["rosters"][label] = {"file": rel, "count": len(names), "missing": drops,
                                    "extra": adds, "hard": hard}
        bucket = problems if hard else warnings
        if drops:
            bucket.append("%s: missing %s" % (label, drops))
        if adds:
            bucket.append("%s: has names the oracle lacks: %s" % (label, adds))

    council_counts, other_counts = scan_counts()
    # each tree's agent_runtime.rs must agree with that tree's own roster
    per_tree = {}
    for label, (rel, names, _h) in rosters.items():
        if names and rel.endswith("/agent_runtime.rs"):
            per_tree["trm_uefi_kernel" if rel.startswith("trm_uefi") else "swarm_v2_rust_kernel"] = len(names)
    for rel, name, val, line in council_counts:
        tree = rel.replace("\\", "/").split("/")[0]
        want = per_tree.get(tree)
        if want is not None and val != want:
            problems.append("%s L%d: %s = %d but that tree's roster has %d" % (rel, line, name, val, want))

    if args.live:
        try:
            with urllib.request.urlopen(args.url + "/council/status", timeout=10) as r:
                live = json.loads(r.read())
            names = [m.get("name") for m in live.get("members", [])]
            stats = live.get("stats", {})
            n, q = stats.get("total_agents"), stats.get("pbft_quorum")
            report["live"] = {"total_agents": n, "pbft_quorum": q, "members": names}
            if n != len(oracle):
                problems.append("live kernel reports %s agents; the oracle has %d" % (n, len(oracle)))
            for a in sorted(oracle - set(names)):
                problems.append("live kernel roster is missing %s" % a)
            if isinstance(n, int) and isinstance(q, int):
                f = (n - 1) // 3
                if q != 2 * f + 1:
                    problems.append("live pbft_quorum = %d, but 2f+1 for N=%d is %d" % (q, n, 2 * f + 1))
        except Exception as exc:
            print("cannot run: --live but %s/council/status failed: %s" % (args.url, exc), file=sys.stderr)
            return 2

    if args.json:
        report["problems"], report["warnings"] = problems, warnings
        report["council_counts"], report["other_populations"] = council_counts, other_counts
        print(json.dumps(report, indent=1))
    else:
        print("oracle roster (%d): %s\n" % (len(oracle), sorted(oracle)))
        for label, info in report["rosters"].items():
            if info.get("unscannable"):
                cnt, tag = " --", "UNSCANNABLE (not a clean bill)"
            else:
                cnt = "%2d" % info["count"] if isinstance(info["count"], int) else " ?"
                tag = ("missing " + str(info["missing"])) if info["missing"] else "matches"
            if info.get("extra"):
                tag = "extra %s" % info["extra"]
            print("  %-7s %-42s %s  %s" % (label.split(":")[0], label.split(":")[-1][-42:], cnt, tag))
        if report["live"]:
            print("\n  LIVE: %s agents, pbft_quorum=%s" % (report["live"]["total_agents"], report["live"]["pbft_quorum"]))
        print("\n  council-count constants:")
        for rel, name, val, line in council_counts:
            print("     %-22s L%-5d %s = %d" % (os.path.basename(rel), line, name, val))
        if other_counts:
            print("  other populations (informational, not the council):")
            for rel, name, val, line in other_counts:
                print("     %-30s L%-5d %s = %d" % (os.path.basename(rel), line, name, val))
        print()
        if warnings:
            print("WARNINGS (%d):" % len(warnings))
            for w in warnings:
                print("   - %s" % w)
        if problems:
            print("VERDICT: DRIFT (%d)" % len(problems))
            for p in problems:
                print("   - %s" % p)
            return 1
        print("VERDICT: consistent")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

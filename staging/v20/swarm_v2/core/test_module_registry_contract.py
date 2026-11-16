"""Contract guard for the module recognition registry.

The registry is DERIVED (swarm_v2/core/build_module_registry.py -> module_registry.json ->
trm_uefi_kernel/src/module_registry_generated.rs). Derived data drifts silently, so this
asserts the relationships rather than any particular count:

  1. every scanned module is reported exactly once (a name collision must not delete one)
  2. the published totals equal the tally of the status list, and the parts sum to it
  3. a name compiled in both trees resolves by the documented rule: an audited fabrication
     or a platform exclusion dominates; otherwise the BEST implementation is reported
  4. every module the registry calls FABRICATED still carries the signature that proves it,
     and no entry whose signature is gone is still published as FABRICATED

Run:  python swarm_v2/core/test_module_registry_contract.py     (exit 0 = pass)
"""

import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
JSON_PATH = os.path.join(ROOT, "module_registry.json")
GEN_PATH = os.path.join(ROOT, "trm_uefi_kernel", "src", "module_registry_generated.rs")

_WARN = ["FABRICATED", "EXCLUDED"]
_RANK = {"REAL": 3, "PORTED": 2, "SKELETON": 1}


def resolve_collision(copies):
    """copies: list of (tree, file, status). The registry's documented resolution rule."""
    warned = [c for c in copies if c[2] in _WARN]
    if warned:
        return max(warned, key=lambda c: -_WARN.index(c[2]))[2]
    return max(copies, key=lambda c: _RANK.get(c[2], 0))[2]


class Unrunnable(Exception):
    """Inputs absent or unparsable: not a pass, not a failure."""


def load():
    if not os.path.exists(JSON_PATH) or not os.path.exists(GEN_PATH):
        raise Unrunnable(f"registry artifacts missing: {JSON_PATH} / {GEN_PATH}")
    with open(JSON_PATH, encoding="utf-8") as fh:
        reg = json.load(fh)
    with open(GEN_PATH, encoding="utf-8") as fh:
        gen = fh.read()
    statuses = dict(re.findall(r'\("([A-Za-z0-9_]+)", "([A-Z_]+)"\)', gen))
    consts = {k: int(v) for k, v in re.findall(r"pub const ([A-Z_]+): u64 = (\d+);", gen)}
    trees = dict((a, int(b)) for a, b in re.findall(
        r'\("(\w+)", (\d+)\)', gen.split("pub const TREES")[1].split(";")[0]))
    return reg, gen, statuses, consts, trees


def check(cond, label, problems):
    print(f"  {'PASS' if cond else 'FAIL'}  {label}")
    if not cond:
        problems.append(label)


def main():
    try:
        reg, gen, statuses, consts, trees = load()
    except Unrunnable as exc:
        print(f"UNRUNNABLE: {exc}")
        return 2

    problems = []
    km = reg["kernel_modules"]
    print("=== MODULE REGISTRY CONTRACT ===")

    # 1. one entry per name, and every scanned file is represented
    check(len({m["name"] for m in km}) == len(km), "no duplicate names in the scan", problems)
    check(len(statuses) == len(km),
          f"STATUSES covers every module ({len(statuses)} == {len(km)})", problems)
    check(set(statuses) == {m["name"] for m in km},
          "STATUSES names == scanned names", problems)

    # 2. parts sum to the whole
    tally = {}
    for st in statuses.values():
        tally[st] = tally.get(st, 0) + 1
    check(consts.get("TOTAL") == len(statuses),
          f"TOTAL == len(STATUSES) ({consts.get('TOTAL')} == {len(statuses)})", problems)
    for key, name in (("REAL", "REAL"), ("PORTED", "PORTED"), ("SKELETON", "SKELETON"),
                      ("EXCLUDED", "EXCLUDED"), ("FABRICATED", "FABRICATED")):
        check(consts.get(key) == tally.get(name, 0),
              f"{key} const == tally ({consts.get(key)} == {tally.get(name, 0)})", problems)
    parts = sum(consts.get(k, 0) for k in
                ("REAL", "PORTED", "SKELETON", "EXCLUDED", "FABRICATED"))
    check(parts == consts.get("TOTAL"), f"parts sum to TOTAL ({parts})", problems)
    check(sum(trees.values()) == consts.get("TOTAL"),
          f"per-tree counts sum to TOTAL ({sum(trees.values())})", problems)

    # 3. collision resolution follows the documented rule
    bad = []
    for name, copies in reg.get("tree_collisions", {}).items():
        parsed = []
        for c in copies:
            src, _, st = c.partition("=")
            tree, _, _fn = src.partition(":")
            parsed.append((tree, src, st))
        want = resolve_collision(parsed)
        got = statuses.get(name)
        if got != want:
            bad.append(f"{name}: published {got}, rule says {want} ({copies})")
    check(not bad, f"every collision resolves by the rule ({len(reg.get('tree_collisions', {}))} checked)", problems)
    for line in bad:
        print(f"        {line}")

    # 4. fabrication claims are live, and stale ones are not published
    audit = reg.get("fabricated_audit", {})
    check(len(audit) > 0, "audit records the fabricated signatures", problems)
    for name, rec in audit.items():
        path = os.path.join(ROOT, rec["source"].replace("/", os.sep))
        live = False
        if os.path.exists(path):
            with open(path, encoding="utf-8", errors="replace") as fh:
                live = re.search(rec["signature"], fh.read()) is not None
        if rec["verified"]:
            check(live, f"{name}: FABRICATED signature still present on disk", problems)
            check(statuses.get(name) == "FABRICATED",
                  f"{name}: published as FABRICATED", problems)
        else:
            check(statuses.get(name) != "FABRICATED",
                  f"{name}: stale audit not published as FABRICATED", problems)

    # 5. the rule above must be able to fail. A stub copy in one crate must not demote a
    #    real implementation in the other -- the defect this registry was fixed for.
    synthetic = [("uefi", "uefi:main.rs", "REAL"), ("dll", "dll:main.rs", "SKELETON")]
    naive_worst = [s for _, _, s in sorted(synthetic, key=lambda c: _RANK.get(c[2], 9))][0]
    check(resolve_collision(synthetic) == "REAL",
          "stub copy does not demote a real implementation", problems)
    check(naive_worst == "SKELETON",
          "self-check: a worst-wins rule would have failed this (rule discriminates)", problems)

    print(f"--- {len(problems)} problem(s) ---")
    for p in problems:
        print(f"    {p}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

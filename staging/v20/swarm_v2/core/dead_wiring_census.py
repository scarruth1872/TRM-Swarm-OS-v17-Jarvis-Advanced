#!/usr/bin/env python3
"""Census: what the compiler says is declared but never used.

The authority is rustc, not a name search: a symbol rustc reports as `never used`
is not dispatched, full stop. The truth-core family (K3 / Kleene / trit /
consensus / PBFT / quorum / verdict / dispatch) is the one every "K3-verified"
and "council quorum" claim depends on, so it is flagged per item.

usage: default = write swarm_v2_artifacts/dead_wiring_census.json and print counts
       --check = exit 1 if any truth-core item is dead
"""
import datetime
import json
import os
import re
import subprocess
import sys
import warnings

warnings.filterwarnings("ignore")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CRATE = os.path.join(ROOT, "swarm_v2_rust_kernel")
OUT = os.path.join(ROOT, "swarm_v2_artifacts", "dead_wiring_census.json")
FAMILY = re.compile(r"k3|kleene|trit|consensus|pbft|bypass|quorum|council|dispatch|ternary|verdict", re.I)
LINE = re.compile(r"warning: (.*?(?:never used|never read))")
LOC = re.compile(r"^\s*(?:--> )?([^\s:]+):(\d+)(?::\d+)?")


def census():
    p = subprocess.run(["cargo", "check", "--release", "--message-format", "short"],
                       cwd=CRATE, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    text = (p.stdout or "") + (p.stderr or "")
    items = []
    for ln in text.splitlines():
        m = LINE.search(ln)
        if not m:
            # rustc's short format puts the location on its own line: attach it to
            # the warning above, so every census entry carries a file:line.
            loc = LOC.search(ln)
            if loc and items and items[-1]["file"] is None:
                items[-1]["file"] = loc.group(1)
                items[-1]["line"] = int(loc.group(2))
            continue
        loc = LOC.search(ln)
        items.append({
            "decl": m.group(1),
            "file": loc.group(1) if loc else None,
            "line": int(loc.group(2)) if loc else None,
            "truth_core": bool(FAMILY.search(m.group(1))),
        })
    return items, p.returncode


def main(argv):
    items, rc = census()
    fam = [i for i in items if i["truth_core"]]
    report = {
        "generated_utc": datetime.datetime.now(datetime.timezone.utc)
        .replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "source": "rustc dead-code warnings from `cargo check --release` (derived, not asserted)",
        "crate": "swarm_v2_rust_kernel",
        "cargo_rc": rc,
        "counts": {"dead_items_total": len(items), "truth_core_dead": len(fam)},
        "truth_core_dead": fam,
        "all_dead": items,
    }
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=1, sort_keys=True)
    print("dead items: %d | truth-core dead: %d" % (len(items), len(fam)))
    for i in fam:
        print("  DEAD %s:%s -- %s" % (i["file"], i["line"], i["decl"][:96]))
    print("artifact: swarm_v2_artifacts/dead_wiring_census.json")
    if "--check" in argv and fam:
        print("WIRING CENSUS: FAIL -- %d truth-core symbol(s) are declared but never used" % len(fam))
        return 1
    if "--check" in argv:
        print("WIRING CENSUS: PASS -- no truth-core symbol is dead")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

"""Compile any non-PE *.dll in a strip tree into a real loadable cdylib.

Why this exists: the strip trees can contain files named `*.dll` whose first bytes are a TRM
marker header (`TRMCDYLI...`) rather than `MZ`. They are not loadable libraries, and a Python
module that binds one silently falls back to its pure-Python path. `swarm_v2/core/verify_strip_binaries.py`
is the gate; this is the repair it points at.

Each marker has a synthesized Rust source beside it, and the sibling tree
`swarm_v2_artifacts/native_voltage_strips/` holds real PEs built with exactly this command:

    rustc --crate-type cdylib -O <src.rs> -o <out.dll>

The name mapping is the trap: sources DROP the `strip_` prefix, so `strip_agent_mailbox.dll` is
built from `agent_mailbox.rs`. Resolution tries both spellings, and an `--edition 2021` retry
covers sources that the 2015 default rejects.

A compile "succeeds" only if the output actually starts with `MZ` afterwards -- a compiler that
exits 0 having written something non-loadable is a failure, not a success.

Usage:  python swarm_v2/core/compile_strip_markers.py [TREE ...]
        (default: the strip trees under the repo)
Exit 0 when no markers remain, 1 when any do.
"""
import os, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DEFAULT_TREES = [
    os.path.join(REPO, "swarm_v2_artifacts", "native_voltage_strips"),
    os.path.join(REPO, "swarm_v2_integrated", "code", "native_voltage_strips"),
]


def is_pe(path):
    try:
        with open(path, "rb") as fh:
            return fh.read(2) == b"MZ"
    except OSError:
        return False


def source_for(tree, dll_name):
    stem = dll_name[:-4]
    for cand in (stem + ".rs", stem[len("strip_"):] + ".rs" if stem.startswith("strip_") else stem + ".rs"):
        p = os.path.join(tree, cand)
        if os.path.exists(p):
            return p
    return None


def build(tree):
    print("tree: %s" % tree, flush=True)
    if not os.path.isdir(tree):
        print("  absent")
        return 0, 0, 0
    ok = fail = nosrc = 0
    for d in sorted(f for f in os.listdir(tree) if f.lower().endswith(".dll")):
        target = os.path.join(tree, d)
        if is_pe(target):
            continue
        src = source_for(tree, d)
        if not src:
            nosrc += 1
            print("  NO SOURCE: %s" % d, flush=True)
            continue
        r = subprocess.run(["rustc", "--crate-type", "cdylib", "-O", src, "-o", target],
                           capture_output=True, text=True, cwd=tree)
        if r.returncode != 0 or not is_pe(target):
            r = subprocess.run(["rustc", "--crate-type", "cdylib", "-O", "--edition", "2021", src, "-o", target],
                               capture_output=True, text=True, cwd=tree)
        if r.returncode == 0 and is_pe(target):
            ok += 1
        else:
            fail += 1
            print("  FAIL: %s :: %s" % (d, (r.stderr or "")[:160].replace("\n", " ")), flush=True)
    remaining = sum(1 for f in os.listdir(tree) if f.lower().endswith(".dll") and not is_pe(os.path.join(tree, f)))
    print("  compiled=%d failed=%d no_source=%d | markers remaining=%d" % (ok, fail, nosrc, remaining), flush=True)
    return ok, fail, remaining


def main(argv):
    trees = argv[1:] or DEFAULT_TREES
    total_fail = 0
    for t in trees:
        _ok, _fail, remaining = build(t)
        total_fail += remaining
    print("\nmarkers remaining across all trees: %d" % total_fail)
    return 0 if total_fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))

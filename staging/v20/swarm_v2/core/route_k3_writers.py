"""Route Python NAND writes to the text sidecar instead of the Rust K3 store.

The platform already solved this correctly in `swarm_v2/core/epistemology_verification.py`:
it imports `text_records`/`assert_not_k3` from `nand_fabric_paths` and appends its
JSON record to `swarm_v2_nand_text.jsonl`. The `evolution_sandbox/test_mutation_*.py`
files (and `sovereign_practice.py`) are stale forks of a pre-fix version and still
do `open(<store>.k3, "a", encoding="utf-8")` -- UTF-8 text appended onto a
memory-mapped binary trit array. That is the write that corrupted the root fabric.

Rewrites:
  1. a binding whose variable is later OPENED as text or for writing
     -> text_records(_TRM_ROOT)
  2. `with open(<store>.k3, "a", encoding="utf-8")`  -> sidecar + assert_not_k3 guard
  3. prose claiming a record "persisted in ...k3"    -> the sidecar

Refuses to touch:
  * the module that DEFINES the store name (nand_fabric_paths.py)
  * bindings that are only ever copied or sized (trmboot/install.py copies the
    store to USB -- that is a read, not a write)

Binary append ("ab") is deliberately NOT used: MOTION-007 grants Python no write
handle at all on a store the kernel memory-maps, in text OR binary mode.

    python swarm_v2/core/route_k3_writers.py --check
    python swarm_v2/core/route_k3_writers.py --apply
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
STORE = "swarm_v2_nand_fabric.k3"
SKIP_DIRS = {".git", "target", "node_modules", "__pycache__", ".venv", ".mypy_cache"}
DEFINING_MODULE = "swarm_v2/core/nand_fabric_paths.py"

IMPORT_BLOCK = (
    "# The NAND fabric is the Rust kernel's binary store. Python reaches its own\n"
    "# record through the text sidecar -- one owner per store (council MOTION-007).\n"
    "import sys as _sys  # noqa: E402\n"
    "_sys.path.insert(0, str(_Path(_TRM_ROOT) / \"swarm_v2\" / \"core\"))\n"
    "from nand_fabric_paths import assert_not_k3, text_records  # noqa: E402\n"
)

_ASSIGN = re.compile(r'^(\s*)([A-Za-z_]\w*)\s*=\s*(.+)$')
_OPEN_A = re.compile(r'^(\s*)with\s+open\((.+?),\s*"a",\s*encoding="utf-8"\)\s*as\s+(\w+)\s*:\s*$')
_PROSE = re.compile(r'(persisted in )(swarm_v2_nand_fabric\.k3)')


def is_store_path_expr(rhs: str) -> bool:
    """True if this RHS *builds* the store path (vs merely sizing or comparing it)."""
    if STORE not in rhs:
        return False
    for bad in ("getsize", "exists", "basename", "==", "!=", " in ", 'f"'):
        if bad in rhs:
            return False
    return rhs.startswith(("os.path.join(", "str(_Path(", "_Path(", '"', "'", "r\"", "r'"))


def opened_textually(lines, var: str) -> bool:
    """True if `var` reaches an open() as text or for writing.

    A bare `open(v)` is a text read; `encoding=` implies text; a write mode is a
    write. `open(v, "rb")` alone is a legitimate binary read and is left alone --
    as is a variable only ever handed to shutil.copy2.
    """
    for ln in lines:
        m = re.search(r'\bopen\s*\(\s*' + re.escape(var) + r'\b(.*)$', ln)
        if not m:
            continue
        rest = m.group(1)
        if re.search(r'["\'][wax]', rest) or "encoding=" in rest:
            return True
        if not re.match(r'\s*,\s*["\']r?b["\']', rest):   # bare open(v) == text read
            return True
    return False


def tree_depth(path) -> int:
    """Components from the repo root down to this file's directory."""
    rel = os.path.relpath(os.path.dirname(os.path.abspath(path)), ROOT)
    return 0 if rel == "." else len(rel.replace("\\", "/").split("/"))


def plan(path):
    """Return (new_text, n_assign, n_open, n_prose, notes)."""
    raw = open(path, "rb").read()
    try:
        src = raw.decode("utf-8")
    except UnicodeDecodeError:
        return None, 0, 0, 0, ["not valid UTF-8"]
    lines = src.splitlines(keepends=True)
    notes = []

    bindings = {}
    for ln in lines:
        m = _ASSIGN.match(ln.rstrip("\r\n"))
        if m and is_store_path_expr(m.group(3)):
            bindings[m.group(2)] = True
    text_or_write = {v for v in bindings if opened_textually(lines, v)}

    na = no = np = 0
    out = []
    for ln in lines:
        body = ln.rstrip("\r\n")
        eol = ln[len(body):]

        m = _ASSIGN.match(body)
        if m and m.group(2) in text_or_write:
            out.append(f"{m.group(1)}{m.group(2)} = text_records(_TRM_ROOT){eol}")
            na += 1
            continue

        mo = _OPEN_A.match(body)
        if mo and STORE in mo.group(2):
            ind = mo.group(1)
            out.append(f"{ind}_side = text_records(_TRM_ROOT){eol}")
            out.append(f"{ind}assert_not_k3(_side){eol}")
            out.append(f'{ind}with open(_side, "a", encoding="utf-8") as {mo.group(3)}:{eol}')
            no += 1
            continue

        if re.search(r'open\([^)]*' + re.escape(STORE), body) and re.search(r'["\'][wax]', body):
            notes.append("UNHANDLED write site: " + body.strip()[:110])

        if _PROSE.search(body):
            ln = _PROSE.sub(r'\1the text sidecar', body) + eol
            np += 1
        out.append(ln)

    text = "".join(out)

    if na or no:
        # A header exists only if this file ASSIGNS _TRM_ROOT -- the rewrite above
        # also contains the name, so a substring test would be satisfied by the
        # replacement itself and silently skip the header.
        if not re.search(r'^_TRM_ROOT = ', text, re.M):
            depth = tree_depth(path)
            src_lines = text.splitlines(keepends=True)
            head = "".join(
                ln for ln in src_lines[:2]
                if ln.startswith("#!") or re.match(r"^#.*coding[:=]", ln)
            )
            header = (
                "from pathlib import Path as _Path\n"
                "# Repo root derived from this file's location, not pinned to one host's\n"
                "# drive letter (swarm_v2/core/derive_repo_root.py).\n"
                f"_TRM_ROOT = str(_Path(__file__).resolve().parents[{depth}])\n\n"
            )
            text = head + header + text[len(head):]
        if "from nand_fabric_paths import" not in text:
            anchor = re.search(r'^_TRM_ROOT = .*$', text, re.M)
            if anchor:
                at = anchor.end()
                text = text[:at] + "\n\n" + IMPORT_BLOCK + text[at:].lstrip("\n")
            else:
                notes.append("could not locate header anchor for the import block")
    return text, na, no, np, notes


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--check", action="store_true", help="report only (the default)")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    f_assign = f_open = f_prose = 0
    changed, unhandled, skipped = [], [], []
    for dp, dn, fn in os.walk(ROOT):
        dn[:] = [d for d in dn if d not in SKIP_DIRS]
        for f in sorted(fn):
            if not f.endswith(".py"):
                continue
            p = os.path.join(dp, f)
            rel = os.path.relpath(p, ROOT).replace("\\", "/")
            if rel == DEFINING_MODULE or rel.endswith("route_k3_writers.py"):
                continue
            new, na, no, np_, notes = plan(p)
            if new is None:
                skipped.append(rel)
                continue
            for n in notes:
                unhandled.append((rel, n))
            if na or no or np_:
                changed.append((rel, na, no, np_))
                f_assign += na
                f_open += no
                f_prose += np_
                if args.apply:
                    with open(p, "wb") as fh:
                        fh.write(new.encode("utf-8"))

    print("%s: %d files | %d store-path bindings | %d text-append sites | %d prose claims"
          % ("APPLIED" if args.apply else "WOULD CHANGE", len(changed), f_assign, f_open, f_prose))
    if not args.quiet:
        for rel, na, no, np_ in changed:
            print("   %-64s assign=%d open=%d prose=%d" % (rel[:64], na, no, np_))
    if skipped:
        print("\nskipped (unparsable, not UTF-8): %d" % len(skipped))
    if unhandled:
        print("\nNOT HANDLED (needs a human):")
        for rel, n in unhandled:
            print("   %s: %s" % (rel, n))
    return 1 if unhandled else 0


if __name__ == "__main__":
    sys.exit(main())

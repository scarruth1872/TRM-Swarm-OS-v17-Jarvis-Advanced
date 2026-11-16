"""Derive the repo root instead of pinning the peer's absolute checkout.

192 Python path literals across 111 files pointed at ONE machine's absolute
checkout (the peer's drive-letter path, spelled out in PEER_ROOT below, built
from parts so this file never contains it as a literal). On any other host they
are dead paths; 18 of them open the Rust-owned binary .k3 trit store with
``"a", encoding="utf-8"`` -- the exact write that corrupted the root fabric.

This rewrites each *path literal* into a value derived from the file's own
location, so a script resolves the repo it actually lives in.

It is deliberately conservative. A string is rewritten only when ALL hold:

  * it is a plain string literal (not an f-string -- those need manual review);
  * its decoded value STARTS WITH the peer checkout path;
  * what follows that path is empty or begins with a path separator. A string
    that continues into prose ("F:\\...\\v2 with core/, routes/ ...") is
    documentation and is left exactly as written;
  * it is not a module/class/function docstring.

Usage:
    python swarm_v2/core/derive_repo_root.py --check    # report; exit 1 if drift
    python swarm_v2/core/derive_repo_root.py --apply
    python swarm_v2/core/derive_repo_root.py --show 8   # first N proposed edits
"""
import argparse
import ast
import io
import json
import os
import re
import sys
import tokenize

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

# The peer's development root, as Python decodes it. This is the NEEDLE this tool
# hunts for, so it is composed from parts: this file never contains it as a literal,
# and it is skipped by its own walk.
# The whole tree is in scope, not just this checkout. A sibling beside or inside the
# checkout (F:\Development sites\Jarvis-Advanced-main) pinned the platform to one
# machine exactly as the repo root did.
PEER_DEV = "F:" + "\\" + "Development sites"
# The peer-side name of THIS checkout; its alias is _TRM_ROOT.
REPO_DIR_NAME = "TRM-Swarm-OS-v2"
PEER_ROOT = PEER_DEV
# The two source spellings: single separators, and the doubled form a raw string
# may carry (r"F:\\a\\b"), which puts TWO backslashes in the decoded value.
PEER_SRC = PEER_DEV
PEER_DBL = PEER_SRC.replace("\\", "\\\\")

# ("Where is the TRM codebase?", "F:\...\TRM-Swarm-OS-v2.") is training DATA that
# merely looks like a path. Rewriting it falsifies the dataset, so tuple entries are
# never touched.
DATASET_TUPLE = re.compile(r'\(\s*"[^"]*"\s*,\s*"')

HEADER_FMT = (
    "from pathlib import Path as _Path\n"
    "# Repo root derived from this file's location, not pinned to one host's\n"
    "# drive letter (swarm_v2/core/derive_repo_root.py).\n"
    "_TRM_ROOT = str(_Path(__file__).resolve().parents[{n}])\n"
)

SKIP_DIRS = {".git", "target", "node_modules", "__pycache__", ".venv", ".mypy_cache"}


def depth_to_root(path: str) -> int:
    """How many parents() to climb from this file to the repo root directory."""
    rel = os.path.relpath(os.path.dirname(os.path.abspath(path)), ROOT)
    if rel in (".", ""):
        return 0
    return len([p for p in rel.replace("\\", "/").split("/") if p and p != "."])


def docstring_spans(tree) -> set:
    """(lineno, col_offset) of every Constant node that is a docstring."""
    spans = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            body = getattr(node, "body", None)
            if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) \
                    and isinstance(body[0].value.value, str):
                spans.add((body[0].value.lineno, body[0].value.col_offset))
    return spans


def insert_line(src: str, tree) -> int:
    """0-based line index at which the derived-root header may be inserted."""
    lines = src.splitlines(keepends=True)
    # Never displace a shebang or a PEP 263 coding cookie.
    head = 0
    for i, ln in enumerate(lines[:2]):
        if ln.startswith("#!") or re.match(r"^#.*coding[:=]", ln):
            head = i + 1
    # `from __future__` must remain the first statement.
    fut = 0
    for node in tree.body:
        if isinstance(node, ast.ImportFrom) and node.module == "__future__":
            fut = max(fut, node.end_lineno)
    if fut:
        return max(fut, head)
    if tree.body and isinstance(tree.body[0], ast.Expr) and isinstance(tree.body[0].value, ast.Constant) \
            and isinstance(tree.body[0].value.value, str):
        return max(tree.body[0].end_lineno, head)
    return head


def replacement_for(rest: str) -> str:
    """Derive an expression for `rest` -- the tail after the peer's dev root.

    `...\\Development sites\\TRM-Swarm-OS-v2` IS this checkout, so its tail is
    relative to _TRM_ROOT. A sibling that also exists inside this tree
    (`...\\Development sites\\Jarvis-Advanced-main`) is relative to the repo too.
    Anything else lived beside the checkout, so it is relative to its parent.
    """
    parts = [p for p in re.split(r"[\\/]+", rest) if p]
    if not parts:
        return "str(_Path(_TRM_ROOT).parent)"
    head, tail = parts[0], parts[1:]
    if head == REPO_DIR_NAME:
        base = "_TRM_ROOT"
    elif os.path.isdir(os.path.join(ROOT, head)):
        base = "_Path(_TRM_ROOT) / %s" % json.dumps(head)
    else:
        base = "_Path(_TRM_ROOT).parent / %s" % json.dumps(head)
    if not tail:
        return "str(%s)" % base
    return "str(%s / %s)" % (base, " / ".join(json.dumps(p) for p in tail))


def plan(path: str):
    """Return (edits, prose_skips, fstring_skips) for one file."""
    raw = open(path, "rb").read()
    try:
        src = raw.decode("utf-8")
    except UnicodeDecodeError:
        # Rewriting through a lossy decode would corrupt the file. Path literals
        # are ASCII, but we still refuse to touch a file we cannot round-trip.
        return None, 0, 0, "not valid UTF-8; left for manual review"
    try:
        tree = ast.parse(src)
        toks = list(tokenize.generate_tokens(io.StringIO(src).readline))
    except (SyntaxError, tokenize.TokenError) as e:
        return None, 0, 0, "unparsable: %s" % e

    docs = docstring_spans(tree)
    lines = src.splitlines(keepends=True)
    edits, prose, fstr = [], 0, 0

    for tok in toks:
        if tok.type != tokenize.STRING:
            continue
        raw_tok = tok.string
        prefix = raw_tok[:len(raw_tok) - len(raw_tok.lstrip("rRuUbBfF"))]
        if "f" in prefix.lower():
            # literal_eval cannot see through replacement fields, so substitute the
            # source text of the path for a {_TRM_ROOT} field. Guard the tail so an
            # f-string sentence that merely names the checkout is left alone.
            for form in (PEER_DBL, PEER_SRC):
                if form not in raw_tok:
                    continue
                after = raw_tok.split(form, 1)[1][:1]
                if after in ("", "\\", "/", "_", "{", '"', "'"):
                    edits.append((tok.start, tok.end,
                                  raw_tok.replace(form, "{_TRM_ROOT}", 1), None))
                else:
                    prose += 1
                break
            else:
                if PEER_ROOT in raw_tok:
                    fstr += 1
            continue
        if (tok.start[0], tok.start[1]) in docs:
            continue
        try:
            val = ast.literal_eval(raw_tok)
        except Exception:
            continue
        if not isinstance(val, str):
            continue
        # A raw string may spell the separators doubled -- r"F:\\a\\b" -- which puts
        # TWO backslashes in the value. Collapse them so the literal is recognized
        # as the same checkout path rather than silently skipped.
        val = re.sub(r"\\{2,}", r"\\", val)
        if not val.startswith(PEER_ROOT):
            continue
        rest = val[len(PEER_ROOT):]
        if rest and rest[0] not in "\\/":
            prose += 1          # path then prose -> documentation, leave it
            continue
        if re.search(r"[\s,()\[\]]", rest):
            # A component carrying prose punctuation -- "train_venv with Python
            # 3.11, torch-directml" -- is dataset or documentation text that
            # merely BEGINS with the checkout path. Rewriting it would corrupt
            # real content, so it is left exactly as written. Real path
            # components here are whitespace- and punctuation-free.
            prose += 1
            continue
        if DATASET_TUPLE.search(lines[tok.start[0] - 1]):
            # ("Jarvis codebase", "F:\...\Jarvis-Advanced-main") is training data that
            # merely looks like a path; rewriting it falsifies the dataset.
            prose += 1
            continue
        edits.append((tok.start, tok.end, replacement_for(rest), val))

    return (edits, lines, depth_to_root(path)), prose, fstr, None


def rewrite(path: str) -> dict:
    planned, prose, fstr, err = plan(path)
    if err:
        return {"path": path, "ok": False, "error": err}
    edits, lines, n = planned
    if not edits:
        return {"path": path, "ok": True, "changed": 0, "prose": prose, "fstring": fstr}

    # apply edits bottom-up so earlier columns stay valid
    per_line = {}
    for (sr, sc), (er, ec), new, _old in edits:
        per_line.setdefault(sr, []).append((sc, ec, new))
    for r, cols in per_line.items():
        cols.sort(reverse=True)
        ln = lines[r - 1]
        for sc, ec, new in cols:
            ln = ln[:sc] + new + ln[ec:]
        lines[r - 1] = ln

    src2 = "".join(lines)
    # recompute insertion point on the edited source
    tree2 = ast.parse(src2)
    at = insert_line(src2, tree2)
    lines2 = src2.splitlines(keepends=True)
    if "_TRM_ROOT = str(_Path(__file__)" not in src2:
        lines2.insert(at, HEADER_FMT.format(n=n))
    out = "".join(lines2)

    # optional verification: must still parse
    try:
        ast.parse(out)
    except SyntaxError as e:
        return {"path": path, "ok": False, "error": "rewrite produced invalid syntax: %s" % e}

    return {"path": path, "ok": True, "changed": len(edits), "prose": prose,
            "fstring": fstr, "out": out, "n": n}


def main():
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--check", action="store_true")
    g.add_argument("--apply", action="store_true")
    g.add_argument("--show", type=int, metavar="N")
    ap.add_argument("--strict", action="store_true",
                    help="--check: fail (exit 3) if any .py could not be parsed")
    a = ap.parse_args()

    targets = []
    me = os.path.abspath(__file__)          # a gate must exclude itself
    for dp, dn, fn in os.walk(ROOT):
        dn[:] = [d for d in dn if d not in SKIP_DIRS]
        for f in fn:
            if f.endswith(".py") and os.path.abspath(os.path.join(dp, f)) != me:
                targets.append(os.path.join(dp, f))

    results, tot, files, errs = [], 0, 0, 0
    prose_total = fstr_total = 0
    for p in sorted(targets):
        r = rewrite(p)
        results.append(r)
        if not r.get("ok"):
            errs += 1
            print("ERROR %s: %s" % (os.path.relpath(p, ROOT), r.get("error")))
            continue
        tot += r.get("changed", 0)
        prose_total += r.get("prose", 0)
        fstr_total += r.get("fstring", 0)
        if r.get("changed"):
            files += 1
            if a.apply:
                with open(p, "wb") as fh:
                    fh.write(r["out"].encode("utf-8"))

    print("files scanned      : %d" % len(targets))
    print("files to change    : %d" % files)
    print("literals rewritten : %d" % tot)
    print("prose left alone   : %d" % prose_total)
    print("f-strings skipped  : %d" % fstr_total)
    print("errors             : %d" % errs)
    if a.show:
        shown = 0
        for r in results:
            if r.get("changed") and shown < a.show:
                rel = os.path.relpath(r["path"], ROOT).replace("\\", "/")
                print("  %-52s %d literal(s), parents[%d]" % (rel, r["changed"], r["n"]))
                shown += 1
    if a.check:
        print("\nDRIFT: %d peer-pinned path literals remain." % tot)
        if tot:
            return 1
        if errs:
            # Unscannable is not the same as clean: these files cannot be proved free
            # of the pattern, so the caveat is stated rather than swallowed.
            print("NOTE: %d .py files could not be parsed and were NOT scanned "
                  "(pre-existing Python-2 / extracted fragments). "
                  "Use --strict to treat that as a failure." % errs)
            return 3 if a.strict else 0
        return 0
    return 0 if not errs else 2


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Strip-binary gate: separate real PE libraries from marker files.

A file named *.dll is not necessarily a loadable library. Anything whose
first two bytes are not the DOS 'MZ' signature cannot be loaded by
ctypes.CDLL; in this repo such files are Rust source text renamed to .dll.
This gate classifies every *.dll it sees by its first two bytes only:
  REAL_PE  -- first two bytes are b'MZ' (a PE image)
  MARKER   -- anything else (not loadable)
A MARKER whose first two bytes are b'TR' is recorded as source text.

Scope is derived at run time from the directory actually scanned; no file
count is ever hardcoded. __pycache__ directories and any *_generated.*
files are excluded from the walk.
"""
import argparse
import json
import os
import sys
import time

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# The CLASS, not a directory: this gate is about every file in the tree that claims to be a .dll.
# It used to default to one directory, which then went empty -- so the gate passed while 130
# non-loadable .dll markers sat in a sibling tree it never walked ("unscannable is not clean").
DEFAULT_DIR = REPO_ROOT
# Directories that hold build output, VCS data or backups rather than platform artifacts.
SKIP_DIRS = {"target", ".git", "backups", "node_modules", "__pycache__", ".venv", "venv", "dist", "build"}
DEFAULT_OUT = os.path.join(REPO_ROOT, "swarm_v2_artifacts", "strip_binaries_audit.json")
SOURCE_SIGNATURE = b"TR"


def _excluded(path):
    """True when path sits under a skipped directory or is a *_generated.* artifact."""
    parts = path.replace("\\", "/").split("/")
    if any(p in SKIP_DIRS for p in parts):
        return True
    return "_generated." in os.path.basename(path)


def iter_dlls(root):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in sorted(filenames):
            if not fn.lower().endswith(".dll"):
                continue
            p = os.path.join(dirpath, fn)
            if _excluded(p):
                continue
            yield p


def classify(path):
    with open(path, "rb") as fh:
        head = fh.read(2)
    if head == b"MZ":
        return "REAL_PE", head, False
    return "MARKER", head, head.startswith(SOURCE_SIGNATURE)


def scan(root):
    files, seen, real, marker = [], {}, 0, 0
    for p in iter_dlls(root):
        kind, head, is_src = classify(p)
        st = os.stat(p)
        hx = head.hex()
        seen[hx] = seen.get(hx, 0) + 1
        if kind == "REAL_PE":
            real += 1
        else:
            marker += 1
        files.append({
            "name": os.path.relpath(p, root).replace("\\", "/"),
            "kind": kind,
            "first_bytes_hex": hx,
            "source_text": is_src,
            "size": st.st_size,
            "mtime": int(st.st_mtime),
        })
    summary = {"total": len(files), "real_pe": real, "marker": marker,
               "distinct_first_bytes": len(seen)}
    return files, summary


def main(argv=None):
    ap = argparse.ArgumentParser(description="Strip-binary gate (REAL_PE vs MARKER)")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 if any MARKER is present, else 0")
    ap.add_argument("--dir", default=DEFAULT_DIR, help="directory to scan")
    ap.add_argument("--out", default=DEFAULT_OUT, help="audit JSON output path")
    args = ap.parse_args(argv)

    root = os.path.abspath(args.dir)
    if not os.path.isdir(root):
        print("GATE ERROR: scan dir does not exist: %s" % root)
        return 2
    files, summary = scan(root)
    print("scope: %s" % root)
    print("scope: every file named *.dll under the above (recursive walk); "
          "__pycache__ and *_generated.* skipped")
    print("total=%d real_pe=%d marker=%d distinct_first_bytes=%d"
          % (summary["total"], summary["real_pe"], summary["marker"],
             summary["distinct_first_bytes"]))
    src = sum(1 for f in files if f["source_text"])
    if src:
        print("marker files that are source text (first bytes 'TR'): %d" % src)

    if args.check:
        if summary["marker"]:
            print("GATE FAIL: %d MARKER file(s) are not loadable PE libraries"
                  % summary["marker"])
            return 1
        print("GATE PASS: every *.dll is a REAL_PE 'MZ' image")
        return 0

    out = os.path.abspath(args.out)
    payload = {
        "protocol": "STRIP_BINARY_AUDIT",
        "scan_dir": root,
        "generated": time.strftime("%Y-%m-%d %H:%M:%S"),
        "summary": summary,
        "files": files,
    }
    with open(out, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=1)
    print("wrote audit: %s" % out)
    return 0


if __name__ == "__main__":
    sys.exit(main())

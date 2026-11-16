#!/usr/bin/env python3
"""Collect the kernel modules the launcher loads, out of the kernel that is being shipped.

The list lives in exactly one place -- the launcher's own table -- and is read from it here, so a
module added there is staged by the next pack with no second list to keep in step. They are copied
flat into one directory, because that is where the launcher looks (/lib/modules/<name>) and because
the load order is the launcher's table, not the kernel tree's.

The kernel's modules live in the WSL filesystem, so the copy happens there -- the same route mkfs.ext4
and mksquashfs take in make_state.py and make_rootfs.py.

Usage:
  make_modules.py --launcher <init/platform_init.c> --out <dir> [--kernel <release>]
"""
import argparse, os, re, subprocess, sys


def to_wsl(path):
    p = os.path.abspath(path)
    if len(p) > 1 and p[1] == ":":
        return "/mnt/" + p[0].lower() + p[2:].replace("\\", "/")
    return p.replace("\\", "/")


def wanted(launcher):
    """The module names from the launcher's table -- the one list there is."""
    src = open(launcher, encoding="utf-8").read()
    m = re.search(r"const char \*mods\[\]\s*=\s*\{(.*?)\}", src, re.S)
    if not m:
        sys.exit("no mods[] table in %s -- this script reads the list from the launcher, so it has "
                 "to be there" % launcher)
    return re.findall(r'"([\w.-]+\.ko)"', m.group(1))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--launcher", required=True, help="init/platform_init.c, whose table is the list")
    ap.add_argument("--out", required=True, help="directory to collect the .ko files into")
    ap.add_argument("--kernel", help="kernel release under /lib/modules (default: this machine's)")
    a = ap.parse_args()

    mods = wanted(a.launcher)
    if not mods:
        sys.exit("the launcher's mods[] table names no modules")
    print("the launcher loads: %s" % " ".join(mods))

    tpl = ("set -e; KV=$(uname -r); [ -n '@KV@' ] && KV='@KV@'; "
           "mkdir -p @OUT@; "
           "for m in @MODS@; do "
           "  f=$(find /lib/modules/$KV/kernel -name \"$m\" 2>/dev/null | head -1); "
           "  if [ -z \"$f\" ]; then echo \"  MISSING  $m (not in /lib/modules/$KV)\"; "
           "  else cp -f \"$f\" @OUT@/ && echo \"  $m <- ${f#/lib/modules/$KV/kernel/}\"; fi; "
           "done; "
           "echo \"  collected: $(ls @OUT@/*.ko 2>/dev/null | wc -l) of @COUNT@\"; "
           "du -sh @OUT@ 2>/dev/null | cut -f1")
    script = (tpl.replace("@KV@", a.kernel or "").replace("@OUT@", to_wsl(a.out))
                 .replace("@MODS@", " ".join(mods)).replace("@COUNT@", str(len(mods))))
    r = subprocess.run(["wsl.exe", "-e", "bash", "-lc", script], capture_output=True, text=True)
    out = ((r.stdout or "") + (r.stderr or "")).strip()
    print(out[-2000:] if out else "(no output)")
    return 0 if r.returncode == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

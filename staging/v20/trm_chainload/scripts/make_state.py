#!/usr/bin/env python3
"""Build the writable state image the platform's root layer lives on.

The platform image is squashfs and read-only, so the flash is never modified where the platform
itself is concerned. State is the one thing that has to be written, and it gets its own small ext4
image beside the platform image on the same volume: the launcher loop-mounts it and hands it to
overlayfs as the upper layer, so what the platform writes is still there after a reboot.

ext4 rather than the volume's own FAT, because an overlayfs upper needs whiteouts and xattrs that
vfat does not carry.

mkfs.ext4 is a Linux tool, so the file is made through WSL -- the same route mksquashfs takes in
make_rootfs.py. An existing image is kept unless --force: rebuilding it discards the state in it.

Usage:
  make_state.py --out <trm-state.ext4> [--size 64] [--force]
"""
import argparse, os, subprocess, sys

SIZE_MB = 64


def to_wsl(path):
    """A Windows path as WSL sees it (/mnt/c/...), because mkfs.ext4 is a Linux tool."""
    p = os.path.abspath(path)
    if len(p) > 1 and p[1] == ":":
        return "/mnt/" + p[0].lower() + p[2:].replace("\\", "/")
    return p.replace("\\", "/")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--size", type=int, default=SIZE_MB, help="megabytes (default %d)" % SIZE_MB)
    ap.add_argument("--force", action="store_true", help="rebuild even if the image exists")
    a = ap.parse_args()

    if os.path.exists(a.out) and not a.force:
        print("state image kept: %s (%d B). Rebuilding would discard the state in it; --force to do "
              "that anyway." % (a.out, os.path.getsize(a.out)))
        return 0

    tpl = ("set -e; rm -f @OUT@; truncate -s @SIZE@M @OUT@; "
           "mkfs.ext4 -q -F -L TRMSTATE @OUT@; stat -c '%s' @OUT@")
    script = tpl.replace("@OUT@", to_wsl(a.out)).replace("@SIZE@", str(a.size))
    r = subprocess.run(["wsl.exe", "-e", "bash", "-lc", script], capture_output=True, text=True)
    if r.returncode != 0 or not os.path.exists(a.out):
        print("mkfs.ext4 failed (exit %d):" % r.returncode)
        print(((r.stderr or "") + (r.stdout or "")).strip()[-2000:])
        return 1
    print("state image built: %s (%s B, %d MB, ext4 label TRMSTATE)" % (a.out, (r.stdout or "").strip(), a.size))
    return 0


if __name__ == "__main__":
    sys.exit(main())

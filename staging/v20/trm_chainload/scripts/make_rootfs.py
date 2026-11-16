#!/usr/bin/env python3
"""Pack the bare-metal rootfs an initramfs, for the Linux pathway (see docs/BARE_METAL_LINUX_PATHWAY_*).

The kernel unpacks this cpio archive into its root filesystem, so the archive IS the machine's
userspace. Two things about the format are easy to get wrong and both fail silently:

  * names are POSIX. Building the archive on Windows with os.path.join yields 'opt\\trm\\...' and the
    kernel then creates files with literal backslashes in their names instead of directories. Every
    path in the archive is normalized to '/'.
  * parents must precede children, so entries are sorted by depth before they are written.

Two shapes come out of the same layout, chosen with --mode:

  * `initramfs` (default) -- the cpio.gz the kernel unpacks into RAM. PID 1 is the launcher, from here.
  * `squashfs` -- the read-only platform image the launcher mounts and pivots into, so the machine runs
    from the volume with a writable layer instead of from RAM, and the flash is never modified.
    mksquashfs is a Linux tool and this builds on Windows, so this mode stages the tree into WSL and
    fixes the modes there: a Windows mount reports every file as 0777, which would ship an image whose
    supervisor config is executable.

Usage:
  make_rootfs.py --platform <host_council_server> --supervisor <host_supervisor> \\
                 --init <platform_init> --dist <dashboard/dist> --out <initrd.img>
  make_rootfs.py --mode squashfs --platform ... --supervisor ... --dist ... --out rootfs.squashfs
  make_rootfs.py --init-only --modules <dir of .ko> --init ... --out initrd.img
"""
import argparse, gzip, io, os, shlex, shutil, subprocess, sys, tempfile

BIN = 0o100755
DIR = 0o40755
FILE = 0o100644


def pack(entries):
    """newc-format cpio. 6-byte magic + thirteen 8-hex fields, then the NUL-terminated name and the
    data, each padded to 4 bytes; TRAILER!!! ends it."""
    out, ino = bytearray(), 1

    def pad4(b):
        return b + b"\0" * ((4 - len(b) % 4) % 4)

    for name, data, mode in entries:
        nm = name.encode() + b"\0"
        fields = (ino, mode, 0, 0, 1, 0, len(data), 0, 0, 3, 1, len(nm), 0)
        hdr = "070701" + "".join("%08X" % v for v in fields)
        out += pad4(hdr.encode() + nm) + pad4(data)
        ino += 1
    out += pad4(b"070701" + b"0" * 104 + b"TRAILER!!!\0")
    return bytes(out)


def build(root, platform, supervisor, init, dist, conf, include_init=True, include_platform=True,
          modules=None, firmware=None, wifi=None, dashboard=None):
    """The layout the platform's own path contract expects, with TRM_REPO_ROOT=/opt/trm.

    The mount points are part of the layout, not decoration: the launcher moves /dev, /proc, /sys, the
    boot volume and both halves of the overlay into this root before pivoting, and each needs a
    directory to land on.
    """
    for d in ("dev", "proc", "sys", "mnt", "mnt/esp", "mnt/root", "mnt/rw", "opt/trm/dashboard/dist",
              "opt/trm/trm_uefi_kernel/host_supervisor"):
        os.makedirs(os.path.join(root, *d.split("/")), exist_ok=True)
    put = lambda src, rel, mode: (shutil.copy2(src, os.path.join(root, *rel.split("/"))),
                                  os.chmod(os.path.join(root, *rel.split("/")), mode))
    if include_init:
        put(init, "init", BIN)
    if wifi:
        # The supplicant travels in the RAM image, not the persistent root: it has to run BEFORE the
        # handover, while the interface the modules just produced still exists and the platform has not
        # yet started, so the machine is on the network from the first second the dashboard answers.
        put(wifi, "trm_wifi", BIN)
    if modules:
        # Flat, one directory: the launcher loads them by name before it asks /dev what it has, because
        # the kernel this pathway ships is a VM kernel that carries no USB stack of its own -- and on a
        # real machine the volume the firmware booted from is behind a USB port.
        lib = os.path.join(root, "lib", "modules")
        os.makedirs(lib, exist_ok=True)
        staged = 0
        for name in sorted(os.listdir(modules)):
            if name.endswith(".ko"):
                shutil.copy2(os.path.join(modules, name), os.path.join(lib, name))
                staged += 1
        if not staged:
            raise SystemExit("--modules %s holds no .ko files" % modules)
        print("   %d kernel modules staged for init to load" % staged)
    if firmware:
        # What the kernel asks for at driver-bind time, out of /lib/firmware. On this pathway the RAM image
        # IS the root filesystem at that moment, so a driver whose firmware is not in it falls back -- or
        # not: the machine's RTL8125B asked for `rtl_nic/rtl8125b-2.fw` and got
        # "Direct firmware load ... failed with error -2". A file is cheaper than an unknown.
        fw = 0
        for dirpath, _, files in os.walk(firmware):
            rel = os.path.relpath(dirpath, firmware).replace("\\", "/")
            tgt = "lib/firmware" if rel == "." else "lib/firmware/" + rel
            dest = os.path.join(root, *tgt.split("/"))
            os.makedirs(dest, exist_ok=True)
            for f in files:
                shutil.copy2(os.path.join(dirpath, f), os.path.join(dest, f))
                fw += 1
        if not fw:
            raise SystemExit("--firmware %s holds no files" % firmware)
        print("   %d firmware files staged into /lib/firmware" % fw)
    if not include_platform:
        return
    put(platform, "opt/trm/host_council_server", BIN)
    put(supervisor, "opt/trm/trm_uefi_kernel/host_supervisor/host_supervisor", BIN)
    put(conf, "opt/trm/trm_uefi_kernel/host_supervisor/supervisor.services.conf", FILE)
    if dashboard:
        # The framebuffer dashboard runs AFTER the handover, beside the platform it draws, so it
        # travels in the persistent root (squashfs) -- never the RAM image, which is gone by then.
        put(dashboard, "opt/trm/trm_fb_dashboard", BIN)
    for dirpath, _, files in os.walk(dist):
        rel = os.path.relpath(dirpath, dist).replace("\\", "/")
        tgt = "opt/trm/dashboard/dist" if rel == "." else "opt/trm/dashboard/dist/" + rel
        os.makedirs(os.path.join(root, *tgt.split("/")), exist_ok=True)
        for f in files:
            put(os.path.join(dirpath, f), tgt + "/" + f, FILE)


def _stage_stores(root, stores):
    """Copy a flat stores directory into /opt/trm/swarm_v2_artifacts.

    The platform's routes resolve these read-only on the metal, so they travel in the
    squashfs image rather than depending on the launcher's store-link list.
    """
    tgt = os.path.join(root, "opt", "trm", "swarm_v2_artifacts")
    os.makedirs(tgt, exist_ok=True)
    staged = 0
    for name in sorted(os.listdir(stores)):
        src = os.path.join(stores, name)
        if os.path.isfile(src):
            shutil.copy2(src, os.path.join(tgt, name))
            staged += 1
    if not staged:
        raise SystemExit("--stores %s holds no files" % stores)
    print("   %d stores staged into /opt/trm/swarm_v2_artifacts" % staged)


def _stage_root_stores(root, stores):
    """Copy a flat dir into /opt/trm (the repo root the platform's `repo_relative()` resolves from).

    The .turbovec fabric stores (`swarm_v2_nand_fabric.<name>.turbovec`) are read by paths relative
    to the executable's parent dir, not under swarm_v2_artifacts/, and the launcher's link_stores()
    list is fixed to the five base fabric names -- so the model-distilled side stores travel in the
    image at /opt/trm/ instead.
    """
    tgt = os.path.join(root, "opt", "trm")
    os.makedirs(tgt, exist_ok=True)
    staged = 0
    for name in sorted(os.listdir(stores)):
        src = os.path.join(stores, name)
        if os.path.isfile(src):
            shutil.copy2(src, os.path.join(tgt, name))
            staged += 1
    if not staged:
        raise SystemExit("--root-stores %s holds no files" % stores)
    print("   %d stores staged into /opt/trm (repo root)" % staged)



def collect(root):
    ents = []
    for dirpath, _, files in os.walk(root):
        rel = os.path.relpath(dirpath, root).replace("\\", "/")
        ents.append(("." if rel == "." else rel, b"", DIR))
        for f in files:
            p = os.path.join(dirpath, f)
            ents.append((f if rel == "." else rel + "/" + f, open(p, "rb").read(),
                         BIN if os.access(p, os.X_OK) else FILE))
    ents.sort(key=lambda e: (e[0].count("/"), e[0]))  # parents before children
    return ents


def to_wsl(path):
    """A Windows path as WSL sees it (/mnt/c/...), because mksquashfs is a Linux tool."""
    p = os.path.abspath(path)
    if len(p) > 1 and p[1] == ":":
        return "/mnt/" + p[0].lower() + p[2:].replace("\\", "/")
    return p.replace("\\", "/")


def make_squashfs(tree, out, dashboard=None):
    """Pack the tree with mksquashfs, staged into WSL so the image carries real modes.

    gzip rather than the mksquashfs default: this pathway's kernel has squashfs built in and gzip is
    the compressor every squashfs build understands. `-all-root` because PID 1 is root. The three
    binaries get their mode from a pseudo-file rule because a Windows mount hands mksquashfs 0777 for
    every file, which would ship an image whose supervisor config is executable.
    """
    stage = "/tmp/trm_squashfs_stage"
    pseudo = stage + "/.pseudo"
    bins = [stage + "/opt/trm/host_council_server",
            stage + "/opt/trm/trm_uefi_kernel/host_supervisor/host_supervisor"]
    pseudo_lines = ["opt/trm/host_council_server m 755 0 0",
                    "opt/trm/trm_uefi_kernel/host_supervisor/host_supervisor m 755 0 0"]
    if dashboard:
        bins.append(stage + "/opt/trm/trm_fb_dashboard")
        pseudo_lines.append("opt/trm/trm_fb_dashboard m 755 0 0")
    chmod_cmd = "chmod 755 " + " ".join(bins)
    pseudo_cmd = "printf '%s\\n' " + " ".join(shlex.quote(l) for l in pseudo_lines) + " > @PSEUDO@"
    tpl = ("set -e; rm -rf @STAGE@; mkdir -p @STAGE@; cp -r @TREE@/. @STAGE@/; "
           "find @STAGE@ -type f -exec chmod 644 {} +; " + chmod_cmd + "; " + pseudo_cmd + "; "
           # -mkfs-time is the one that matters for the image itself: -all-time pins every file's
           # timestamp but leaves the superblock's mkfs_time at the second the build ran, which is the only
           # byte that differed between two packs of the same tree (measured: offset 8, 3 seconds apart).
           "mksquashfs @STAGE@ @OUT@ -comp gzip -b 128K -Xcompression-level 9 -no-exports "
           "-all-time 0 -mkfs-time 0 -processors 1 -all-root -pf @PSEUDO@ -noappend -quiet; "
           "stat -c '%s' @OUT@")
    script = (tpl.replace("@STAGE@", stage).replace("@TREE@", to_wsl(tree))
                 .replace("@PSEUDO@", pseudo).replace("@OUT@", to_wsl(out)))
    r = subprocess.run(["wsl.exe", "-e", "bash", "-lc", script], capture_output=True, text=True)
    if r.returncode != 0 or not os.path.exists(out):
        print("mksquashfs failed (exit %d):" % r.returncode)
        print(((r.stderr or "") + (r.stdout or "")).strip()[-2000:])
        return 1
    size = ((r.stdout or "").strip().splitlines() or ["?"])[-1]
    print("squashfs built: %s (%s B) -- %s files staged from %s"
          % (out, size, sum(len(f) for _, _, f in os.walk(tree)), tree))
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=("initramfs", "squashfs"), default="initramfs")
    ap.add_argument("--platform", required=True)
    ap.add_argument("--supervisor", required=True)
    ap.add_argument("--init", help="PID 1 for the RAM image (not used in squashfs mode)")
    ap.add_argument("--modules", help="directory of .ko files to stage into /lib/modules, for the "
                                      "launcher to load before it asks /dev: this kernel is a VM kernel "
                                      "with no USB stack, and on a real machine the boot volume is "
                                      "behind a USB port")
    ap.add_argument("--init-only", action="store_true",
                    help="pack just init: the RAM image for the persistent setup, where the platform "
                         "comes from rootfs.squashfs instead of being duplicated in RAM")
    ap.add_argument("--wifi", help="trm_wifi binary staged as /trm_wifi in the RAM image, run by the "
                                   "launcher before the handover so the radio is associated, if it can "
                                   "be, before the platform starts")
    ap.add_argument("--dashboard", help="trm_fb_dashboard binary staged as /opt/trm/trm_fb_dashboard "
                                        "in the persistent root, run by the launcher after the handover "
                                        "so the machine's own screen shows the dashboard natively")
    ap.add_argument("--stores", help="directory staged into /opt/trm/swarm_v2_artifacts: the runtime "
                                     "stores the platform's routes read (threat index, DDR antibodies, "
                                     "CVE fold, retrieval corpus). Read-only on the metal, so they can "
                                     "live in the squashfs image.")
    ap.add_argument("--root-stores", help="directory staged into /opt/trm (the repo root the "
                                          "platform's repo_relative() resolves from): the .turbovec "
                                          "fabric stores, including the model-distilled side stores.")
    ap.add_argument("--firmware", help="directory staged into /lib/firmware, for drivers that request "
                                       "firmware when they bind (a real NIC does; this pathway's kernel "
                                       "carries several as modules)")
    ap.add_argument("--dist", required=True)
    ap.add_argument("--conf", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--gzip", type=int, nargs="?", const=6, default=0,
                    help="compress the RAM image with gzip at this level (default: no compression). The "
                         "compressed output depends on the zlib IMPLEMENTATION, not only on the level: the "
                         "same archive is 3,976,503 B through Windows' zlib-ng and 3,909,028 B through "
                         "Linux's stock zlib, so a compressed image reproduces only in the environment that "
                         "built it. An uncompressed initramfs has no compressor to vary and the kernel "
                         "takes it as it is.")
    a = ap.parse_args()

    root = tempfile.mkdtemp(prefix="trm_rootfs_")
    try:
        include_init = a.mode == "initramfs"
        if include_init and not a.init:
            ap.error("--init is required in initramfs mode: it is the PID 1 the kernel runs")
        build(root, a.platform, a.supervisor, a.init, a.dist, a.conf, include_init=include_init,
              include_platform=not a.init_only, modules=a.modules, firmware=a.firmware, wifi=a.wifi,
              dashboard=a.dashboard)
        if a.stores:
            _stage_stores(root, a.stores)
        if a.root_stores:
            _stage_root_stores(root, a.root_stores)
        if a.mode == "squashfs":
            return make_squashfs(root, a.out, dashboard=a.dashboard)
        ents = collect(root)
        raw = pack(ents)
        out_bytes = gzip.compress(raw, a.gzip, mtime=0) if a.gzip else raw
        io.open(a.out, "wb").write(out_bytes)
        total = sum(len(d) for _, d, _ in ents)
        print("%d entries, %d B unpacked -> %.1f MB %s -> %s" %
              (len(ents), total, len(out_bytes) / 1048576,
               ("gzip -%d" % a.gzip) if a.gzip else "uncompressed", a.out))
        if a.gzip:
            print("   (level %d through a different zlib implementation is a different file: %d B here, "
                  "3,976,503 B through zlib-ng -- an image is comparable only with one built the same way)"
                  % (a.gzip, len(out_bytes)))
        for n, d, _ in ents:
            if d:
                print("   %-58s %9d B" % (n, len(d)))
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())

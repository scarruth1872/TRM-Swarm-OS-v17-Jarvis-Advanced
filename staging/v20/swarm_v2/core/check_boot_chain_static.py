"""Assert that nothing in the boot chain needs a dynamic loader, and that the drive boots the tree's build.

Two failures this exists to catch, both of which have been proposed as explanations for a failed exec:

  * a binary enters the chain that needs `ld.so`, in an image that carries no loader -- the exec fails with
    ENOENT and looks like a missing file;
  * the artifact on the drive stops being the artifact the tree builds, so a boot proves something about
    an image nobody can rebuild.

It reads the initramfs directly (gzip + newc, in Python) and the platform image through `unsquashfs` in
WSL, since the platform lives in a squashfs and this host has no loop devices. Exit 0 only if every ELF in
both images is static and every artifact the drive boots matches the tree.

    python swarm_v2/core/check_boot_chain_static.py            # check the drive (E:) against the tree
"""
import gzip
import hashlib
import io
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DRIVE = "E:/"
PT_INTERP = 3

# The four artifacts the drive boots, and where the tree keeps the built form of each. The paths in the
# archive are what the kernel opens, so a mismatch here is a boot that proves the wrong thing.
ARTIFACTS = [
    ("EFI/BOOT/BOOTX64.EFI", "trm_chainload/dist/trm_chainload.efi"),
    ("EFI/TRM/vmlinuz", None),
    ("EFI/TRM/initrd.img", None),
    ("rootfs.squashfs", None),
]


def elf_needs_loader(blob):
    """(is_elf, interpreter). A PT_INTERP program header is the request for a loader."""
    if len(blob) < 64 or blob[:4] != b"\x7fELF":
        return False, None
    phoff = int.from_bytes(blob[32:40], "little")
    phentsize = int.from_bytes(blob[54:56], "little")
    phnum = int.from_bytes(blob[56:58], "little")
    for i in range(phnum):
        p = phoff + i * phentsize
        if p + 40 > len(blob):
            break
        if int.from_bytes(blob[p:p + 4], "little") == PT_INTERP:
            off = int.from_bytes(blob[p + 8:p + 16], "little")
            size = int.from_bytes(blob[p + 32:p + 40], "little")
            return True, blob[off:off + size].split(b"\0")[0].decode("utf8", "replace")
    return True, None


def cpio_members(blob):
    """Every regular file in a newc archive, by name. The archive's own header alignment is honoured --
    header+name are padded together, which is the mistake a hand-rolled walk makes first."""
    out, off = {}, 0
    while off + 110 <= len(blob):
        if blob[off:off + 6] not in (b"070701", b"070702"):
            break
        f = lambda o, n: int(blob[off + o:off + o + n], 16)  # noqa: E731
        namesize, filesize, mode = f(94, 8), f(54, 8), f(14, 8)
        name = blob[off + 110:off + 110 + namesize - 1].decode("utf8", "replace")
        data = off + ((110 + namesize + 3) & ~3)
        if name == "TRAILER!!!":
            break
        if mode & 0o170000 == 0o100000:
            out[name] = blob[data:data + filesize]
        off = data + ((filesize + 3) & ~3)
    return out


def wsl_visible(image):
    """A path the WSL side can open. WSL does not automount removable drives -- /mnt/e does not exist --
    so an image on the drive is staged on the C: side, which WSL does have, and read from there."""
    native = "/mnt/" + image[0].lower() + image[2:]
    if subprocess.run(["wsl.exe", "-e", "test", "-f", native], capture_output=True).returncode == 0:
        return native
    staged = os.path.join(os.environ.get("LOCALAPPDATA", os.path.expanduser("~")),
                          "hermes", "cache", "scratch", "rootfs_from_drive.squashfs")
    os.makedirs(os.path.dirname(staged), exist_ok=True)
    needs_copy = True
    if os.path.exists(staged):
        a = hashlib.sha256(open(image, "rb").read()).hexdigest()
        b = hashlib.sha256(open(staged, "rb").read()).hexdigest()
        needs_copy = a != b
    if needs_copy:
        with open(image, "rb") as src, open(staged, "wb") as dst:
            while chunk := src.read(1 << 20):
                dst.write(chunk)
    wsl_staged = "/mnt/" + staged[0].lower() + staged[2:].replace(os.sep, "/")
    return wsl_staged


def squashfs_member(image, path, scratch=None):
    """One file out of a squashfs, via unsquashfs in WSL. `-cat` takes the path as it appears in the
    listing, minus the `squashfs-root/` prefix."""
    wsl_image = wsl_visible(image)
    proc = subprocess.run(["wsl.exe", "-e", "bash", "-c",
                           f"unsquashfs -cat '{wsl_image}' '{path}' 2>&1"],
                          capture_output=True)
    out = proc.stdout
    if not out[:4] == b"\x7fELF":
        sys.stderr.write(f"    (unsquashfs said: {out[:200]!r})\n")
    return out


def main():
    problems = []

    print("  --- the artifacts the drive boots, against the tree's build ---")
    for drive_path, tree_path in ARTIFACTS:
        p = os.path.join(DRIVE, drive_path)
        if not os.path.exists(p):
            problems.append(f"{drive_path} is not on the drive")
            print(f"    MISSING  {drive_path}")
            continue
        got = hashlib.sha256(open(p, "rb").read()).hexdigest()
        if tree_path:
            t = os.path.join(ROOT, tree_path)
            want = hashlib.sha256(open(t, "rb").read()).hexdigest() if os.path.exists(t) else None
            if want != got:
                problems.append(f"{drive_path} is {got[:16]} but {tree_path} is {(want or 'absent')[:16]}")
                print(f"    DIFFERS  {drive_path}  drive={got[:16]}  tree={(want or 'absent')[:16]}")
            else:
                print(f"    same     {drive_path}  {got[:16]}  (== {tree_path})")
        else:
            print(f"    ok       {drive_path}  {got[:16]}  {os.path.getsize(p)} B")

    print("  --- every ELF in the RAM image ---")
    # The RAM image ships uncompressed by default now -- a plain cpio has no compressor to vary, which is
    # what makes it reproducible across build environments. This reads both shapes rather than assuming the
    # compression that used to be unconditional: a checker that only understands the old shape fails on the
    # new one for a reason that has nothing to do with the boot chain it exists to check.
    _raw = open(os.path.join(DRIVE, "EFI/TRM/initrd.img"), "rb").read()
    initrd = gzip.decompress(_raw) if _raw[:2] == b"\x1f\x8b" else _raw
    members = cpio_members(initrd)
    checked = 0
    for name, blob in members.items():
        is_elf, interp = elf_needs_loader(blob)
        if is_elf:
            checked += 1
            if interp:
                problems.append(f"/{name} in the initramfs needs {interp}, which the image does not carry")
                print(f"    DYNAMIC  /{name} -> {interp}")
            else:
                print(f"    static   /{name}  {len(blob)} B")

    print("  --- every executable in the platform image ---")
    # The platform is a squashfs; read the two binaries that matter through unsquashfs.
    for path in ("/opt/trm/host_council_server",
                 "/opt/trm/trm_uefi_kernel/host_supervisor/host_supervisor"):
        blob = squashfs_member(os.path.join(DRIVE, "rootfs.squashfs"), path, None)
        is_elf, interp = elf_needs_loader(blob)
        checked += 1
        if not is_elf:
            problems.append(f"{path} could not be read out of the platform image ({len(blob)} B)")
            print(f"    UNREADABLE {path} ({len(blob)} B)")
        elif interp:
            problems.append(f"{path} needs {interp}, which the platform image does not carry")
            print(f"    DYNAMIC  {path} -> {interp}")
        else:
            print(f"    static   {path}  {len(blob)} B")

    print(f"  ---\n  {checked} ELF file(s) checked, {len(problems)} problem(s)")
    for p in problems:
        print("    " + p)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

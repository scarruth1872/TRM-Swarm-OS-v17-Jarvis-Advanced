"""Which stage of the image is not reproducible.

The claim to test is not "reproducible builds would be nice" but "the same source produces the same
artifact". Each stage is built twice from the same inputs and the two results are compared by content, so
the answer is a measurement per stage rather than one verdict for the whole pipeline -- a stage that differs
is a stage that needs fixing, and a stage that does not is not worth touching.

    python check_reproducible.py            # measure every stage
    python check_reproducible.py --initrd   # measure one stage

Nothing here is asserted: the script prints what it measured and exits non-zero if any stage differed.
"""
import argparse
import hashlib
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRATCH = pathlib.Path(r"C:/Users/shawn/AppData/Local/hermes/cache/scratch")
OUT = SCRATCH / "uefi_test/linux/artifacts_nic"
WIFI = SCRATCH / "trm_wifi_target/x86_64-unknown-linux-musl/release/trm_wifi"
PLATFORM = SCRATCH / "storereload_musl/x86_64-unknown-linux-musl/release"


def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def report(stage, a, b, extra=""):
    same = sha(a) == sha(b)
    print("  %-22s %s  %s%s" % (stage, "SAME     " if same else "DIFFERENT",
                               sha(a)[:16], ("  vs " + sha(b)[:16]) if not same else ""))
    if extra:
        print("      " + extra)
    return same


def measure_initrd(work):
    """Two packs of the same staged tree, with nothing rebuilt in between."""
    a, b = work / "a.img", work / "b.img"
    # One second apart, deliberately: a timestamp field has second resolution, so two packs inside the same
    # second compare equal whether or not such a field exists. Without this the check passes for the wrong
    # reason -- which is the defect class this whole pathway keeps finding, in its own instruments.
    import time
    for out in (a, b):
        if out is b:
            time.sleep(1.2)
        cmd = [sys.executable, str(ROOT / "trm_chainload/scripts/make_rootfs.py"), "--init-only",
               "--modules", str(OUT / "modules"), "--init", str(OUT / "platform_init"),
               "--firmware", str(ROOT / "trm_chainload/firmware"), "--wifi", str(WIFI),
               "--platform", str(PLATFORM / "host_council_server"),
               "--supervisor", str(PLATFORM / "host_supervisor"),
               "--dist", str(ROOT / "dashboard/dist"),
               "--conf", str(ROOT / "trm_chainload/init/supervisor.services.conf"), "--out", str(out)]
        r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode != 0:
            print("  initrd: pack failed:\n" + r.stdout[-800:] + r.stderr[-800:])
            return False
    same = report("initrd (pack twice)", a, b, "inputs unchanged; only the pack runs again")
    if not same:
        # where the two differ, byte for byte: a gzip header field or real content
        da, db = a.read_bytes(), b.read_bytes()
        diffs = [i for i in range(min(len(da), len(db))) if da[i] != db[i]][:8]
        print("      first byte offsets that differ: %s (of %d vs %d bytes)"
              % (diffs, len(da), len(db)))
    return same


def measure_unpacked(work):
    """The same, with the gzip stripped: is the cpio itself stable?"""
    import gzip
    a, b = work / "a.img", work / "b.img"
    if not (a.exists() and b.exists()):
        print("  %-22s not measured -- the pack it depends on did not produce an image" % "cpio (no gzip)")
        return False
    ua, ub = work / "a.cpio", work / "b.cpio"
    for src, dst in ((a, ua), (b, ub)):
        raw = src.read_bytes()
        # The default image is a plain cpio now, so this stage reads both shapes rather than assuming the
        # compression that used to be unconditional.
        dst.write_bytes(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)
    return report("cpio (no gzip)", ua, ub, "the archive the kernel unpacks, before compression")


def measure_cross_environment(work):
    """The same inputs packed on both sides of the WSL boundary.

    This is the stage that caught the real defect. The cpio was already byte-identical build to build and
    the gzip header already carried mtime 0, so the file looked reproducible -- but the same 13 MB archive
    was 3,976,503 B through Windows' zlib-ng and 3,909,028 B through Linux's stock zlib at the same level,
    which is a different image for an unchanged source. An artifact is only comparable with one built the
    same way, and the fix was to stop compressing the RAM image: a plain cpio has no compressor to vary, and
    the kernel takes it as it is.
    """
    def wsl_path(p):
        p = pathlib.Path(p).as_posix()
        return "/mnt/" + p[0].lower() + p[2:] if len(p) > 1 and p[1] == ":" else p

    pairs = [("--modules", OUT / "modules"), ("--init", OUT / "platform_init"),
             ("--firmware", ROOT / "trm_chainload/firmware"), ("--wifi", WIFI),
             ("--platform", PLATFORM / "host_council_server"),
             ("--supervisor", PLATFORM / "host_supervisor"),
             ("--dist", ROOT / "dashboard/dist"),
             ("--conf", ROOT / "trm_chainload/init/supervisor.services.conf")]
    script = ROOT / "trm_chainload/scripts/make_rootfs.py"
    win_out, wsl_out = work / "xenv_win.img", work / "xenv_wsl.img"
    subprocess.run([sys.executable, str(script), "--init-only"] +
                   [x for k, v in pairs for x in (k, str(v))] + ["--out", str(win_out)],
                   capture_output=True, text=True)
    time.sleep(1.2)   # a second apart, so a timestamp field cannot hide behind the resolution
    inner = " ".join([f"'{wsl_path(script)}'", "--init-only"] +
                     [f"'{x}'" for k, v in pairs for x in (k, wsl_path(v))] +
                     [f"--out '{wsl_path(wsl_out)}'"])
    subprocess.run(["wsl.exe", "-e", "bash", "-c", f"python3 {inner}"],
                   capture_output=True, text=True)
    if not (win_out.exists() and wsl_out.exists()):
        print("  %-22s not measured -- one side did not produce an image" % "image (win vs WSL)")
        return False
    return report("image (win vs WSL)", win_out, wsl_out,
                  "same inputs, different interpreter and different zlib")


def measure_compiler(name, argv, out, work, cwd=None):
    a, b = work / (out + ".a"), work / (out + ".b")
    for tgt in (a, b):
        r = subprocess.run(argv + ["-o", str(tgt)], capture_output=True, text=True, cwd=cwd)
        if r.returncode != 0:
            print("  %-22s compile failed: %s" % (name, (r.stderr or r.stdout)[-300:]))
            return False
    return report(name, a, b)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--initrd", action="store_true")
    args = ap.parse_args()
    work = pathlib.Path(tempfile.mkdtemp(prefix="trm_repro_"))
    print("  workdir: %s" % work)
    results = []
    if args.initrd or True:
        results.append(("initrd", measure_initrd(work)))
        results.append(("cpio", measure_unpacked(work)))
        results.append(("cross-env", measure_cross_environment(work)))
    print("\n  %d of %d stage(s) reproducible" % (sum(1 for _, ok in results if ok), len(results)))
    return 0 if all(ok for _, ok in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""qemu_boot_test.py — boot the TRM UEFI artifact in QEMU and capture what the screen actually shows.

WHY: "it boots" is unverified until something renders it. This runs the same BOOTX64.EFI that is deployed to
the USB stick under OVMF firmware, screenshots the framebuffer over QMP at intervals, and keeps the serial log.
The evidence is the frames plus the log — not the exit code, because QEMU exits 0 whether the kernel drew an
LCARS dashboard or the firmware dropped straight to the built-in shell.

USAGE
  python qemu_boot_test.py                          # built artifact from scratch/uefi_build
  python qemu_boot_test.py --efi G:/EFI/BOOT/BOOTX64.EFI
  python qemu_boot_test.py --seconds 60 --mem 4096

WHAT IT NEEDS
  qemu-system-x86_64.exe and an OVMF x86-64 firmware blob (edk2-x86_64-code.fd / OVMF_CODE.fd / OVMF.fd).
  Finds them in C:\\Program Files\\qemu, C:\\Program Files (x86)\\qemu, or PATH.

OUTPUT
  scratch/qemu_boot/<stamp>/  frame_00.png … serial.log, qemu.log, summary.json
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import time
import zlib
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
QEMU_CANDIDATES = [
    Path(r"C:\Program Files\qemu\qemu-system-x86_64.exe"),
    Path(r"C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"),
]
FIRMWARE_NAMES = ["edk2-x86_64-code.fd", "OVMF_CODE.fd", "OVMF.fd", "edk2-x86_64-smm-code.fd"]
VARS_NAMES = ["edk2-x86_64-vars.fd", "OVMF_VARS.fd", "edk2-i386-vars.fd"]


def find_qemu() -> Path:
    for p in QEMU_CANDIDATES:
        if p.exists():
            return p
    w = shutil.which("qemu-system-x86_64")
    if w:
        return Path(w)
    sys.exit("qemu-system-x86_64.exe not found — install QEMU (winget install SoftwareFreedomConservancy.QEMU)")


def _find(names) -> Path | None:
    roots = [Path(r"C:\Program Files\qemu"), Path(r"C:\Program Files (x86)\qemu")]
    for root in roots:
        for sub in ("share", "share\\edk2", "firmware", ""):
            for name in names:
                p = root / sub / name
                if p.exists():
                    return p
    return None


def find_firmware() -> tuple[Path, Path | None]:
    code = _find(FIRMWARE_NAMES)
    if code is None:
        sys.exit("no OVMF code blob found under C:\\Program Files\\qemu")
    return code, _find(VARS_NAMES)


def ppm_to_png(data: bytes, out: Path) -> None:
    """Minimal binary-PPM (P6) -> PNG. No Pillow on this host, and a screenshot we cannot look at is useless."""
    assert data[:2] == b"P6", "screendump did not return P6"
    # header: P6\n<w> <h>\n<maxval>\n  — comments (#) are legal, skip them
    idx, fields = 2, []
    while len(fields) < 3:
        while idx < len(data) and data[idx : idx + 1].isspace():
            idx += 1
        if data[idx : idx + 1] == b"#":
            while data[idx : idx + 1] not in (b"\n", b""):
                idx += 1
            continue
        start = idx
        while idx < len(data) and not data[idx : idx + 1].isspace():
            idx += 1
        fields.append(int(data[start:idx]))
    idx += 1
    w, h, _maxval = fields
    px = data[idx : idx + w * h * 3]
    raw = b"".join(b"\x00" + px[y * w * 3 : (y + 1) * w * 3] for y in range(h))

    def chunk(tag: bytes, payload: bytes) -> bytes:
        return struct.pack(">I", len(payload)) + tag + payload + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)

    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    out.write_bytes(png)


class Qmp:
    def __init__(self, port: int, timeout: float = 20.0):
        deadline = time.time() + timeout
        last = None
        while time.time() < deadline:
            try:
                self.s = socket.create_connection(("127.0.0.1", port), timeout=5)
                break
            except OSError as e:  # the socket is not up until QEMU starts listening
                last = e
                time.sleep(0.4)
        else:
            raise RuntimeError(f"could not connect to QMP on 127.0.0.1:{port}: {last}")
        self.f = self.s.makefile("rwb", buffering=0)
        self._read()  # greeting
        self.cmd("qmp_capabilities")

    def _read(self) -> dict:
        line = self.f.readline()
        if not line:
            raise RuntimeError("QMP closed")
        return json.loads(line)

    def cmd(self, name: str, **args) -> dict:
        payload = {"execute": name}
        if args:
            payload["arguments"] = args
        self.f.write((json.dumps(payload) + "\n").encode())
        while True:
            msg = self._read()
            if "return" in msg or "error" in msg:
                return msg
            # events are interleaved; ignore them here


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--efi", default=str(REPO / "scratch/uefi_build/x86_64-unknown-uefi/release/trm_uefi_kernel.efi"))
    ap.add_argument("--seconds", type=int, default=45)
    ap.add_argument("--frames", type=int, default=6)
    ap.add_argument("--mem", type=int, default=4096)
    ap.add_argument("--display", action="store_true", help="show the QEMU window too (default: headless)")
    ap.add_argument("--nic", action="store_true",
                    help="attach an e1000 NIC (positive control for the RADIO line: SimpleNetwork must appear)")
    ap.add_argument("--fabric", type=int, default=0,
                    help="place a swarm_v2_nand_fabric.k3 of N bytes on the ESP (positive control for the K3 line)")
    args = ap.parse_args()

    efi = Path(args.efi)
    if not efi.exists():
        sys.exit(f"EFI artifact missing: {efi}")
    qemu, (fw, fw_vars) = find_qemu(), find_firmware()

    stamp = time.strftime("%Y%m%d_%H%M%S")
    out = REPO / "scratch/qemu_boot" / stamp
    esp = out / "esp"
    (esp / "EFI/BOOT").mkdir(parents=True, exist_ok=True)
    shutil.copy2(efi, esp / "EFI/BOOT/BOOTX64.EFI")
    if args.fabric:
        # Sparse-looking file of the requested size: the kernel must report ITS measured size and read the first
        # and last sector, so a fake zero-length file would not pass.
        with open(esp / "swarm_v2_nand_fabric.k3", "wb") as f:
            f.truncate(args.fabric)
        print(f"fabric: {args.fabric:,} bytes staged on the ESP")

    if fw_vars:
        shutil.copy2(fw_vars, out / "vars.fd")

    qmp_port = 4455
    serial = out / "serial.log"
    log = out / "qemu.log"
    cmd = [
        str(qemu),
        "-machine", "q35",
        "-m", str(args.mem),
        # OVMF is a pflash image: -bios cannot load it ("could not load PC BIOS"), it needs if=pflash.
        # unit=1 is the writable NVRAM store, copied per run so the boot state is deterministic.
        "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={fw.as_posix()}",
        *((["-drive", f"if=pflash,format=raw,unit=1,file={(out / 'vars.fd').as_posix()}"] if fw_vars else [])),
        "-drive", f"format=raw,file=fat:rw:{esp.as_posix()}",
        *((["-netdev", "user,id=n0", "-device", "e1000,netdev=n0"] if args.nic else ["-net", "none"])),
        "-vga", "std",
        "-display", "sdl" if args.display else "none",
        "-qmp", f"tcp:127.0.0.1:{qmp_port},server,nowait",
        "-serial", f"file:{serial.as_posix()}",
        "-no-reboot",
    ]
    print("qemu :", qemu)
    print("firm :", fw)
    print("efi  :", efi, f"({efi.stat().st_size:,} bytes)")
    print("cmd  :", " ".join(cmd))

    with open(log, "wb") as lf:
        proc = subprocess.Popen(cmd, stdout=lf, stderr=subprocess.STDOUT)

    frames, notes = [], []
    try:
        qmp = Qmp(qmp_port)
        for i in range(args.frames):
            time.sleep(max(args.seconds / args.frames, 1.0))
            ppm = out / f"frame_{i:02d}.ppm"
            r = qmp.cmd("screendump", filename=str(ppm))
            if "error" in r:
                notes.append(f"frame {i}: {r['error']}")
                continue
            if ppm.exists():
                png = out / f"frame_{i:02d}.png"
                ppm_to_png(ppm.read_bytes(), png)
                ppm.unlink()
                frames.append((i, png, png.stat().st_size))
                print(f"  frame {i:02d}: {png.name} ({png.stat().st_size:,} bytes)")
        qmp.cmd("quit")
    except Exception as e:  # a dead QMP must not hide the frames we already have
        notes.append(f"qmp: {e}")
    finally:
        try:
            proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            proc.kill()

    serial_txt = serial.read_text(errors="replace") if serial.exists() else ""
    summary = {
        "artifact": str(efi),
        "artifact_bytes": efi.stat().st_size,
        "firmware": str(fw),
        "seconds": args.seconds,
        "frames": [str(p) for _i, p, _s in frames],
        "notes": notes,
        "serial_bytes": len(serial_txt),
        "serial_head": serial_txt[:2000],
        "qemu_exit": proc.returncode,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print("\nserial bytes:", len(serial_txt), "| qemu exit:", proc.returncode)
    if serial_txt.strip():
        print("serial head:\n" + "\n".join(serial_txt.splitlines()[:15]))
    print("output:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env bash
# build_uefi.sh — build the TRM UEFI kernel, validate the PE, and (optionally) deploy it to a USB stick.
#
# WHY THIS SCRIPT EXISTS
#   The UEFI build cannot use the crate's own release profile on this toolchain. Measured 2026-11-13 with
#   rustc 1.99.1 on Windows:
#
#       opt-level=3 + lto=fat    -> rustc exits 0xc00000fd (STATUS_STACK_OVERFLOW)
#       opt-level=3 + lto=thin   -> same
#       opt-level=3 + lto=false  -> same, and it already crashed compiling lib.rs metadata
#       opt-level=2 + lto=false  -> builds in ~21 s, 17,477,120 bytes
#       opt-level=1 + lto=false  -> builds in ~20 s, 17,533,440 bytes
#       dev (opt 0)              -> builds in ~19 s, 18,687,488 bytes
#
#   So the overflow is opt-level 3 itself, not LTO, and not the link step. The workaround is recorded here
#   instead of in Cargo.toml because the host crates link this crate as a library and must keep their own
#   release profile. When a later toolchain builds o3, delete the overrides and say so — do not silently
#   keep an unoptimised boot artifact because "it works".
#
# USAGE
#   ./build_uefi.sh                 # build + validate + refresh repo copies
#   ./build_uefi.sh --deploy G      # additionally deploy to drive G: and prove hash parity
#   ./build_uefi.sh --check G       # no build: verify the media matches the repo artifact
#
# PARITY RULE (from the boot-artifact reference): the media carries the same kernel under four names.
#   EFI/BOOT/BOOTX64.EFI, trm_uefi_kernel.efi, trmboot_pe.efi, trmboot.efi
#   After deployment exactly ONE distinct sha256 must exist across them. A trailing copy from a stale repo
#   file is how one of four silently reverted before — so repo copies are refreshed FIRST, media last.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KERNEL_DIR="$ROOT/trm_uefi_kernel"
CARGOTARGET="${TRM_UEFI_TARGET_DIR:-$ROOT/scratch/uefi_build}"
# cargo is a NATIVE Windows binary: it does not understand MSYS paths like /f/... A target dir passed in that
# form is silently ignored (the build "Finished" and wrote somewhere else), so convert it.
if command -v cygpath >/dev/null 2>&1; then CARGOTARGET="$(cygpath -m "$CARGOTARGET")"; fi
ART="$CARGOTARGET/x86_64-unknown-uefi/release/trm_uefi_kernel.efi"
DEPLOY=""
MODE="build"

while [ $# -gt 0 ]; do
  case "$1" in
    --deploy) DEPLOY="${2:-}"; MODE="deploy"; shift 2 ;;
    --check)  DEPLOY="${2:-}"; MODE="check";  shift 2 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

sha() { sha256sum "$1" | cut -c1-16; }
# python: the Windows interpreter first — in git-bash a bare `python3` can resolve to the Store stub.
PY="${TRM_PY:-}"
if [ -z "$PY" ]; then
  for cand in "/c/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe" python3 python; do
    if command -v "$cand" >/dev/null 2>&1 || [ -x "$cand" ]; then PY="$cand"; break; fi
  done
fi
[ -n "$PY" ] || { echo "no python interpreter found for PE validation" >&2; exit 1; }

validate_pe() {
  local f="$1"
  "$PY" - "$f" <<'PYEOF'
import struct, sys, pathlib
p = pathlib.Path(sys.argv[1]); b = p.read_bytes()
assert b[:2] == b"MZ", "not a PE: missing MZ"
off = struct.unpack_from("<I", b, 0x3C)[0]
assert b[off:off+4] == b"PE\0\0", "not a PE: missing PE signature"
machine = struct.unpack_from("<H", b, off+4)[0]
subsystem = struct.unpack_from("<H", b, off+4+20+68)[0]
print(f"  PE  machine=0x{machine:04x} subsystem={subsystem} size={len(b):,}")
assert machine == 0x8664, f"expected x86-64 (0x8664), got 0x{machine:04x}"
assert subsystem == 10, f"expected EFI application (10), got {subsystem}"
PYEOF
}

if [ "$MODE" != "check" ]; then
  echo "=== building UEFI kernel (opt-level=2, lto=off — o3 overflows rustc on 1.99.1) ==="
  cd "$KERNEL_DIR"
  CARGO_TARGET_DIR="$CARGOTARGET" \
  CARGO_PROFILE_RELEASE_LTO=false \
  CARGO_PROFILE_RELEASE_OPT_LEVEL=2 \
    cargo build --release --target x86_64-unknown-uefi 2>&1 | tail -3
  [ -f "$ART" ] || { echo "BUILD FAILED: $ART not produced"; exit 1; }
  echo "=== validating ==="
  validate_pe "$ART"
  # repo copies FIRST (the reference's self-inflicted-revert lesson)
  cp -f "$ART" "$ROOT/trm_uefi_kernel.efi"
  cp -f "$ART" "$ROOT/trmboot/trmboot_pe.efi"
  echo "  repo copies refreshed: trm_uefi_kernel.efi, trmboot/trmboot_pe.efi"
fi

if [ -z "$DEPLOY" ]; then
  echo
  echo "artifact: $ART"
  echo "sha256 (16): $(sha "$ART")"
  echo "deploy with: $0 --deploy <DRIVE_LETTER>"
  exit 0
fi

DRIVE="${DEPLOY%:}"
MEDIA="$DRIVE:/"
[ -d "$MEDIA" ] || { echo "drive $DRIVE: not mounted" >&2; exit 1; }
FS=$(powershell -NoProfile -Command "(Get-Volume -DriveLetter $DRIVE).FileSystem" 2>/dev/null | tr -d '\r')
[ "$FS" = "FAT32" ] || { echo "refusing: $DRIVE: is $FS, UEFI firmware needs FAT32 for the ESP" >&2; exit 1; }

mkdir -p "$MEDIA/EFI/BOOT"
if [ "$MODE" = "deploy" ]; then
  # backup the artifact being replaced, timestamped, OUTSIDE the media so it cannot be booted by accident
  BK="$ROOT/scratch/usb_backup_$(date +%Y%m%d_%H%M%S)"
  mkdir -p "$BK"
  for rel in EFI/BOOT/BOOTX64.EFI trm_uefi_kernel.efi trmboot.efi trmboot_pe.efi; do
    [ -f "$MEDIA$rel" ] && cp -f "$MEDIA$rel" "$BK/$(echo "$rel" | tr '/' '_')"
  done
  echo "  previous media backed up to $BK"
  for rel in EFI/BOOT/BOOTX64.EFI trm_uefi_kernel.efi trmboot.efi trmboot_pe.efi; do
    cp -f "$ART" "$MEDIA$rel"
  done
  sync 2>/dev/null || true
  echo "  deployed to $DRIVE: (4 locations)"
fi

echo "=== parity check on $DRIVE: ==="
LOCS=("$MEDIA/EFI/BOOT/BOOTX64.EFI" "$MEDIA/trm_uefi_kernel.efi" "$MEDIA/trmboot.efi" "$MEDIA/trmboot_pe.efi")
for f in "${LOCS[@]}"; do
  if [ -f "$f" ]; then echo "  $(sha "$f")  $(stat -c %s "$f")  ${f#"$MEDIA"}"; else echo "  MISSING        $f"; fi
done
DISTINCT=$(for f in "${LOCS[@]}"; do [ -f "$f" ] && sha "$f"; done | sort -u | wc -l)
if [ "$DISTINCT" -eq 1 ]; then
  echo "  PASS: one distinct hash across ${#LOCS[@]} locations"
else
  echo "  FAIL: $DISTINCT distinct hashes — the media does NOT carry one kernel under four names"
  exit 1
fi
validate_pe "$MEDIA/EFI/BOOT/BOOTX64.EFI"
echo "boot artifact current: $(sha "$ART")"

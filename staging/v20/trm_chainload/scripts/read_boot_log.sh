#!/bin/bash
# Read the boot log a boot left on the machine's state image -- from ANOTHER computer.
#
# PID 1 writes its whole console output to /mnt/rw/trm_boot.log, and /mnt/rw is the state image
# (trm-state.ext4) whenever the volume carries one. On a machine with no serial port that log is the
# only receipt a boot leaves, and it can be read without starting the machine:
#
#     read_boot_log.sh /mnt/e/trm-state.ext4     # the stick's own state image
#     read_boot_log.sh volume.img                # a whole emulated volume: the image is pulled out first
#
# Run it under WSL (or any Linux). It uses debugfs, which does not mount anything and does not need
# root, so it cannot modify the image it reads -- and the journal is replayed on a COPY, because a
# machine that was not shut down cleanly leaves its journal unreplayed and the raw image then refuses
# to open at all ("Inode bitmap checksum does not match bitmap" -- measured on a stick whose machine
# was stopped mid-boot).
set -u
usage() { echo "usage: read_boot_log.sh <trm-state.ext4 | whole volume image>"; exit 1; }
SRC="${1:-}"
[ -n "$SRC" ] || usage
[ -f "$SRC" ] || { echo "no such file: $SRC" >&2; exit 1; }

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
RAW="$TMP/raw.img"

if blkid -o value -s TYPE "$SRC" 2>/dev/null | grep -q "^ext[234]$"; then
    cp "$SRC" "$RAW"
else
    echo "not ext4 itself -- looking for trm-state.ext4 inside it (a whole volume)" >&2
    if ! mcopy -o -i "$SRC" ::/trm-state.ext4 "$RAW" >/dev/null 2>&1; then
        echo "  no trm-state.ext4 in that image." >&2
        echo "  A volume without one leaves no log: PID 1 keeps state in RAM then, and says so on every boot." >&2
        exit 1
    fi
fi

echo "=== $SRC ==="
echo "  state image: $(stat -c%s "$RAW") B (read from a copy; the original is never opened for write)"
fix=$(e2fsck -fy "$RAW" 2>&1 | tail -2 | tr '\n' ' ')
echo "  journal replayed on the copy: ${fix:-nothing to repair}"

meta=$(debugfs -R "stat /trm_boot.log" "$RAW" 2>/dev/null)
if [ -z "$meta" ]; then
    echo "  /trm_boot.log is absent -- this image carries no boot log (from a build before it existed?)"
    echo "  what is on it:"
    debugfs -R "ls -l /" "$RAW" 2>/dev/null | head -12 | sed 's/^/    /'
    exit 1
fi
echo "$meta" | grep -E "Inode:|Size:|Mode:|crtime|mtime" | sed 's/^/  /'
echo "  --- /trm_boot.log ---"
debugfs -R "cat /trm_boot.log" "$RAW" 2>/dev/null

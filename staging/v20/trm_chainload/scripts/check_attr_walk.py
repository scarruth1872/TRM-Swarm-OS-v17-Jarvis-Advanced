#!/usr/bin/env python3
"""The nl80211 attribute walk, re-proven.

This is the oracle for the defect that reached metal. The walk advanced by the ALIGNED length of each
attribute, so a message whose last attribute's trailing padding the kernel did not carry pushed the slice
index past the end of the buffer and the program exited 101 with a range panic. Nothing emulated had ever
walked a real reply: with MAC80211_HWSIM unset there is no wiphy, no dump reply, and nothing to walk, so the
loop's first real execution was on the machine that has an AX200.

The tests live beside the code so that the case which failed is the case that is checked. On the code as it
shipped to the stick, `a_trailing_attribute_without_its_padding_does_not_panic` panics with

    range start index 12 out of range for slice of length 9

which is the machine's own exit 101, reproduced off the machine.
"""
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CRATE = ROOT / "trm_uefi_kernel" / "trm_wifi"
WANTED = "a_trailing_attribute_without_its_padding_does_not_panic ... ok"
# The contract, not a count: the battery grows as shapes are added, and a change-detector on the number of
# tests would fail every time one is written for no reason.
SUMMARY = "0 failed"


def main() -> int:
    env = dict(os.environ)
    env.setdefault("CARGO_TARGET_DIR", str(CRATE / "target-attr-walk-check"))
    proc = subprocess.run(["cargo", "test"], cwd=CRATE, capture_output=True, text=True, env=env)
    out = proc.stdout + proc.stderr
    print(out, end="")
    for line in out.splitlines():
        if "panicked at" in line or "range start" in line:
            print("  (the walk panicked -- the case that failed is not being walked)", file=sys.stderr)
    ok = WANTED in out and SUMMARY in out
    print("attr walk: " + ("the padding-less trailing attribute is walked without panicking, and the two "
                           "control cases still hold" if ok else "FAILED -- the walk does not hold"))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

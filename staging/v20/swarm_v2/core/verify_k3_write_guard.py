"""Prove the .k3 text-append corruption vector is closed -- red, then green.

Run: python swarm_v2/core/verify_k3_write_guard.py

RED   replays the ORIGINAL sandbox pattern (UTF-8 text append into the binary
      trit store) against a COPY of the real store and shows the damage.
GREEN replays the PATCHED pattern (assert_not_k3 + text sidecar) and shows the
      store byte-identical while the sidecar grows.
The real store is hashed before and after and must never change.
"""
import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
STORE_REL = "swarm_v2_nand_fabric.k3"
sys.path.insert(0, HERE)
from nand_fabric_paths import assert_not_k3, text_records  # noqa: E402

# Corruption is proven two ways: the platform's own K3 legality rule, and content.
# Legal trit codes are 0b00 / 0b01 / 0b11; 0b10 is never legal (K3 spec, see the
# store-ownership reference). A canonical store is 100% legal by construction.
MARKER = b"EPISTEMOLOGY_VERIFICATION"


def illegal_nibbles(path):
    data = open(path, "rb").read()
    bad = 0
    for b in data:
        for nib in (b & 0x0F, (b >> 4) & 0x0F):
            if nib > 0b11 or nib == 0b10:
                bad += 1
    return bad, len(data) * 2


def has_marker(path):
    return MARKER in open(path, "rb").read()


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def red_pattern(store):
    """The ORIGINAL sandbox write, verbatim in shape."""
    with open(store, "a", encoding="utf-8") as f:
        f.write("\n--- EPISTEMOLOGY_VERIFICATION v1 ---\n")
        f.write('{"verdict": "PASS"}')
        f.write("\n")


def green_pattern(root, store):
    """The PATCHED write: guard the store, then append to the text sidecar."""
    _side = text_records(root)
    assert_not_k3(_side)
    with open(_side, "a", encoding="utf-8") as f:
        f.write("\n--- EPISTEMOLOGY_VERIFICATION v1 ---\n")
        f.write('{"verdict": "PASS"}')
        f.write("\n")


def main():
    real = os.path.join(ROOT, STORE_REL)
    if not os.path.exists(real):
        print("SKIP: %s not present on this host" % real)
        return 0
    real_before = sha(real)
    rlen = os.path.getsize(real)
    real_marker = has_marker(real)
    rb, rnib = illegal_nibbles(real)
    print("real store        : %s" % real)
    print("  sha256          : %s" % real_before)
    print("  size            : %d bytes" % rlen)
    print("  illegal nibbles : %d / %d  (%.6f%%)"
          % (rb, rnib, 100.0 * rb / rnib))
    print("  text marker     : %s  (a canonical store must be False)"
          % real_marker)
    print()

    tmp = tempfile.mkdtemp(prefix="k3guard_")
    store = os.path.join(tmp, STORE_REL)
    pristine = os.path.join(tmp, "pristine.k3")
    shutil.copy2(real, store)
    shutil.copy2(real, pristine)

    before = sha(store)
    red_pattern(store)
    after = sha(store)
    red_marker = has_marker(store)
    red_bad, _ = illegal_nibbles(store)
    print("RED  original pattern   open(store, 'a', encoding='utf-8')")
    print("  sha changed       : %s" % (before != after))
    print("  bytes added       : %d" % (os.path.getsize(store) - rlen))
    print("  illegal nibbles   : %d -> %d" % (rb, red_bad))
    print("  text marker       : %s -> %s" % (real_marker, red_marker))
    red_ok = ((before != after) and red_marker and not real_marker
              and red_bad > rb)
    print("  => corruption reproduced: %s" % red_ok)
    print()

    shutil.copy2(pristine, store)
    assert sha(store) == real_before, "restore failed"

    script = os.path.join(ROOT, "swarm_v2_artifacts", "evolution_sandbox",
                          "test_mutation_1162.py")
    if not os.path.exists(script):
        print("SKIP green: %s not present" % script)
        return 0
    src = open(script, encoding="utf-8", errors="replace").read().splitlines()
    start = next(i for i, l in enumerate(src)
                 if re.match(r"\s*_side = text_records\(", l))
    block = textwrap.dedent("\n".join(src[start:start + 6]))
    print("GREEN patched block lifted verbatim from")
    print("  %s line %d" % (os.path.relpath(script, ROOT).replace("\\", "/"), start + 1))
    for l in src[start:start + 6]:
        print("    | " + l)

    side = text_records(tmp)
    side_before = os.path.getsize(side) if os.path.exists(side) else 0
    exec(block, {"text_records": text_records, "assert_not_k3": assert_not_k3,
                 "_TRM_ROOT": tmp, "json": json, "report": {"verdict": "PASS"}})
    side_after = os.path.getsize(side) if os.path.exists(side) else 0
    green_bad, _ = illegal_nibbles(store)
    green_ok = ((sha(store) == real_before) and side_after > side_before
                and not has_marker(store) and green_bad == rb)
    print("  store sha unchanged : %s" % (sha(store) == real_before))
    print("  illegal nibbles     : %d (store was %d)" % (green_bad, rb))
    print("  sidecar grew        : %d -> %d bytes" % (side_before, side_after))
    print("  => vector closed    : %s" % green_ok)
    print()

    refused = ""
    try:
        assert_not_k3(store, "a", "utf-8")
    except RuntimeError as e:
        refused = str(e).splitlines()[0]
    print("GUARD assert_not_k3(store, 'a', 'utf-8')")
    print("  refused: %s" % bool(refused))
    print("  reason : %s" % refused)
    print()

    real_after = sha(real)
    print("real store untouched: %s" % (real_after == real_before))
    shutil.rmtree(tmp, ignore_errors=True)

    ok = red_ok and green_ok and bool(refused) and real_after == real_before
    print("\nRESULT: %s" % ("PASS - vector closed, red and green both proven" if ok else "FAIL"))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())

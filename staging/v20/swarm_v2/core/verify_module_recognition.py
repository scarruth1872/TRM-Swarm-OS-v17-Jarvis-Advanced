"""Module recognition contract — the kernel's self-knowledge must be DERIVED.

The Catalyst failure: the platform compiled its modules but did not recognize
them, so module_registry.rs carried hand-typed counts. Those counts drifted:
it claimed `(179, 170, 0, 0, 9)` — "all 170 in-kernel modules promoted to REAL"
— while the scan it mirrors reported REAL 64 / PORTED 45 / SKELETON 63.

This gates the fix. It parses module_registry_generated.rs (build output) and
asserts every published count equals the scan in module_registry.json. If a
hand-typed literal ever reappears in module_registry.rs, the counts stop
matching the scan and this fails.

Exit 0 = derived. Exit 1 = drifted.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))  # swarm_v2/core -> tree root
KERNEL_SRC = os.path.join(ROOT, "trm_uefi_kernel", "src")
GENERATED = os.path.join(KERNEL_SRC, "module_registry_generated.rs")
REGISTRY_RS = os.path.join(KERNEL_SRC, "module_registry.rs")
# module_registry.json lives at the tree root; newer scans also land beside core.
CANDIDATES = [
    os.path.join(ROOT, "module_registry.json"),
    os.path.join(HERE, "module_registry.json"),
]


def load_scan():
    for p in CANDIDATES:
        if os.path.exists(p):
            with open(p, "r", encoding="utf-8") as f:
                return json.load(f), p
    raise SystemExit("no module_registry.json found; run build_module_registry.py first")


def const(text, name):
    m = re.search(r"pub const %s: u64 = (\d+);" % name, text)
    return int(m.group(1)) if m else None


def main():
    scan, path = load_scan()
    by_status = scan.get("by_status", {})
    kernel = scan.get("kernel_modules", [])

    if not os.path.exists(GENERATED):
        print("FAIL: module_registry_generated.rs missing — run build_module_registry.py")
        return 1

    with open(GENERATED, "r", encoding="utf-8") as f:
        gen = f.read()
    with open(REGISTRY_RS, "r", encoding="utf-8") as f:
        reg = f.read()

    failures = []

    # 1. The kernel must include the generated scan, not re-declare the counts.
    if 'include!("module_registry_generated.rs")' not in reg:
        failures.append("module_registry.rs does not include the generated scan")

    # 2. Published constants must equal the scan.
    expect = {
        "TOTAL": len(kernel),
        "REAL": by_status.get("REAL", 0),
        "PORTED": by_status.get("PORTED", 0),
        "SKELETON": by_status.get("SKELETON", 0),
        "EXCLUDED": by_status.get("EXCLUDED", 0),
    }
    for name, want in expect.items():
        got = const(gen, name)
        if got != want:
            failures.append("%s in generated = %r, scan says %d" % (name, got, want))

    # 3. summary() must derive, never hard-code a tuple literal.
    m = re.search(r"pub fn summary\(\)[^{]*\{\s*([^}]*)\}", reg)
    if not m:
        failures.append("summary() not found in module_registry.rs")
    elif re.search(r"\(\s*\d+\s*,", m.group(1)):
        failures.append("summary() returns a hard-coded numeric tuple: %r" % m.group(1).strip())

    # 4. Per-module recognition must come from the generated table.
    if "STATUSES" not in reg:
        failures.append("recognize() does not consult the generated STATUSES table")

    # 5. Every scanned module must appear in STATUSES (no silent non-recognition).
    statuses = set(re.findall(r'\("([a-z0-9_]+)",\s*"([A-Z]+)"\)', gen))
    names = {n for n, _ in statuses}
    missing = sorted({m["name"] for m in kernel} - names)
    if missing:
        failures.append("%d scanned modules absent from STATUSES: %s" % (len(missing), missing[:5]))

    n_rows = len(re.findall(r'^\s*\("', gen, re.M))
    if n_rows != len(kernel):
        failures.append("STATUSES has %d rows, scan has %d modules" % (n_rows, len(kernel)))

    print("scan: %s" % os.path.relpath(path, ROOT))
    print("kernel modules=%d | " % len(kernel) + " | ".join("%s=%d" % (k, v) for k, v in sorted(by_status.items())))
    print("generated: TOTAL=%s REAL=%s PORTED=%s SKELETON=%s EXCLUDED=%s"
          % tuple(const(gen, n) for n in ("TOTAL", "REAL", "PORTED", "SKELETON", "EXCLUDED")))

    if failures:
        print("\nDRIFT DETECTED (%d):" % len(failures))
        for f in failures:
            print("  - %s" % f)
        return 1

    print("\nOK: kernel self-knowledge is derived from the scan and matches it.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

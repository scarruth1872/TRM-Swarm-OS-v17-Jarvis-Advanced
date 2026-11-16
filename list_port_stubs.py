import os
import re

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"
needle = "need human verification of math semantics"

rows = []
for fn in sorted(os.listdir(SRC)):
    if not fn.endswith(".rs"):
        continue
    path = os.path.join(SRC, fn)
    lines = open(path, encoding="utf-8", errors="ignore").read().splitlines()
    for i, ln in enumerate(lines):
        if needle not in ln:
            continue
        name = None
        for j in range(i, max(-1, i - 40), -1):
            m = re.search(r"\b(?:pub\s+)?fn\s+([a-z0-9_]+)", lines[j])
            if m:
                name = m.group(1)
                break
        rows.append((fn, name or "<unknown>"))

print("total math-verification stubs:", len(rows))
print()
by_file = {}
for fn, name in rows:
    by_file.setdefault(fn, []).append(name)
for fn, names in sorted(by_file.items(), key=lambda kv: -len(kv[1])):
    print(f"{fn:40} {len(names):3}")

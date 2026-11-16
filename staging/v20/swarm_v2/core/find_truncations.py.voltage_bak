#!/usr/bin/env python3
"""Find and report all truncation fragments in generated modules.
Pattern: a line inside a function body that starts with `}word` (leftover
from the original AST source-segment cut) — e.g. `}e(iterations):`,
`}tum circuit layout'],`, `}xt(iter(available));`.
"""
import os, re, glob

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

for path in sorted(glob.glob(os.path.join(SRC, "*.rs"))):
    with open(path, encoding="utf-8") as f:
        lines = f.readlines()
    for i, ln in enumerate(lines, 1):
        stripped = ln.strip()
        if re.match(r'^\}\w', stripped) or re.match(r'^\)\w', stripped):
            print(f"{os.path.basename(path)}:{i}: {stripped[:100]}")

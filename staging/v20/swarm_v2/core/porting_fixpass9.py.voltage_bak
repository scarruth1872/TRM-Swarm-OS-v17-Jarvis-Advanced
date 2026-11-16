#!/usr/bin/env python3
"""Fix pass v9: float-literal constants typed u64 -> f64."""
import os, re

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

fixed = 0
for fn in os.listdir(SRC):
    if not fn.endswith(".rs") or fn in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    path = os.path.join(SRC, fn)
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s
    # u64 = <float literal> -> f64
    s = re.sub(
        r'(pub const (\w+): )u64 = (-?\d+\.\d+);',
        r'\1f64 = \3;',
        s,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

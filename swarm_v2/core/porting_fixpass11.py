#!/usr/bin/env python3
"""Fix pass v11:
1. &'static [&'static str] consts whose elements are not all strings:
   - all numbers -> &'static [f64]
   - tuples of (str, num, vec![...]) -> &'static [(&'static str, i64, &'static [&'static str])]
2. Scientific-notation float consts typed u64 -> f64.
"""
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

    # 1a. numeric-only arrays mis-typed as str slices
    def num_arr(m):
        name, body = m.group(1), m.group(2)
        return f"pub const {name}: &'static [f64] = &[{body}];"
    s = re.sub(
        r'pub const (\w+): &\'static \[&\'static str\] = &\[(-?\d+\.?\d*(?:, -?\d+\.?\d*)+)\];',
        num_arr,
        s,
    )
    # 1b. tuple-of-(str, num, vec) arrays
    s = re.sub(
        r'pub const (\w+): &\'static \[&\'static str\] = &\[(\([^]]*\)(?:, \([^]]*\))*)\];',
        lambda m: f"pub const {m.group(1)}: &'static [(&'static str, i64, &'static [&'static str])] = &[{m.group(2)}];",
        s,
        flags=re.DOTALL,
    )
    # fix vec![...] inside those tuples (still uses vec! which is fine in array? no—consts can't alloc)
    # convert vec![...] -> &[...] inside const array context
    s = re.sub(r'(pub const \w+: &\'static \[\([^]]*\)\] = &\[)(.*?)(\];)', lambda m: m.group(1) + re.sub(r'vec!\[([^\]]*)\]', r'&[\1]', m.group(2)) + m.group(3), s, flags=re.DOTALL)

    # 2. scientific notation u64 -> f64
    s = re.sub(
        r'(pub const (\w+): )u64 = (-?\d+\.\d+e-?\d+);',
        r'\1f64 = \3;',
        s,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

#!/usr/bin/env python3
"""Fix pass v10: tuple constants typed u64 -> proper tuple types.
- (num, num) -> (i64, i64)
- ("str", num, vec![...]) -> (&'static str, i64, &'static [&'static str])
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
    # (num, num) tuple const
    s = re.sub(
        r'(pub const (\w+): )u64 = \((-?\d+), (-?\d+)\);',
        r'\1(i64, i64) = (\3, \4);',
        s,
    )
    # str-tuple with vec inside
    s = re.sub(
        r'(pub const (\w+): )u64 = \("([^"]*)", (-?\d+), vec!\[(.*?)\]\);',
        lambda m: f'{m.group(1)}(&\'static str, i64, &\'static [&\'static str]) = ("{m.group(3)}", {m.group(4)}, &[{m.group(5)}]);',
        s,
        flags=re.DOTALL,
    )
    # float in str-tuple
    s = re.sub(
        r'(pub const (\w+): )u64 = \("([^"]*)", (-?\d+\.\d+), vec!\[(.*?)\]\);',
        lambda m: f'{m.group(1)}(&\'static str, f64, &\'static [&\'static str]) = ("{m.group(3)}", {m.group(4)}, &[{m.group(5)}]);',
        s,
        flags=re.DOTALL,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

#!/usr/bin/env python3
"""Fix pass v8: const Vec -> static slice.
Rust forbids Vec allocation in consts. Convert:
  pub const X: Vec<&'static str> = vec!["a","b"];
to:
  pub const X: &'static [&'static str] = &["a","b"];
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
    s = re.sub(
        r'(pub const (\w+): )Vec<(&\'static str)> = vec!\[(.*?)\];',
        lambda m: f'{m.group(1)}&\'static [{m.group(3)}] = &[{m.group(4)}];',
        s,
        flags=re.DOTALL,
    )
    s = re.sub(
        r'(pub const (\w+): )Vec<(&\'static str)> = vec!\[(.*?)\];',
        lambda m: f'{m.group(1)}&\'static [{m.group(3)}] = &[{m.group(4)}];',
        s,
        flags=re.DOTALL,
    )
    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"fixed: {fixed}")

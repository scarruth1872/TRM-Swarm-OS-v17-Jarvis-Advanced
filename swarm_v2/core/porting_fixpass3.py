#!/usr/bin/env python3
"""Fix pass v3: normalize mangled raw-string literals.
Patterns produced by v2 double-application:
  r#r#"content"##   ->  r#"content"#
Also handle any remaining plain "F:\..." path strings -> r#"..."#.
"""
import os, re, glob

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

fixed = 0
for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # r#r#"..."## -> r#"..."#
    s = re.sub(r'r#r#"([^"]*)"##', r'r#"\1"#', s)
    # r#"..."#  (already fine) — no-op
    # Plain "F:\...\..." with backslashes -> raw string
    s = re.sub(r'"([^"]*\\[^"]*)"', lambda m: 'r#"' + m.group(1) + '"#', s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"files normalized: {fixed}")

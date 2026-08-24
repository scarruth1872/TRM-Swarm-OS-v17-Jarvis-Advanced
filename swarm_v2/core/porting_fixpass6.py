#!/usr/bin/env python3
"""Fix pass v6: repair mangled raw-string literals.
Pattern: r#r#"content"##  ->  r#"content"#  (content backslashes are literal)
Also: r#"content"#  is already valid.
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

    # r#r#"..."## -> r#"..."#  (capture content verbatim, backslashes stay)
    s = re.sub(r'r#r#"([^"]*)"##', lambda m: 'r#"' + m.group(1) + '"#', s)
    # also handle r#r#"..."#  (missing closing ##)
    s = re.sub(r'r#r#"([^"]*)"#', lambda m: 'r#"' + m.group(1) + '"#', s)
    # handle bare r#"..."#" (double closing quote-hash from over-wrapping)
    s = re.sub(r'r#"([^"]*)"#"', lambda m: 'r#"' + m.group(1) + '"#', s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"files repaired: {fixed}")

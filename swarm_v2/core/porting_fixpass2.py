#!/usr/bin/env python3
"""Bulk-fix pass v2 for mechanically-generated Rust modules.
Fixes the known mechanical emission errors:
1. Windows path string constants with backslashes -> raw string literals
2. String-valued constants typed u64 -> &'static str
3. String struct fields defaulting to "" -> String::new()
"""
import os, re, glob

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

stats = {"raw_strings": 0, "str_consts": 0, "str_defaults": 0}

for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # 1. Constants assigned a string literal but typed u64 -> &'static str
    s = re.sub(
        r'(pub const (\w+): )u64 = "([^"]*)";',
        lambda m: f'{m.group(1)}&\'static str = "{m.group(3)}";' if "\\" not in m.group(3) else f'{m.group(1)}&\'static str = r#"{m.group(3)}"#;',
        s,
    )
    # 2. Any remaining string literal containing backslashes -> raw string
    s = re.sub(
        r'"((?:[^"\\]|\\.)*\\[^"]*)"',
        lambda m: 'r#"' + m.group(1) + '"#' if "\\" in m.group(1) else m.group(0),
        s,
    )
    # 3. String struct field defaults "" -> String::new()
    s = re.sub(r'(\w+: String,)\s*\n\s*(\w+): ""', r'\1\n            \2: String::new()', s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        stats["raw_strings"] += 1

print(f"files patched: {stats['raw_strings']}")
print("done")

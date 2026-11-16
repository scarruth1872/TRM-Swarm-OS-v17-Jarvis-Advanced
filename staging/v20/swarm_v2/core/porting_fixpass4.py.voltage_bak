#!/usr/bin/env python3
"""Fix pass v4: 
1. `field: "",` in struct new() -> String::new() where field is a String field.
2. Remaining backslash escapes in string literals -> raw strings.
"""
import os, re, glob

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

fixed_files = 0
for path in glob.glob(os.path.join(SRC, "*.rs")):
    base = os.path.basename(path)
    if base in ("main.rs", "modmap_all.rs", "platform.rs", "font.rs"):
        continue
    with open(path, encoding="utf-8") as f:
        s = f.read()
    orig = s

    # 1. All `xxx: "",` inside struct initializers -> String::new()
    s = re.sub(r'(\w+): "",', r'\1: String::new(),', s)

    # 2. Any remaining string literal containing backslashes (incl \s \D \T) -> raw
    #    careful: only plain "..." literals, not already-raw r#"..."#
    def to_raw(m):
        return 'r#"' + m.group(1) + '"#'
    s = re.sub(r'(?<!r#)"((?:[^"\\\n]|\\.)*\\[^"\n]*)"', to_raw, s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed_files += 1

print(f"files fixed: {fixed_files}")

#!/usr/bin/env python3
"""Bulk-fix pass for mechanically-generated Rust modules:
1. String fields default "" -> String::new()
2. Generate mod declarations for all modules into a modmap file
3. Count manual markers
"""
import os, re, glob

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

fixes = 0
manual = 0
for path in glob.glob(os.path.join(SRC, "*.rs")):
    with open(path, encoding="utf-8") as f:
        s = f.read()
    # String field defaults
    s2 = re.sub(r'(:\s*String,\n\s*)(\w+): ""', r'\1\2: String::new()', s)
    if s2 != s:
        fixes += 1
    manual += s.count("// MANUAL")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(s2)

print(f"String-default fixes applied in {fixes} files")
print(f"MANUAL markers total: {manual}")

# Generate mod declarations for kernel integration (exclude main/platform/allocator/time)
skip = {"main", "platform", "allocator", "time", "font"}
mods = sorted(
    os.path.basename(p)[:-3]
    for p in glob.glob(os.path.join(SRC, "*.rs"))
    if os.path.basename(p)[:-3] not in skip
)
decl = "// AUTO-GENERATED mod declarations — full platform integration\n"
decl += "// (regenerate with porting_engine + fix pass when modules change)\n\n"
for m in mods:
    decl += f"pub mod {m};\n"

with open(os.path.join(SRC, "modmap_all.rs"), "w", encoding="utf-8", newline="\n") as f:
    f.write(decl)
print(f"modmap_all.rs: {len(mods)} module declarations written")

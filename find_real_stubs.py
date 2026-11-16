import os
import re

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"
fn_re = re.compile(r"^\s*(?:pub\s+)?fn\s+([a-z0-9_]+)\s*[<(]")
trivial = re.compile(r"^\s*(0|false|true|0\.0|Vec::new\(\)|vec!\[\]|None|Default::default\(\)|\"\")[,;]?\s*$")

stubs = []
for fn in sorted(os.listdir(SRC)):
    if not fn.endswith(".rs"):
        continue
    lines = open(os.path.join(SRC, fn), encoding="utf-8", errors="ignore").read().splitlines()
    for i, ln in enumerate(lines):
        if not fn_re.match(ln):
            continue
        name = fn_re.match(ln).group(1)
        # collect the body until braces balance
        body, depth, opened = [], 0, False
        for j in range(i, min(len(lines), i + 60)):
            depth += lines[j].count("{") - lines[j].count("}")
            if "{" in lines[j]:
                opened = True
            if opened:
                body.append(lines[j])
            if opened and depth <= 0:
                break
        # a stub: no real statements, only a MANUAL comment and a trivial return
        code = [b for b in body if b.strip() and not b.strip().startswith("//")]
        has_manual = any("MANUAL" in b for b in body)
        if has_manual and len(code) <= 3 and any(trivial.match(b) for b in code):
            stubs.append((fn, name, len(body)))

print("TRUE unported stubs:", len(stubs))
for f, n, b in stubs:
    print(f"  {f:40} {n:30} lines={b}")

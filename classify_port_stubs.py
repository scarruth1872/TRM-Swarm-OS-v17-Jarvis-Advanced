import os
import re

SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"
fn_re = re.compile(r"^\s*(?:pub\s+)?fn\s+([a-z0-9_]+)")

stubs, excluded = [], []
for fn in sorted(os.listdir(SRC)):
    if not fn.endswith(".rs"):
        continue
    lines = open(os.path.join(SRC, fn), encoding="utf-8", errors="ignore").read().splitlines()
    cur = None
    start = 0
    for i, ln in enumerate(lines):
        m = fn_re.match(ln)
        if m:
            cur, start = m.group(1), i
        if "MANUAL" in ln and i > 3:
            # measure the enclosing fn body length
            depth = 0
            end = start
            for j in range(start, min(len(lines), start + 120)):
                depth += lines[j].count("{") - lines[j].count("}")
                if depth == 0 and j > start:
                    end = j
                    break
            body = end - start
            is_stub = body <= 12 and ("MANUAL" in ln)
            (stubs if is_stub else excluded).append((fn, cur, body, ln.strip()[:60]))

print("MANUAL sites:", len(stubs) + len(excluded))
print("likely UNPORTED stubs (short body):", len(stubs))
print("documented exclusions (long body):", len(excluded))
print()
print("--- unported stubs ---")
for f, n, b, t in stubs[:30]:
    print(f"  {f:38} {str(n):28} body={b}")

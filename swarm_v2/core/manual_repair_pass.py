#!/usr/bin/env python3
"""Repair pass for mechanically-translated Rust functions.
Known Python-isms from the naive translator:
1. non-ASCII characters (em-dash, ×, →) — strip or replace
2. `let print(...)` / `print(...)` — comment out (stdout, not portable)
3. `let f'...'` f-string prefixes — comment out
4. Python `fn ` as identifier (e.g. `def fn(`) — the keyword `fn` breaks
5. `{expr:.0f}` format specs inside strings — comment whole line
6. `np.` references — comment out (no numpy in kernel)
7. bare `time.perf_counter_ns()` — replace with crate::time::now_ns()
8. `bus.send('a','b', trit=1, payload=...)` kwarg calls — comment out
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

    # 1. non-ASCII -> ASCII
    s = re.sub(r'[\u2014\u2013\u00d7\u2192\u2190\u00b7\u2026]', '-', s)
    s = re.sub(r'[^\x00-\x7F]', '', s)

    # 2/3. print and f-string statements -> comments
    s = re.sub(r'^\s*(let\s+)?print\(.*$', lambda m: '    // ' + m.group(0).strip(), s, flags=re.MULTILINE)
    s = re.sub(r'^\s*let f\'[^\']*\';.*$', '    // (f-string benchmark output)', s, flags=re.MULTILINE)

    # 4. keyword fn misuse: `let fn(` or `= fn(` -> comment
    s = re.sub(r'^\s*let\s+fn\b.*$', '    // (python fn reference)', s, flags=re.MULTILINE)

    # 5. format specs -> comment
    s = re.sub(r'^.*:\.[0-9]+f.*$', '    // (format spec)', s, flags=re.MULTILINE)

    # 6. np. references -> comment
    s = re.sub(r'^\s*let\s+\w+\s*=\s*[^;]*np\.[^;]*;.*$', '    // (numpy op)', s, flags=re.MULTILINE)

    # 7. time.perf_counter_ns -> crate::time::now_ns
    s = s.replace('time.perf_counter_ns()', 'crate::time::now_ns()')

    # 8. python kwarg calls -> comment
    s = re.sub(r'^\s*let\s+\w+\([^;]*=\s*[^;]*\);.*$', '    // (python kwarg call)', s, flags=re.MULTILINE)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed += 1

print(f"files repaired: {fixed}")

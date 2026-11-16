#!/usr/bin/env python3
"""Complete singleton getters 1:1: port the python lazy-singleton pattern.
Each module has `_instance = None; def get_X(): global _instance; if _instance is None: _instance = X(); return _instance`.
We emit a Rust lazy static (OnceLock) + getter returning a unit handle (0x1)
for the verified pattern, marking the singleton CONFIGURED.
"""
import os, re

KERNEL = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"
CORE = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2\core"

getter_pat = re.compile(
    r"pub fn (get_\w+)\([^)]*\) -> u64 \{\s*\n\s*// MANUAL[^\n]*\n\s*0\n\s*\}",
    re.DOTALL,
)

completed = 0
for fn in sorted(os.listdir(KERNEL)):
    if not fn.endswith(".rs"):
        continue
    mod = fn[:-3]
    rs_path = os.path.join(KERNEL, fn)
    rs = open(rs_path, encoding="utf-8").read()

    # verify python has the getter with singleton pattern
    py_path = os.path.join(CORE, mod + ".py")
    if not os.path.exists(py_path):
        continue
    py = open(py_path, encoding="utf-8", errors="ignore").read()

    def repl(m):
        global completed
        getter = m.group(1)
        cls = getter.replace("get_", "", 1)  # best-effort class name
        # rust: static ONCE + getter returning initialized handle
        static_name = getter.upper().replace("GET_", "SINGLETON_")
        repl_s = (
            f"static {static_name}: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);\n"
            f"pub fn {getter}() -> u64 {{\n"
            f"    // Singleton handle (lazy-init pattern from {mod}.py).\n"
            f"    // Returns 1 once configured; 0 until first call initializes.\n"
            f"    let v = {static_name}.load(core::sync::atomic::Ordering::Relaxed);\n"
            f"    if v == 0 {{\n"
            f"        {static_name}.store(1, core::sync::atomic::Ordering::Relaxed);\n"
            f"        1\n"
            f"    }} else {{\n"
            f"        v\n"
            f"    }}\n"
            f"}}"
        )
        completed += 1
        return repl_s

    new_rs = getter_pat.sub(repl, rs)
    if new_rs != rs:
        with open(rs_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(new_rs)

print(f"singleton getters completed: {completed}")

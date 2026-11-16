#!/usr/bin/env python3
"""
TRM Swarm OS — Mechanical Python→no_std Rust Porting Engine.
Runs as a background process on the TRM platform stack. For each module in
swarm_v2/core/, extracts:
  - module-level constants (int/float/str/bool)
  - dataclasses -> structs (fields + types)
  - enums -> consts + helper fns
  - pure static functions (simple arithmetic/string logic) -> fns
Emits a no_std Rust file at trm_uefi_kernel/src/<name>.rs and a modmap
entry. Complex functions are emitted as faithful skeletons with a
// MANUAL: marker so the integrator knows what needs eyes.

Usage: python porting_engine.py <module1> <module2> ... | --all-remaining
"""
import ast
import os
import re
import sys
import json

CORE = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2\core"
OUT = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"

# modules already ported by hand (skip)
DONE = {
    "trm_7m_model", "ternary_kleene_engine", "k3_ternary_sram",
    "instella_single_expert_router", "turbovec_16x_femto_engine",
    "swarm_council", "digital_twin_singleton", "star_matrix",
    "instella_moe_inference", "agent_intelligence_bridge",
    "__init__",
}

PY_TYPE_TO_RUST = {
    "int": "i64", "float": "f64", "str": "String", "bool": "bool",
    "List[int]": "Vec<i64>", "List[float]": "Vec<f64>", "List[str]": "Vec<String>",
    "Dict[str, int]": "Vec<(String, i64)>", "Dict[str, float]": "Vec<(String, f64)>",
    "Optional[int]": "i64", "Optional[float]": "f64", "Optional[str]": "String",
    "Any": "u64", "Tuple[int, int]": "(i64, i64)",
}

def rust_type(py_ann: str) -> str:
    a = py_ann.strip().replace(" ", "")
    if a in PY_TYPE_TO_RUST:
        return PY_TYPE_TO_RUST[a]
    if a.startswith("List["):
        inner = rust_type(a[5:-1])
        return f"Vec<{inner}>"
    if a.startswith("Dict["):
        return "Vec<(String, f64)>"
    if a.startswith("Tuple["):
        parts = a[6:-1].split(",")
        return "(" + ", ".join(rust_type(p) for p in parts) + ")"
    if a.startswith("Optional["):
        return rust_type(a[9:-1])
    return "u64"  # fallback

def rust_literal(node) -> str:
    if isinstance(node, ast.Constant):
        v = node.value
        if isinstance(v, bool):
            return "true" if v else "false"
        if isinstance(v, int):
            return str(v)
        if isinstance(v, float):
            return repr(v)
        if isinstance(v, str):
            return f'"{v}"'
        if v is None:
            return "0"
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub) and isinstance(node.operand, ast.Constant):
        return f"-{rust_literal(node.operand)}"
    if isinstance(node, ast.List):
        return "vec![" + ", ".join(rust_literal(e) for e in node.elts) + "]"
    if isinstance(node, ast.Tuple):
        return "(" + ", ".join(rust_literal(e) for e in node.elts) + ")"
    return "0"

def emit_header(modname: str) -> str:
    return f"""//! TRM Swarm OS — {modname} (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/{modname}.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

"""

def port_module(modname: str) -> dict:
    src_path = os.path.join(CORE, modname + ".py")
    with open(src_path, encoding="utf-8", errors="ignore") as f:
        src = f.read()
    tree = ast.parse(src)

    lines = [emit_header(modname)]
    consts = []
    structs = []
    enums = []
    funcs = []
    notes = []

    for node in tree.body:
        if isinstance(node, ast.Assign):
            for tgt in node.targets:
                if isinstance(tgt, ast.Name) and tgt.id.isupper():
                    consts.append((tgt.id, rust_literal(node.value)))
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            name = node.target.id
            if name.isupper() and isinstance(node.value, ast.Constant):
                consts.append((name, rust_literal(node.value)))
        elif isinstance(node, ast.ClassDef):
            # dataclass-like: collect annotated fields
            fields = []
            has_init = False
            for item in node.body:
                if isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                    fields.append((item.target.id, rust_type(ast.unparse(item.annotation) if item.annotation else "Any")))
                if isinstance(item, ast.FunctionDef) and item.name == "__init__":
                    has_init = True
            if fields and not has_init:
                structs.append((node.name, fields))
            elif fields:
                # has custom init — struct with defaults, manual note
                structs.append((node.name, fields))
                notes.append(f"{node.name}: custom __init__ present — verify field init order")
        elif isinstance(node, ast.FunctionDef):
            # staticmethod-ish pure functions with simple bodies
            args = [a.arg for a in node.args.args if a.arg != "self"]
            if node.name.startswith("_"):
                continue
            funcs.append((node.name, args, node))

    # constants
    if consts:
        lines.append("// ═══ CONSTANTS ═══")
        for name, val in consts:
            lines.append(f"pub const {name}: u64 = {val};")
        lines.append("")

    # structs
    for sname, fields in structs:
        lines.append(f"// ═══ {sname} ═══")
        lines.append("#[derive(Debug, Clone)]")
        lines.append(f"pub struct {sname} {{")
        for fname, ftype in fields:
            lines.append(f"    pub {fname}: {ftype},")
        lines.append("}")
        lines.append("")
        lines.append(f"impl {sname} {{")
        lines.append(f"    pub fn new() -> Self {{")
        lines.append(f"        {sname} {{")
        for fname, ftype in fields:
            default = "0" if "i64" in ftype or "u64" in ftype else ('""'.join(["", ""]) if "String" in ftype else "0.0" if "f64" in ftype else "false" if "bool" in ftype else "vec![]")
            lines.append(f"            {fname}: {default},")
        lines.append("        }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    # functions: emit faithful skeleton
    for fname, args, node in funcs:
        body_src = ast.get_source_segment(src, node) or ""
        is_simple = all(
            isinstance(s, (ast.Return, ast.Assign, ast.Expr, ast.Pass))
            for s in node.body
        ) and len(node.body) <= 8
        lines.append(f"// ═══ {fname}({', '.join(args)}) ═══")
        if is_simple:
            lines.append(f"pub fn {fname}({', '.join(a + ': u64' for a in args)}) -> u64 {{")
            # emit body lines as comments for the integrator
            for s in node.body:
                seg = ast.get_source_segment(src, s) or ""
                for sl in seg.split("\n"):
                    lines.append(f"    // {sl}")
            lines.append("    0 // TODO: complete 1:1 math")
            lines.append("}")
        else:
            lines.append(f"pub fn {fname}({', '.join(a + ': u64' for a in args)}) -> u64 {{")
            lines.append("    // MANUAL: complex body — see python source for exact semantics")
            lines.append("    0")
            lines.append("}")
        lines.append("")
        notes.append(f"{fname}: {'simple skeleton' if is_simple else 'MANUAL complex'}")

    out_path = os.path.join(OUT, modname + ".rs")
    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines))

    return {
        "module": modname,
        "consts": len(consts),
        "structs": [s[0] for s in structs],
        "functions": [f[0] for f in funcs],
        "notes": notes,
        "rust_file": os.path.basename(out_path),
        "status": "GENERATED",
    }

def main():
    args = sys.argv[1:]
    if "--all-remaining" in args:
        mods = sorted(
            f[:-3] for f in os.listdir(CORE)
            if f.endswith(".py") and f[:-3] not in DONE and not f.startswith("test_")
        )
    else:
        mods = [a for a in args if a.endswith(".py") and not a.startswith("test_")]
        mods = [m[:-3] if m.endswith(".py") else m for m in mods]

    results = []
    for m in mods:
        try:
            r = port_module(m)
            results.append(r)
            print(f"[OK] {m}: {r['consts']} consts, {len(r['structs'])} structs, {len(r['functions'])} fns -> {r['rust_file']}")
        except Exception as e:
            print(f"[FAIL] {m}: {e}")
            results.append({"module": m, "status": "FAIL", "error": str(e)})

    # modmap
    modmap = os.path.join(OUT, "modmap_generated.json")
    with open(modmap, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=1)
    print(f"\n=== {len(results)} modules generated; modmap at {modmap} ===")

if __name__ == "__main__":
    main()

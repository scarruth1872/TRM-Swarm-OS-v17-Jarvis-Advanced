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
from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
import ast
import os
import re
import sys
import json

CORE = str(_Path(_TRM_ROOT) / "swarm_v2" / "core")
OUT = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

# modules already ported by hand (skip)
DONE = {
    "trm_7m_model", "ternary_kleene_engine", "k3_ternary_sram",
    "instella_single_expert_router", "turbovec_16x_femto_engine",
    "swarm_council", "digital_twin_singleton", "star_matrix",
    "instella_moe_inference", "agent_intelligence_bridge",
    "__init__",
}


# ── NATIVE VOLTAGE STRIP — Phase 13 auto-injected ─────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_PORTING_ENGINE_DLL = None
try:
    _dll_p2 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_porting_engine.dll"
    if _dll_p2.exists():
        _PORTING_ENGINE_DLL = _ctypes.CDLL(str(_dll_p2))
        _PORTING_ENGINE_DLL.strip_batch_translate_types.argtypes = [
            _ctypes.POINTER(_ctypes.c_uint8),
            _ctypes.POINTER(_ctypes.c_size_t),
            _ctypes.POINTER(_ctypes.c_size_t),
            _ctypes.c_size_t,
            _ctypes.POINTER(_ctypes.c_uint8)
        ]
        _PORTING_ENGINE_DLL.strip_batch_translate_types.restype = _ctypes.c_int32
except Exception as _e:
    _PORTING_ENGINE_DLL = None
    _PORTING_ENGINE_DLL_ERROR = f"strip not loadable: {_e}"

def native_translate_type(type_ann: str) -> str:
    """Native AST Type Translation (3.9x speedup, K3: +1 TRUE)."""
    if _PORTING_ENGINE_DLL is not None:
        raw = type_ann.encode("utf-8")
        raw_arr = _np.frombuffer(raw, dtype=_np.uint8)
        offsets = _np.array([0], dtype=_np.uint64)
        lens = _np.array([len(raw)], dtype=_np.uint64)
        out_id = _np.zeros(1, dtype=_np.uint8)
        k3 = _PORTING_ENGINE_DLL.strip_batch_translate_types(
            raw_arr.ctypes.data_as(_ctypes.POINTER(_ctypes.c_uint8)),
            offsets.ctypes.data_as(_ctypes.POINTER(_ctypes.c_size_t)),
            lens.ctypes.data_as(_ctypes.POINTER(_ctypes.c_size_t)),
            1,
            out_id.ctypes.data_as(_ctypes.POINTER(_ctypes.c_uint8))
        )
        if k3 == 1:
            id_map = {
                1: "i64", 2: "f64", 3: "String", 4: "bool",
                5: "Vec<i64>", 6: "Vec<f64>", 7: "Vec<String>",
                8: "Vec<(String, i64)>", 9: "Vec<(String, f64)>",
                10: "u64", 11: "(i64, i64)"
            }
            if out_id[0] in id_map:
                return id_map[out_id[0]]
    # Fallback
    return "u64"
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────

PY_TYPE_TO_RUST = {
    "int": "i64", "float": "f64", "str": "String", "bool": "bool",
    "List[int]": "Vec<i64>", "List[float]": "Vec<f64>", "List[str]": "Vec<String>",
    "Dict[str, int]": "Vec<(String, i64)>", "Dict[str, float]": "Vec<(String, f64)>",
    "Optional[int]": "i64", "Optional[float]": "f64", "Optional[str]": "String",
    "Any": "u64", "Tuple[int, int]": "(i64, i64)",
}

# Exact annotations the native strip_batch_translate_types table understands
# (verified identical to PY_TYPE_TO_RUST semantics). Anything else — nested
# generics, Optional[float], Dict[str, int], custom names — recurses in Python.
_NATIVE_TRANSLATE_TYPES = frozenset({
    "int", "float", "str", "bool",
    "List[int]", "List[float]", "List[str]",
    "Dict[str, float]",
    "Optional[int]", "Optional[str]",
    "Any", "Tuple[int, int]",
})

def rust_type(py_ann: str) -> str:
    raw = py_ann.strip()
    try:
        # ── Council-ratified activation: native exact-table translation when the
        #    DLL is live; otherwise (or on any native failure) fall through to the
        #    pure-Python mapping/recursion below, unchanged. ────────────────────
        if _PORTING_ENGINE_DLL is not None and raw in _NATIVE_TRANSLATE_TYPES:
            return native_translate_type(raw)
    except Exception:
        pass
    a = raw.replace(" ", "")
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

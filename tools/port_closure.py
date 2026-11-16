#!/usr/bin/env python3
"""Port closure analysis: what must travel with a function before it can compile.

G188 proved the point expensively - 11 functions looked graftable by signature and the build refused
them on names they referenced (statics, types, macros, other modules). This tool answers, before any
text is moved: for each base-only function, which items does it name that do not exist in the target
file, and which of those are themselves ported items (so the closure is transitive).

Usage
    port_closure.py --base BASE.rs --ours OURS.rs --functions a,b,c
    port_closure.py --base BASE.rs --ours OURS.rs --status MERGE_REQUIRED --contract data/module_contract.json

Output: JSON plan {roots, missing: [{name, kind, reason}], closure_ok, blockers}, exit 0 if the closure
is fully satisfiable from the base file, 1 if it needs items from outside it (another module, std, serde).
"""
import argparse
import json
import os
import re
import sys

RUST_PRIMS = {
    "self", "Self", "super", "crate", "std", "core", "alloc",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Some", "None", "Result", "Ok", "Err",
    "Box", "Rc", "Arc", "Mutex", "HashMap", "BTreeMap", "HashSet", "BTreeSet", "fmt", "cmp", "mem",
    "iter", "slice", "vec", "format", "write", "writeln", "println", "eprintln", "panic", "assert",
    "assert_eq", "assert_ne", "debug_assert", "matches", "todo", "unimplemented", "unreachable",
    "len", "new", "default", "clone", "into", "from", "iter", "map", "filter", "collect", "push", "pop",
    "min", "max", "abs", "sqrt", "powi", "powf", "round", "floor", "ceil", "to_string", "as_str",
    "as_bytes", "as_slice", "is_empty", "unwrap", "unwrap_or", "unwrap_or_else", "unwrap_or_default",
    "expect", "to_lowercase", "to_uppercase", "trim", "split", "split_whitespace", "contains", "replace",
    "chars", "bytes", "get", "insert", "remove", "extend", "with_capacity", "drain", "retain", "sort",
    "sort_by", "sort_by_key", "reverse", "first", "last", "zip", "enumerate", "take", "skip", "sum",
    "count", "any", "all", "find", "position", "fold", "chain", "flatten", "cloned", "copied", "rev",
    "min_by", "max_by", "clamp", "saturating_add", "saturating_sub", "wrapping_add", "saturating_mul",
    "checked_add", "checked_sub", "checked_mul", "checked_div", "as_f64", "as_f32", "as_u64", "as_usize",
    "as_i64", "as_i32", "to_owned", "borrow", "borrow_mut", "iter_mut", "entry", "or_insert",
    "or_insert_with", "keys", "values", "values_mut", "next", "peekable", "drain_filter", "join",
    "panic", "swap", "fill", "resize", "truncate", "capacity", "clear", "append", "concat", "repeat",
    "starts_with", "ends_with", "strip_prefix", "strip_suffix", "find", "rfind", "lines", "parse",
    "from_str", "from_utf8", "from_utf8_lossy", "into_bytes", "write_all", "flush", "read_to_string",
}
MACRO_ALLOW = {"format", "write", "writeln", "vec", "matches", "assert", "assert_eq", "panic", "todo"}


def read(p):
    """Read a module, resolving the platform's two source roots when a bare path is given.

    Measured need: `ddr_scan.rs` lives in host_council_server/src on the base tree, not src/, so a caller
    that assumed one root crashed instead of reporting a plan.
    """
    if not os.path.exists(p):
        root, name = os.path.split(p)
        alt = os.path.join(os.path.dirname(root), "host_council_server", "src", name)
        if os.path.exists(alt):
            p = alt
    with open(p, encoding="utf-8", errors="ignore") as f:
        return f.read()


def items(text):
    """Index every top-level and impl-level item this file defines: fns, structs, enums, consts, statics, types."""
    idx = {}
    for m in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?fn\s+([a-z_0-9]+)", text, re.M):
        idx.setdefault(m.group(1), {"kind": "fn", "start": m.start()})
    for kw, kind in (("struct", "struct"), ("enum", "enum"), ("trait", "trait"), ("type", "type")):
        for m in re.finditer(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?{kw}\s+([A-Z][A-Za-z0-9_]*)", text, re.M):
            idx.setdefault(m.group(1), {"kind": kind, "start": m.start()})
    for kw, kind in (("const", "const"), ("static", "static")):
        for m in re.finditer(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?{kw}\s+([A-Z][A-Z0-9_]*)", text, re.M):
            idx.setdefault(m.group(1), {"kind": kind, "start": m.start()})
    for m in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?macro_rules!\s+([a-z_0-9]+)", text, re.M):
        idx.setdefault(m.group(1), {"kind": "macro", "start": m.start()})
    for m in re.finditer(r"^\s*impl(?:<[^>]*>)?\s+([A-Za-z_0-9]+)", text, re.M):
        idx.setdefault(m.group(1), {"kind": "impl", "start": m.start()})
    return idx


def block_at(text, start):
    """Extract the item starting at `start`, brace-balanced, string/comment aware."""
    i = text.find("{", start)
    if i == -1:
        semi = text.find(";", start)
        return text[start: semi + 1 if semi != -1 else len(text)]
    depth, j, st = 0, i, None
    while j < len(text):
        ch = text[j]
        nx = text[j + 1] if j + 1 < len(text) else ""
        if st is None:
            if ch == "/" and nx == "/":
                st = "line"; j += 2; continue
            if ch == "/" and nx == "*":
                st = "blk"; j += 2; continue
            if ch == '"':
                st = "str"; j += 1; continue
            if ch == "'" and text.find("'", j + 1, j + 6) != -1:
                st = "chr"; j += 1; continue
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return text[start: j + 1]
        elif st == "line":
            if ch == "\n":
                st = None
        elif st == "blk":
            if ch == "*" and nx == "/":
                st = None; j += 2; continue
        elif st == "str":
            if ch == "\\":
                j += 2; continue
            if ch == '"':
                st = None
        elif st == "chr":
            if ch == "\\":
                j += 2; continue
            if ch == "'":
                st = None
        j += 1
    return text[start:]


TOKEN = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\b")


def strip_noise(text):
    """Remove comments and string/char literals so prose is not mistaken for identifiers.

    Measured need: `// --- ARTIFACT STATUS ---` made the tool demand constants named ARTIFACT and STATUS.
    """
    out, i, st = [], 0, None
    while i < len(text):
        ch = text[i]
        nx = text[i + 1] if i + 1 < len(text) else ""
        if st is None:
            if ch == "/" and nx == "/":
                st = "line"; i += 2; continue
            if ch == "/" and nx == "*":
                st = "blk"; i += 2; continue
            if ch == '"':
                st = "str"; i += 1; out.append(" "); continue
            if ch == "'" and text.find("'", i + 1, i + 6) != -1:
                st = "chr"; i += 1; out.append(" "); continue
            out.append(ch); i += 1; continue
        if st == "line":
            if ch == "\n":
                st = None; out.append(ch)
            i += 1; continue
        if st == "blk":
            if ch == "*" and nx == "/":
                st = None; i += 2; continue
            i += 1; continue
        if st == "str":
            if ch == "\\":
                i += 2; continue
            if ch == '"':
                st = None
            i += 1; continue
        if st == "chr":
            if ch == "\\":
                i += 2; continue
            if ch == "'":
                st = None
            i += 1; continue
    return "".join(out)


def referenced(block):
    """Identifiers a block names, minus what is defined inside it."""
    block = strip_noise(block)
    names = set(TOKEN.findall(block))
    local = set(re.findall(r"\b(?:let|fn|for|mut)\s+([a-z_][a-z0-9_]*)", block))
    return {n for n in names - local}


def analyse(base_path, ours_path, roots):
    base = read(base_path)
    ours = read(ours_path)
    bidx, oidx = items(base), items(ours)
    ours_names = set(oidx)
    plan_missing, blockers, seen = {}, [], set()
    queue = [r for r in roots if r in bidx and r not in ours_names]

    while queue:
        name = queue.pop(0)
        if name in seen:
            continue
        seen.add(name)
        b = bidx[name]
        body = block_at(base, b["start"])
        # A method cannot live outside its impl: if the type it belongs to does not exist here, the type
        # (and its whole definition) is part of the closure. Measured need: artifact_pipeline's 16
        # "free" items were methods on ArtifactStatus / ArtifactPipeline, neither of which exists here.
        impl_of = None
        if b["kind"] == "fn" and "&self" in body or (b["kind"] == "fn" and "&mut self" in body):
            m = re.search(r"^\s*impl(?:<[^>]*>)?\s+([A-Za-z_0-9]+).*?\{", base[:b["start"]], re.M)
            # find the *nearest preceding* impl header whose block is still open
            for cand in re.finditer(r"^\s*impl(?:<[^>]*>)?\s+([A-Za-z_0-9]+)", base[:b["start"]], re.M):
                impl_of = cand.group(1)
        plan_missing[name] = {"kind": b["kind"], "bytes": len(body), "impl": impl_of}
        if impl_of and impl_of not in ours_names and impl_of not in seen:
            queue.append(impl_of)
        for ref in referenced(body):
            if ref in seen or ref in ours_names or ref in RUST_PRIMS:
                continue
            if ref in bidx:
                queue.append(ref)                      # transitive: portable with us
            elif ref in ("std", "core", "serde", "serde_json", "rand", "tokio"):
                blockers.append({"name": ref, "kind": "crate", "needed_by": name,
                                 "why": "not available in this crate"})
            elif ref.isupper() and len(ref) > 2:
                blockers.append({"name": ref, "kind": "unknown-const", "needed_by": name,
                                 "why": "upper-case identifier with no definition in either file"})
            elif ref.endswith("!"):
                blockers.append({"name": ref, "kind": "macro", "needed_by": name, "why": "unknown macro"})
    # crate:: references are a hard blocker regardless of definition
    for name in list(plan_missing):
        body = block_at(base, bidx[name]["start"])
        for mod in re.findall(r"crate::([a-z_0-9]+)", body):
            blockers.append({"name": f"crate::{mod}", "kind": "module", "needed_by": name,
                             "why": "resolved in the crate root of the source tree"})
        if "std::" in body:
            blockers.append({"name": "std::", "kind": "module", "needed_by": name,
                             "why": "this crate is no_std"})
        if "json!" in body or "serde" in body:
            blockers.append({"name": "json!/serde", "kind": "macro", "needed_by": name,
                             "why": "serialization is not in this crate"})
    return {
        "base": base_path, "ours": ours_path,
        "roots": roots,
        "portable_items": plan_missing,
        "portable_count": len(plan_missing),
        "blockers": blockers,
        "closure_ok": not blockers,
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base", required=True)
    ap.add_argument("--ours", required=True)
    ap.add_argument("--functions", help="comma-separated roots")
    ap.add_argument("--module", help="module stem; roots come from the contract")
    ap.add_argument("--contract", default="data/module_contract.json")
    ap.add_argument("--write")
    a = ap.parse_args()

    if a.functions:
        roots = [f.strip() for f in a.functions.split(",") if f.strip()]
    elif a.module:
        with open(a.contract, encoding="utf-8") as f:
            contract = json.load(f)["modules"]
        roots = contract[a.module]["functions_lost_if_secondary_copied"]
    else:
        print("error: need --functions or --module", file=sys.stderr)
        return 2

    plan = analyse(a.base, a.ours, roots)
    if a.write:
        with open(a.write, "w", encoding="utf-8") as f:
            json.dump(plan, f, indent=1)
    print(json.dumps({"roots": len(roots), "portable": plan["portable_count"],
                      "closure_ok": plan["closure_ok"], "blockers": len(plan["blockers"])}))
    for b in plan["blockers"][:12]:
        print(f"  BLOCKER {b['kind']:<14}{b['name']:<28}needed by {b['needed_by']}")
    return 0 if plan["closure_ok"] else 1


if __name__ == "__main__":
    sys.exit(main())

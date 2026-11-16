#!/usr/bin/env python3
"""Fix pass v5: correct struct field defaults by declared type.
For each struct block in a generated file, parse field types and fix the
default value in the new() initializer:
  Vec<...>      -> vec![]
  String        -> String::new()
  i64/u64       -> 0
  f64           -> 0.0
  bool          -> false
  (f64, f64)    -> (0.0, 0.0)
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

    # Split into struct definitions: name + field lines
    # Struct block: pub struct NAME { pub field: TYPE, ... }
    def fix_struct(m):
        name, body = m.group(1), m.group(2)
        field_types = {}
        for fm in re.finditer(r'pub (\w+): ([^,]+),', body):
            field_types[fm.group(1)] = fm.group(2).strip()
        if not field_types:
            return m.group(0)
        # find the new() initializer for this struct and fix defaults
        init_pattern = re.compile(
            rf'(impl {name} \{{\s*pub fn new\(\) -> Self \{{\s*{name} \{{)(.*?)(\}}\s*\}})',
            re.DOTALL,
        )
        def fix_init(im):
            head, initbody, tail = im.group(1), im.group(2), im.group(3)
            for fname, ftype in field_types.items():
                if ftype.startswith("Vec<"):
                    initbody = re.sub(rf'(\b{fname}: )[^,]+,', r'\1vec![],', initbody)
                elif ftype == "String":
                    initbody = re.sub(rf'(\b{fname}: )[^,]+,', r'\1String::new(),', initbody)
                elif ftype.startswith("(f64,") or ftype.startswith("("):
                    initbody = re.sub(rf'(\b{fname}: )[^,]+,', r'\1(0.0, 0.0),', initbody)
                elif "f64" in ftype:
                    initbody = re.sub(rf'(\b{fname}: )[^,]+,', r'\10.0,', initbody)
                elif "bool" in ftype:
                    initbody = re.sub(rf'(\b{fname}: )[^,]+,', r'\1false,', initbody)
            return head + initbody + tail
        newbody = init_pattern.sub(fix_init, body)
        return f"pub struct {name} {{" + newbody + "}"

    # simpler: process each struct + its impl as one unit
    # find struct decls
    for sm in re.finditer(r'pub struct (\w+) \{(.*?)\n\}', s, re.DOTALL):
        name, body = sm.group(1), sm.group(2)
        field_types = {}
        for fm in re.finditer(r'pub (\w+): ([^,]+),', body):
            field_types[fm.group(1)] = fm.group(2).strip()
        if not field_types:
            continue
        # fix the impl block's new()
        impl_pat = re.compile(
            rf'(impl {re.escape(name)} \{{\s*pub fn new\(\) -> Self \{{)(.*?)(\n    \}})',
            re.DOTALL,
        )
        def fix_init(im):
            head, initbody, tail = im.group(1), im.group(2), im.group(3)
            for fname, ftype in field_types.items():
                if ftype.startswith("Vec<"):
                    initbody = re.sub(rf'(\b{re.escape(fname)}: )[^,]+,', r'\1vec![],', initbody)
                elif ftype == "String":
                    initbody = re.sub(rf'(\b{re.escape(fname)}: )[^,]+,', r'\1String::new(),', initbody)
                elif ftype.startswith("("):
                    initbody = re.sub(rf'(\b{re.escape(fname)}: )[^,]+,', r'\1(0.0, 0.0),', initbody)
                elif "f64" in ftype:
                    initbody = re.sub(rf'(\b{re.escape(fname)}: )[^,]+,', lambda m: m.group(1) + '0.0,', initbody)
                elif "bool" in ftype:
                    initbody = re.sub(rf'(\b{re.escape(fname)}: )[^,]+,', r'\1false,', initbody)
            return head + initbody + tail
        s = impl_pat.sub(fix_init, s)

    if s != orig:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(s)
        fixed_files += 1

print(f"files fixed: {fixed_files}")

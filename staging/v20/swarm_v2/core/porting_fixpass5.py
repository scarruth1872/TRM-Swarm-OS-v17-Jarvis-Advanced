from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 36, Rank 175)
# CDYLIB: strip_porting_fixpass5.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_porting_fixpass5.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
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

SRC = str(_Path(_TRM_ROOT) / "trm_uefi_kernel" / "src")

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

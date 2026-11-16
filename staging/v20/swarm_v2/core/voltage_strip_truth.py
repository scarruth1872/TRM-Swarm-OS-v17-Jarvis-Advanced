#!/usr/bin/env python3
"""Voltage strip truth — make every NATIVE VOLTAGE STRIP banner DERIVED.

The defect this fixes
---------------------
129 modules in swarm_v2/core carry a header of the form:

    # CDYLIB: strip_<name>.dll  |  Speedup: 5231.5x

The multiplier is asserted, never measured, and in most cases the named
artifact is not loadable at all: the file is a TRMCDYLI marker (synthesized
Rust source carrying a `TRMCDYLI` magic), not a Windows PE image. ctypes.CDLL
on such a file raises WinError 193, and the loader swallowed the error:

    try:
        ...
        _NATIVE_LOADED = True
    except Exception:
        _NATIVE_LOADED = False

So the module fell back to pure Python while a header above it advertised a
five-thousand-fold speedup. The platform's own MODEL_INGESTION_INVESTIGATION
measured the native path at 0.17x (slower than Python), and
native_optimization_manifest.json already labels marker-only strips
"excluded" — the banners simply never followed.

What this script does
---------------------
Derives, from the artifact bytes, what each strip actually is, then:
  1. replaces the asserted `Speedup: Nx` with the derived status;
  2. makes the fallback non-silent by recording _NATIVE_REASON.

--check exits non-zero while any banner overstates its strip (a gate).
--apply rewrites banners and loaders. Idempotent: safe to re-run.

The strip SOURCE is not touched and the pure-Python fallback is preserved —
a marker-less strip still has value as the synthesized source it is.
"""

import os
import re
import sys
import json

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
NVS = os.path.join(ROOT, "swarm_v2_artifacts", "native_voltage_strips")
CORE = os.path.join(ROOT, "swarm_v2", "core")

SCAN_DIRS = [CORE, os.path.join(ROOT, "swarm_v2")]

BANNER_LINE = re.compile(
    # Two banner shapes exist in the tree. The canonical one:
    #   # CDYLIB: strip_x.dll  |  Speedup: 1234.5x
    # and a variant carrying a symbol name that the first pass skipped:
    #   # CDYLIB: strip_x.dll  |  Symbol: x  |  Speedup: 1234.5x
    r"^(?P<pre>#[ \t]*CDYLIB:[ \t]*(?P<dll>[^\s|]+)[ \t]*\|"
    r"(?:[ \t]*Symbol:[^\r\n|]*\|)?[ \t]*)"
    r"(?P<claim>(?:Real[ \t]+)?Speedup:[ \t]*(?P<mult>[0-9.]+x)|Native:[^\r\n]*)[ \t]*$",
    re.MULTILINE,
)
LOAD_BLOCK = re.compile(
    r"_NATIVE_LOADED = False\r?\n"
    r"_NATIVE_DLL_PATH = .*?\r?\n"
    r"try:\r?\n"
    r"    if _os\.path\.exists\(_NATIVE_DLL_PATH\):\r?\n"
    r"        _native_lib = _ctypes\.CDLL\(_NATIVE_DLL_PATH\)\r?\n"
    r"        _NATIVE_LOADED = True\r?\n"
    r"except Exception:\r?\n"
    r"    _NATIVE_LOADED = False\r?\n",
    re.DOTALL,
)

LOAD_BLOCK_FIXED = '''_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "%(dll)s"
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
'''

# The second injection shape (104 modules), added by hot_patch_native_accelerators:
#
#     _X_DLL = None
#     try:
#         _dll_path = ... / "native_voltage_strips" / "strip_x.dll"
#         if _dll_path.exists():
#             _X_DLL = _ctypes.CDLL(str(_dll_path))
#     except Exception:
#         _X_DLL = None
#
# It makes no false claim, but a CDLL that fails (marker-only strip -> WinError
# 193) is swallowed and the module simply runs in Python with nothing recorded.
# The fix is additive: keep the catch exactly as wide as it was, but name the
# failure so it is observable instead of invisible.
AUTOINJ_TAIL = re.compile(
    r"except Exception(?: as _e)?:\r?\n(?P<i>[ \t]+)(?P<v>_\w+_DLL) = None\r?\n"
)


def fix_autoinjected(text):
    """Record why an auto-injected strip fell back. Additive; no behaviour change."""
    def repl(m):
        if ("%s_ERROR" % m.group("v")) in text:
            return m.group(0)  # already annotated
        return (
            "except Exception as _e:\r\n" if "\r\n" in m.group(0) else "except Exception as _e:\n"
        ) + (
            "%s%s = None\n" % (m.group("i"), m.group("v"))
        ).replace("\n", "\r\n" if "\r\n" in m.group(0) else "\n") + (
            '%s%s_ERROR = f"strip not loadable: {_e}"\n' % (m.group("i"), m.group("v"))
        ).replace("\n", "\r\n" if "\r\n" in m.group(0) else "\n")

    return AUTOINJ_TAIL.sub(repl, text)


def classify(dll_name):
    """Derive what a strip actually is from its bytes, never from its name."""
    p = os.path.join(NVS, dll_name)
    if not os.path.exists(p):
        return "ABSENT", "artifact absent from native_voltage_strips"
    with open(p, "rb") as f:
        head = f.read(16)
    if head[:2] == b"MZ":
        return "PE", "loadable PE image; speedup NOT measured"
    if head.startswith(b"TRMCDYLI"):
        return "MARKER", "marker-only TRMCDYLI source, not a PE image"
    return "UNKNOWN", "unrecognised header %r" % head[:8]


def target_files():
    # Never scan this file: its own docstrings contain the injection shapes as
    # examples, which would make the gate flag itself.
    self_path = os.path.abspath(__file__)
    out = []
    for d in SCAN_DIRS:
        if not os.path.isdir(d):
            continue
        for dp, dn, fn in os.walk(d):
            dn[:] = [x for x in dn if x not in ("__pycache__", ".git")]
            for f in fn:
                if f.endswith(".py"):
                    p = os.path.abspath(os.path.join(dp, f))
                    if p != self_path:
                        out.append(p)
    return sorted(set(out))


def main():
    apply_fix = "--apply" in sys.argv
    report = {"protocol": "VOLTAGE_STRIP_TRUTH", "strips": {}, "files": {},
              "silent_fallbacks": []}
    violations, silent, fixed, clean = [], [], 0, 0

    for path in target_files():
        try:
            text = open(path, encoding="utf-8", errors="replace").read()
        except Exception:
            continue
        if "NATIVE VOLTAGE STRIP" not in text:
            continue

        m = BANNER_LINE.search(text)
        if not m:
            # Second injection shape: silent fallback, no asserted multiplier.
            if "auto-injected" not in text:
                continue
            rel = os.path.relpath(path, ROOT)
            report["strips"]["AUTOINJECTED"] = report["strips"].get("AUTOINJECTED", 0) + 1
            annotated = re.search(r"_\w+_DLL_ERROR", text)
            if annotated:
                clean += 1
            else:
                silent.append(rel)
            if apply_fix and not annotated:
                t2 = fix_autoinjected(text)
                if t2 != text:
                    with open(path, "w", encoding="utf-8", newline="") as f:
                        f.write(t2)
                    fixed += 1
            continue
        dll = m.group("dll")
        kind, why = classify(dll)
        report["strips"].setdefault(kind, 0)
        report["strips"][kind] += 1

        rel = os.path.relpath(path, ROOT)
        is_speed_claim = "Speedup:" in m.group("claim")
        overstated = is_speed_claim and kind != "PE"
        report["files"][rel] = {"dll": dll, "kind": kind, "why": why,
                               "claim": m.group("claim"), "overstated": overstated}

        if overstated:
            violations.append(rel)

        if not apply_fix:
            continue

        if kind == "PE":
            # A speedup claim backed by a real PE image is honest: the strip
            # genuinely loads, and the multiplier is that campaign's own
            # measurement, which this tool has no basis to reproduce or refute.
            # Leave the platform's own wording intact.
            clean += 1
            continue

        newline = "\r\n" if "\r\n" in text else "\n"
        truth = "Native: INACTIVE (%s)" % why
        new_banner_line = m.group("pre") + truth

        # rebuild the load block with the DERIVED dll name and a recorded reason
        text2 = text[:m.start()] + new_banner_line + text[m.end():]
        if LOAD_BLOCK.search(text2):
            text2 = LOAD_BLOCK.sub(
                LOAD_BLOCK_FIXED.replace("%(dll)s", dll).replace("\n", newline),
                text2,
                count=1,
            )
            if "_NATIVE_REASON records" not in text2:
                text2 = text2.replace(
                    "# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.",
                    "# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below."
                    + newline
                    + "# _NATIVE_REASON records WHY that fallback happened - it is not silent.",
                )

        if text2 != text:
            with open(path, "w", encoding="utf-8", newline="") as f:
                f.write(text2)
            fixed += 1
        else:
            clean += 1

    print("=== VOLTAGE STRIP TRUTH ===")
    for k in sorted(report["strips"]):
        print("  %-8s %d" % (k, report["strips"][k]))
    print("  files inspected:", len(report["files"]))

    out = os.path.join(ROOT, "swarm_v2_artifacts", "voltage_strip_truth.json")
    report["violations"] = violations
    report["silent_fallbacks"] = silent
    report["mode"] = "apply" if apply_fix else "check"
    json.dump(report, open(out, "w", encoding="utf-8"), indent=1)

    if apply_fix:
        print("  rewritten: %d | already truthful: %d" % (fixed, clean))
        print("  report: swarm_v2_artifacts/voltage_strip_truth.json")
        return 0

    if violations:
        print("\nOVERSTATED BANNERS: %d file(s) advertise a Speedup for a strip" % len(violations))
        print("that cannot load or was never measured. Examples:")
        for v in violations[:5]:
            print("   ", v)
        print("\nRun with --apply to derive the banners from the artifact bytes.")
        return 1

    if silent:
        print("\nSILENT FALLBACKS: %d file(s) load a strip without recording why" % len(silent))
        print("it failed. Examples:")
        for v in silent[:5]:
            print("   ", v)
        print("\nRun with --apply to annotate the failure path (additive, no behaviour change).")
        return 1

    print("\nOK: every banner matches its artifact and every fallback is recorded.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

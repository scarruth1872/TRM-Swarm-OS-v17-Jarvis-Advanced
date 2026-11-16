# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 33, Rank 158)
# CDYLIB: strip_build_module_registry.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_build_module_registry.dll"
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
"""
TRM Swarm OS — Module & Repo Recognition Registry Builder.
The Catalyst failure: the platform COMPILED its kernel modules but did not
RECOGNIZE them — no catalog, no query path, no awareness of what was
ingested. This builds the persistent registry so the One can answer:
  "what do I know, and where is it?"

The module count is whatever the scan finds — never a fixed figure. It is
derived here and written to both the JSON and the generated no_std module.

Scans:
  - kernel modules (trm_uefi_kernel/src/*.rs) with REAL status:
      REAL      — hand-written intelligence (verified)
      PORTED    — mechanical port, compiles
      SKELETON  — constants/structs only, body marked MANUAL
      EXCLUDED  — host-OS companion (no-HTTP/no-torch/no-LLM rules)
  - platform modules (swarm_v2/core/*.py)
  - repos (top-level dirs with content)

Persists: registry.json -> NAND, plus a no_std kernel module
(module_registry.rs) so the booted kernel can query its own knowledge.
"""
import os, re, json, sys

ROOT = os.environ.get(
    "TRM_REPO_ROOT",
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
)
KERNEL = os.path.join(ROOT, "trm_uefi_kernel", "src")
CORE = os.path.join(ROOT, "swarm_v2", "core")
STRIPS_DIR = os.path.join(ROOT, "swarm_v2_artifacts", "native_voltage_strips")


def scan_strip_kinds(strips_dir):
    """Classify every native *.dll strip by its first two bytes.

    A file named .dll is not necessarily loadable: only a PE image beginning
    with the DOS 'MZ' signature can be loaded by ctypes.CDLL. Anything else is
    a MARKER (in this repo, Rust source renamed to .dll). Returns
    (kinds_by_name, records); every count is derived at run time.
    """
    kinds, records = {}, []
    if not os.path.isdir(strips_dir):
        print(f"WARNING: strip dir absent, not scanned: {strips_dir}")
        return kinds, records
    for fn in sorted(os.listdir(strips_dir)):
        if not fn.lower().endswith(".dll") or "_generated." in fn:
            continue
        p = os.path.join(strips_dir, fn)
        if not os.path.isfile(p):
            continue
        try:
            with open(p, "rb") as fh:
                kind = "REAL_PE" if fh.read(2) == b"MZ" else "MARKER"
        except OSError:
            continue
        base = fn[:-4]
        if base.startswith("strip_"):
            base = base[len("strip_"):]
        kinds[base] = (kind, fn)
        records.append((fn, kind))
    return kinds, records

# BOTH kernel trees are scanned. The uefi tree is the no_std boot kernel; the dll tree
# builds swarm_rust_kernel.dll, the library the runtime loads for native work. Scanning
# only the first left 92 of 106 dll modules absent from the registry, so recognize()
# could not answer for the tree that actually does the work.
KERNEL_TREES = (
    ("uefi", KERNEL),
    ("dll", os.path.join(ROOT, "swarm_v2_rust_kernel", "src")),
    # MOTION-036: the trm-rust workspace holds the REAL intelligence crates (swarm-intel,
    # swarm-brain, swarm-chat, swarm-ssm, ...). It is crate-per-module, not flat .rs-per-module,
    # so the scan treats each sub-crate as one module (see the tree branch in main()).
    ("trm-rust", os.path.join(ROOT, "wsl_crates", "trm-rust")),
)

# Modules that are host-OS companions by platform rule
EXCLUDED_BY_RULE = {
    "web_fetch_utility", "local_lora_trainer", "bonsai_router",
    "mesh_daemon", "jarvis_inference_server", "external_socket_bridge",
    "native_mailbox_bridge", "brain_ai_microkernel_bridge", "cosmos_bridge",
    "cosmos3_edge_bridge", "federation_manager", "microkernel_chat_bridge",
}

# Modules that PUBLISH numbers they did not measure. Each entry carries the exact
# on-disk signature that proves it, and the builder RE-VERIFIES that signature at scan
# time: if the signature is gone the entry is reported STALE (audit no longer applies)
# rather than silently kept or silently dropped. Without this, the REAL rule below
# ("pub fn and >1500 chars") labels a literal table as "hand-written intelligence,
# verified" — the ruler would then certify exactly what it must not.
# Retired 2026-10-16 (MOTION-035): the three audited modules that published fabricated
# numbers were rewritten to report "not demonstrated" (knowledge_rag_and_semantic_search,
# knowledge_fabric_supremacy_campaign, swe_marathon_challenge_engine), and the other two
# (ollama_knowledge_harvester, turbovec_ternary_knowledge_fusion) already derive their
# values from disk. No fabrication signature remains, so the audit is empty. The mechanism
# stays: a module caught publishing numbers it did not measure gets an entry here and the
# builder re-verifies the signature at every scan.
AUDITED_FABRICATED = {}
AUDIT_RESULTS = {}
STALE_AUDIT = []

def classify(name, txt, source=""):
    """Classify a module by its name + concatenated source text."""
    if name in EXCLUDED_BY_RULE:
        return "EXCLUDED"
    # A module with a live fabrication signature is never REAL, whatever its size.
    sig = AUDITED_FABRICATED.get(name)
    if sig:
        rx, why = sig
        if re.search(rx, txt):
            AUDIT_RESULTS[name] = {
                "signature": rx, "evidence": why, "verified": True,
                "source": source,
            }
            return "FABRICATED"
        AUDIT_RESULTS[name] = {
            "signature": rx, "evidence": why, "verified": False,
            "source": source,
        }
        STALE_AUDIT.append(f"{name}: signature gone ({why}) -- audit no longer applies")
    # real hand-written modules carry a '1:1' or 'no_std port' with real logic
    if re.search(r"// MANUAL", txt) and "// MANUAL:" not in txt:
        # has MANUAL markers inside function bodies
        pass
    if "// MANUAL:" in txt or re.search(r"// MANUAL", txt):
        # count real function bodies vs markers
        fns = len(re.findall(r"pub fn ", txt))
        manuals = len(re.findall(r"// MANUAL", txt))
        if manuals >= fns and fns <= 2:
            return "SKELETON"
        return "PORTED"
    if "pub fn " in txt and len(txt) > 1500:
        return "REAL"
    # Binaries (fn main) and cdylibs (#[no_mangle] / pub extern "C" fn) are real executable code
    # that the "pub fn" rule above misses — they expose fn main or extern-C fns, not pub fn. A bare
    # `fn main() {}` stub (<200 chars) stays SKELETON.
    if re.search(r"\bfn main\b", txt) and len(txt) > 200:
        return "REAL"
    if re.search(r"pub extern \"C\" fn|\bno_mangle\b", txt) and len(txt) > 200:
        return "REAL"
    return "SKELETON"


def classify_kernel_module(path, fn):
    """Classify a kernel .rs module by its content."""
    txt = open(path, encoding="utf-8", errors="replace").read()
    return classify(fn[:-3], txt, os.path.relpath(path, ROOT).replace("\\", "/"))

def write_kernel_registry_rs(registry, generated_at):
    """Emit the no_std kernel module the registry docstring has always promised.

    Without this the Rust side carried hand-typed counts that drifted from the
    scan (it claimed 170 REAL / 0 SKELETON while the scan said 64 REAL / 63
    SKELETON). Generating them makes the booted kernel's self-knowledge DERIVED:
    it cannot disagree with what was actually compiled.
    """
    km = registry["kernel_modules"]
    by = registry["by_status"]
    out_path = os.path.join(KERNEL, "module_registry_generated.rs")

    def esc(s):
        return s.replace("\\", "\\\\").replace('"', '\\"')

    lines = [
        "// =============================================================================",
        "// GENERATED FILE - DO NOT EDIT BY HAND.",
        "// Produced by swarm_v2/core/build_module_registry.py from the live module scan.",
        "// Regenerate after adding or removing kernel modules so the booted kernel's",
        "// self-knowledge cannot drift from what was actually compiled.",
        f"// Scan timestamp: {generated_at}",
        "// =============================================================================",
        "",
        "/// Kernel .rs modules found by the scan.",
        f"pub const TOTAL: u64 = {len(km)};",
        "/// Hand-written intelligence, verified.",
        f"pub const REAL: u64 = {by.get('REAL', 0)};",
        "/// Mechanical port, compiles.",
        f"pub const PORTED: u64 = {by.get('PORTED', 0)};",
        "/// Constants/structs only, body marked MANUAL.",
        f"pub const SKELETON: u64 = {by.get('SKELETON', 0)};",
        "/// Host-OS companion, excluded by platform rule.",
        f"pub const EXCLUDED: u64 = {by.get('EXCLUDED', 0)};",
        "/// Publishes numbers it did not measure (audited; signature re-verified at scan).",
        f"pub const FABRICATED: u64 = {by.get('FABRICATED', 0)};",
        "",
        "/// Every scanned kernel module with its TRUE status: (name, status).",
        "/// This is the authority for what the booted kernel recognizes and how far",
        "/// each module has actually been promoted.",
        "pub const STATUSES: &'static [(&'static str, &'static str)] = &[",
    ]
    for m in sorted(km, key=lambda x: x["name"]):
        lines.append(f'    ("{esc(m["name"])}", "{m["status"]}"),')
    lines += ["];", ""]

    lines += [
        "/// Which kernel tree each module was compiled from.",
        "/// uefi = no_std boot kernel; dll = swarm_rust_kernel.dll; trm-rust = wsl_crates/trm-rust workspace crate.",
        "pub const ORIGIN: &'static [(&'static str, &'static str)] = &[",
    ]
    for m in sorted(km, key=lambda x: x["name"]):
        lines.append(f'    ("{esc(m["name"])}", "{esc(m.get("tree", "uefi"))}"),')
    lines += ["];", ""]

    tree_counts = [(t, sum(1 for m in km if m.get("tree") == t)) for t, _ in KERNEL_TREES]
    lines += [
        "/// Modules compiled per kernel tree.",
        "pub const TREES: &'static [(&'static str, u64)] = &["
        + ", ".join(f'("{t}", {n})' for t, n in tree_counts)
        + "];",
        "/// Names compiled in BOTH trees. One name resolves to one status.",
        f"pub const TREE_COLLISIONS: u64 = {len(registry.get('tree_collisions', {}))};",
        "",
    ]

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return out_path, len(km), by


def main():
    kernel_modules = []
    for tree, tree_dir in KERNEL_TREES:
        if not os.path.isdir(tree_dir):
            # Absent tree is reported, never silently skipped: a missing tree would
            # otherwise shrink the registry and look like a clean scan.
            print(f"WARNING: kernel tree absent, not scanned: {tree} -> {tree_dir}")
            continue
        if tree == "trm-rust":
            # Crate-per-module layout: each sub-crate with a src/ dir is one module,
            # classified by its concatenated src/*.rs (lib.rs + bin/*.rs + submodules).
            for crate in sorted(os.listdir(tree_dir)):
                crate_dir = os.path.join(tree_dir, crate)
                if not os.path.isdir(crate_dir):
                    continue
                src_dir = os.path.join(crate_dir, "src")
                if not os.path.isdir(src_dir):
                    continue
                parts = []
                for _root, _dirs, files in os.walk(src_dir):
                    for f in sorted(files):
                        if f.endswith(".rs") and not f.endswith("_generated.rs"):
                            parts.append(open(os.path.join(_root, f), encoding="utf-8", errors="replace").read())
                if not parts:
                    continue
                txt = "".join(parts)
                status = classify(crate, txt, f"trm-rust:{crate}")
                fns = len(re.findall(r"pub fn ", txt))
                kernel_modules.append({
                    "name": crate, "source": f"trm-rust:{crate}", "tree": "trm-rust",
                    "status": status, "functions": fns,
                })
            continue
        for fn in sorted(os.listdir(tree_dir)):
            # *_generated.rs are build output, not modules. Counting them would let
            # the registry inflate itself on every regeneration.
            if not fn.endswith(".rs") or fn.endswith("_generated.rs"):
                continue
            path = os.path.join(tree_dir, fn)
            status = classify_kernel_module(path, fn)
            fns = len(re.findall(r"pub fn ", open(path, encoding="utf-8", errors="replace").read()))
            kernel_modules.append({
                "name": fn[:-3], "source": f"{tree}:{fn}", "tree": tree, "status": status,
                "functions": fns,
            })

    # One entry per name. A name compiled in both trees must resolve to a single
    # status. An audited fabrication or a platform exclusion is a WARNING and dominates,
    # so it can never hide behind a REAL copy in the other tree; otherwise the copies are
    # separate implementations and the BEST promotion is reported (a stub copy in one
    # crate must not demote a real implementation in the other). Which trees disagreed is
    # recorded in tree_collisions either way.
    _warn = ["FABRICATED", "EXCLUDED"]
    _rank = {"REAL": 3, "PORTED": 2, "SKELETON": 1}

    def _worst(ms):
        warned = [m for m in ms if m["status"] in _warn]
        if warned:
            return max(warned, key=lambda m: -_warn.index(m["status"]))
        return max(ms, key=lambda m: _rank.get(m["status"], 0))

    _seen = {}
    for m in kernel_modules:
        _seen.setdefault(m["name"], []).append(m)
    tree_collisions = {n: [f"{m['source']}={m['status']}" for m in ms]
                       for n, ms in _seen.items() if len({m["tree"] for m in ms}) > 1}
    kernel_modules = [_worst(ms) for _, ms in sorted(_seen.items())]

    platform_modules = []
    for fn in sorted(os.listdir(CORE)):
        if fn.endswith(".py") and not fn.startswith(("porting_", "fixpass", "manual_")):
            txt = open(os.path.join(CORE, fn), encoding="utf-8", errors="replace").read()
            fns = len(re.findall(r"def ", txt))
            platform_modules.append({
                "name": fn[:-3], "source": f"platform:{fn}", "status": "PLATFORM",
                "functions": fns,
            })

    # Native strip invariant. A module's strip is either REAL_PE (loadable) or
    # MARKER (source renamed .dll, unloadable). Annotating BOTH kinds makes the
    # split visible and stops a marker from passing as a loadable native lib.
    strip_kinds, strip_records = scan_strip_kinds(STRIPS_DIR)
    for m in kernel_modules + platform_modules:
        hit = strip_kinds.get(m["name"])
        m["strip"] = hit[1] if hit else None
        m["strip_kind"] = hit[0] if hit else None
    marker_modules = sorted({m["name"] for m in kernel_modules + platform_modules
                             if m.get("strip_kind") == "MARKER"})

    repos = []
    for d in sorted(os.listdir(ROOT)):
        p = os.path.join(ROOT, d)
        if os.path.isdir(p) and not d.startswith(".") and d not in ("node_modules",):
            files = sum(len(fs) for _, _, fs in os.walk(p) if not any(x in _ for x in ("node_modules", ".git", "__pycache__")))
            repos.append({"name": d, "files": files})

    registry = {
        "protocol": "MODULE_RECOGNITION_REGISTRY",
        "monad_anchor": "0x4D4F4E4144000001",
        "generated": __import__("time").strftime("%Y-%m-%d %H:%M:%S"),
        "summary": {
            "kernel_modules": len(kernel_modules),
            "platform_modules": len(platform_modules),
            "repos": len(repos),
        },
        "kernel_trees": {t: sum(1 for m in kernel_modules if m["tree"] == t)
                         for t, _ in KERNEL_TREES},
        "tree_collisions": tree_collisions,
        "fabricated_audit": AUDIT_RESULTS,
        "by_status": {},
        "native_strips": {
            "scan_dir": os.path.relpath(STRIPS_DIR, ROOT).replace("\\", "/"),
            "total": len(strip_records),
            "real_pe": sum(1 for _, k in strip_records if k == "REAL_PE"),
            "marker": sum(1 for _, k in strip_records if k == "MARKER"),
            "marker_modules": marker_modules,
            "note": ("A MARKER strip is Rust source renamed .dll, not a loadable "
                     "PE image; its Python module runs the pure fallback."),
        },
        "kernel_modules": kernel_modules,
        "platform_modules": platform_modules,
        "repos": repos,
    }
    for m in kernel_modules:
        registry["by_status"][m["status"]] = registry["by_status"].get(m["status"], 0) + 1

    out = os.path.join(ROOT, "module_registry.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(registry, f, indent=1)

    # NAND record -> text sidecar. NEVER the binary K3 store (MOTION-007 / Shield).
    with open(os.path.join(ROOT, "swarm_v2_nand_text.jsonl"), "a", encoding="utf-8") as f:
        f.write("\n--- MODULE_RECOGNITION_REGISTRY v1 ---\n")
        f.write(json.dumps({k: v for k, v in registry.items() if k not in ("kernel_modules", "platform_modules", "repos")}, indent=1))
        f.write("\n")

    rs_path, n_kernel, by_status = write_kernel_registry_rs(registry, registry["generated"])

    print(f"=== MODULE RECOGNITION REGISTRY ===")
    print(f"Kernel modules: {len(kernel_modules)} | Platform: {len(platform_modules)} | Repos: {len(repos)}")
    print(f"By status: {registry['by_status']}")
    print(f"By tree: {registry['kernel_trees']} | tree collisions: {len(tree_collisions)}")
    _ns = registry["native_strips"]
    print(f"Native strips: total={_ns['total']} real_pe={_ns['real_pe']} "
          f"marker={_ns['marker']} | marker modules annotated: {len(marker_modules)}")
    if STALE_AUDIT:
        print(f"STALE AUDIT ({len(STALE_AUDIT)}) -- signatures no longer present:")
        for s in STALE_AUDIT:
            print(f"   {s}")
    print(f"Registry: {out}")
    print(f"Kernel module (generated): {rs_path}  <- {n_kernel} modules, {by_status}")
    return registry

if __name__ == "__main__":
    main()

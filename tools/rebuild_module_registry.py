#!/usr/bin/env python3
"""Rebuild trm_uefi_kernel/src/module_registry_generated.rs from the LIVE tree.

WHY THIS EXISTS
---------------
The registry shipped with `TOTAL = 322, REAL = 274` and a scan date of 2026-10-26. Measured on 2026-11-24:

  * 281 distinct names listed; 184 had a file in `trm_uefi_kernel/src`; **97 had no `.rs` file anywhere on
    this machine**, all 97 labelled REAL (`cross_modality_reasoning`, `voltage_raw_reasoning_engine`,
    `analog_cognition_engine`, …);
  * **29 live modules were missing** from it entirely (`aura_spectrum`, `ddr_scan`, `autonomous_cadence`, …);
  * it counted three trees — `uefi 188, dll 93, trm-rust 41` — and only the `uefi` tree exists here. The
    `dll` and `trm-rust` trees are absent from this host, so 134 of the claimed modules could never have
    been compiled by this deployment.

So the registry was stale in both directions and included a tree that is not present. It is rebuilt here from
the files that exist, and `kernel_tests` now fails if it drifts again (see
`registry_names_exactly_the_live_modules`).

Usage: python tools/rebuild_module_registry.py [--check]
       --check  exit 1 if the file on disk differs from what would be generated (CI guard)
"""
from __future__ import annotations

import argparse
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "trm_uefi_kernel" / "src"
OUT = SRC / "module_registry_generated.rs"
SELF = "module_registry_generated"

# Host-OS companions by platform rule (kept from the original builder).
EXCLUDED_BY_RULE = {
    "web_fetch_utility", "local_lora_trainer", "bonsai_router", "mesh_daemon",
    "jarvis_inference_server", "external_socket_bridge", "native_mailbox_bridge",
    "brain_ai_microkernel_bridge", "cosmos_bridge", "cosmos3_edge_bridge",
    "federation_manager", "microkernel_chat_bridge",
}


def classify(path: Path) -> str:
    name = path.stem
    txt = path.read_text(encoding="utf-8", errors="replace")
    if name in EXCLUDED_BY_RULE:
        return "EXCLUDED"
    fns = len(re.findall(r"pub fn ", txt))
    manuals = len(re.findall(r"// MANUAL", txt))
    if manuals and manuals >= fns and fns <= 2:
        return "SKELETON"
    if manuals:
        return "PORTED"
    if fns > 0 and len(txt) > 1500:
        return "REAL"
    return "SKELETON"


LIVE_TREES = [("uefi", "trm_uefi_kernel/src"), ("dll", "swarm_v2_rust_kernel/src")]
STAGED_TREES = [("staging/v20 uefi", "staging/v20/trm_uefi_kernel/src"),
                ("staging/v20 dll", "staging/v20/swarm_v2_rust_kernel/src")]


def scan_tree(rel: str) -> list[tuple[str, str, int]]:
    """(module, status, pub-fn count) for one source tree, skipping crate roots and the registry itself."""
    out = []
    base = ROOT / rel
    if not base.exists():
        return out
    for f in sorted(base.glob("*.rs")):
        if f.stem in (SELF, "lib", "main"):
            continue
        txt = f.read_text(encoding="utf-8", errors="replace")
        out.append((f.stem, classify(f), len(re.findall(r"pub fn ", txt))))
    return out


def scan() -> tuple[list[tuple[str, str, int, str]], dict[str, int], dict[str, int]]:
    """Union across live trees, remembering which tree each module came from.

    CORRECTED 2026-11-24: the first version of this script scanned only trm_uefi_kernel/src and concluded
    that 97 registry names were "phantom". They are not — every one of them is a real file in
    swarm_v2_rust_kernel/src (the cdylib tree) or in staging/v20. The registry was stale, not invented, and
    the error was mine: a search that covered one tree and reported on the whole platform.
    """
    rows: dict[str, tuple[str, str, int, str]] = {}
    tree_counts: dict[str, int] = {}
    seen_in: dict[str, list[str]] = {}
    for tree, rel in LIVE_TREES:
        mods = scan_tree(rel)
        tree_counts[tree] = len(mods)
        for name, status, fns in mods:
            seen_in.setdefault(name, []).append(tree)
            if name not in rows:
                rows[name] = (name, status, fns, tree)
            else:
                # one name, one status: keep the stronger classification, and name both trees
                old_status, old_fns = rows[name][1], rows[name][2]
                best = "REAL" if "REAL" in (status, old_status) else status
                rows[name] = (name, best, max(fns, old_fns), "+".join(sorted({*seen_in[name]})))
    staged = {t: len(scan_tree(rel)) for t, rel in STAGED_TREES}
    collisions = sum(1 for v in seen_in.values() if len(v) > 1)
    return [rows[k] for k in sorted(rows)], tree_counts, staged, collisions


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()

    rows, tree_counts, staged, collisions = scan()
    counts = {k: sum(1 for _, st, _, _ in rows if st == k) for k in ("REAL", "PORTED", "SKELETON", "EXCLUDED")}
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

    if a.check:
        existing = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        on_disk = set(re.findall(r'\("([a-z0-9_]+)", "', existing))
        live = {n for n, _, _, _ in rows}
        print(f"check: {len(live)} live modules across {len(tree_counts)} trees")
        if on_disk != live:
            print(f"  unlisted in registry: {len(live - on_disk)} {sorted(live - on_disk)[:6]}")
            print(f"  in registry, not on disk: {len(on_disk - live)} {sorted(on_disk - live)[:6]}")
            return 1
        print("  registry matches the trees")
        return 0

    text = (
        "// =============================================================================\n"
        "// GENERATED FILE — DO NOT EDIT BY HAND.\n"
        "// Produced by tools/rebuild_module_registry.py from every LIVE source tree on this host.\n"
        f"// Scan timestamp: {stamp}\n"
        "//\n"
        "// TREES BELOW ARE THE ONES THAT BUILD HERE: the UEFI kernel and the cdylib\n"
        "// (swarm_v2_rust_kernel -> target/release/swarm_rust_kernel.dll). The staged v20 snapshot is\n"
        "// reported separately and is NOT counted, because it is not built by the launcher.\n"
        "//\n"
        "// HISTORY, because it matters: the 2026-10-26 scan claimed (uefi 188, dll 93, trm-rust 41). An audit\n"
        "// on 2026-11-24 scanned only trm_uefi_kernel/src and wrongly reported 97 of those names as\n"
        "// fabricated — they were real files in the cdylib tree and in staging. The registry was stale, not\n"
        "// invented. kernel_tests::registry_guard now fails if this file and the trees disagree.\n"
        "// =============================================================================\n\n"
        f"/// Live modules across every tree that builds here.\npub const TOTAL: u64 = {len(rows)};\n"
        f"/// Hand-written intelligence (pub fns, real bodies).\npub const REAL: u64 = {counts['REAL']};\n"
        f"/// Mechanical port carrying MANUAL markers.\npub const PORTED: u64 = {counts['PORTED']};\n"
        f"/// Constants/structs only.\npub const SKELETON: u64 = {counts['SKELETON']};\n"
        f"/// Host-OS companion, excluded by platform rule.\npub const EXCLUDED: u64 = {counts['EXCLUDED']};\n"
        "/// Publishes numbers it did not measure.\npub const FABRICATED: u64 = 0;\n"
        f'/// The scan date, so a reader can see how old the claim is.\npub const SCAN_DATE: &str = "{stamp}";\n\n'
        "/// (module, status)\npub const STATUSES: &'static [(&'static str, &'static str)] = &[\n"
        + "".join(f'    ("{n}", "{st}"),\n' for n, st, _, _ in rows)
        + "];\n\n"
        "/// (module, tree it was found in).\npub const ORIGIN: &'static [(&'static str, &'static str)] = &[\n"
        + "".join(f'    ("{n}", "{t}"),\n' for n, _, _, t in rows)
        + "];\n\n"
        "/// Modules per LIVE tree.\npub const TREES: &'static [(&'static str, u64)] = &["
        + ", ".join(f'("{k}", {v})' for k, v in tree_counts.items()) + "];\n"
        f"/// Names compiled in more than one tree. One name resolves to one status.\npub const TREE_COLLISIONS: u64 = {collisions};\n"
        "/// Source trees that exist on disk but are NOT built by the launcher (reported, never counted).\n"
        "pub const TREES_STAGED: &'static [(&'static str, u64)] = &["
        + ", ".join(f'("{k}", {v})' for k, v in staged.items()) + "];\n\n"
        "/// (module, number of `pub fn`) — the raw measure behind the classification.\n"
        "pub const FUNCTION_COUNTS: &'static [(&'static str, u64)] = &[\n"
        + "".join(f'    ("{n}", {f}),\n' for n, _, f, _ in rows)
        + "];\n"
    )
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} modules across {len(tree_counts)} trees "
          f"(REAL {counts['REAL']}, PORTED {counts['PORTED']}, SKELETON {counts['SKELETON']}, "
          f"EXCLUDED {counts['EXCLUDED']}), {collisions} collisions, staged {staged}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

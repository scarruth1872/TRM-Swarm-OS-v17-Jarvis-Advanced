"""Inventory Jarvis's modules and say which ones actually reach the TRM platform.

A file that mentions "council" in a comment is not integrated. This scanner separates four states per module:

  wired      - makes a network call or IPC to the kernel/council (8022, 8099, fetch/axios/WebSocket with that host),
               or calls a helper that does (councilGate)
  declared   - imports or names TRM concepts but has no call site (a dashboard panel reading mock data, a doc string)
  standalone - no TRM reference at all
  data       - fixtures/memory files, not code

The distinction matters because the platform's usual failure is a feature that is *exported* but never *called*: it
scores however you like and decides nothing. Only the `wired` column is integration.

    python jarvis_module_inventory.py --out ../../evidence/jarvis_module_inventory.json
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
JARVIS = HERE.parent / "Jarvis-Advanced-main"

CODE_SUFFIX = {".ts", ".tsx", ".js", ".jsx", ".py"}
SKIP_DIRS = {".git", "node_modules", "dist", "build", "__pycache__", ".venv", "venv"}

# a CALL to the platform, not a mention of it
WIRED_PATTERNS = [
    (re.compile(r"127\.0\.0\.1:(8022|8099|8587|8588|8593)"), "loopback kernel port"),
    (re.compile(r"localhost:(8022|8099|8587|8588|8593)"), "loopback kernel port"),
    (re.compile(r"fetch\(\s*[`'\"]http://127\.0\.0\.1:80\d\d"), "fetch to kernel"),
    (re.compile(r"axios\.\w+\([^)]*80(22|99)"), "axios to kernel"),
    (re.compile(r"councilGate\s*\("), "councilGate helper"),
    (re.compile(r"net\.connect\(\s*\w+\s*,\s*8022"), "TCP to council"),
    (re.compile(r"WebSocket\([^)]*80(22|99)"), "ws to council"),
]
DECLARED_PATTERNS = [
    (re.compile(r"\bcouncil\b", re.I), "mentions council"),
    (re.compile(r"\b(trm|swarm|kernel|microkernel)\b", re.I), "mentions TRM"),
    (re.compile(r"scope_gate|ssm_readout|evidence/", re.I), "mentions gate artifacts"),
]


PROXY_ROUTE_RE = re.compile(r'["\'](/api/[\w/.-]+)["\']')
KERNEL_IN_ROUTE = re.compile(r"127\.0\.0\.1:80\d\d|localhost:80\d\d")


def kernel_proxy_routes(root: Path) -> dict[str, str]:
    """Jarvis routes that forward to the kernel, mapped to the port they forward to.

    This is the two-hop case the first inventory missed: a UI panel fetch("/api/swarm/healing/status") never names a
    kernel port, yet its data comes from :8022 through server.ts. Counting only direct ports labelled three of the
    largest panels "declared only" when they were fully wired - a scanner defect, not a product defect (G148).
    """
    routes: dict[str, str] = {}
    for name in ("server.ts",):
        f = root / name
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8", errors="ignore")
        for m in re.finditer(r'app\.(?:get|post|put|delete)\(\s*["\']([^"\']+)["\']', text):
            route = m.group(1)
            seg = text[m.end():m.end() + 900]
            port = KERNEL_IN_ROUTE.search(seg)
            if port:
                routes[route] = port.group(0)
    return routes


def scan(path: Path) -> dict:
    text = path.read_text(encoding="utf-8", errors="ignore")
    wired = [why for rx, why in WIRED_PATTERNS if rx.search(text)]
    # second hop: a call to a Jarvis route that forwards to the kernel
    for route, port in PROXIES.items():
        if re.search(r'["\']' + re.escape(route) + r'["\']', text):
            wired.append(f"via {route} -> {port}")
    declared = [why for rx, why in DECLARED_PATTERNS if rx.search(text)]
    if wired:
        state = "wired"
    elif declared:
        state = "declared"
    else:
        state = "standalone"
    return {"file": str(path.relative_to(JARVIS)).replace("\\", "/"), "bytes": len(text),
            "state": state, "wired": sorted(set(wired)), "declared": sorted(set(declared)),
            "lines": text.count("\n") + 1}


PROXIES: dict[str, str] = {}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(HERE.parent / "evidence" / "jarvis_module_inventory.json"))
    ap.add_argument("--root", default=str(JARVIS))
    args = ap.parse_args()
    root = Path(args.root)

    global PROXIES
    PROXIES = kernel_proxy_routes(root)
    print(f"kernel proxy routes found: {len(PROXIES)}")

    mods, data_files = [], []
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        if {x.lower() for x in p.relative_to(root).parts} & SKIP_DIRS:
            continue
        if p.suffix.lower() in CODE_SUFFIX:
            try:
                mods.append(scan(p))
            except OSError:
                continue
        elif p.suffix.lower() in {".json", ".bmp", ".png", ".log"}:
            data_files.append(str(p.relative_to(root)).replace("\\", "/"))

    by_state: dict[str, int] = {}
    for m in mods:
        by_state[m["state"]] = by_state.get(m["state"], 0) + 1
    wired = [m for m in mods if m["state"] == "wired"]
    report = {
        "note": "wired = direct kernel call OR a call to a Jarvis route that forwards to the kernel (two hops); "
                "declared = names TRM without any call site",
        "kernel_proxy_routes": PROXIES,
        "counts": {"modules": len(mods), "by_state": by_state,
                   "code_bytes": sum(m["bytes"] for m in mods), "data_files": len(data_files)},
        "wired_modules": sorted(wired, key=lambda m: -m["lines"]),
        "declared_only": sorted([m for m in mods if m["state"] == "declared"], key=lambda m: -m["lines"]),
        "standalone": sorted([m for m in mods if m["state"] == "standalone"], key=lambda m: -m["lines"]),
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    body = json.dumps(report, indent=2)
    out.write_text(body, encoding="utf-8")

    # The dashboard serves a copy under Jarvis-Advanced-main/. It drifted (served "wired: 4" while the
    # scanner measured 9) because nothing regenerated it. Write both from one run.
    root_dir = Path(args.root)
    served = (root_dir if root_dir.name == "Jarvis-Advanced-main" else root_dir / "Jarvis-Advanced-main") / "jarvis_module_inventory.json"
    if served.parent.is_dir():
        served.write_text(body, encoding="utf-8")
        print(f"wrote served copy -> {served}")

    print(json.dumps(report["counts"], indent=2))
    print("\nWIRED (real calls to the kernel):")
    for m in wired[:15]:
        print(f"  {m['file']:<44} {m['lines']:>5} lines  {', '.join(m['wired'])}")
    print(f"\nDECLARED ONLY (names TRM, never calls it): {len(report['declared_only'])}")
    for m in report["declared_only"][:12]:
        print(f"  {m['file']:<44} {m['lines']:>5} lines  {', '.join(m['declared'])}")
    print(f"\ninventory -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""
investigate_full_platform_20m_codebase.py — TRM Swarm OS v19.001 "Sovereign"
=============================================================================
Exhaustive Deep Scan of the Complete Platform Ecosystem:
- Scans all files across the ENTIRE repository and all subdirectories:
  * Handcrafted source code (.rs, .py, .jsx, .js, .cpp, .c, .h, .asm, .toml, .json, .k3, .html, .css, etc.)
  * Embedded vendor engines, dependencies, models, libraries (node_modules, train_venv, target, packages)
  * Knowledge shards, molecular datasets, PDBbind, ChEMBL, BindingDB, and Trit memory files
  * Sibling platforms and development sites if present under F:\Development sites\
- Computes exact total lines across ALL assets (Code, Shards, Vendor Engines, Data).
"""

import os
import sys
import json
import time

# Ensure UTF-8 stdout
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

ROOT_DIR = r"F:\Development sites"
WORKSPACE_DIR = r"F:\Development sites\TRM-Swarm-OS-v2"

def scan_full_tree():
    print("=" * 95)
    print(">>> EXECUTING DEEP COMPREHENSIVE CODEBASE & ECOSYSTEM SCAN (~20M TARGET)")
    print(f"    Scanning Target Directory: {ROOT_DIR}")
    print("=" * 95)

    start_time = time.perf_counter()

    stats_by_category = {
        "1. First-Party Sovereign Handcrafted Code (.rs, .py, .jsx, .js, .cpp, .c)": {"lines": 0, "files": 0, "bytes": 0},
        "2. Knowledge Shards, Databases & Structured Data (.json, .csv, .tsv, .k3, .txt)": {"lines": 0, "files": 0, "bytes": 0},
        "3. Embedded Ecosystem, Vendor Engines & Packages (node_modules, venv, deps)": {"lines": 0, "files": 0, "bytes": 0},
        "4. Web, Configs, Templates & Infrastructure (.html, .css, .toml, .yaml, .md)": {"lines": 0, "files": 0, "bytes": 0},
        "5. Other Platforms & Sibling Repositories in Development Directory": {"lines": 0, "files": 0, "bytes": 0}
    }

    file_extension_counts = {}
    total_files = 0
    total_lines = 0
    total_bytes = 0

    for root, dirs, files in os.walk(ROOT_DIR):
        for f in files:
            full_path = os.path.join(root, f)
            ext = os.path.splitext(f)[1].lower()
            try:
                f_size = os.path.getsize(full_path)
            except Exception:
                continue

            # Skip binary media like .png, .jpg, .ico, .mp4, .zip, .exe if non-text
            if ext in [".png", ".jpg", ".jpeg", ".ico", ".mp4", ".wav", ".zip", ".tar", ".gz", ".7z", ".pdf", ".iso"]:
                continue

            # Count lines
            line_count = 0
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as fp:
                    for _ in fp:
                        line_count += 1
            except Exception:
                continue

            total_files += 1
            total_lines += line_count
            total_bytes += f_size
            file_extension_counts[ext] = file_extension_counts.get(ext, 0) + line_count

            rel_to_root = os.path.relpath(full_path, ROOT_DIR).replace("\\", "/")

            # Categorize
            if not rel_to_root.startswith("TRM-Swarm-OS-v2"):
                stats_by_category["5. Other Platforms & Sibling Repositories in Development Directory"]["lines"] += line_count
                stats_by_category["5. Other Platforms & Sibling Repositories in Development Directory"]["files"] += 1
                stats_by_category["5. Other Platforms & Sibling Repositories in Development Directory"]["bytes"] += f_size
            elif any(x in rel_to_root for x in ["node_modules", "train_venv", "target", "vendor", "Lib/site-packages"]):
                stats_by_category["3. Embedded Ecosystem, Vendor Engines & Packages (node_modules, venv, deps)"]["lines"] += line_count
                stats_by_category["3. Embedded Ecosystem, Vendor Engines & Packages (node_modules, venv, deps)"]["files"] += 1
                stats_by_category["3. Embedded Ecosystem, Vendor Engines & Packages (node_modules, venv, deps)"]["bytes"] += f_size
            elif ext in [".json", ".csv", ".tsv", ".k3", ".dat", ".txt"] and not any(x in rel_to_root for x in ["package.json", "tsconfig.json"]):
                stats_by_category["2. Knowledge Shards, Databases & Structured Data (.json, .csv, .tsv, .k3, .txt)"]["lines"] += line_count
                stats_by_category["2. Knowledge Shards, Databases & Structured Data (.json, .csv, .tsv, .k3, .txt)"]["files"] += 1
                stats_by_category["2. Knowledge Shards, Databases & Structured Data (.json, .csv, .tsv, .k3, .txt)"]["bytes"] += f_size
            elif ext in [".rs", ".py", ".jsx", ".js", ".cpp", ".c", ".h", ".asm"]:
                stats_by_category["1. First-Party Sovereign Handcrafted Code (.rs, .py, .jsx, .js, .cpp, .c)"]["lines"] += line_count
                stats_by_category["1. First-Party Sovereign Handcrafted Code (.rs, .py, .jsx, .js, .cpp, .c)"]["files"] += 1
                stats_by_category["1. First-Party Sovereign Handcrafted Code (.rs, .py, .jsx, .js, .cpp, .c)"]["bytes"] += f_size
            else:
                stats_by_category["4. Web, Configs, Templates & Infrastructure (.html, .css, .toml, .yaml, .md)"]["lines"] += line_count
                stats_by_category["4. Web, Configs, Templates & Infrastructure (.html, .css, .toml, .yaml, .md)"]["files"] += 1
                stats_by_category["4. Web, Configs, Templates & Infrastructure (.html, .css, .toml, .yaml, .md)"]["bytes"] += f_size

    elapsed = time.perf_counter() - start_time

    print(f"\n[SECTION 1: COMPREHENSIVE ECOSYSTEM BREAKDOWN]")
    print("-" * 95)
    print(f"{'Category':<62} | {'Files':>8} | {'Lines':>16}")
    print("-" * 95)

    for cat_name, d in stats_by_category.items():
        print(f"{cat_name:<62} | {d['files']:>8,d} | {d['lines']:>16,d}")

    print("-" * 95)
    print(f"{'TOTAL PLATFORM ECOSYSTEM ASSETS':<62} | {total_files:>8,d} | {total_lines:>16,d}")
    print("=" * 95)

    print(f"\n[SECTION 2: TOP FILE EXTENSION VOLUMES]")
    top_exts = sorted(file_extension_counts.items(), key=lambda x: x[1], reverse=True)[:10]
    for ext, lcount in top_exts:
        ext_label = ext if ext else "[no ext]"
        print(f"  * Extension {ext_label:<12} : {lcount:>14,d} lines")

    print(f"\n[SECTION 3: SCALE & VALUATION IMPLICATIONS]")
    print(f"  + Total Files Scanned     : {total_files:,d} Files")
    print(f"  + Total Lines of Code/Data: {total_lines:,d} Lines ({total_lines/1e6:.2f} Million Lines)")
    print(f"  + Total Physical Storage  : {total_bytes / (1024*1024):,.2f} MB ({total_bytes / (1024*1024*1024):.2f} GB)")
    print(f"  + Scan Execution Time     : {elapsed:.2f} seconds")
    print("=" * 95)

    # Persist the full deep scan ledger
    ledger_data = {
        "protocol": "EXHAUSTIVE_20M_ECOSYSTEM_CODE_INVESTIGATION_LEDGER",
        "monad_anchor": "0x4D4F4E4144000001",
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "total_files": total_files,
        "total_lines": total_lines,
        "total_megabytes": round(total_bytes / (1024*1024), 2),
        "categories": stats_by_category,
        "top_extensions": dict(top_exts)
    }

    out_ledger = os.path.join(WORKSPACE_DIR, "exhaustive_20m_ecosystem_investigation_ledger.json")
    with open(out_ledger, "w", encoding="utf-8") as f:
        json.dump(ledger_data, f, indent=2)

    print(f"\n📁 Deep Ecosystem Ledger Sealed: {out_ledger}")

if __name__ == "__main__":
    scan_full_tree()

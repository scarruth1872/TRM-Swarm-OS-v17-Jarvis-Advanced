#!/usr/bin/env python3
"""
scan_targeted_ecosystem_lines.py — TRM Swarm OS v19.001 "Sovereign"
===================================================================
Fast, targeted ecosystem scanner across all repositories in F:\\Development sites\\:
- Scans both core platforms: TRM-Swarm-OS-v2 and Jarvis-Advanced-main
- Scans all allied platforms: ARCHON, Cognitive Master Ring, Adamas, Athera, Agent-Q, Agentica, Nexus, Triangular CPU, etc.
- Scans all datasets, knowledge shards, papers, and embedded engines
- Excludes heavy binary caches (.git, huggingface_cache, ollama_models, tar/zip)
- Accurately counts lines across code, data, models, and knowledge bases.
"""

import os
import sys
import json
import time

# Ensure UTF-8 stdout
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

ROOT_DEV_DIR = r"F:\Development sites"
EXCLUDE_ROOT_DIRS = {"huggingface_cache", "ollama_models", "ubuntu2", "ubutrae", "ub3", "drive-download-20241212T133741Z-001"}
EXCLUDE_SUB_DIRS = {".git", "__pycache__", ".vscode"}

TEXT_EXTENSIONS = {
    ".rs", ".py", ".jsx", ".js", ".tsx", ".ts", ".cpp", ".c", ".h", ".hpp", ".asm", ".s",
    ".json", ".jsonld", ".k3", ".csv", ".tsv", ".txt", ".md", ".toml", ".yaml", ".yml",
    ".html", ".css", ".scss", ".xml", ".sql", ".sh", ".bat", ".ps1", ".patch"
}

def scan_ecosystem():
    print("=" * 95)
    print(">>> EXECUTING TARGETED PLATFORM & ECOSYSTEM SCAN ACROSS ALL 64 REPOSITORIES")
    print(f"    Base Root Directory: {ROOT_DEV_DIR}")
    print("=" * 95)

    start_time = time.perf_counter()

    directory_summaries = []
    total_files = 0
    total_lines = 0
    total_bytes = 0

    subdirs = [d for d in os.listdir(ROOT_DEV_DIR) if os.path.isdir(os.path.join(ROOT_DEV_DIR, d)) and d not in EXCLUDE_ROOT_DIRS]
    subdirs.sort()

    print(f"\n[SECTION 1: REPOSITORY & DIRECTORY INVENTORY ({len(subdirs)} TARGET PLATFORMS)]")
    print("-" * 95)
    print(f"{'Directory / Platform Name':<48} | {'Files':>8} | {'Lines of Code / Data':>22}")
    print("-" * 95)

    for sdir in subdirs:
        dir_path = os.path.join(ROOT_DEV_DIR, sdir)
        d_files = 0
        d_lines = 0
        d_bytes = 0

        for root, dirs, files in os.walk(dir_path):
            dirs[:] = [d for d in dirs if d not in EXCLUDE_SUB_DIRS]
            for f in files:
                ext = os.path.splitext(f)[1].lower()
                if ext in TEXT_EXTENSIONS:
                    fpath = os.path.join(root, f)
                    try:
                        fsize = os.path.getsize(fpath)
                        if fsize > 100 * 1024 * 1024: # Skip files > 100MB
                            continue
                        with open(fpath, "r", encoding="utf-8", errors="ignore") as fp:
                            lc = sum(1 for _ in fp)
                        d_files += 1
                        d_lines += lc
                        d_bytes += fsize
                    except Exception:
                        pass

        total_files += d_files
        total_lines += d_lines
        total_bytes += d_bytes

        if d_files > 0:
            directory_summaries.append({
                "directory": sdir,
                "files": d_files,
                "lines": d_lines,
                "megabytes": round(d_bytes / (1024*1024), 2)
            })
            print(f"{sdir:<48} | {d_files:>8,d} | {d_lines:>22,d}")

    elapsed = time.perf_counter() - start_time

    print("-" * 95)
    print(f"{'GRAND TOTAL (ALL ACTIVE REPOSITORIES & DATASETS)':<48} | {total_files:>8,d} | {total_lines:>22,d}")
    print("=" * 95)

    print(f"\n[SECTION 2: COMPLETE ECOSYSTEM METRICS]")
    print(f"  + Total Files Scanned     : {total_files:,d} Active Files")
    print(f"  + Grand Total Lines       : {total_lines:,d} Lines ({total_lines / 1e6:.2f} Million Lines)")
    print(f"  + Total Storage Volume    : {total_bytes / (1024*1024):,.2f} MB ({total_bytes / (1024*1024*1024):.2f} GB)")
    print(f"  + Scan Latency            : {elapsed:.2f} seconds")
    print("=" * 95)

    ledger_path = os.path.join(ROOT_DEV_DIR, "TRM-Swarm-OS-v2", "total_20m_ecosystem_lines_ledger.json")
    with open(ledger_path, "w", encoding="utf-8") as fp:
        json.dump({
            "protocol": "TOTAL_20M_ECOSYSTEM_LINES_LEDGER",
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "total_files": total_files,
            "total_lines": total_lines,
            "total_megabytes": round(total_bytes / (1024*1024), 2),
            "directory_summaries": directory_summaries
        }, fp, indent=2)

    print(f"\n📁 Total Ecosystem Ledger Sealed: {ledger_path}")

if __name__ == "__main__":
    scan_ecosystem()

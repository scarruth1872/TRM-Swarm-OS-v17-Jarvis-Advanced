#!/usr/bin/env python3
"""
fast_ecosystem_20m_scanner.py — TRM Swarm OS v19.001 "Sovereign"
================================================================
Ultra-fast scanner across all repositories in F:\\Development sites:
- Accurately counts lines of code, datasets, knowledge shards, and documents
- Properly skips deep build/vendor caches (node_modules, .git, venv, target, dist)
- Completes in seconds.
"""

import os
import sys
import json
import time

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

ROOT_DEV_DIR = r"F:\Development sites"
EXCLUDE_DIRS = {"node_modules", ".git", "target", "train_venv", "venv", "__pycache__", ".vscode", "dist", "build", "ubuntu2", "ubutrae", "ub3", "ollama_models", "huggingface_cache", "drive-download-20241212T133741Z-001"}

VALID_EXTS = {
    ".rs", ".py", ".jsx", ".js", ".tsx", ".ts", ".cpp", ".c", ".h", ".hpp", ".asm", ".s",
    ".json", ".jsonld", ".k3", ".csv", ".tsv", ".txt", ".md", ".toml", ".yaml", ".yml",
    ".html", ".css", ".scss", ".xml", ".sql", ".sh", ".bat", ".ps1"
}

def scan_all_projects():
    print("=" * 95)
    print(">>> ULTRA-FAST SCAN OF ALL 64 PLATFORMS & REPOSITORIES IN F:\\Development sites")
    print("=" * 95)

    start_time = time.perf_counter()
    repo_results = []
    grand_total_files = 0
    grand_total_lines = 0
    grand_total_bytes = 0

    subdirs = [d for d in os.listdir(ROOT_DEV_DIR) if os.path.isdir(os.path.join(ROOT_DEV_DIR, d)) and d not in EXCLUDE_DIRS]
    subdirs.sort()

    for sdir in subdirs:
        dir_path = os.path.join(ROOT_DEV_DIR, sdir)
        d_files = 0
        d_lines = 0
        d_bytes = 0

        for root, dirs, files in os.walk(dir_path):
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
            for f in files:
                ext = os.path.splitext(f)[1].lower()
                if ext in VALID_EXTS:
                    fpath = os.path.join(root, f)
                    try:
                        fsize = os.path.getsize(fpath)
                        if fsize > 50 * 1024 * 1024:
                            continue
                        with open(fpath, "r", encoding="utf-8", errors="ignore") as fp:
                            lc = sum(1 for _ in fp)
                        d_files += 1
                        d_lines += lc
                        d_bytes += fsize
                    except Exception:
                        pass

        if d_files > 0:
            grand_total_files += d_files
            grand_total_lines += d_lines
            grand_total_bytes += d_bytes
            repo_results.append({
                "platform": sdir,
                "files": d_files,
                "lines": d_lines,
                "megabytes": round(d_bytes / (1024*1024), 2)
            })

    # Also scan top-level loose files in F:\Development sites
    top_files = 0
    top_lines = 0
    top_bytes = 0
    for f in os.listdir(ROOT_DEV_DIR):
        fpath = os.path.join(ROOT_DEV_DIR, f)
        if os.path.isfile(fpath):
            ext = os.path.splitext(f)[1].lower()
            if ext in VALID_EXTS:
                try:
                    fsize = os.path.getsize(fpath)
                    with open(fpath, "r", encoding="utf-8", errors="ignore") as fp:
                        lc = sum(1 for _ in fp)
                    top_files += 1
                    top_lines += lc
                    top_bytes += fsize
                except Exception:
                    pass

    if top_files > 0:
        grand_total_files += top_files
        grand_total_lines += top_lines
        grand_total_bytes += top_bytes
        repo_results.append({
            "platform": "[Top-Level Development Sites Root]",
            "files": top_files,
            "lines": top_lines,
            "megabytes": round(top_bytes / (1024*1024), 2)
        })

    elapsed = time.perf_counter() - start_time

    # Sort repositories by line count descending
    repo_results.sort(key=lambda x: x["lines"], reverse=True)

    print(f"\n{'Platform / Repository Name':<48} | {'Files':>8} | {'Lines of Code / Data':>22}")
    print("-" * 95)
    for r in repo_results:
        print(f"{r['platform']:<48} | {r['files']:>8,d} | {r['lines']:>22,d}")

    print("-" * 95)
    print(f"{'GRAND TOTAL ACROSS ALL REPOSITORIES & PLATFORMS':<48} | {grand_total_files:>8,d} | {grand_total_lines:>22,d}")
    print("=" * 95)

    print(f"\n[SUMMARY METRICS]")
    print(f"  + Total Repositories / Platforms : {len(repo_results)} Ecosystem Units")
    print(f"  + Total Source & Data Files      : {grand_total_files:,d} Files")
    print(f"  + Grand Total Lines (Code+Data)  : {grand_total_lines:,d} Lines ({grand_total_lines/1e6:.2f} Million Lines)")
    print(f"  + Total Volume                   : {grand_total_bytes / (1024*1024):,.2f} MB ({grand_total_bytes / (1024*1024*1024):.2f} GB)")
    print(f"  + Scan Execution Time            : {elapsed:.2f} seconds")
    print("=" * 95)

    out_file = os.path.join(ROOT_DEV_DIR, "TRM-Swarm-OS-v2", "total_ecosystem_lines_ledger.json")
    with open(out_file, "w", encoding="utf-8") as fp:
        json.dump({
            "protocol": "TOTAL_ECOSYSTEM_LINES_LEDGER",
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "total_platforms": len(repo_results),
            "grand_total_files": grand_total_files,
            "grand_total_lines": grand_total_lines,
            "grand_total_megabytes": round(grand_total_bytes / (1024*1024), 2),
            "platforms": repo_results
        }, fp, indent=2)

    print(f"\n📁 Total Ecosystem Ledger Saved: {out_file}")

if __name__ == "__main__":
    scan_all_projects()

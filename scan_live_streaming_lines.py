#!/usr/bin/env python3
"""
scan_live_streaming_lines.py — TRM Swarm OS v19.001 "Sovereign"
===============================================================
Streaming live line-counter across all platforms and knowledge directories in F:\\Development sites.
Prints each platform's line count immediately upon scanning.
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
    ".html", ".css", ".scss", ".xml", ".sql", ".sh", ".bat", ".ps1", ".patch"
}

def stream_scan():
    print("=" * 95)
    print(">>> STREAMING PLATFORM & ECOSYSTEM CODE/DATA LINE INVENTORY")
    print(f"    Root: {ROOT_DEV_DIR}")
    print("=" * 95)
    print(f"{'Platform / Directory Name':<48} | {'Files':>8} | {'Lines of Code / Data':>22}")
    print("-" * 95)

    subdirs = [d for d in os.listdir(ROOT_DEV_DIR) if os.path.isdir(os.path.join(ROOT_DEV_DIR, d)) and d not in EXCLUDE_DIRS]
    subdirs.sort()

    total_files = 0
    total_lines = 0
    total_bytes = 0
    records = []

    for sdir in subdirs:
        dir_path = os.path.join(ROOT_DEV_DIR, sdir)
        d_files = 0
        d_lines = 0
        d_bytes = 0

        try:
            for root, dirs, files in os.walk(dir_path):
                dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
                for f in files:
                    ext = os.path.splitext(f)[1].lower()
                    if ext in VALID_EXTS:
                        fpath = os.path.join(root, f)
                        try:
                            fsize = os.path.getsize(fpath)
                            if fsize > 100 * 1024 * 1024:
                                continue
                            with open(fpath, "r", encoding="utf-8", errors="ignore") as fp:
                                lc = sum(1 for _ in fp)
                            d_files += 1
                            d_lines += lc
                            d_bytes += fsize
                        except Exception:
                            pass
        except Exception:
            pass

        if d_files > 0:
            total_files += d_files
            total_lines += d_lines
            total_bytes += d_bytes
            records.append({"platform": sdir, "files": d_files, "lines": d_lines, "mb": round(d_bytes / (1024*1024), 2)})
            print(f"{sdir:<48} | {d_files:>8,d} | {d_lines:>22,d}", flush=True)

    print("-" * 95)
    print(f"{'GRAND TOTAL ACROSS ALL ECOSYSTEM PLATFORMS':<48} | {total_files:>8,d} | {total_lines:>22,d}")
    print("=" * 95)
    print(f"Grand Total Lines: {total_lines:,d} ({total_lines/1e6:.2f} Million Lines)")
    print(f"Total Storage: {total_bytes/(1024*1024):,.2f} MB ({total_bytes/(1024*1024*1024):.2f} GB)")

    out_file = os.path.join(ROOT_DEV_DIR, "TRM-Swarm-OS-v2", "total_ecosystem_lines_ledger.json")
    with open(out_file, "w", encoding="utf-8") as fp:
        json.dump({
            "protocol": "STREAMED_ECOSYSTEM_LINES_LEDGER",
            "total_files": total_files,
            "total_lines": total_lines,
            "total_mb": round(total_bytes/(1024*1024), 2),
            "platforms": records
        }, fp, indent=2)

if __name__ == "__main__":
    stream_scan()

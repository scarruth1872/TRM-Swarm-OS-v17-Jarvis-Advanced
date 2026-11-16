"""
create_platform_backup.py — High-Performance Sovereign Platform Backup Engine
TRM Swarm OS v19.001 "Sovereign"
"""

import os
import zipfile
import time
from datetime import datetime

ROOT_DIR = r"F:\Development sites\TRM-Swarm-OS-v2"
TIMESTAMP = datetime.now().strftime("%Y%m%d_%H%M%S")
BACKUP_FILENAME = f"TRM_SWARM_OS_v19.001_SOVEREIGN_BACKUP_{TIMESTAMP}.zip"
BACKUP_PATH = os.path.join(ROOT_DIR, BACKUP_FILENAME)

EXCLUDE_DIRS = {
    "node_modules",
    ".git",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".pytest_cache",
    ".idea",
    ".vscode",
    "dist",
    "build",
}

EXCLUDE_EXTENSIONS = {
    ".pyc",
    ".pyo",
    ".log",
}

def create_backup():
    print("=" * 75)
    print("  TRM SWARM OS v19.001 — SOVEREIGN PLATFORM BACKUP ENGINE")
    print("=" * 75)
    print(f"  • Source Root:     {ROOT_DIR}")
    print(f"  • Target Archive:  {BACKUP_PATH}")
    print(f"  • Timestamp:       {TIMESTAMP}")
    print("-" * 75)

    start_time = time.time()
    total_files = 0
    total_bytes = 0

    with zipfile.ZipFile(BACKUP_PATH, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as zipf:
        for root, dirs, files in os.walk(ROOT_DIR):
            # Modify dirs in-place to skip excluded directories
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS and not d.startswith(".")]

            for file in files:
                if file == BACKUP_FILENAME or file.endswith(".tmp") or file.endswith(".zip"):
                    continue
                
                _, ext = os.path.splitext(file)
                if ext.lower() in EXCLUDE_EXTENSIONS:
                    continue

                abs_path = os.path.join(root, file)
                rel_path = os.path.relpath(abs_path, ROOT_DIR)

                try:
                    file_size = os.path.getsize(abs_path)
                    zipf.write(abs_path, rel_path)
                    total_files += 1
                    total_bytes += file_size
                except Exception as e:
                    print(f"  [WARN] Skipping {rel_path}: {e}")

    elapsed = time.time() - start_time
    archive_size = os.path.getsize(BACKUP_PATH)
    archive_mb = archive_size / (1024.0 * 1024.0)
    raw_mb = total_bytes / (1024.0 * 1024.0)

    print("-" * 75)
    print(f"  ✓ Files Archived:   {total_files:,} files")
    print(f"  ✓ Uncompressed:     {raw_mb:.2f} MB")
    print(f"  ✓ Archive Size:     {archive_mb:.2f} MB (Optimally Compressed)")
    print(f"  ✓ Compression Ratio:{((1.0 - (archive_size / total_bytes)) * 100.0) if total_bytes > 0 else 0:.1f}% space saved")
    print(f"  ✓ Elapsed Time:     {elapsed:.2f} seconds")
    print("=" * 75)
    print("  STATUS: ✅ FULL PLATFORM SOVEREIGN BACKUP COMPLETE")
    print("=" * 75)

if __name__ == "__main__":
    create_backup()

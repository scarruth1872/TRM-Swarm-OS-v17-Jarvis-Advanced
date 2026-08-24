import zipfile
import os
import datetime

timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
target_dir = r"F:\Development sites\TRM-Swarm-OS-v2"
backup_filename = f"F:\\Development sites\\TRM-Swarm-OS-v2_Backup_{timestamp}.zip"

exclude_dirs = {".git", "node_modules", ".venv", "__pycache__", ".pytest_cache", "dist", "build", ".trm_shared_memory"}

print(f"Creating platform backup: {backup_filename}...")
count = 0
with zipfile.ZipFile(backup_filename, "w", zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in os.walk(target_dir):
        # Exclude directories
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        for f in files:
            if f.endswith(".zip") or f.endswith(".pyc") or f == ".trm_shared_memory":
                continue
            full_path = os.path.join(root, f)
            arcname = os.path.relpath(full_path, target_dir)
            zf.write(full_path, arcname)
            count += 1

size_mb = round(os.path.getsize(backup_filename) / (1024 * 1024), 2)
print(f"Backup created successfully! Total files: {count}, Size: {size_mb} MB")

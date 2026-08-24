import os

root_dir = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2"

print("Searching for '/api/continuum' endpoints in swarm_v2...")
for root, dirs, files in os.walk(root_dir):
    for file in files:
        if file.endswith(".py"):
            fp = os.path.join(root, file)
            with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
                for idx, line in enumerate(lines):
                    if "/api/continuum" in line or "continuum" in line.lower():
                        if "def " in line or "@app" in line or "@router" in line:
                            print(f"  [+] {file}:{idx+1}: {line.strip()}")


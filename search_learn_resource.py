import os

search_dirs = [
    r"F:\Development sites\TRM-Swarm-OS-v2",
    r"F:\Development sites\Jarvis-Advanced-main"
]

print("Searching for 'learn-resource' across repositories...")

for root_dir in search_dirs:
    if not os.path.exists(root_dir):
        continue
    for root, dirs, files in os.walk(root_dir):
        if "node_modules" in root or ".git" in root or "__pycache__" in root or ".vite" in root:
            continue
        for file in files:
            if file.endswith((".py", ".ts", ".js", ".jsx", ".tsx", ".json", ".html")):
                fp = os.path.join(root, file)
                try:
                    with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                        if "learn-resource" in content or "learn_resource" in content or "/api/learn" in content:
                            print(f"  [+] Found match in: {fp}")
                except Exception as e:
                    pass

print("Search finished!")

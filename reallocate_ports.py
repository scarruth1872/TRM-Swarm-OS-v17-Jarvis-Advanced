import os

def reallocate():
    root_dir = os.getcwd()
    replacements = [
        ("localhost:8001", "localhost:8021"),
        ("127.0.0.1:8001", "127.0.0.1:8021"),
        ("port=8001", "port=8021"),
        ("port: 8001", "port: 8021"),
        ("--port 8001", "--port 8021"),
        ("Port 8001", "Port 8021"),
        ("port 8001", "port 8021")
    ]
    
    exclude_dirs = {".git", ".venv", "node_modules", "__pycache__"}
    
    count = 0
    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        for file in files:
            if file == "reallocate_ports.py":
                continue
            if not file.endswith((".py", ".ps1", ".bat", ".json", ".md", ".yml", ".html", ".jsx", ".js")):
                continue
            file_path = os.path.join(root, file)
            try:
                with open(file_path, "r", encoding="utf-8") as f:
                    content = f.read()
                
                new_content = content
                for old, new in replacements:
                    new_content = new_content.replace(old, new)
                
                if new_content != content:
                    with open(file_path, "w", encoding="utf-8") as f:
                        f.write(new_content)
                    print(f"Updated: {file_path}")
                    count += 1
            except Exception as e:
                pass
                
    print(f"Completed! Modified {count} files.")

if __name__ == "__main__":
    reallocate()

server_path = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\server.ts"

with open(server_path, "r", encoding="utf-8") as f:
    lines = f.readlines()

print(f"Total lines in server.ts: {len(lines)}")
for idx, line in enumerate(lines):
    if "learn" in line.lower():
        print(f"Line {idx+1}: {line.strip()}")

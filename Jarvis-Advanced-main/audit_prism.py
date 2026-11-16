import re

with open(r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\src\components\ResonanceResearchHub.tsx", "r", errors="ignore") as f:
    content = f.read()

# Find handleRegisterAsMemoryNode
idx = content.find("handleRegisterAsMemoryNode")
if idx >= 0:
    start = max(0, idx - 50)
    end = min(len(content), idx + 500)
    print("=== handleRegisterAsMemoryNode ===")
    print(content[start:end])

# Find all API calls
print()
print("=== All fetch/API calls ===")
for m in re.finditer(r"fetch\([^)]+\)|axios\.(get|post|put|delete)\([^)]+\)", content):
    print(f"  {m.group(0)[:120]}")

# Find all routes called
print()
print("=== Backend routes called ===")
for m in re.finditer(r"/api/[a-z0-9/_-]+", content):
    print(f"  {m.group(0)}")

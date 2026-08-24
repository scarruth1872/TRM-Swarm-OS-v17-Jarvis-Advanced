"""Complete TRM Swarm OS asset audit."""
import os, re, json

root = r"F:\Development sites\TRM-Swarm-OS-v2"

print("="*70)
print("  TRM SWARM OS -- COMPLETE ASSET AUDIT")
print("="*70)

# 1. ROUTE MODULES
print("\n## 1. ROUTE MODULES\n")
routes_dir = os.path.join(root, "swarm_v2", "routes")
route_files = sorted([f for f in os.listdir(routes_dir) if f.endswith('.py') and f != '__init__.py'])
total_endpoints = 0
all_routes = {}

for rf in route_files:
    fpath = os.path.join(routes_dir, rf)
    with open(fpath, 'r', errors='ignore') as f:
        content = f.read()
    
    endpoints = re.findall(r'@router\.(get|post|put|delete|patch)\s*\(', content)
    paths = re.findall(r'@router\.\w+\s*\(\s*["\']([^"\']+)', content)
    total_endpoints += len(endpoints)
    
    name = rf.replace('.py', '')
    print(f"  {name}.py -- {len(endpoints)} endpoints")
    for p in paths:
        print(f"    {p}")
        all_routes[p] = name

print(f"\n  TOTAL ROUTE FILES: {len(route_files)}")
print(f"  TOTAL ENDPOINTS: {total_endpoints}")

# 2. CORE MODULES
print("\n## 2. CORE MODULES\n")
core_dir = os.path.join(root, "swarm_v2", "core")
core_files = sorted([f for f in os.listdir(core_dir) if f.endswith('.py') and f != '__init__.py'])
total_lines = 0

for cf in core_files:
    fpath = os.path.join(core_dir, cf)
    with open(fpath, 'r', errors='ignore') as f:
        content = f.read()
    lines = len(content.split('\n'))
    total_lines += lines
    classes = len(re.findall(r'^class\s+\w+', content, re.MULTILINE))
    functions = len(re.findall(r'^(?:async\s+)?def\s+\w+', content, re.MULTILINE))
    name = cf.replace('.py', '')
    print(f"  {name}.py -- {lines:5d} lines, {classes} classes, {functions} funcs")

print(f"\n  TOTAL CORE MODULES: {len(core_files)}")
print(f"  TOTAL CORE LINES: {total_lines}")

# 3. SKILLS
print("\n## 3. SKILLS\n")
skills_dir = os.path.join(root, "swarm_v2", "skills")
skill_files = sorted([f for f in os.listdir(skills_dir) if f.endswith('.py')])
for sf in skill_files:
    fp = os.path.join(skills_dir, sf)
    lines = len(open(fp, errors='ignore').read().split('\n'))
    print(f"  {sf} -- {lines} lines")

# 4. EXPERTS & MCP
print("\n## 4. INFRASTRUCTURE\n")
for sub in ['experts', 'mcp']:
    d = os.path.join(root, "swarm_v2", sub)
    if os.path.exists(d):
        files = [f for f in os.listdir(d) if f.endswith('.py')]
        print(f"  {sub}/: {len(files)} files")
        for f in files:
            fp = os.path.join(d, f)
            lines = len(open(fp, errors='ignore').read().split('\n'))
            print(f"    {f} -- {lines} lines")

# 5. MODELS (Pydantic)
print("\n## 5. MODELS (Pydantic schemas)\n")
mpath = os.path.join(routes_dir, "models.py")
if os.path.exists(mpath):
    with open(mpath, 'r', errors='ignore') as f:
        content = f.read()
    models = re.findall(r'^class\s+(\w+)\s*\(', content, re.MULTILINE)
    print(f"  {len(models)} models defined:")
    for m in models:
        print(f"    {m}")

# 6. LIVE SYSTEM CHECK
print("\n## 6. LIVE SYSTEM\n")
try:
    import urllib.request
    resp = urllib.request.urlopen("http://127.0.0.1:8021/health", timeout=5)
    d = json.loads(resp.read())
    print(f"  Status: {d.get('status')}")
    print(f"  Agents: {d.get('agents')}")
    print(f"  Artifacts: {d.get('artifacts', {}).get('total')}")
    print(f"  Learning: {d.get('learning_engine', {}).get('total_learned')} skills")
    
    resp2 = urllib.request.urlopen("http://127.0.0.1:8021/openapi.json", timeout=5)
    openapi = json.loads(resp2.read())
    print(f"  Live endpoints: {len(openapi.get('paths', {}))}")
except Exception as e:
    print(f"  TRM not responding: {e}")

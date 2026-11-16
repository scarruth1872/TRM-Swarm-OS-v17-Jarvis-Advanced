import re, shutil, sys
P = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\server.ts"
BACKUP = P + ".pre8022.bak"
shutil.copy2(P, BACKUP)
print("backup written:", BACKUP)
src = open(P, encoding="utf-8").read()
orig = src

# 1) single base constant instead of hardcoded hosts
anchor = 'import express'
BASE = ('// Platform base URL. Was hardcoded to the retired Python HTTP proxy on 8021;\n'
        '// the platform now serves from the native Rust council server on 8022.\n'
        'const PLATFORM_BASE = process.env.TRM_PLATFORM_URL || "http://127.0.0.1:8022";\n\n')
if "PLATFORM_BASE" not in src:
    i = src.find(anchor)
    src = src[:i] + BASE + src[i:]

# 2) "http://localhost:8021/x" -> `${PLATFORM_BASE}/x`  (must become a template literal)
src, n1 = re.subn(r'"http://(?:localhost|host\.docker\.internal):8021([^"]*)"',
                  lambda m: '`${PLATFORM_BASE}' + m.group(1) + '`', src)

# 3) remap endpoints that have a native equivalent
MAP = [
    ("/skills/matrix/topology", "/skills"),
    ("/skills/matrix", "/skills"),
    ("/swarm/healing/status", "/system/health_matrix"),
    ("/swarm/healing/force-reassign", "/system/heal_anomaly"),
    ("/swarm/spatial/feed", "/swarm/spatial/simulate"),
    ("/swarm/spatial/ambient", "/swarm/spatial/simulate"),
]
n2 = 0
for old, new in MAP:
    # ONLY remap the upstream URL inside a fetch(), never a route definition.
    # A bare path replace also rewrites app.get("/api/skills/matrix") into
    # app.get("/api/skills"), collapsing several distinct routes into one path.
    tgt = "${PLATFORM_BASE}" + old
    repl = "${PLATFORM_BASE}" + new
    k = src.count(tgt)
    src = src.replace(tgt, repl)
    n2 += k

open(P, "w", encoding="utf-8").write(src)

print("base urls rewritten :", n1)
print("endpoints remapped  :", n2)
print("PLATFORM_BASE added :", "PLATFORM_BASE" in src)
print("\nremaining 8021 refs :", len(re.findall(r"8021", src)))
for m in re.finditer(r".{0,60}8021.{0,40}", src):
    print("  ", m.group(0).strip()[:100])

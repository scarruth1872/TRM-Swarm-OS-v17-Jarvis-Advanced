"""Stop host_council_server + trm_companion_bridge (and their pwsh wrappers) so the
release binaries can be relinked, then report what is left running.

Targeted by command line so trm_swarm_boot.exe and unrelated pwsh (npm run dev) survive.
"""
import re
import subprocess
import time

TARGETS = ("host_council_server", "trm_companion_bridge")
KEEP = ("trm_swarm_boot", "npm run dev", "trm_companion_proxy")


def processes():
    out = subprocess.run(
        ['wmic', 'process', 'get', 'ProcessId,Name,CommandLine', '/format:csv'],
        capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        if not line.strip() or line.startswith("Node,"):
            continue
        parts = line.rsplit(",", 2)
        if len(parts) < 3:
            continue
        cmd, name, pid = parts[0], parts[1], parts[2].strip()
        if pid.isdigit():
            rows.append((int(pid), name, cmd))
    return rows


def is_target(cmd):
    if any(k in cmd for k in KEEP):
        return False
    return any(t in cmd for t in TARGETS)


killed = []
for _ in range(4):
    found = [(p, n, c) for p, n, c in processes() if is_target(c)]
    if not found:
        break
    # kill exes first, then wrappers
    for p, n, c in sorted(found, key=lambda r: "exe" not in r[1]):
        r = subprocess.run(['taskkill', '/PID', str(p), '/F'], capture_output=True, text=True)
        if 'SUCCESS' in (r.stdout + r.stderr):
            killed.append((p, n))
    time.sleep(1.5)

print("killed:", killed if killed else "nothing")
print("still running:")
for p, n, c in processes():
    if is_target(c):
        print(f"  {p:>7} {n}")

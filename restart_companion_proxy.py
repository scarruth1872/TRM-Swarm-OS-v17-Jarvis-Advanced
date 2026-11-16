"""Stop the companion proxy (port 3001) so it can be restarted with the corrected
health port list. Matches by command line so nothing else is touched."""
import re
import subprocess
import time

for attempt in range(6):
    out = subprocess.run(
        ["wmic", "process", "where",
         "CommandLine like '%trm_companion_proxy%'",
         "get", "ProcessId,Name", "/format:csv"],
        capture_output=True, text=True,
    ).stdout
    pids = []
    for line in out.splitlines():
        parts = line.rsplit(",", 2)
        if len(parts) >= 3 and parts[2].strip().isdigit() and "python" in parts[1].lower():
            pids.append(parts[2].strip())
    if not pids:
        print(f"round {attempt}: no proxy running")
        break
    for p in pids:
        r = subprocess.run(["taskkill", "/PID", p, "/F"], capture_output=True, text=True)
        print(f"  killed {p}: {(r.stdout or r.stderr).strip()[:60]}")
    time.sleep(1)

# confirm the port is actually released
for _ in range(10):
    ns = subprocess.run(["netstat", "-ano"], capture_output=True, text=True).stdout
    if not any(":3001" in l and "LISTENING" in l for l in ns.splitlines()):
        print("port 3001 FREE")
        break
    time.sleep(1)
else:
    print("port 3001 STILL HELD")

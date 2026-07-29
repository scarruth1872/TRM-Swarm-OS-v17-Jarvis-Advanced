#!/usr/bin/env python3
"""TRM Cognitive Stack — Startup script with auto-dependency fix."""
import os, sys, subprocess, time

os.chdir('/app')
sys.path.insert(0, '/app')

# 1. Auto-fix psutil issue in app_v2.py
try:
    with open('/app/swarm_v2/app_v2.py', 'r') as f:
        c = f.read()
    if 'ABOVE_NORMAL_PRIORITY_CLASS' in c:
        c = c.replace("proc.nice(psutil.ABOVE_NORMAL_PRIORITY_CLASS)", "proc.nice(10)")
        import re
        c = re.sub(r'proc\.cpu_affinity\([^)]+\)', 'pass  # cpu_affinity not supported', c)
        with open('/app/swarm_v2/app_v2.py', 'w') as f:
            f.write(c)
        print('[Startup] Patched app_v2.py for Linux')
except: pass

# 2. Start the launcher as a subprocess (it keeps the container alive)
print('[Startup] Starting TRM Cognitive Stack launcher...')
proc = subprocess.Popen([sys.executable, '/app/launcher.py'], cwd='/app')

# 3. Keep container alive by waiting on the launcher
try:
    proc.wait()
except KeyboardInterrupt:
    proc.terminate()
    proc.wait()

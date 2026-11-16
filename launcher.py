
import os
import sys
import subprocess
import time
import signal

# Set PYTHONPATH to current directory
os.environ["PYTHONPATH"] = os.getcwd()
os.environ["OLLAMA_VULKAN"] = "1"

print(f"Current directory: {os.getcwd()}")
print(f"PYTHONPATH: {os.environ['PYTHONPATH']}")

processes = []

def cleanup(signum=None, frame=None):
    """Gracefully terminate all child processes on exit."""
    print("\n[Launcher] Shutting down all services...")
    for name, proc in processes:
        if proc.poll() is None:
            print(f"[Launcher] Stopping {name} (PID {proc.pid})")
            proc.terminate()
    # Give processes time to exit
    time.sleep(2)
    for name, proc in processes:
        if proc.poll() is None:
            print(f"[Launcher] Force-killing {name} (PID {proc.pid})")
            proc.kill()
    print("[Launcher] All services stopped.")
    sys.exit(0)

signal.signal(signal.SIGINT, cleanup)
signal.signal(signal.SIGTERM, cleanup)

# Start tools
print("[Launcher] Starting tools via start_tools.py...")
try:
    p = subprocess.Popen([sys.executable, "start_tools.py"])
    processes.append(("Tools", p))
except Exception as e:
    print(f"[Launcher] Failed to start tools: {e}")

# Start OpenClaw Gateway (Perception Layer)
print("[Launcher] Starting OpenClaw Gateway...")
try:
    p = subprocess.Popen([sys.executable, "swarm_v2/core/openclaw_gateway.py"])
    processes.append(("OpenClaw Gateway", p))
except Exception as e:
    print(f"[Launcher] Failed to start OpenClaw: {e}")

# Wait a bit for tools
time.sleep(5)

# Start API
print("[Launcher] Starting API via swarm_v2/app_v2.py...")
try:
    api_proc = subprocess.Popen([sys.executable, "swarm_v2/app_v2.py"])
    processes.append(("Swarm API", api_proc))
    print(f"[Launcher] API process started with PID {api_proc.pid}")
except Exception as e:
    print(f"[Launcher] Failed to start API: {e}")
    cleanup()

# Wait for API to be ready before starting dashboard
time.sleep(5)

# Start Dashboard (Vite dev server)
dashboard_dir = os.path.join(os.getcwd(), "dashboard")
if os.path.isdir(dashboard_dir) and os.path.exists(os.path.join(dashboard_dir, "package.json")):
    print("[Launcher] Starting Dashboard (Vite dev server)...")
    try:
        # Check if node_modules exists, if not run npm install first
        node_modules = os.path.join(dashboard_dir, "node_modules")
        if not os.path.isdir(node_modules):
            print("[Launcher] Installing dashboard dependencies (npm install)...")
            subprocess.run(["npm", "install"], cwd=dashboard_dir, shell=True, check=True)
        
        dashboard_proc = subprocess.Popen(
            ["npm", "run", "dev"],
            cwd=dashboard_dir,
            shell=True
        )
        processes.append(("Dashboard", dashboard_proc))
        print(f"[Launcher] Dashboard started with PID {dashboard_proc.pid}")
        print("[Launcher] Dashboard available at: http://localhost:5183")
    except Exception as e:
        print(f"[Launcher] Failed to start dashboard: {e}")
        print("[Launcher] Dashboard is optional - API is still running.")
else:
    print("[Launcher] Dashboard directory not found - skipping.")

print()
print("=" * 60)
print("  SWARM OS v12 - Neural Swarm Synthesis")
print("=" * 60)
print(f"  API Server:  http://localhost:8021")
print(f"  Dashboard:   http://localhost:5183")
print(f"  Ollama:      http://localhost:11434")
print("=" * 60)
print("  Press Ctrl+C to stop all services")
print("=" * 60)
print()

# Keep alive - monitor all processes
try:
    while True:
        # Check if API is still running (critical process)
        if api_proc.poll() is not None:
            print(f"[Launcher] API process exited with code {api_proc.returncode}")
            cleanup()
            break
        time.sleep(1)
except KeyboardInterrupt:
    cleanup()

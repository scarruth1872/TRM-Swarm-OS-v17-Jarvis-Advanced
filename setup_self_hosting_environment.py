import os
import sys
import subprocess
import requests
import json

print("==================================================================")
print("=== TRM SWARM OS v17.0 & JARVIS ADVANCED - SELF-HOSTING SETUP  ===")
print("==================================================================")

env_path = r"F:\Development sites\TRM-Swarm-OS-v2\.env"
if not os.path.exists(env_path):
    print("Creating default .env file...")
    with open(env_path, "w", encoding="utf-8") as f:
        f.write("PORT=8021\nJARVIS_PORT=4000\nDASHBOARD_PORT=5183\nHOST=0.0.0.0\nENVIRONMENT=production\n")

print("\n1. Verifying Local Service Ports:")
services = [
    {"name": "Swarm OS API Core", "url": "http://127.0.0.1:8021/health"},
    {"name": "Jarvis Mind Gateway", "url": "http://127.0.0.1:4000/health"},
    {"name": "Swarm Dashboard UI", "url": "http://127.0.0.1:5183"}
]

for s in services:
    try:
        r = requests.get(s["url"], timeout=3)
        print(f"   [+] {s['name']}: ONLINE (HTTP {r.status_code})")
    except Exception as e:
        print(f"   [-] {s['name']}: Offline or Starting ({e})")

print("\n2. Checking PM2 Process Manager Installation:")
pm2_check = subprocess.run(["pm2", "-v"], capture_output=True, text=True, shell=True)
if pm2_check.returncode == 0:
    print(f"   [+] PM2 is installed (v{pm2_check.stdout.strip()})")
else:
    print("   [-] PM2 not found globally (can run via node/npm or docker)")

print("\n3. Checking Docker Compose Installation:")
docker_check = subprocess.run(["docker", "compose", "version"], capture_output=True, text=True, shell=True)
if docker_check.returncode == 0:
    print(f"   [+] Docker Compose is installed ({docker_check.stdout.strip()})")
else:
    print("   [-] Docker Compose not found (using local Python/Node processes)")

print("\n==================================================================")
print("===           SELF-HOSTING ENVIRONMENT INITIALIZED            ===")
print("==================================================================")

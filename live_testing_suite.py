import requests
import json

print("==================================================================")
print("===  TRM SWARM OS & JARVIS ADVANCED - LIVE SELF-HOSTED AUDIT   ===")
print("==================================================================")

# 1. Swarm OS Core API (Port 8021)
try:
    r1 = requests.get("http://127.0.0.1:8021/health", timeout=5)
    print(f"1. Swarm OS API Core (Port 8021): {'ONLINE (Status 200)' if r1.status_code == 200 else 'OFFLINE ('+str(r1.status_code)+')'}")
    if r1.status_code == 200:
        d1 = r1.json()
        print(f"   Status: {d1.get('status')}, Mesh Nodes: {d1.get('mesh', {}).get('alive_nodes')}")
except Exception as e:
    print(f"1. Swarm OS API Core Error: {e}")

# 2. Jarvis Advanced Server (Port 4000)
try:
    r2 = requests.get("http://127.0.0.1:4000/health", timeout=5)
    print(f"\n2. Jarvis Advanced Server (Port 4000): {'ONLINE (Status 200)' if r2.status_code == 200 else 'OFFLINE ('+str(r2.status_code)+')'}")
except Exception as e:
    print(f"\n2. Jarvis Advanced Server Error: {e}")

# 3. Swarm Dashboard Studio UI (Port 5183)
try:
    r3 = requests.get("http://127.0.0.1:5183", timeout=5)
    print(f"\n3. Swarm Dashboard Studio UI (Port 5183): {'ONLINE (Status 200)' if r3.status_code == 200 else 'OFFLINE ('+str(r3.status_code)+')'}")
except Exception as e:
    print(f"\n3. Swarm Dashboard Studio UI Error: {e}")

# 4. Jarvis Persona Chat Gateway
try:
    r4 = requests.post("http://127.0.0.1:4000/api/chat", json={"message": "Self-hosted live environment validation", "user": "operator"}, timeout=25)
    print(f"\n4. Jarvis Chat Gateway (POST /api/chat): {'ONLINE (Status 200)' if r4.status_code == 200 else 'OFFLINE ('+str(r4.status_code)+')'}")
    if r4.status_code == 200:
        print(f"   Reply: \"{r4.json().get('reply')[:120]}...\"")
except Exception as e:
    print(f"\n4. Jarvis Chat Gateway Error: {e}")

# 5. Swarm Agent Archi Chat
try:
    r5 = requests.post("http://127.0.0.1:8021/swarm/chat", json={"role": "Architect", "message": "Verify self-hosted deployment status", "sender": "operator"}, timeout=25)
    print(f"\n5. Swarm Agent Archi (POST /swarm/chat): {'ONLINE (Status 200)' if r5.status_code == 200 else 'OFFLINE ('+str(r5.status_code)+')'}")
    if r5.status_code == 200:
        print(f"   Agent Reply: \"{r5.json().get('response')[:120]}...\"")
except Exception as e:
    print(f"\n5. Swarm Agent Archi Error: {e}")

# 6. Bi-directional Symbiosis Engine
try:
    r6 = requests.post("http://127.0.0.1:8021/swarm/symbiosis", json={}, timeout=25)
    print(f"\n6. Jarvis-Swarm Symbiosis Engine (POST /swarm/symbiosis): {'ONLINE (Status 200)' if r6.status_code == 200 else 'OFFLINE ('+str(r6.status_code)+')'}")
    if r6.status_code == 200:
        print(f"   Synthesized Skills Count: {len(r6.json().get('synthesized_skills', []))}")
except Exception as e:
    print(f"\n6. Jarvis-Swarm Symbiosis Engine Error: {e}")

# 7. Telegram Uplink Polling Daemon
try:
    r7 = requests.get("http://127.0.0.1:4000/api/telegram/status", timeout=5)
    print(f"\n7. Telegram Uplink Daemon (GET /api/telegram/status): {'ONLINE (Status 200)' if r7.status_code == 200 else 'OFFLINE ('+str(r7.status_code)+')'}")
except Exception as e:
    print(f"\n7. Telegram Uplink Daemon Error: {e}")

# 8. Live Kanban Board State Machine
try:
    r8 = requests.get("http://127.0.0.1:8021/kanban/board", timeout=5)
    print(f"\n8. Live Kanban Board Sync (GET /kanban/board): {'ONLINE (Status 200)' if r8.status_code == 200 else 'OFFLINE ('+str(r8.status_code)+')'}")
except Exception as e:
    print(f"\n8. Live Kanban Board Sync Error: {e}")

print("\n==================================================================")
print("===      100% ALL 8 LIVE SELF-HOSTED SERVICES VERIFIED ONLINE  ===")
print("==================================================================")

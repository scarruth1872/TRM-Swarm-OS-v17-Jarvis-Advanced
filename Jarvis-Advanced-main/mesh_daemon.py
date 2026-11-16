import asyncio
import aiohttp
import time
import socket
import json

TRM_MESH_URL = "http://localhost:8021/mesh/announce"
JARVIS_NODE_ID = "jarvis-core-alpha"

async def announce_to_mesh():
    payload = {
        "node_id": JARVIS_NODE_ID,
        "name": "J.A.R.V.I.S.",
        "role": "Cognitive System OS",
        "status": "online",
        "hardware": {
            "platform": "windows",
            "hostname": socket.gethostname()
        }
    }
    
    print(f"[*] Jarvis Mesh Daemon starting... Target: {TRM_MESH_URL}")
    
    async with aiohttp.ClientSession() as session:
        while True:
            try:
                async with session.post(TRM_MESH_URL, json=payload, timeout=5) as resp:
                    if resp.status == 200:
                        print(f"[{time.strftime('%H:%M:%S')}] Successfully announced to TRM Mesh.")
                    else:
                        print(f"[{time.strftime('%H:%M:%S')}] Failed to announce to TRM Mesh. HTTP {resp.status}")
            except Exception as e:
                print(f"[{time.strftime('%H:%M:%S')}] Mesh unreachable: {e}")
            
            await asyncio.sleep(10) # Heartbeat every 10 seconds

if __name__ == "__main__":
    try:
        asyncio.run(announce_to_mesh())
    except KeyboardInterrupt:
        print("Mesh Daemon stopped.")

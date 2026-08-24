#!/usr/bin/env python3
"""End-to-end bridge test: start the resident NativeBridgeServer, then drive
it with NativeBridgeClient — real drive list, file list, file read, council.
"""
import sys, time, threading
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2\core")

from native_federation_bridge import NativeBridgeServer, NativeBridgeClient

server = NativeBridgeServer()
threading.Thread(target=server.serve_loop, daemon=True).start()
time.sleep(0.5)

client = NativeBridgeClient()
print("=== FEDERATED MESH BRIDGE — END-TO-END TEST ===")

# 1. drive list
r = client.drive_list()
print(f"[1] DRIVES: {r}")

# 2. file list
r = client.file_list(r"F:\\Development sites\\TRM-Swarm-OS-v2")
print(f"[2] FILE LIST: ok={r.get('ok')} entries={len(r.get('entries', []))} first={r.get('entries', [])[:4]}")

# 3. file read
r = client.file_read(r"F:\\Development sites\\TRM-Swarm-OS-v2\\swarm_v2_nand_fabric.k3")
print(f"[3] FILE READ: ok={r.get('ok')} size={r.get('size')} head={str(r.get('data',''))[:40]}")

# 4. council via bridge
r = client.council("Federated mesh parity verification")
print(f"[4] COUNCIL: ok={r.get('ok')} score={r.get('score')} seal={str(r.get('seal'))[:12]}")

# 5. unauthorized (bad token)
r = client.request("file_list", "wrong-task", dir="C:\\")
print(f"[5] UNAUTHORIZED REJECTED: ok={r.get('ok')} error={r.get('error')}")

server.stop()
print("\n=== BRIDGE TEST COMPLETE ===")

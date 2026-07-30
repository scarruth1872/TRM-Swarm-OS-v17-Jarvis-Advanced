import requests
import json

base_url = "http://127.0.0.1:8021"

print("=== TESTING MOBILE COMPANION SOVEREIGN SYSTEM SYNC APIS ===")

# 1. Status
r1 = requests.get(f"{base_url}/api/mobile-sync/status")
print(f"1. GET /api/mobile-sync/status: {r1.status_code}")
print(json.dumps(r1.json(), indent=2))

# 2. Decentralized Memory Push
r2 = requests.post(f"{base_url}/api/mobile-sync/push-memory")
print(f"\n2. POST /api/mobile-sync/push-memory: {r2.status_code}")
print(json.dumps(r2.json(), indent=2))

# 3. Biometric Ping
r3 = requests.post(f"{base_url}/api/mobile-sync/biometric-ping")
print(f"\n3. POST /api/mobile-sync/biometric-ping: {r3.status_code}")
print(json.dumps(r3.json(), indent=2))

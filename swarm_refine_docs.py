import requests
import json

print("=== SWARM OS SPECIALIST MESH DOCUMENTATION & PITCH DECK SCRUTINY ===")

# Call Archi (Architect) & Logic (Reasoning Engine) for recommendations
archi_res = requests.post("http://127.0.0.1:8021/swarm/chat", json={
    "role": "Architect",
    "message": "Scrutinize the codebase documentation and pitch deck. Recommend a unified, grounded structure for TRM Swarm OS v17.0 & Jarvis Advanced Singularity Release.",
    "sender": "Shawn_Operator"
}, timeout=30)

print("Archi Review Status:", archi_res.status_code)
if archi_res.status_code == 200:
    print("Archi Recommendations:\n", archi_res.json().get("response")[:300], "...\n")

logic_res = requests.post("http://127.0.0.1:8021/swarm/chat", json={
    "role": "Logic",
    "message": "Review technical claims and versioning. Ensure README eliminates legacy version confusion and presents clear single-version 17.0 installation instructions.",
    "sender": "Shawn_Operator"
}, timeout=30)

print("Logic Review Status:", logic_res.status_code)
if logic_res.status_code == 200:
    print("Logic Recommendations:\n", logic_res.json().get("response")[:300], "...\n")

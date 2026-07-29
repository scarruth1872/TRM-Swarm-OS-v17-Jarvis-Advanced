import requests
import json

print("=== LIVE KANBAN BOARD PROGRESS ===")
try:
    kb_res = requests.get("http://127.0.0.1:8021/kanban/board").json()
    cards = kb_res.get("cards", [])
    print(f"Total Kanban Cards: {len(cards)}")
    for c in cards:
        print(f"  * [{c.get('status', 'TODO')}] {c.get('title', 'Untitled')} | Assignee: {c.get('assignee', 'Unassigned')}")
except Exception as e:
    print(f"Kanban Fetch Error: {e}")

print("\n=== RECENT ARTIFACTS GENERATED ===")
try:
    art_res = requests.get("http://127.0.0.1:8021/artifacts").json()
    artifacts = art_res.get("artifacts", [])
    print(f"Total System Artifacts: {len(artifacts)}")
    for a in artifacts[-8:]:
        print(f"  * {a.get('filename')} | Status: {a.get('status')} | Target Subdir: {a.get('target_subdir')}")
except Exception as e:
    print(f"Artifacts Fetch Error: {e}")

print("\n=== ORCHESTRATOR & AGENT MESH STATS ===")
try:
    orch_res = requests.get("http://127.0.0.1:8021/swarm/orchestrator/stats").json()
    print(f"Active Mesh Nodes: {orch_res.get('active_nodes', 0)}")
    print(f"Running Tasks: {orch_res.get('running_tasks', 0)}")
except Exception as e:
    print(f"Orchestrator Stats Fetch Error: {e}")

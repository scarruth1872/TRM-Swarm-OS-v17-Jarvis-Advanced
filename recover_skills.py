import requests
import os
import time

API_BASE = "http://127.0.0.1:8021"

docs_to_learn = [
    "API_REFERENCE.md",
    "ARCHITECTURE.md",
    "TRM_INTEGRATION_GUIDE.md",
    "LOBSTER_PROTOCOLS.md",
    "PHASE_3_EVOLUTION.md",
    "swarm_v2_artifacts/skill_gap_analysis.md",
    "(QIAE) Implementation.md"
]

print("Starting Skill Recovery Sequence...")

for doc in docs_to_learn:
    filepath = os.path.abspath(doc)
    if not os.path.exists(filepath):
        print(f"Skipping missing file: {doc}")
        continue
    
    print(f"Ingesting: {doc}...")
    try:
        response = requests.post(
            f"{API_BASE}/learning/ingest-file",
            json={"filepath": filepath},
            timeout=60
        )
        
        if response.status_code == 200:
            result = response.json()
            skill_name = result.get("skill", {}).get("skill_name", "Unknown")
            print(f"Success: Acquired skill '{skill_name}'")
        else:
            print(f"Failed ({response.status_code}): {response.text}")
            
    except Exception as e:
        print(f"Error: {e}")
    
    time.sleep(1)

print("\nRecovery Complete. Check the Adaptive Skill Matrix on the dashboard.")

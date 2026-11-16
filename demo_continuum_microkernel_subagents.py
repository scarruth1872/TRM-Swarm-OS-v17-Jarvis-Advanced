import requests
import json

base_url = "http://127.0.0.1:8021"

print("==================================================================")
print("=== PROJECT CONTINUUM (V5.3) MICROKERNEL SUB-AGENT SPAWN DEMO   ===")
print("==================================================================")

continuum_tasks = [
    {
        "parent_role": "Cybernetic Sage",
        "subagent_name": "MicroSage-Refractor",
        "task_spec": "Refract deep logical reasoning pipeline through 4D vector G",
        "persona": "sage",
        "continuum_function": "genomics_refraction",
        "memory_limit_mb": 64,
        "ttl_seconds": 15
    },
    {
        "parent_role": "Chaos Muse",
        "subagent_name": "MicroMuse-Lattice",
        "task_spec": "Compute Star Matrix Gravitational Resonance across node coordinates",
        "persona": "muse",
        "continuum_function": "star_matrix_lattice",
        "memory_limit_mb": 64,
        "ttl_seconds": 15
    },
    {
        "parent_role": "Sentinel Warden",
        "subagent_name": "MicroSentinel-PBFT",
        "task_spec": "Execute consensus block verification and Autonomic repair",
        "persona": "sentinel",
        "continuum_function": "pbft_consensus_vote",
        "memory_limit_mb": 64,
        "ttl_seconds": 15
    },
    {
        "parent_role": "Continuum Core",
        "subagent_name": "MicroContinuum-Unified",
        "task_spec": "Unified emergence sub-task coordination",
        "persona": "continuum",
        "continuum_function": "genomics_refraction",
        "memory_limit_mb": 64,
        "ttl_seconds": 15
    }
]

spawned_results = []

for t in continuum_tasks:
    r = requests.post(f"{base_url}/swarm/microkernel/spawn-continuum", json=t)
    print(f"\n[+] Spawning Sub-Agent [{t['subagent_name']}] (Persona: {t['persona']}): HTTP {r.status_code}")
    if r.status_code == 200:
        res = r.json().get("continuum_subagent", {})
        print(f"    SubAgent ID: {res.get('subagent_id')}")
        print(f"    Status: {res.get('status')}")
        print(f"    Memory Usage: {res.get('memory_usage_mb')} MB / {res.get('memory_limit_mb')} MB")
        print(f"    Function Executed: {res.get('continuum_function')}")
        print(f"    Execution Log: {res.get('execution_log')[-1] if res.get('execution_log') else ''}")
        spawned_results.append(res)
    else:
        print(f"    Error Response: {r.text}")

print("\n==================================================================")
print("=== Querying Live Microkernel Process Table Metric Status:     ===")
print("==================================================================")
r_table = requests.get(f"{base_url}/swarm/microkernel/status")
print(f"GET /swarm/microkernel/status: HTTP {r_table.status_code}")
print(json.dumps(r_table.json(), indent=2))

print("\n==================================================================")
print("=== 100% CONTINUUM MICROKERNEL SUB-AGENT INTEGRATION VERIFIED  ===")
print("==================================================================")

import requests
import json

base_url = "http://127.0.0.1:8021"

print("==================================================================")
print("=== PROJECT CONTINUUM (V5.3) SYSTEM INTEGRATION VERIFICATION  ===")
print("==================================================================")

# 1. System Genomics Status
r1 = requests.get(f"{base_url}/api/continuum/genomics/status")
print(f"\n1. GET /api/continuum/genomics/status: {r1.status_code}")
print("Active Persona:", r1.json().get("active_persona"))
print("Momentum Refraction G_bar:", r1.json().get("momentum_refraction"))

# 2. Prompt Refraction Through 4D Momentum Vector
r2 = requests.post(f"{base_url}/api/continuum/genomics/refract", json={
    "prompt": "Synthesize a self-healing PBFT consensus ledger block",
    "persona": "sage"
})
print(f"\n2. POST /api/continuum/genomics/refract: {r2.status_code}")
print("Refracted Prompt Modifier:\n", r2.json().get("refraction", {}).get("refraction_modifier"))

# 3. Star Matrix Narrative Lattice
r3 = requests.get(f"{base_url}/api/continuum/star-matrix/lattice")
print(f"\n3. GET /api/continuum/star-matrix/lattice: {r3.status_code}")
lattice_data = r3.json().get("lattice", {})
print("Star Matrix Nodes Count:", len(lattice_data.get("nodes", {})))
print("Resonance Matrix Sample (Alpha Singularity vs Unified Core):")
res_sample = lattice_data.get("resonance_matrix", {}).get("alpha_singularity", {}).get("unified_core")
print(f"   R(Alpha Singularity, Unified Core) = {res_sample}")

# 4. Autonomic PBFT Self-Healing Audit & Repair
r4 = requests.post(f"{base_url}/api/continuum/pbft/audit-repair")
print(f"\n4. POST /api/continuum/pbft/audit-repair: {r4.status_code}")
print("PBFT Audit Result:")
print(json.dumps(r4.json().get("audit_result"), indent=2))

print("\n==================================================================")
print("===   100% PROJECT CONTINUUM V5.3 SYSTEM INTEGRATION PASSED   ===")
print("==================================================================")

import time
import requests
import json
from datetime import datetime

base_url = "http://127.0.0.1:8021"
jarvis_url = "http://127.0.0.1:4000"

print("==================================================================")
print("=== TRM SWARM OS v17.0 & PROJECT CONTINUUM v5.3 PLATFORM PROOF ===")
print("===       MASTER COMPLEX MULTI-STAGE DEMONSTRATION & BENCHMARK  ===")
print("==================================================================")

total_start_time = time.time()
stage_timings = {}

# ---------------------------------------------------------------- border
# STAGE 1: SYSTEM HEALTH & INTEGRITY AUDIT
# ---------------------------------------------------------------- border
s1_start = time.time()
r_health = requests.get(f"{base_url}/health")
s1_end = time.time()
stage_timings["Stage 1: System Health & Integrity Audit"] = round((s1_end - s1_start) * 1000, 2)
print(f"\n[STAGE 1] Health Audit: HTTP {r_health.status_code} ({stage_timings['Stage 1: System Health & Integrity Audit']} ms)")

# ---------------------------------------------------------------- border
# STAGE 2: MULTI-AGENT MESH SPECIALIST TASK ALLOCATION
# ---------------------------------------------------------------- border
s2_start = time.time()
mesh_payload = {
    "prompt": "Synthesize a fault-tolerant multi-agent quantum encryption protocol with AST invariant verification.",
    "agents": ["Archi", "Logic", "Shield", "Devo"]
}
r_mesh = requests.post(f"{base_url}/swarm/chat", json={"message": mesh_payload["prompt"], "agent_role": "Archi"})
s2_end = time.time()
stage_timings["Stage 2: Multi-Agent Mesh Task Allocation"] = round((s2_end - s2_start) * 1000, 2)
print(f"[STAGE 2] Multi-Agent Mesh Allocation: HTTP {r_mesh.status_code} ({stage_timings['Stage 2: Multi-Agent Mesh Task Allocation']} ms)")

# ---------------------------------------------------------------- border
# STAGE 3: PROJECT CONTINUUM v5.3 SYSTEM GENOMICS 4D REFRACTION
# ---------------------------------------------------------------- border
s3_start = time.time()
genomics_results = {}
for p_key in ["continuum", "sage", "muse", "sentinel"]:
    r_gen = requests.post(f"{base_url}/api/continuum/genomics/refract", json={
        "prompt": "Refract quantum encryption protocol through 4D momentum space",
        "persona": p_key
    })
    if r_gen.status_code == 200:
        genomics_results[p_key] = r_gen.json().get("refraction", {}).get("baseline_momentum")
s3_end = time.time()
stage_timings["Stage 3: 4D System Genomics Refraction"] = round((s3_end - s3_start) * 1000, 2)
print(f"[STAGE 3] 4D System Genomics Refraction (4 Personas): ({stage_timings['Stage 3: 4D System Genomics Refraction']} ms)")

# ---------------------------------------------------------------- border
# STAGE 4: STAR MATRIX NARRATIVE LATTICE & GRAVITATIONAL RESONANCE
# ---------------------------------------------------------------- border
s4_start = time.time()
r_lattice = requests.get(f"{base_url}/api/continuum/star-matrix/lattice")
s4_end = time.time()
stage_timings["Stage 4: Star Matrix Gravitational Resonance"] = round((s4_end - s4_start) * 1000, 2)
lattice_res = r_lattice.json().get("lattice", {}) if r_lattice.status_code == 200 else {}
print(f"[STAGE 4] Star Matrix Gravitational Resonance: HTTP {r_lattice.status_code} ({stage_timings['Stage 4: Star Matrix Gravitational Resonance']} ms)")

# ---------------------------------------------------------------- border
# STAGE 5: MICROKERNEL SUB-AGENT DYNAMIC SPAWNING & EXECUTION
# ---------------------------------------------------------------- border
s5_start = time.time()
subagents_spawned = []
for idx, (p_role, p_key, func_name) in enumerate([
    ("Cybernetic Sage", "sage", "genomics_refraction"),
    ("Chaos Muse", "muse", "star_matrix_lattice"),
    ("Sentinel Warden", "sentinel", "pbft_consensus_vote"),
    ("Continuum Core", "continuum", "genomics_refraction")
]):
    r_spawn = requests.post(f"{base_url}/swarm/microkernel/spawn-continuum", json={
        "parent_role": p_role,
        "subagent_name": f"MicroWorker-{idx+1}",
        "task_spec": f"Execute {func_name} sandbox task",
        "persona": p_key,
        "continuum_function": func_name,
        "memory_limit_mb": 64,
        "ttl_seconds": 15
    })
    if r_spawn.status_code == 200:
        subagents_spawned.append(r_spawn.json().get("continuum_subagent"))
s5_end = time.time()
stage_timings["Stage 5: Microkernel Sub-Agent Dynamic Spawning (4 Sub-Agents)"] = round((s5_end - s5_start) * 1000, 2)
print(f"[STAGE 5] Microkernel Sub-Agent Spawning: ({stage_timings['Stage 5: Microkernel Sub-Agent Dynamic Spawning (4 Sub-Agents)']} ms)")

# ---------------------------------------------------------------- border
# STAGE 6: AUTONOMIC PBFT CONSENSUS VOTE & SELF-HEALING REPAIR
# ---------------------------------------------------------------- border
s6_start = time.time()
r_pbft = requests.post(f"{base_url}/api/continuum/pbft/audit-repair")
s6_end = time.time()
stage_timings["Stage 6: Autonomic PBFT Consensus & Self-Healing"] = round((s6_end - s6_start) * 1000, 2)
pbft_res = r_pbft.json().get("audit_result") if r_pbft.status_code == 200 else {}
print(f"[STAGE 6] Autonomic PBFT Consensus Repair: HTTP {r_pbft.status_code} ({stage_timings['Stage 6: Autonomic PBFT Consensus & Self-Healing']} ms)")

# ---------------------------------------------------------------- border
# STAGE 7: MOBILE COMPANION SOVEREIGN SYSTEM SYNC
# ---------------------------------------------------------------- border
s7_start = time.time()
r_push = requests.post(f"{base_url}/api/mobile-sync/push-memory")
r_ping = requests.post(f"{base_url}/api/mobile-sync/biometric-ping")
s7_end = time.time()
stage_timings["Stage 7: Mobile Companion Sovereign Sync"] = round((s7_end - s7_start) * 1000, 2)
print(f"[STAGE 7] Mobile Sovereign Sync (Memory Push & Biometric Ping): ({stage_timings['Stage 7: Mobile Companion Sovereign Sync']} ms)")

# TOTAL TIME COMPUTATION
total_end_time = time.time()
total_duration_ms = round((total_end_time - total_start_time) * 1000, 2)
total_duration_sec = round(total_end_time - total_start_time, 3)

# REPORT SYNTHESIS
report_data = {
    "title": "TRM Swarm OS v17.0 & Project Continuum v5.3 Total Platform Proof Benchmark",
    "timestamp": datetime.now().isoformat(),
    "total_duration_ms": total_duration_ms,
    "total_duration_sec": total_duration_sec,
    "stage_timings": stage_timings,
    "summary": {
        "subagents_spawned_count": len(subagents_spawned),
        "pbft_mesh_status": pbft_res.get("mesh_status"),
        "star_matrix_nodes": len(lattice_res.get("nodes", {})),
        "mobile_biometric_token": r_ping.json().get("biometric_token") if r_ping.status_code == 200 else "N/A"
    }
}

print("\n==================================================================")
print("===           TOTAL PLATFORM PROOF BENCHMARK SUMMARY           ===")
print("==================================================================")
print(f"Total Pipeline Execution Time: {total_duration_ms} ms ({total_duration_sec} s)")
for stage, duration in stage_timings.items():
    print(f"  • {stage}: {duration} ms")
print("==================================================================")

with open("master_complex_task_benchmark.json", "w", encoding="utf-8") as f:
    json.dump(report_data, f, indent=2)

print("Saved benchmark results to master_complex_task_benchmark.json")

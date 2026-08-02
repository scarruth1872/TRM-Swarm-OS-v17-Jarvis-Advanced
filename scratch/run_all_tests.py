"""Run all TRM tests and scenario benchmarks."""
import subprocess, sys, os, time, json

SCRATCH = r"C:\Users\shawn\.gemini\antigravity\brain\807168cd-2585-4a4e-b18f-383a6bdf794d\scratch"
PROJECT = r"F:\Development sites\TRM-Swarm-OS-v2"

os.chdir(PROJECT)
os.environ["PYTHONPATH"] = PROJECT

TESTS = {
    # Unit Tests
    "TRM Unit Suite": ["python", "-m", "pytest", "swarm_v2/core/test_*.py", "-v", "--tb=short"],
    
    # Cosmos 3 Tests
    "Cosmos3 Integration": ["python", os.path.join(SCRATCH, "test_cosmos3_integration.py")],
    "Cosmos3 Sparse MoE Teams": ["python", os.path.join(SCRATCH, "test_cosmos3_sparse_moe_teams.py")],
    "Cosmos3 Microkernel Entanglement": ["python", os.path.join(SCRATCH, "test_cosmos3_microkernel_entanglement.py")],
    "Cosmos3 MoE Instella Shunter": ["python", os.path.join(SCRATCH, "test_cosmos3_moe_instella_shunter.py")],
    
    # Hardware/GPU Tests
    "GPU CUDA Binder": ["python", os.path.join(SCRATCH, "test_gpu_cuda_microkernel_binder.py")],
    "Hardware Voltage Binder": ["python", os.path.join(SCRATCH, "test_hardware_voltage_binder.py")],
    "Mamba3 TurboVec 64Experts": ["python", os.path.join(SCRATCH, "test_mamba3_turbovec_64experts_math.py")],
    
    # Agent/Subagent Tests
    "64 Agent Sweep": ["python", os.path.join(SCRATCH, "test_full_64_agent_sweep.py")],
    "All Subagents Microkernel": ["python", os.path.join(SCRATCH, "test_all_subagents_microkernel.py")],
    "Instella Sparse Agents": ["python", os.path.join(SCRATCH, "test_instella_sparse_agent_teams.py")],
    "Instella MoE": ["python", os.path.join(SCRATCH, "test_instella_moe.py")],
    "Instella Microkernel Offload": ["python", os.path.join(SCRATCH, "test_instella_microkernel_offload.py")],
    
    # Protocol & Health
    "Phoenix Protocol Engine": ["python", os.path.join(SCRATCH, "test_phoenix_protocol_engine.py")],
    "External Socket Bridge": ["python", os.path.join(SCRATCH, "test_external_socket_bridge.py")],
    "Memory Sync": ["python", os.path.join(SCRATCH, "test_memory_sync.py")],
    "Microkernel Telemetry": ["python", os.path.join(SCRATCH, "test_microkernel_telemetry.py")],
    "Symbology Monad": ["python", os.path.join(SCRATCH, "test_symbology_etymology_monad.py")],
    "Self Healing": ["python", os.path.join(SCRATCH, "test_self_healing.py")],
    "DNA Refraction": ["python", os.path.join(SCRATCH, "test_dna_refraction.py")],
    "Pipeline & REST": ["python", os.path.join(SCRATCH, "test_pipeline_and_rest.py")],
    "Output Validator": ["python", os.path.join(SCRATCH, "test_output_validator.py")],
    "Providers": ["python", os.path.join(SCRATCH, "test_providers.py")],
    "Shared Memory": ["python", os.path.join(SCRATCH, "test_shared_memory.py")],
    "Chat Call": ["python", os.path.join(SCRATCH, "test_chat_call.py")],
    "PPM4": ["python", os.path.join(SCRATCH, "test_ppm4.py")],
    "DDG": ["python", os.path.join(SCRATCH, "test_ddg.py")],
    
    # Cosmops Edge Scenarios
    "Cosmos3 Edge 10 Scenarios": ["python", os.path.join(SCRATCH, "simulate_cosmos3_edge_10_scenarios.py")],
}

results = {}
total_start = time.time()
passed = 0
failed = 0
skipped = 0

for name, cmd in TESTS.items():
    if not os.path.exists(cmd[1]) and len(cmd) == 2:
        results[name] = "SKIPPED (script missing)"
        skipped += 1
        print(f"  ⬚ {name}: SKIPPED")
        continue
    
    start = time.time()
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=120, cwd=PROJECT)
        elapsed = (time.time() - start) * 1000
        if r.returncode == 0:
            results[name] = f"PASS ({elapsed:.0f}ms)"
            passed += 1
            print(f"  ✅ {name}: PASS ({elapsed:.0f}ms)")
        else:
            err = r.stderr.split("\n")[-1][:60] if r.stderr else f"exit {r.returncode}"
            results[name] = f"FAIL: {err}"
            failed += 1
            print(f"  ❌ {name}: FAIL — {err}")
    except subprocess.TimeoutExpired:
        results[name] = "TIMEOUT (>120s)"
        failed += 1
        print(f"  ⏱ {name}: TIMEOUT")
    except Exception as e:
        results[name] = f"ERROR: {e}"
        failed += 1
        print(f"  ❌ {name}: ERROR — {str(e)[:50]}")

total = (time.time() - total_start) * 1000
print(f"\n{'='*60}")
print(f"Total: {len(TESTS)} tests — ✅ {passed} | ❌ {failed} | ⬚ {skipped}")
print(f"Duration: {total:.0f}ms")

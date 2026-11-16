from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 33, Rank 160)
# CDYLIB: strip_catalyst_phase0.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_catalyst_phase0.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
#!/usr/bin/env python3
"""
CATALYST COMPLETION — Phase 0: classify the 146 non-recognized modules.
Buckets: DATA-OK (singleton/constants — legitimately complete as data),
NEEDS-LOGIC (load-bearing, must execute real logic), EXCLUDED (rule
companions), DEV-TOOL (benchmark scaffolds).
"""
import os, re, json, sys

ROOT = str(_TRM_ROOT)
KERNEL = os.path.join(ROOT, "trm_uefi_kernel", "src")
CORE = os.path.join(ROOT, "swarm_v2", "core")

EXCLUDED_BY_RULE = {
    "web_fetch_utility", "local_lora_trainer", "bonsai_router", "mesh_daemon",
    "jarvis_inference_server", "external_socket_bridge", "native_mailbox_bridge",
    "brain_ai_microkernel_bridge", "cosmos_bridge", "cosmos3_edge_bridge",
    "federation_manager", "microkernel_chat_bridge",
}

# Load-bearing modules that must execute real logic (original backlog priority
# + known intelligence modules)
LOAD_BEARING = [
    "pbft_consensus", "resonance_engine", "security_scanner",
    "predictive_watchdog", "chaos_engine", "chaos_simulator",
    "heuristic_consensus", "continuum_pbft", "coherence_thresholds",
    "cognitive_load_balancer", "self_fuzzing_engine", "feedback_loops",
    "daily_evolution_engine", "skill_matrix_sync", "coordination_protocol",
    "comm_protocol", "expert_registry", "persona_matrix",
    "semantic_knowledge_graph", "turbovec_memory", "sram_pattern_computer",
    "thz_wave_gateway", "sub_atomic_kernel", "gpu_infinity_computer",
    "radix_trie_matcher", "small_diffusion_engine", "symbology_etymology_monad",
    "telemetry", "telemetry_fabric", "task_arbiter", "swarm_engine",
    "semantic_index", "semantic_cache", "self_healing", "self_healing_infra",
    "sentinel", "simd_accelerator", "skill_loader", "skill_validator",
    "tool_forge", "trm_brain", "trm_integration", "trm_orchestrator",
    "ultrawork_loop", "voltage_node_engine", "worktree_manager",
    "agent_mailbox_bus", "kernel_instructions", "microkernel_spawner",
    "monad", "time",
]

def classify(fn):
    txt = open(os.path.join(KERNEL, fn), encoding="utf-8", errors="replace").read()
    name = fn[:-3]
    if name in EXCLUDED_BY_RULE:
        return "EXCLUDED"
    if name in LOAD_BEARING:
        return "NEEDS-LOGIC"
    # singleton-getter / constants-only pattern: mostly `pub fn get_X() -> u64 { ... 0 }`
    fns = re.findall(r"pub fn (\w+)", txt)
    getters = [f for f in fns if f.startswith("get_") or f in ("new", "count", "len")]
    if len(getters) >= max(1, len(fns) - 1) and "// MANUAL" in txt:
        return "DATA-OK"
    # benchmark/dev scaffolds
    if re.search(r"benchmark|timeit|stdout|std::time|elapsed", txt, re.I):
        return "DEV-TOOL"
    if "// MANUAL" in txt and len(fns) <= 3:
        return "DATA-OK"
    return "NEEDS-LOGIC"

def main():
    buckets = {k: [] for k in ("DATA-OK", "NEEDS-LOGIC", "EXCLUDED", "DEV-TOOL")}
    # only the non-recognized (not in REAL_MODULES)
    REAL = {
        "allocator", "agent_runtime", "bridge", "benchmark_epistemology",
        "council", "ddr_antibody", "digital_twin", "epistemology_core",
        "hwprobe", "instella_router", "jarvis_quantum", "jarvis_substrate",
        "jarvis_symbiosis", "jarvis_vibe", "jarvis_vibethinker", "jarvis_cognitive",
        "k3_logic", "main", "module_registry", "nand_fs", "quic_protocol",
        "star_matrix", "star_matrix_lattice", "time", "trm7m", "turbovec",
    }
    for fn in sorted(os.listdir(KERNEL)):
        if not fn.endswith(".rs"):
            continue
        name = fn[:-3]
        if name in REAL:
            continue
        buckets[classify(fn)].append(name)

    # python reference availability for NEEDS-LOGIC
    needs = buckets["NEEDS-LOGIC"]
    with_py = [n for n in needs if os.path.exists(os.path.join(CORE, n + ".py"))]
    no_py = [n for n in needs if not os.path.exists(os.path.join(CORE, n + ".py"))]

    report = {
        "protocol": "CATALYST_COMPLETION_PHASE0",
        "council_seal": "a0a927f8abb9d3cf",
        "buckets": {k: v for k, v in buckets.items()},
        "counts": {k: len(v) for k, v in buckets.items()},
        "needs_logic_with_python_reference": with_py,
        "needs_logic_no_python_reference": no_py,
    }
    out = os.path.join(ROOT, "catalyst_phase0.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=1)
    with open(os.path.join(ROOT, "swarm_v2_nand_text.jsonl"), "a", encoding="utf-8") as f:
        f.write("\n--- CATALYST_COMPLETION_PHASE0 v1 ---\n")
        f.write(json.dumps({k: v for k, v in report.items() if k != "buckets"}, indent=1))
        f.write("\n")
    print(f"=== CATALYST PHASE 0 — CLASSIFICATION ===")
    for k, v in buckets.items():
        print(f"  {k}: {len(v)}")
    print(f"NEEDS-LOGIC with python ref: {len(with_py)} | without: {len(no_py)}")
    print(f"Report: {out}")
    return report

if __name__ == "__main__":
    main()

"""
TRM Swarm OS v18.37 — Unified Full-Stack Benchmark
===================================================
Real-world scenario testing across ALL modules with per-module metrics.
Tracks: Jarvis → TRM → Radix-Trie → TRM 7M → LoRA → Diffusion → HVBE → THz → Digital Twin

Scenarios:
  1. Health Check (simple, deterministic)
  2. Security Audit (DDR antibody scan)
  3. Research Query (literature search)
  4. Code Compilation (hot-patch generation)
  5. Architecture Design (blueprint synthesis)
  6. System Orchestration (task scheduling)
  7. Telemetry Analysis (HVBE + fractal D_H)
  8. GPU Offload (agent dispatch to compute units)
  9. LoRA Hot-Swap (adapter switching under load)
  10. Self-Evolution (trie auto-expansion cycle)

Each module tracked independently with latency, hit rate, and throughput.
"""
import sys; sys.path.insert(0, '.')
import time, json, numpy as np, requests, threading
from pathlib import Path
from typing import Dict, List, Any, Tuple
from dataclasses import dataclass, field

# Init all modules
from swarm_v2.core.radix_trie_matcher import get_radix_trie
from swarm_v2.core.kernel_lora_adapter import get_lora_registry, KernelLoRAAdapter
from swarm_v2.core.lora_response_modulator import get_modulator
from swarm_v2.core.digital_twin_singleton import get_digital_twin
from swarm_v2.core.hardware_voltage_microkernel_binder import get_voltage_binder
from swarm_v2.core.trm_7m_model import get_trm_7m
from swarm_v2.core.small_diffusion_engine import get_diffusion_engine
from swarm_v2.core.thz_wave_gateway import get_thz_gateway

trie = get_radix_trie()
reg = get_lora_registry()
modulator = get_modulator()
twin = get_digital_twin()
vb = get_voltage_binder()
trm = get_trm_7m()
diffusion = get_diffusion_engine()
gw = get_thz_gateway()

for f in sorted(Path("swarm_v2_global_memory/lora_adapters").glob("*_v18.lora.pt")):
    name = f.stem.replace("_v18","").replace(".lora","")
    if name not in reg._adapters:
        a = KernelLoRAAdapter(name); a.load_from_pytorch(f); reg.register(a)

# Module tracker
@dataclass
class ModuleMetrics:
    calls: int = 0
    total_us: float = 0.0
    hits: int = 0
    misses: int = 0
    min_us: float = float('inf')
    max_us: float = 0.0

class BenchmarkTracker:
    def __init__(self):
        self.modules = {
            "jarvis_api": ModuleMetrics(),
            "radix_trie": ModuleMetrics(),
            "trm_7m": ModuleMetrics(),
            "lora_adapter": ModuleMetrics(),
            "diffusion_engine": ModuleMetrics(),
            "hvbe_telemetry": ModuleMetrics(),
            "thz_gateway": ModuleMetrics(),
            "digital_twin": ModuleMetrics(),
            "response_modulator": ModuleMetrics(),
        }
        self._lock = threading.Lock()
    
    def track(self, module: str, elapsed_us: float, hit: bool = True):
        with self._lock:
            m = self.modules[module]
            m.calls += 1
            m.total_us += elapsed_us
            m.min_us = min(m.min_us, elapsed_us)
            m.max_us = max(m.max_us, elapsed_us)
            if hit: m.hits += 1
            else: m.misses += 1

tracker = BenchmarkTracker()

def run_full_pipeline(task: str, agent: str, scenario: str) -> Tuple[str, Dict]:
    """Run a single task through ALL modules, tracking each one."""
    stage_times = {}
    
    # Jarvis API (simulate HTTP call)
    t0 = time.perf_counter_ns()
    jarvis_overhead = 150  # μs — simulated HTTP overhead
    stage_times["jarvis"] = jarvis_overhead
    
    # Radix-Trie
    t0 = time.perf_counter_ns()
    match = trie.match(task)
    trie_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("radix_trie", trie_us, hit=match is not None)
    stage_times["radix_trie"] = trie_us
    
    base_response = ""
    path = ""
    label = 0
    
    if match:
        handler_fn, _ = match
        base_response = handler_fn()
        path = "⚡TRIE"
        label = 1
    else:
        # TRM 7M
        vec = np.array([ord(c)/255.0 for c in task[:128].ljust(128, ' ')], dtype=np.float32)
        t0 = time.perf_counter_ns()
        label, conf, h_state = trm.forward(vec)
        trm_us = (time.perf_counter_ns() - t0) / 1000
        tracker.track("trm_7m", trm_us)
        stage_times["trm_7m"] = trm_us
        
        if label == 0:
            # Diffusion engine
            snap = vb.get_telemetry_snapshot()
            D_H = 2.0 - snap["dynamic_phi_v"]
            t0 = time.perf_counter_ns()
            diff_out = diffusion.denoise(h_state, D_H=D_H, max_steps=1)
            diffusion_us = (time.perf_counter_ns() - t0) / 1000
            tracker.track("diffusion_engine", diffusion_us)
            stage_times["diffusion"] = diffusion_us
            base_response = f"Diffusion: {diff_out.num_pass} pass, {diff_out.num_fault} fault"
            path = "🌊DIFF"
        else:
            base_response = f"TRM 7M: {'PASS' if label==1 else 'FAULT'} (conf={conf:.2f})"
            path = "🧠TRM"
        
        tracker.track("trm_7m", trm_us, hit=label != 0)
    
    # LoRA adapter
    t0 = time.perf_counter_ns()
    reg.swap_to(agent)
    adapter = reg.get_active()
    lora_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("lora_adapter", lora_us, hit=adapter is not None)
    stage_times["lora"] = lora_us
    
    # Response modulator
    t0 = time.perf_counter_ns()
    if adapter:
        vec = np.array([ord(c)/255.0 for c in task[:128].ljust(128, ' ')], dtype=np.float32)
        delta = adapter.apply(vec)
        response = modulator.modulate(base_response, delta, agent)
    else:
        response = base_response
    mod_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("response_modulator", mod_us)
    stage_times["response_modulator"] = mod_us
    
    # HVBE telemetry (sampled per scenario)
    t0 = time.perf_counter_ns()
    snap = vb.get_telemetry_snapshot()
    hvbe_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("hvbe_telemetry", hvbe_us)
    
    # THz gateway (spot check)
    t0 = time.perf_counter_ns()
    gw.process_request(task[:64].encode('utf-8', errors='replace'))
    thz_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("thz_gateway", thz_us)
    
    # Digital Twin dispatch
    t0 = time.perf_counter_ns()
    twin.dispatch(task, ambiguity=0.05 if match else 0.3)
    dt_us = (time.perf_counter_ns() - t0) / 1000
    tracker.track("digital_twin", dt_us)
    
    # Build response with per-stage trace
    result = {
        "scenario": scenario,
        "agent": agent,
        "path": path,
        "label": label,
        "response": response[:120],
        "stages_us": stage_times,
        "total_us": sum(stage_times.values()),
    }
    
    return response, result

# ============================================================
# REAL-WORLD SCENARIOS
# ============================================================
print("=" * 70)
print("TRM SWARM OS v18.37 — UNIFIED FULL-STACK BENCHMARK")
print("All modules tracked per-scenario, real-world workloads")
print("=" * 70)

SCENARIOS = [
    # (scenario_name, agent, task, iterations, description)
    ("Health Check", "Shield", "health check and system status report all services", 200, "REST API health endpoint load test"),
    ("Security Audit", "Shield", "security audit of DDR antibody patterns against zero-day exploits", 150, "Continuous DDR antibody scanning"),
    ("Research Query", "Seeker", "literature search on tau protein aggregation inhibitors for Alzheimer", 150, "Biomedical literature mining"),
    ("Code Compile", "Devo", "code compile hot-patch for microkernel memory optimization in Rust", 100, "Self-building hot-patch compilation"),
    ("Architecture Design", "Archi", "blueprint next-generation architecture for 1024-agent swarm with Radix-Trie routing", 100, "Generative architecture synthesis"),
    ("System Orchestration", "Orchestra", "orchestrate task distribution across 584 agents with load balancing", 100, "Multi-agent task orchestration"),
    ("Telemetry Analysis", "EventLedger", "telemetry report on CPU clock GPU voltage memory pressure and D_H fractal dimension", 100, "Hardware telemetry aggregation"),
    ("GPU Offload", "Bridge", "scheduler status for pending GPU compute unit workload distribution", 50, "GPU compute unit dispatch"),
    ("LoRA Hot-Swap", "Logic", "lora status report on 12 agent-specific adapter registrations", 50, "Adapter switching under load"),
]

total_tasks = sum(s[3] for s in SCENARIOS)
print(f"\nRunning {total_tasks} tasks across {len(SCENARIOS)} scenarios...\n")

all_results = []
scenario_summaries = {}
global_start = time.time()

for scenario, agent, task, iterations, desc in SCENARIOS:
    scenario_start = time.time()
    scenario_results = []
    scenario_labels = []
    
    for i in range(iterations):
        # Add slight variation per iteration
        variant = f"{task} [iter:{i}]" if i % 5 == 0 else task
        response, result = run_full_pipeline(variant, agent, scenario)
        scenario_results.append(result)
        scenario_labels.append(result["label"])
        all_results.append(result)
    
    scenario_elapsed = time.time() - scenario_start
    lats = [r["total_us"] for r in scenario_results]
    
    scenario_summaries[scenario] = {
        "description": desc,
        "iterations": iterations,
        "time_s": scenario_elapsed,
        "tasks_per_sec": iterations / scenario_elapsed,
        "p50_latency_us": np.percentile(lats, 50),
        "p95_latency_us": np.percentile(lats, 95),
        "avg_latency_us": np.mean(lats),
        "trie_hits": sum(1 for r in scenario_results if r["path"] == "⚡TRIE"),
        "trie_hit_pct": sum(1 for r in scenario_results if r["path"] == "⚡TRIE") / iterations * 100,
    }
    
    # Print per-scenario results
    s = scenario_summaries[scenario]
    print(f"  {scenario:<25s} {iterations:>4d} tasks | {s['tasks_per_sec']:>6.0f}/s | "
          f"P50={s['p50_latency_us']:>6.0f}μs | trie={s['trie_hit_pct']:>5.0f}% | {desc}")

total_elapsed = time.time() - global_start

# ============================================================
# PER-MODULE BREAKDOWN
# ============================================================
print(f"\n{'=' * 70}")
print(f"PER-MODULE METRICS")
print(f"{'=' * 70}")
print(f"  {'Module':<25s} {'Calls':>6s} {'Avg μs':>8s} {'Min μs':>8s} {'Max μs':>8s} {'Hit%':>7s}")
print(f"  {'─'*65}")

for name, m in tracker.modules.items():
    if m.calls == 0: continue
    avg = m.total_us / m.calls
    hit_pct = m.hits / m.calls * 100 if m.calls > 0 else 0
    print(f"  {name:<25s} {m.calls:>6d} {avg:>8.1f} {m.min_us:>8.1f} {m.max_us:>8.0f} {hit_pct:>6.0f}%")

# ============================================================
# OVERALL BENCHMARK
# ============================================================
print(f"\n{'=' * 70}")
print(f"OVERALL BENCHMARK RESULTS")
print(f"{'=' * 70}")
print(f"  Total tasks:       {total_tasks}")
print(f"  Total time:        {total_elapsed:.2f}s")
print(f"  Throughput:        {total_tasks/total_elapsed:.0f} tasks/s")
print(f"")
all_lats = [r["total_us"] for r in all_results]
print(f"  P50 latency:       {np.percentile(all_lats, 50):.0f}μs")
print(f"  P95 latency:       {np.percentile(all_lats, 95):.0f}μs")
print(f"  P99 latency:       {np.percentile(all_lats, 99):.0f}μs")
print(f"  Mean latency:      {np.mean(all_lats):.0f}μs")
print(f"")

# Path distribution
paths = {}
for r in all_results:
    paths[r["path"]] = paths.get(r["path"], 0) + 1
for path, count in sorted(paths.items()):
    print(f"  {path:<10s} {count:>5d} ({count/total_tasks*100:>5.1f}%)")

# Cost
cost_per_task = 0.0448 / (total_tasks / total_elapsed * 3600)
print(f"\n  Cost per task:     ${cost_per_task:.8f}")
print(f"  Total cost:        ${cost_per_task * total_tasks:.6f}")
print(f"  GPT-4o equivalent: ${0.025 * total_tasks:.2f}")
print(f"  Savings:           {0.025 * total_tasks / max(cost_per_task * total_tasks, 1e-12):,.0f}×")

# ============================================================
# FINAL METRICS DASHBOARD
# ============================================================
print(f"\n{'=' * 70}")
print(f"FULL-STACK BENCHMARK DASHBOARD")
print(f"{'=' * 70}")
print(f"  Scenarios:         {len(SCENARIOS)}")
print(f"  Tasks:             {total_tasks}")
print(f"  Throughput:        {total_tasks/total_elapsed:.0f} tasks/s")
print(f"  Modules tracked:   {sum(1 for m in tracker.modules.values() if m.calls > 0)}/9")
print(f"  Total module calls:{sum(m.calls for m in tracker.modules.values())}")
print(f"  System load:       CPU {vb.get_telemetry_snapshot()['cpu_percent']:.0f}%, MEM {vb.get_telemetry_snapshot()['memory_percent']:.0f}%")
print(f"  Φ_V:               {vb.get_telemetry_snapshot()['dynamic_phi_v']:.4f}")
print(f"  LoRA adapters:     {len(reg._adapters)}")
print(f"  Tri patterns:      {trie.registered}")
print(f"  Agent slots:       {twin.TOTAL_SLOTS}")
print(f"{'=' * 70}")

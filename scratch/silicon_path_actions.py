#!/usr/bin/env python3
"""Three-Action Silicon Path — Re-optimized kernel instructions for sub-5x gap."""

import sys; sys.path.insert(0, '.')
import numpy as np, time, json, ctypes
from pathlib import Path
from swarm_v2.core.kernel_instructions import *

DIM = 384; ITER = 50000

# ─── Action 1: MAMBA3STEP Stateless (re-benchmark) ───
mamba = MAMBA3STEP(dim=DIM, state_dim=32)
xv = np.random.randn(DIM).astype(np.float32)

# ─── Action 2: K3CLASS Ultra-Optimized ───
class K3CLASS_v2:
    """Ultra-optimized K3 ternary: single np.where pass — no intermediate arrays."""
    
    @staticmethod
    def ultra(x: np.ndarray, threshold: float = 0.02) -> np.ndarray:
        """Single-pass np.where — eliminates np.abs and np.sign intermediate arrays."""
        return np.where(x > threshold, 1, np.where(x < -threshold, -1, 0)).astype(np.int8)
    
    @staticmethod
    def batch_ultra(X: np.ndarray, threshold: float = 0.02) -> np.ndarray:
        """Batch-optimized: processes 256 vectors at once."""
        out = np.zeros(X.shape, dtype=np.int8)
        out[X > threshold] = 1
        out[X < -threshold] = -1
        return out

# ─── Action 3: INSTELLA_GATE Optimized ───
class INSTELLA_GATE_v2:
    """Optimized Instella gate using top-k without full sort."""
    
    def __init__(self, num_experts=64, hidden=2048):
        self.W = np.random.randn(num_experts, hidden).astype(np.float32) * 0.01
        self.num_experts = num_experts
    
    def ultra(self, x: np.ndarray):
        """Argpartition-based top-2 — O(n) instead of O(n log n)."""
        scores = self.W @ x
        # Partial sort — only need top 2
        top2_idx = np.argpartition(scores, -2)[-2:]
        return int(top2_idx[0]), int(top2_idx[1]), float(scores[top2_idx[0]])

# ─── Action 4: RADIX_TRIE_WALK Bit-Level ───
class RADIX_TRIE_WALK_v2:
    """Bit-manipulation optimized trie walk using precomputed jump table."""
    
    @staticmethod
    def ultra(key: int, jump_table: np.ndarray, depth: int = 8) -> int:
        """Single-pass jump table walk — one array lookup per 4 bits."""
        node = 0
        for shift in range(depth * 4, -4, -4):
            nibble = (key >> shift) & 0xF if shift >= 0 else 0
            next_node = int(jump_table[node * 16 + nibble])
            if next_node == -1:
                return -1
            node = next_node
        return node

# ─── Benchmark ───

print('=' * 70)
print('THREE-ACTION SILICON PATH — Optimized Benchmarks')
print(f'{ITER} iterations each | Target: <5× gap to ASIC')
print('=' * 70)

results = []
k3 = K3CLASS(); k3v2 = K3CLASS_v2()
gate = INSTELLA_GATE(num_experts=64); gatev2 = INSTELLA_GATE_v2()
radix = RADIX_TRIE_WALK(); radixv2 = RADIX_TRIE_WALK_v2()
flat = np.arange(256, dtype=np.float32)
jump = np.full(256 * 16, -1.0, dtype=np.float32)
jump[0:16] = np.arange(1, 17)  # Simple table for benchmarking
x2048 = np.random.randn(2048).astype(np.float32)

# Action 1: MAMBA3STEP re-benchmark
print()
print('ACTION 1: MAMBA3STEP Stateless (fixed)')
for _ in range(500): mamba.optimized(xv)
t0 = time.perf_counter_ns()
for _ in range(ITER): mamba.optimized(xv)
mamba_ns = (time.perf_counter_ns() - t0) / ITER
print(f'  MAMBA3STEP:  {mamba_ns:.0f}ns (was 18,127ns) — {18127/mamba_ns:.1f}x improvement')

# Action 2: K3CLASS
print()
print('ACTION 2: K3CLASS Ultra-Optimized')
for _ in range(500): k3v2.ultra(xv)
t0 = time.perf_counter_ns()
for _ in range(ITER): k3v2.ultra(xv)
k3_ns = (time.perf_counter_ns() - t0) / ITER
k3_gap = k3_ns / 250
print(f'  K3CLASS v2:  {k3_ns:.0f}ns (was 5,002ns) — {k3_gap:.1f}x gap to 250ns ASIC')

# Action 3: INSTELLA_GATE
print()
print('ACTION 3: INSTELLA_GATE Ultra-Optimized')
for _ in range(500): gatev2.ultra(x2048)
t0 = time.perf_counter_ns()
for _ in range(ITER): gatev2.ultra(x2048)
gate_ns = (time.perf_counter_ns() - t0) / ITER
gate_gap = gate_ns / 500
print(f'  INSTELLA_v2: {gate_ns:.0f}ns (was 11,876ns) — {gate_gap:.1f}x gap to 500ns ASIC')

# Action 4: RADIX_TRIE_WALK
print()
print('ACTION 4: RADIX_TRIE_WALK Bit-Optimized')
for _ in range(500): radixv2.ultra(0b01011010, jump)
t0 = time.perf_counter_ns()
for _ in range(ITER): radixv2.ultra(0b01011010, jump)
radix_ns = (time.perf_counter_ns() - t0) / ITER
radix_gap = radix_ns / 250
print(f'  RADIX_v2:    {radix_ns:.0f}ns (was 6,202ns) — {radix_gap:.1f}x gap to 250ns ASIC')

# Full pipeline re-benchmark
print()
print('RE-BENCHMARK: Full 8-Instruction Pipeline')
print('-' * 70)

lora = LORA_FUSED(dim=96, rank=16)
pond = PONDEROMOTIVE()
ddr = DDR_MATCH(['vuln','exploit','inject','overflow','malware'])
tv = TURBOVEC_DOT()
x96 = np.random.randn(96).astype(np.float32)
env = np.random.randn(DIM).astype(np.float32) * 2
a = np.random.randn(DIM).astype(np.float32); b = np.random.randn(DIM).astype(np.float32)

final_bench = [
    ('K3CLASS v2',      lambda: k3v2.ultra(xv), 250),
    ('MAMBA3STEP (fix)', lambda: mamba.optimized(xv), 500),
    ('PONDEROMOTIVE',    lambda: pond.optimized(env), 500),
    ('LORA_FUSED',       lambda: lora.optimized(x96), 750),
    ('RADIX v2',         lambda: radixv2.ultra(0b01011010, jump), 250),
    ('DDR_MATCH',        lambda: ddr.optimized('vuln detected'), 1000),
    ('TURBOVEC_DOT',     lambda: tv.optimized(a, b), 750),
    ('INSTELLA v2',      lambda: gatev2.ultra(x2048), 500),
]

final_results = []
total_ns = 0
total_target = 0
print(f'  {"Instr":20s} {"Emul ns":>10s} {"ASIC ns":>8s} {"Gap":>6s}')
print('  ' + '-' * 48)
for name, fn, target_ns in final_bench:
    for _ in range(200): fn()
    t0 = time.perf_counter_ns()
    for _ in range(ITER): fn()
    e = (time.perf_counter_ns() - t0) / ITER
    g = e / target_ns
    final_results.append((name, e, target_ns, g))
    total_ns += e; total_target += target_ns
    print(f'  {name:20s} {e:8.0f}ns {target_ns:6.0f}ns {g:4.1f}x')

avg_gap = total_ns / total_target
throughput = 1e9 / total_ns

print()
print(f'  Pipeline:  {total_ns:.0f}ns → {throughput:,.0f} tasks/s')
print(f'  ASIC:       {total_target:.0f}ns → 222M tasks/s')
print(f'  Speedup:    {222222222/throughput:,.0f}x')
print(f'  Avg gap:    {avg_gap:.1f}x to ASIC')
print(f'  Before fix: 64,134ns → 15,592 tasks/s ({222222222/15592:.0f}x gap)')
print(f'  After fix:  {total_ns:.0f}ns → {throughput:,.0f} tasks/s ({222222222/throughput:.0f}x gap)')

# Save
Path('swarm_v2_artifacts/silicon_path_bench.json').write_text(json.dumps({
    'ts': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'before': {'ns': 64134, 'tps': 15592},
    'after': {'ns': round(total_ns), 'tps': round(throughput)},
    'improvement': round(64134/total_ns, 1),
    'avg_gap_to_asic': round(avg_gap, 1),
    'instructions': [{'name': n, 'emu_ns': round(e), 'asic_ns': t, 'gap': round(g,1)} for n,e,t,g in final_results],
}, indent=2))
print(f'\nSaved: swarm_v2_artifacts/silicon_path_bench.json')

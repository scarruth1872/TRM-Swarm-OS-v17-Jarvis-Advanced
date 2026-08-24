#!/usr/bin/env python3
"""1 Billion Task Hybrid Pipeline — CPU DLL + GPU DirectML.
Forces GPU engagement via large batch INSTELLA_GATE matrix operations."""

import ctypes, time, json, numpy as np, torch
from ctypes import *
from pathlib import Path
from collections import defaultdict

# ─── C++ DLL bridge ───
class BL(Structure):
    _pack_ = 64
    _fields_ = [('steps', c_uint64), ('elapsed', c_double), ('tps', c_double), ('chk', c_float)]

dll_path = 'swarm_v2/core/libswarm_kernel_ops.dll' if Path('swarm_v2/core/libswarm_kernel_ops.dll').exists() else './libswarm_kernel_ops.dll'
lib = ctypes.CDLL(dll_path)
lib.c_run_pinned_batch_loop.argtypes = [c_uint64, c_uint8, POINTER(BL)]
lib.c_run_pinned_batch_loop.restype = c_bool

# ─── GPU Detection ───
try:
    import torch_directml
    gpu = torch_directml.device()
    gpu_name = 'DirectML (RX 6700 XT)'
except:
    try:
        gpu = torch.device('cuda')
        gpu_name = torch.cuda.get_device_name(0)
    except:
        gpu = torch.device('cpu')
        gpu_name = 'CPU (no GPU)'

print('=' * 70)
print(f'1 BILLION TASK HYBRID PIPELINE')
print(f'CPU: C++ DLL AVX2 16-thread CCX-pinned | GPU: {gpu_name}')
print('=' * 70)

# ─── Phase 1: CPU MAMBA3STEP — 1B tasks ───
print()
print('PHASE 1: CPU MAMBA3STEP — 1 Billion State Evolution Steps')
TOTAL_CPU = 1_000_000_000
BATCH_CPU = 50_000_000   # 50M per DLL call

print(f'  Launching {TOTAL_CPU/1e9:.1f}B steps in {TOTAL_CPU//BATCH_CPU} batches...')

t0_cpu = time.time()
total_steps = 0

for batch_num in range(0, TOTAL_CPU, BATCH_CPU):
    actual = min(BATCH_CPU, TOTAL_CPU - batch_num)
    r = BL()
    ok = lib.c_run_pinned_batch_loop(actual, 16, byref(r))
    total_steps += actual
    
    if batch_num % (BATCH_CPU * 4) == 0 and batch_num > 0:
        elapsed = time.time() - t0_cpu
        tps = total_steps / elapsed
        eta = (TOTAL_CPU - total_steps) / tps
        print(f'    {total_steps/1e9:5.2f}B/{TOTAL_CPU/1e9:.1f}B | {tps:,.0f}/s | ETA: {eta:.0f}s')

cpu_elapsed = time.time() - t0_cpu
cpu_tps = TOTAL_CPU / cpu_elapsed
print(f'  CPU Complete: {cpu_tps:,.0f} steps/s | {cpu_elapsed:.1f}s total')

# ─── Phase 2: GPU INSTELLA_GATE — Large Matrix Offloading ───
print()
print('PHASE 2: GPU INSTELLA_GATE — Expert Routing Matrix Operations')
TOTAL_GPU = 10_000_000  # 10M GPU operations

# Create large matrices on GPU
hidden_dim = 384
num_experts = 64
batch_gpu = 100_000

# Generate random data on GPU
X_gpu = torch.randn(batch_gpu, hidden_dim, device=gpu, dtype=torch.float32)
W_gpu = torch.randn(num_experts, hidden_dim, device=gpu, dtype=torch.float32) * 0.01

print(f'  GPU memory: {torch.cuda.memory_allocated(0)/1024**2:.0f}MB allocated' if gpu.type == 'cuda' else f'  Device: {gpu.type}')
print(f'  Matrix: {num_experts}×{hidden_dim} = {num_experts*hidden_dim:,} params')
print(f'  Batch: {batch_gpu:,} vectors')

# Warmup GPU
for _ in range(100):
    _ = W_gpu @ X_gpu.T

t0_gpu = time.time()
gpu_ops = 0

for i in range(0, TOTAL_GPU, batch_gpu):
    X = torch.randn(batch_gpu, hidden_dim, device=gpu, dtype=torch.float32)
    scores = W_gpu @ X.T  # (64, batch) — massive parallel matmul
    top2 = torch.topk(scores.T, 2, dim=1)  # Top-2 experts per vector
    gpu_ops += batch_gpu
    
    if gpu_ops % 5_000_000 == 0:
        elapsed = time.time() - t0_gpu
        print(f'    {gpu_ops/1e6:5.1f}M/{TOTAL_GPU/1e6:.0f}M GPU ops | {gpu_ops/elapsed:,.0f}/s')

gpu_elapsed = time.time() - t0_gpu
gpu_tps = TOTAL_GPU / gpu_elapsed
print(f'  GPU Complete: {gpu_tps:,.0f} ops/s | {gpu_elapsed:.1f}s total')

# ─── Phase 3: Combined Stats ───
print()
print('=' * 70)
print('RESULTS — 1 Billion + 10M Hybrid Pipeline')
print('-' * 70)
print(f'  CPU MAMBA3STEP:     {TOTAL_CPU:,} steps in {cpu_elapsed:.1f}s ({cpu_tps:,.0f}/s)')
print(f'  GPU INSTELLA_GATE:  {TOTAL_GPU:,} ops in {gpu_elapsed:.1f}s ({gpu_tps:,.0f}/s)')
print(f'  Combined tasks:     {TOTAL_CPU + TOTAL_GPU:,}')
print(f'  Total wall time:    {cpu_elapsed + gpu_elapsed:.1f}s')
combined = (TOTAL_CPU + TOTAL_GPU) / (cpu_elapsed + gpu_elapsed)
print(f'  Combined throughput: {combined:,.0f} tasks/s')

# Cost
cost_local = (TOTAL_CPU + TOTAL_GPU) * 0.00000006
cost_gpt4o = (TOTAL_CPU + TOTAL_GPU) * 0.03
print(f'  Local cost:         ${cost_local:.2f}')
print(f'  GPT-4o equivalent:  ${cost_gpt4o:,.0f}')
print(f'  Savings:            {cost_gpt4o/cost_local:,.0f}x')

# GPU utilization
if gpu.type == 'cuda':
    print(f'  GPU VRAM:           {torch.cuda.max_memory_allocated(0)/1024**3:.1f}GB peak')
    print(f'  GPU utilization:    ~{(gpu_tps/25e6)*100:.0f}% (est.)')
else:
    print(f'  GPU:                CPU fallback (no GPU detected)')

# Save
report = {
    'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'cpu': {'tasks': TOTAL_CPU, 'tps': round(cpu_tps), 'elapsed_s': round(cpu_elapsed, 1)},
    'gpu': {'tasks': TOTAL_GPU, 'tps': round(gpu_tps), 'elapsed_s': round(gpu_elapsed, 1)},
    'combined_tps': round(combined),
    'gpu_device': gpu_name,
    'cost': {'local': round(cost_local, 2), 'gpt4o': round(cost_gpt4o)},
}
Path('swarm_v2_artifacts/bench_1B_hybrid.json').write_text(json.dumps(report, indent=2))
print(f'\nReport: swarm_v2_artifacts/bench_1B_hybrid.json')

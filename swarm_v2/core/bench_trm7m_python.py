#!/usr/bin/env python3
"""Python reference benchmark: 1000 forward calls, P50/P95/avg/tps.
Apples-to-apples vs the Rust host_test benchmark."""
import numpy as np
import sys, os, time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, INPUT_DIM

model = TRM7MModel()

# warm up
x = np.random.randn(INPUT_DIM).astype(np.float32)
model.forward(x)

times = []
for _ in range(1000):
    x = np.random.randn(INPUT_DIM).astype(np.float32)
    out = model.forward(x)
    times.append(out.latency_ns)

times_ns = np.array(times)
p50 = np.percentile(times_ns, 50)
p95 = np.percentile(times_ns, 95)
avg = times_ns.mean()
tps = 1e9 / avg
print(f"PYTHON BENCH n=1000 p50={p50:.0f}ns p95={p95:.0f}ns avg={avg:.0f}ns tps={tps:.0f}")

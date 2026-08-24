#!/usr/bin/env python3
"""Python reference: same LCG input vector as the Rust host test, run through
the canonical seed-42 TRM 7M. Compares trit/confidence 1:1 with Rust."""
import numpy as np
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, INPUT_DIM

# Same deterministic LCG input the Rust host test uses
x = np.zeros(INPUT_DIM, dtype=np.float32)
s = 42
for j in range(INPUT_DIM):
    s = (s * 6364136223846793005 + 1) & 0xFFFFFFFFFFFFFFFF
    x[j] = ((s >> 33) / (1 << 31)) - 1.0

model = TRM7MModel()
out = model.forward(x)
label_name = {1: "PASS(+1)", 0: "AMBIGUOUS(0)", -1: "FAULT(-1)"}
print(f"PYTHON trit={out.predicted_label} conf={out.confidence:.6f} ns={out.latency_ns}")
print(f"  label={label_name[out.predicted_label]} probs={out.trits}")

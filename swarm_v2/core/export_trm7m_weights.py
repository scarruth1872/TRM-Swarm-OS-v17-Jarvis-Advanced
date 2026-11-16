#!/usr/bin/env python3
"""Export TRM 7M canonical seed-42 weights to a flat binary blob for the Rust kernel.

Layout (all float32 LE):
  [0]      input_embed  (STATE_DIM x INPUT_DIM = 465x128)
  [1..3]   block i: input_proj, conv1d_weight, dt_proj, A, B, C, D, out_proj
  [N]      output_head  (3 x STATE_DIM)
  [N+1]    output_bias  (3)

This guarantees 1:1 parity: the Rust kernel loads the EXACT same weights the
Python platform uses, so outputs + benchmark behavior match bit-for-bit.
"""
import numpy as np
import struct
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from swarm_v2.core.trm_7m_model import TRM7MModel, STATE_DIM, INPUT_DIM, OUTPUT_TRITS, NUM_BLOCKS, D_CONV, DT_RANK

model = TRM7MModel()
print(f"Exporting TRM 7M (seed-42 canonical):")
print(f"  blocks={NUM_BLOCKS} state_dim={model.state_dim} input_dim={INPUT_DIM}")
print(f"  total_params={model.total_params:,}")

out = bytearray()
def emit(arr: np.ndarray):
    a = np.ascontiguousarray(arr, dtype=np.float32)
    out.extend(a.tobytes())

emit(model.input_embed)
for block in model.blocks:
    for name in ['input_proj', 'conv1d_weight', 'dt_proj', 'A', 'B', 'C', 'D', 'out_proj']:
        emit(getattr(block, name))
emit(model.output_head)
emit(model.output_bias)

path = os.path.join(os.path.dirname(__file__), '..', '..', 'trm_uefi_kernel', 'assets', 'trm7m_weights.bin')
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, 'wb') as f:
    f.write(bytes(out))

print(f"  exported: {len(out):,} bytes -> {path}")
print(f"  expected params: {model.total_params:,} x 4B = {model.total_params*4:,} bytes")
print(f"  match: {'OK' if len(out) == model.total_params*4 else 'MISMATCH'}")

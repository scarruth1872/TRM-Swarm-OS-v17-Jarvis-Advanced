import sys
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
import torch
from swarm_v2.core.trm_brain import TRMBrain

b = TRMBrain()
b._ensure_loaded()

toks = [0] * 9 + [10] * (1024 - 9)
inp = torch.tensor(toks).unsqueeze(0)
batch = {"inputs": inp, "puzzle_identifiers": torch.tensor([0])}

print("input shape:", inp.shape)
carry = b._model.initial_carry(batch)
print("carry keys:", list(carry.keys()) if isinstance(carry, dict) else type(carry))

for i in range(3):
    carry, out = b._model(carry, batch)
    print(f"cycle {i}: out keys={list(out.keys())}")
    for k, v in out.items():
        if hasattr(v, "shape"):
            print(f"   {k}: {tuple(v.shape)}")
        else:
            print(f"   {k}: {type(v)}")
    if i == 0:
        break

import sys; sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

import torch
p = r"F:\Development sites\TRM agent swarm\tiny-recursive-weights\step_155718"
try:
    sd = torch.load(p, map_location="cpu", weights_only=False)
except Exception as e:
    print("load err:", e); raise SystemExit
print("type:", type(sd).__name__)
keys = list(sd.keys()) if hasattr(sd, "keys") else []
print("num keys:", len(keys))
print("first 10:", keys[:10])
emb = [k for k in keys if "puzzle" in k.lower()]
print("puzzle-related keys:", emb[:6])

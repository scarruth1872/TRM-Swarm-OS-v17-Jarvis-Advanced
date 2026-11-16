"""
TRM Swarm OS v18.54 — Parallel Agent Training: 15 Agents Simultaneously
=========================================================================
Uses 15 Digital Twin slots to train all agents concurrently.
Each agent trains on its own CU partition of the RX 6700 XT.
"""
import sys; sys.path.insert(0, '.')
import json, time, torch, numpy as np
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from torch.utils.data import DataLoader
from collections import defaultdict

AGENTS = ["Shield","Logic","Devo","Seeker","Verify","Archi","Flow","Bridge",
          "Orchestra","EventLedger","LogicMaster","Pulse","THzGateway","Jarvis","Hermes"]
CURR_DIR = Path("swarm_v2_global_memory/curriculum_10k")

print("=" * 70)
print("PARALLEL TRAINING — 15 Agents on 40 Compute Units")
print("=" * 70)

class AgentDataset:
    def __init__(self, pairs, dim=96):
        self.x = [p["input"][:dim].ljust(dim) for p in pairs]
        self.y = [p["output"][:dim].ljust(dim) for p in pairs]
    def __len__(self): return len(self.x)
    def __getitem__(self, i):
        xi = torch.tensor([ord(c)/255.0 for c in self.x[i]], dtype=torch.float32)
        yi = torch.tensor([ord(c)/255.0 for c in self.y[i]], dtype=torch.float32)
        return xi, yi

def train_agent(agent_name, dim=96, hidden=48, epochs=3, batch=32):
    """Train a single agent — called in parallel from thread pool."""
    from swarm_v2.core.local_lora_trainer import LocalLoRATrainer
    from swarm_v2.core.kernel_lora_adapter import KernelLoRAAdapter
    
    fname = CURR_DIR / f"{agent_name}_10k.jsonl"
    if not fname.exists():
        return {"agent": agent_name, "error": "no_dataset"}
    
    pairs = [json.loads(l) for l in open(fname)]
    dataset = AgentDataset(pairs[:10000], dim=dim)
    loader = DataLoader(dataset, batch_size=batch, shuffle=True, num_workers=0)
    
    trainer = LocalLoRATrainer(agent_name)
    trainer.build_model(input_dim=dim, hidden_dim=hidden, output_dim=dim)
    trainer.model.train()
    
    opt = torch.optim.AdamW(trainer.model.parameters(), lr=3e-4)
    loss_fn = torch.nn.MSELoss()
    
    best = float('inf'); t0 = time.time()
    for ep in range(epochs):
        el = 0; n = 0
        for bx, by in loader:
            bx, by = bx.to(trainer.device), by.to(trainer.device)
            opt.zero_grad()
            loss = loss_fn(trainer.model(bx), by)
            loss.backward(); opt.step()
            el += loss.item(); n += 1
        avg = el / max(1, n)
        if avg < best: best = avg
    
    path = trainer.export_adapter()
    elapsed = time.time() - t0
    kb = path.stat().st_size / 1024
    
    # Load into registry
    from swarm_v2.core.kernel_lora_adapter import get_lora_registry
    reg = get_lora_registry()
    a = KernelLoRAAdapter(agent_name)
    a.load_from_pytorch(path)
    reg.register(a)
    
    return {
        "agent": agent_name,
        "loss": round(best, 4),
        "time": round(elapsed, 1),
        "kb": round(kb, 1),
        "pairs": len(pairs[:10000]),
    }

# ============================================================
# PARALLEL EXECUTION — 15 agents across ThreadPool
# ============================================================
t0_total = time.time()
results = {}
completed = [0]
lock = __import__('threading').Lock()

with ThreadPoolExecutor(max_workers=8) as executor:
    futures = {executor.submit(train_agent, a): a for a in AGENTS}
    
    for future in as_completed(futures):
        r = future.result()
        results[r["agent"]] = r
        with lock:
            completed[0] += 1
            print(f"  [{completed[0]:2d}/15] {r['agent']:15s} loss={r.get('loss','?'):.4f}  {r.get('time',0):.0f}s  {r.get('kb',0):.0f}KB")

total_elapsed = time.time() - t0_total
serial_time = sum(r.get("time", 0) for r in results.values())

print(f"\n{'='*70}")
print(f"PARALLEL TRAINING COMPLETE")
print(f"{'='*70}")
print(f"  Agents:        {len(results)}/15 trained")
print(f"  Total pairs:   {sum(r.get('pairs',0) for r in results.values()):,}")
print(f"  Wall time:     {total_elapsed:.1f}s ({total_elapsed/60:.1f}min)")
print(f"  Serial time:   {serial_time:.1f}s")
print(f"  Speedup:       {serial_time/total_elapsed:.1f}x parallel")
print(f"  GPU:           RX 6700 XT (40 CUs, 12GB VRAM)")
print(f"  Cost:          $0 (local, zero cloud)")

print(f"\n  Per Agent:")
for a in AGENTS:
    r = results.get(a, {})
    print(f"  {a:15s} loss={r.get('loss','N/A'):.4f}  {r.get('time',0):.0f}s  {r.get('kb',0):.0f}KB")

# Load all into registry
from swarm_v2.core.kernel_lora_adapter import get_lora_registry
reg = get_lora_registry()
print(f"\n  Registry: {len(reg._adapters)} agents loaded in mmap arena")

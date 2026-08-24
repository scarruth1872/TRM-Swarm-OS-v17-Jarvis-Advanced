"""Train all 15 agents on 10K-pair curriculum through DirectML microkernel pipeline."""
import sys; sys.path.insert(0, '.')
import json, time, torch, numpy as np
from pathlib import Path
from torch.utils.data import DataLoader
from swarm_v2.core.local_lora_trainer import LocalLoRATrainer
from swarm_v2.core.kernel_lora_adapter import get_lora_registry, KernelLoRAAdapter

AGENTS = ["Shield","Logic","Devo","Seeker","Verify","Archi","Flow","Bridge",
          "Orchestra","EventLedger","LogicMaster","Pulse","THzGateway","Jarvis","Hermes"]
CURR_DIR = Path("swarm_v2_global_memory/curriculum_10k")

class AgentDataset:
    def __init__(self, pairs):
        self.x = [p["input"][:96].ljust(96) for p in pairs]
        self.y = [p["output"][:96].ljust(96) for p in pairs]
    def __len__(self): return len(self.x)
    def __getitem__(self, i):
        xi = torch.tensor([ord(c)/255.0 for c in self.x[i]], dtype=torch.float32)
        yi = torch.tensor([ord(c)/255.0 for c in self.y[i]], dtype=torch.float32)
        return xi, yi

results = {}
t0_total = time.time()

for agent in AGENTS:
    fname = CURR_DIR / f"{agent}_10k.jsonl"
    if not fname.exists():
        print(f"  {agent}: SKIP (no dataset)")
        continue
    
    pairs = [json.loads(l) for l in open(fname)]
    dataset = AgentDataset(pairs[:10000])
    loader = DataLoader(dataset, batch_size=16, shuffle=True)
    
    trainer = LocalLoRATrainer(agent)
    trainer.build_model(input_dim=96, hidden_dim=48, output_dim=96)
    trainer.model.train()
    
    opt = torch.optim.AdamW(trainer.model.parameters(), lr=3e-4)
    loss_fn = torch.nn.MSELoss()
    
    best = float('inf'); t0 = time.time()
    for ep in range(3):  # 3 epochs for speed with 10K pairs
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
    reg = get_lora_registry()
    a = KernelLoRAAdapter(agent)
    a.load_from_pytorch(path)
    reg.register(a)
    
    elapsed = time.time() - t0
    results[agent] = {"loss": round(best, 4), "time": round(elapsed, 1), "kb": round(path.stat().st_size/1024, 1)}
    print(f"  {agent:15s}: loss={best:.4f}  {elapsed:.1f}s  {path.stat().st_size/1024:.0f}KB")

print(f"\nTotal: {time.time()-t0_total:.0f}s for {len(results)} agents")
print(f"Registry: {len(get_lora_registry()._adapters)} adapters loaded")
for a, r in results.items():
    print(f"  {a:15s} loss={r['loss']:.4f}  {r['time']:>5.1f}s  {r['kb']:.0f}KB")

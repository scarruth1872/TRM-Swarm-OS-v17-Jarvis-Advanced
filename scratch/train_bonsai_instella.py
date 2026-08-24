#!/usr/bin/env python3
"""TRM 7M + Bonsai K3 + Instella MoE — Combined Intelligence Architecture.
Trains per-agent models with ternary classification and expert routing."""

import sys; sys.path.insert(0, '.')
import json, torch, numpy as np, time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from torch.utils.data import DataLoader

AGENTS = ['Shield','Logic','Devo','Seeker','Verify','Archi','Flow','Bridge',
          'Orchestra','EventLedger','LogicMaster','Pulse','THzGateway','Jarvis','Hermes']

class BonsaiInstellaModel(torch.nn.Module):
    """TRM 7M enhanced with Bonsai K3 ternary + Instella MoE expert routing.
    
    Architecture:
    1. Input → 128-dim embedding
    2. Mamba-3 SSM block (Bonsai ternary core)
    3. Instella MoE: 64 experts, top-2 gating
    4. K3 classifier: {-1, 0, +1} output
    """
    
    def __init__(self, dim=128, state_dim=32, num_experts=64, top_k=2):
        super().__init__()
        self.dim = dim
        self.state_dim = state_dim
        self.num_experts = num_experts
        self.top_k = top_k
        
        # Bonsai K3: Mamba-3 SSM weights
        self.A = torch.nn.Parameter(torch.randn(state_dim, state_dim) * 0.01)
        self.B = torch.nn.Parameter(torch.randn(state_dim, dim) * 0.01)
        self.C = torch.nn.Parameter(torch.randn(dim, state_dim) * 0.01)
        self.D = torch.nn.Parameter(torch.randn(dim, dim) * 0.01)
        
        # Instella MoE: 64 expert networks + gating
        self.experts = torch.nn.ModuleList([
            torch.nn.Sequential(
                torch.nn.Linear(dim, 64),
                torch.nn.ReLU(),
                torch.nn.Linear(64, dim),
            ) for _ in range(num_experts)
        ])
        
        # Gating network: selects which experts to use
        self.gate = torch.nn.Sequential(
            torch.nn.Linear(dim, num_experts),
            torch.nn.Softmax(dim=-1),
        )
        
        # K3 classifier head
        self.classifier = torch.nn.Sequential(
            torch.nn.Linear(dim, 64),
            torch.nn.ReLU(),
            torch.nn.Linear(64, 3),  # 3 classes: {-1, 0, +1}
        )
        
        # Output projection
        self.out_proj = torch.nn.Linear(dim, dim)
        self.norm = torch.nn.LayerNorm(dim)
    
    def forward(self, x):
        # x: (batch, dim)
        batch = x.shape[0]
        
        # 1. Bonsai K3: Mamba-3 SSM state evolution
        h = torch.zeros(batch, self.state_dim, device=x.device)
        dh = h @ self.A.T + x @ self.B.T
        h = h + 0.05 * dh
        
        # Bonsai output: y = C·h + D·x
        bonsai_out = h @ self.C.T + x @ self.D.T
        
        # 2. Instella MoE: top-k expert routing
        gate_weights = self.gate(x)  # (batch, num_experts)
        topk_vals, topk_idx = torch.topk(gate_weights, self.top_k, dim=-1)
        
        # Route through selected experts
        moe_out = torch.zeros_like(x)
        for k in range(self.top_k):
            for i in range(batch):
                expert_idx = topk_idx[i, k].item()
                expert_out = self.experts[expert_idx](x[i:i+1])
                moe_out[i:i+1] += topk_vals[i, k] * expert_out
        
        # 3. Combine Bonsai + MoE
        combined = bonsai_out + 0.3 * moe_out
        combined = self.out_proj(torch.relu(combined))
        combined = self.norm(x + combined)
        
        # 4. K3 classification: {-1, 0, +1}
        k3_logits = self.classifier(combined)  # (batch, 3)
        
        return {
            'k3_logits': k3_logits,
            'bonsai_out': bonsai_out,
            'moe_out': moe_out,
            'combined': combined,
            'gate_weights': gate_weights,
            'topk_idx': topk_idx,
        }

class BonsaiInstellaTrainer:
    def __init__(self, agent, device='cpu'):
        self.agent = agent
        self.device = torch.device(device)
        self.model = BonsaiInstellaModel(dim=96, state_dim=16, num_experts=8, top_k=2).to(self.device)
        self.optimizer = torch.optim.AdamW(self.model.parameters(), lr=0.0005)
        self.k3_loss = torch.nn.CrossEntropyLoss()
        self.recon_loss = torch.nn.MSELoss()
    
    def load_data(self, fname, limit=5000):
        data = []
        with open(fname) as f:
            for i, line in enumerate(f):
                if i >= limit: break
                p = json.loads(line)
                msg = p['messages'][-1]['content']
                vec = np.array([ord(c)/255.0 for c in msg[:96].ljust(96,' ')], dtype=np.float32)
                data.append(vec)
        return torch.tensor(np.stack(data))
    
    def train(self, epochs=5, batch_size=64):
        fname = f'swarm_v2_global_memory/curriculum_50k/{self.agent}_50k.jsonl'
        data = self.load_data(fname, limit=5000)
        y_recon = torch.roll(data, -1, 0)
        
        # K3 targets: 0=BLOCK, 1=ANALYZE, 2=PROCEED
        k3_targets = torch.ones(len(data), dtype=torch.long) * 2  # Default PROCEED
        
        dataset = torch.utils.data.TensorDataset(data, y_recon, k3_targets)
        loader = DataLoader(dataset, batch_size=batch_size, shuffle=True)
        
        best_loss = float('inf')
        t0 = time.time()
        
        for epoch in range(epochs):
            total_loss = 0
            total_k3_loss = 0
            total_recon_loss = 0
            
            for x, y_rec, k3_y in loader:
                x = x.to(self.device)
                y_rec = y_rec.to(self.device)
                k3_y = k3_y.to(self.device)
                
                self.optimizer.zero_grad()
                outputs = self.model(x)
                
                # K3 classification loss
                k3_loss = self.k3_loss(outputs['k3_logits'], k3_y)
                
                # Reconstruction loss (next-token prediction)
                recon_loss = self.recon_loss(outputs['combined'], y_rec)
                
                # Combined loss
                loss = k3_loss + 0.1 * recon_loss
                loss.backward()
                self.optimizer.step()
                
                total_loss += loss.item()
                total_k3_loss += k3_loss.item()
                total_recon_loss += recon_loss.item()
            
            avg = total_loss / len(loader)
            if avg < best_loss:
                best_loss = avg
            
            if epoch % 2 == 0 or epoch == epochs - 1:
                print(f'  {self.agent:12s} ep{epoch} loss={avg:.4f} k3={total_k3_loss/len(loader):.4f} recon={total_recon_loss/len(loader):.4f}')
        
        elapsed = time.time() - t0
        params = sum(p.numel() for p in self.model.parameters())
        return best_loss, elapsed, params

def train_one(agent):
    t = BonsaiInstellaTrainer(agent)
    loss, elapsed, params = t.train(epochs=5, batch_size=64)
    
    # Save
    out_dir = Path('swarm_v2_global_memory/trm_7m_agents')
    out_dir.mkdir(exist_ok=True)
    torch.save({
        'model_state': t.model.state_dict(),
        'architecture': 'Bonsai-K3 + Instella-MoE',
        'dim': 96, 'state_dim': 16, 'num_experts': 8,
        'loss': loss, 'params': params,
        'agent': agent,
    }, out_dir / f'{agent}_bonsai_instella.pt')
    
    return agent, loss, elapsed, params

print('=' * 70)
print('TRM 7M + Bonsai K3 + Instella MoE — Combined Training')
print('Architecture: Mamba-3 SSM + 8 Experts (top-2) + K3 Classifier')
print('=' * 70)

t0 = time.time()
results = []

with ThreadPoolExecutor(max_workers=8) as ex:
    for agent in AGENTS:
        f = ex.submit(train_one, agent)
        results.append((agent, f))

for agent, f in results:
    try:
        a, loss, elapsed, params = f.result()
        print(f'  {a:12s} loss={loss:.4f}  {elapsed:.1f}s  {params:,} params')
    except Exception as e:
        print(f'  {agent:12s} FAILED: {e}')

total = time.time() - t0
print(f'\nTrained: {len(results)}/15 | {total:.1f}s wall time')
print(f'Models: swarm_v2_global_memory/trm_7m_agents/*_bonsai_instella.pt')

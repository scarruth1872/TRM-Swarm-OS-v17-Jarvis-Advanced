#!/usr/bin/env python3
"""Corrected Autonomous Training — Fixed NaN divergence.
Lower LR, normalized inputs, Xavier init, gradient monitoring."""

import sys; sys.path.insert(0, '.')
import json, time, torch, numpy as np
from pathlib import Path
from datetime import datetime

# ─── Config ───
TRAINING_DURATION_S = 3 * 3600
CHECKPOINT_S = 300
CURRICULUM_DIR = Path('swarm_v2_global_memory/curriculum_50k')
OUT_DIR = Path('swarm_v2_artifacts/training_session_v2')
OUT_DIR.mkdir(parents=True, exist_ok=True)

AGENTS = ['Shield','Logic','Devo','Seeker','Verify','Archi','Flow','Bridge',
          'Orchestra','EventLedger','LogicMaster','Pulse','THzGateway','Jarvis','Hermes']

LORA_RANK = 8       # Reduced from 16
STATE_DIM = 32
INPUT_DIM = 96
LR = 0.00003        # 10x lower
DT = 0.002

print('='*70)
print('CORRECTED 3-HOUR TRAINING')
print(f'LR: {LR} | LoRA rank: {LORA_RANK} | State dim: {STATE_DIM}')
print('='*70)

# ─── Load curriculum (normalized) ───
print()
print('LOADING NORMALIZED CURRICULUM...')
curriculum = {}
total_pairs = 0
for agent in AGENTS:
    fname = CURRICULUM_DIR / f'{agent}_50k.jsonl'
    if fname.exists():
        data = []
        with open(fname) as f:
            for line in f:
                try:
                    p = json.loads(line)
                    msg = p.get('messages', [{}])[-1].get('content', '')
                    vec = np.array([ord(c)/255.0 for c in msg[:INPUT_DIM].ljust(INPUT_DIM,' ')], dtype=np.float32)
                    data.append(vec)
                except: pass
        arr = np.stack(data[:30000])  # 30K per agent
        # NORMALIZE
        arr = (arr - arr.mean()) / (arr.std() + 1e-8)
        curriculum[agent] = arr
        total_pairs += len(arr)
        print(f'  {agent:15s}: {len(arr):,} normalized')
    else:
        curriculum[agent] = np.random.randn(5000, INPUT_DIM).astype(np.float32) * 0.01

print(f'  TOTAL: {total_pairs:,} pairs')

# ─── Corrected Model ───
class CorrectedLoRA(torch.nn.Module):
    def __init__(self, name):
        super().__init__()
        self.name = name
        
        # Xavier init for stability
        self.W_in = torch.nn.Parameter(torch.empty(STATE_DIM, INPUT_DIM))
        self.W_out = torch.nn.Parameter(torch.empty(INPUT_DIM, STATE_DIM))
        torch.nn.init.xavier_normal_(self.W_in, gain=0.1)
        torch.nn.init.xavier_normal_(self.W_out, gain=0.1)
        
        # LoRA: low-rank with small init
        self.lora_A = torch.nn.Parameter(torch.zeros(LORA_RANK, STATE_DIM))
        self.lora_B = torch.nn.Parameter(torch.zeros(STATE_DIM, LORA_RANK))
        torch.nn.init.normal_(self.lora_A, std=0.001)
        torch.nn.init.normal_(self.lora_B, std=0.001)
        
        self.A_diag = torch.nn.Parameter(torch.full((STATE_DIM,), -0.5))
        self.dt = DT
    
    def forward(self, x):
        h = x @ self.W_in.T
        delta = self.lora_B @ self.lora_A  # [state, state]
        h_new = h * (1.0 + self.A_diag * self.dt) + (h @ delta.T) * self.dt * 0.1
        return h_new @ self.W_out.T

# ─── Build ───
models = {}
optimizers = {}
for agent in AGENTS:
    m = CorrectedLoRA(agent)
    models[agent] = m
    optimizers[agent] = torch.optim.AdamW(m.parameters(), lr=LR, weight_decay=1e-4)

loss_fn = torch.nn.MSELoss()
param_count = sum(p.numel() for p in models[AGENTS[0]].parameters())
print(f'\n  Models built: {param_count:,} params each')

# ─── Training ───
print()
print(f'BEGINNING CORRECTED TRAINING — {datetime.now().strftime("%H:%M:%S")}')

t0 = time.time()
last_ckpt = t0
stats = {'loss': [], 'steps': 0, 'errors': 0}
epoch = 0

try:
    while time.time() - t0 < TRAINING_DURATION_S:
        epoch += 1
        epoch_loss = []
        
        for agent in AGENTS:
            data = torch.tensor(curriculum[agent], dtype=torch.float32)
            y = torch.roll(data, -1, 0)
            ds = torch.utils.data.TensorDataset(data, y)
            dl = torch.utils.data.DataLoader(ds, batch_size=64, shuffle=True)
            
            for xb, yb in dl:
                optimizers[agent].zero_grad()
                pred = models[agent](xb)
                loss = loss_fn(pred, yb)
                
                # NaN detection
                if torch.isnan(loss) or torch.isinf(loss):
                    stats['errors'] += 1
                    continue
                
                loss.backward()
                
                # Gradient clipping
                gn = torch.nn.utils.clip_grad_norm_(models[agent].parameters(), 0.5)
                
                # Skip if gradient exploded
                if gn > 100:
                    stats['errors'] += 1
                    continue
                
                optimizers[agent].step()
                epoch_loss.append(loss.item())
                stats['steps'] += 1
        
        if epoch_loss:
            avg = np.mean(epoch_loss)
            stats['loss'].append(avg)
        
        elapsed = time.time() - t0
        remaining = TRAINING_DURATION_S - elapsed
        
        if epoch % 20 == 1 and stats['loss']:
            print(f'  E{epoch:4d} | Loss: {stats["loss"][-1]:.6f} | '
                  f'Best: {min(stats["loss"]):.6f} | '
                  f'Steps: {stats["steps"]:,} | '
                  f'Err: {stats["errors"]} | '
                  f'{elapsed/60:.0f}m/{remaining/60:.0f}m')
        
        # Checkpoint
        if time.time() - last_ckpt >= CHECKPOINT_S and stats['loss']:
            ckpt = OUT_DIR / f'ckpt_e{epoch}_{int(elapsed)}s.pt'
            torch.save({
                'epoch': epoch,
                'models': {a: m.state_dict() for a, m in models.items()},
                'stats': stats,
                'elapsed': elapsed,
            }, ckpt)
            last_ckpt = time.time()
            print(f'  [CKPT] {ckpt.name}')

except KeyboardInterrupt:
    print('\nInterrupted')
except Exception as e:
    print(f'\nFATAL: {e}')
    import traceback; traceback.print_exc()

# ─── Final ───
elapsed = time.time() - t0
print()
print('='*70)
print('CORRECTED TRAINING COMPLETE')
print('-'*70)
print(f'  Duration:   {elapsed/60:.1f}min')
print(f'  Epochs:     {epoch}')
print(f'  Steps:      {stats["steps"]:,}')
print(f'  Errors:     {stats["errors"]} ({stats["errors"]/max(stats["steps"],1)*100:.2f}%)')
if stats['loss']:
    print(f'  Best loss:  {min(stats["loss"]):.6f}')
    print(f'  Final loss: {stats["loss"][-1]:.6f}')

# Save
final = OUT_DIR / 'final_model.pt'
torch.save({a: m.state_dict() for a, m in models.items()}, final)
print(f'\nModel: {final} ({final.stat().st_size/1024**2:.1f}MB)')

report = {
    'ts': datetime.now().isoformat(),
    'duration_min': round(elapsed/60, 1),
    'epochs': epoch,
    'steps': stats['steps'],
    'errors': stats['errors'],
    'error_rate_pct': round(stats['errors']/max(stats['steps'],1)*100, 2),
    'best_loss': round(min(stats['loss']), 6) if stats['loss'] else None,
    'final_loss': round(stats['loss'][-1], 6) if stats['loss'] else None,
    'config': {'lr': LR, 'lora_rank': LORA_RANK, 'state_dim': STATE_DIM},
}
(OUT_DIR / 'report.json').write_text(json.dumps(report, indent=2))
print(f'Report: {OUT_DIR / "report.json"}')
print('='*70)

"""
TRM Swarm OS v18.21 — THz Gateway: 100 Complex Tasks → LoRA Training Pipeline
==============================================================================
1. Run 100 diverse HTTP/web payloads through THz Fourier-Mamba3-Ponderomotive pipeline
2. Collect (state_vector, trit_label) training pairs from Kleene outputs
3. Feed into LoRA trainer (DirectML on AMD GPU)
4. Hot-swap updated adapters into kernel mmap registry
"""
import sys; sys.path.insert(0, '.')
import time, json, numpy as np
from pathlib import Path

from swarm_v2.core.thz_wave_gateway import get_thz_gateway
from swarm_v2.core.kernel_lora_adapter import get_lora_registry, KernelLoRAAdapter
from swarm_v2.core.local_lora_trainer import LocalLoRATrainer, AgentTrainingDataset

print("=" * 70)
print("THz Gateway — 100 Complex Tasks → LoRA Training")
print("=" * 70)

# Phase 1: Run 100 tasks through THz gateway
gw = get_thz_gateway()

# Diverse payloads simulating real web traffic
PAYLOADS = [
    # REST API calls
    b'GET /api/v1/health HTTP/1.1\r\nHost: swarm.local\r\nAccept: application/json',
    b'POST /api/v1/agents/shield/audit HTTP/1.1\r\nContent-Type: application/json\r\n{"action":"ddr_scan"}',
    b'PUT /api/v1/config/hvbe HTTP/1.1\r\nContent-Type: application/json\r\n{"phi_v":0.76}',
    b'DELETE /api/v1/tasks/expired HTTP/1.1\r\nHost: swarm.local',
    b'PATCH /api/v1/adapters/logic/lora HTTP/1.1\r\n{"rank":8,"alpha":16.0}',
    # WebSocket frames
    b'\x81\x0b{"op":"ping"}',
    b'\x82\x7f{"op":"subscribe","channel":"telemetry"}',
    b'\x88\x02\x03\xe8',  # Close frame
    # Security probes
    b'\x00\x00\x00\x00\xFF\xFF\xFF\xFF' * 8,  # Null/FF pattern
    b"' OR '1'='1' -- ",  # SQL injection probe
    b'<script>alert(1)</script>',  # XSS probe
    b'../../../etc/passwd',  # Path traversal
    # GraphQL
    b'{"query":"{ agents { name status } }"}',
    b'{"mutation":{"upsertMemory":{"key":"thz_test","value":"gateway_trained"}}}',
    # gRPC-like binary
    b'\x00\x00\x00\x00\x08\x0a\x06\x53\x68\x69\x65\x6c\x64',
    # DNS query
    b'\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07\x73\x77\x61\x72\x6d\x02\x6f\x73\x00\x00\x01\x00\x01',
    # Normal JSON
    json.dumps({"role":"Seeker","message":"find tau inhibitors","sender":"benchmark"}).encode(),
    json.dumps({"role":"Archi","message":"blueprint 1024-agent swarm","sender":"benchmark"}).encode(),
    json.dumps({"role":"Logic","message":"theorem prove K3 completeness","sender":"benchmark"}).encode(),
    json.dumps({"role":"Shield","message":"security audit DDR patterns","sender":"benchmark"}).encode(),
    # Minimal
    b'PING',
    b'{}',
    b'\x00',
    # Large payload
    b'X' * 256,
    # Binary protocol
    b'\x01\x00\x00\x00' + b'\x00' * 32,
]

# Generate 100 tasks by repeating and varying
TASKS = []
for i in range(100):
    base = PAYLOADS[i % len(PAYLOADS)]
    # Add sequence number to vary payload
    variant = base[:max(1, len(base)//2)] + bytes([i & 0xFF, (i>>8) & 0xFF])
    TASKS.append(variant)

print(f"\nPhase 1: Running {len(TASKS)} tasks through THz gateway...")
training_pairs = []
start = time.time()

for i, payload in enumerate(TASKS):
    resp = gw.process_request(payload)
    data = json.loads(resp.decode())
    
    # Collect state vector + majority trit as training label
    packet = gw.ssm  # Access SSM for latest state vector
    
    # Build input features from payload
    vec = np.zeros(128, dtype=np.float32)
    for j, b in enumerate(payload[:128]):
        vec[j] = b / 255.0
    
    # Label: majority trit from Kleene output
    trits = data['kleene_trits']
    if trits['pass'] > trits['fail']:
        label = 1   # PASS
    elif trits['fail'] > 0:
        label = -1  # FAULT
    else:
        label = 0   # AMBIGUOUS
    
    training_pairs.append((vec, label))
    
    if (i+1) % 25 == 0:
        elapsed = time.time() - start
        print(f"  {i+1:3d}/100 | {elapsed:.2f}s | {trits}")

elapsed = time.time() - start
print(f"\nPhase 1 complete: {len(training_pairs)} training pairs in {elapsed:.2f}s ({len(training_pairs)/elapsed:.1f} tasks/s)")

# Stats on collected data
labels = [p[1] for p in training_pairs]
print(f"  Pass (+1):     {labels.count(1)}")
print(f"  Fault (-1):    {labels.count(-1)}")
print(f"  Ambiguous (0): {labels.count(0)}")

# Phase 2: Export training data for LoRA
print(f"\nPhase 2: Exporting training data...")
export_path = Path("swarm_v2_global_memory/thz_training_data.jsonl")
with open(export_path, 'w') as f:
    for vec, label in training_pairs:
        entry = {"state_vector": vec.tolist(), "trit_label": label}
        f.write(json.dumps(entry) + '\n')
print(f"  Exported to: {export_path} ({export_path.stat().st_size/1024:.1f}KB)")

# Phase 3: Train LoRA adapters on THz data
print(f"\nPhase 3: Training 'THz Gateway' LoRA adapter on collected data...")

# Create dataset from collected pairs
class THzDataset:
    def __init__(self, pairs):
        self.X = np.array([p[0] for p in pairs], dtype=np.float32)
        self.y = np.array([p[1] for p in pairs], dtype=np.int64)
    def __len__(self):
        return len(self.X)
    def __getitem__(self, idx):
        import torch
        return torch.tensor(self.X[idx]), torch.tensor(self.y[idx])

trainer = LocalLoRATrainer("THzGateway")
trainer.build_model()

# Convert to torch dataset and train
import torch
from torch.utils.data import DataLoader, TensorDataset

dataset = TensorDataset(
    torch.tensor(np.array([p[0] for p in training_pairs], dtype=np.float32)),
    torch.tensor(np.array([p[1] for p in training_pairs], dtype=np.int64))
)
loader = DataLoader(dataset, batch_size=8, shuffle=True)

trainer.model.train()
loss_fn = torch.nn.CrossEntropyLoss()
best_loss = float('inf')

for epoch in range(20):
    epoch_loss = 0
    for batch_x, batch_y in loader:
        batch_x = batch_x.to(trainer.device)
        batch_y = batch_y.to(trainer.device)
        trainer.optimizer.zero_grad()
        loss = loss_fn(trainer.model(batch_x), batch_y)
        loss.backward()
        trainer.optimizer.step()
        epoch_loss += loss.item()
    avg_loss = epoch_loss / len(loader)
    if avg_loss < best_loss:
        best_loss = avg_loss
    if (epoch+1) % 5 == 0:
        print(f"  Epoch {epoch+1:2d}/20 | Loss: {avg_loss:.4f}")

print(f"  Training complete | Best loss: {best_loss:.4f}")

# Phase 4: Export and hot-swap
adapter_path = trainer.export_adapter()
size_kb = adapter_path.stat().st_size / 1024

# Load into kernel registry
reg = get_lora_registry()
thz_adapter = KernelLoRAAdapter("THzGateway")
thz_adapter.load_from_pytorch(adapter_path)
reg.register(thz_adapter)
reg.swap_to("THzGateway")

print(f"\nPhase 4: Hot-swapped into kernel registry")
print(f"  Adapter: {size_kb:.1f}KB | Agent: THzGateway")
print(f"  Registry: {len(reg._adapters)} adapters active")

# Verify
active = reg.get_active()
if active:
    test_vec = np.ones(128, dtype=np.float32) * 0.5
    delta = active.apply(test_vec)
    print(f"  Inference test: delta std={delta.std():.4f}")

print(f"\n{'=' * 70}")
print(f"DONE — THz Gateway training complete")
print(f"  100 tasks processed, {len(training_pairs)} pairs collected")
print(f"  THzGateway LoRA trained ({size_kb:.1f}KB)")
print(f"  Hot-swapped into {len(reg._adapters)}-adapter kernel registry")
print(f"{'=' * 70}")

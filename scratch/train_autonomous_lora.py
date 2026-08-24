#!/usr/bin/env python3
"""
TRM Autonomous Microkernel Training — LoRA Pipeline
====================================================
Trains all 15 agents on 50K curriculum pairs through LoRA adapters.
Spec: rank-16, 96-dim embedding, 5 epochs, 8 workers parallel.

Curriculum:  swarm_v2_global_memory/curriculum_50k/{Agent}_50k.jsonl
Output:      swarm_v2_global_memory/lora_adapters/autonomous_{timestamp}.pt
"""
import sys; sys.path.insert(0, '.')
import json, time, torch, numpy as np
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
from torch.utils.data import DataLoader, TensorDataset

# ============================================================
# CONFIGURATION
# ============================================================
AGENTS = [
    "Shield", "Logic", "Devo", "Seeker", "Verify",
    "Archi", "Flow", "Bridge", "Orchestra", "EventLedger",
    "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes"
]

LORA_RANK = 16            # Low-rank decomposition rank
LORA_ALPHA = 32           # Scaling factor (alpha/r)
EMBED_DIM = 96            # Embedding dimension
EPOCHS = 5                # Training epochs
BATCH_SIZE = 64           # Batch size
LEARNING_RATE = 3e-4      # AdamW learning rate
MAX_WORKERS = 8           # Parallel training workers
MAX_PAIRS = 50000         # Pairs per agent
CURR_DIR = Path("swarm_v2_global_memory/curriculum_50k")
OUT_DIR = Path("swarm_v2_global_memory/lora_adapters")

# ============================================================
# LoRA MODULE — rank-16, 96-dim
# ============================================================
class LoRALinear(torch.nn.Module):
    """Low-Rank Adaptation wrapper: W = W₀ + (α/r) * B @ A"""

    def __init__(self, in_features, out_features, r=LORA_RANK, alpha=LORA_ALPHA):
        super().__init__()
        self.r = r
        self.alpha = alpha
        self.scaling = alpha / r

        # Frozen base weights
        self.W0 = torch.nn.Linear(in_features, out_features, bias=False)
        self.W0.weight.requires_grad = False

        # Trainable LoRA matrices — A: (in_features, r), B: (r, out_features)
        self.A = torch.nn.Parameter(torch.zeros(in_features, r))
        self.B = torch.nn.Parameter(torch.zeros(r, out_features))
        torch.nn.init.kaiming_uniform_(self.A, a=5 ** 0.5)
        torch.nn.init.zeros_(self.B)
        self.dropout = torch.nn.Dropout(0.05)

    def forward(self, x):
        base = self.W0(x)
        lora = (self.dropout(x) @ self.A @ self.B) * self.scaling
        return base + lora

    def export_lora(self):
        return {
            "A": self.A.detach().cpu(),
            "B": self.B.detach().cpu(),
            "alpha": self.alpha,
            "rank": self.r,
        }


class LoRAAgentModel(torch.nn.Module):
    """2-layer model with LoRA on the first layer. 3-class output (PASS/AMBIGUOUS/FAULT)."""

    def __init__(self, input_dim=EMBED_DIM, hidden_dim=48, output_dim=3, rank=LORA_RANK):
        super().__init__()
        self.lora = LoRALinear(input_dim, hidden_dim, r=rank)
        self.activation = torch.nn.ReLU()
        self.classifier = torch.nn.Linear(hidden_dim, output_dim)

        self._nparams = sum(p.numel() for p in self.parameters())
        self._trainable = sum(p.numel() for p in self.parameters() if p.requires_grad)

    def forward(self, x):
        x = self.lora(x)
        x = self.activation(x)
        return self.classifier(x)

    def export_adapter(self):
        return self.lora.export_lora()


# ============================================================
# DATA LOADER — embed curriculum text to 96-dim vectors
# ============================================================
def load_curriculum(agent_name, limit=MAX_PAIRS, embed_dim=EMBED_DIM):
    """Convert curriculum JSONL to (vector, label) pairs by embedding assistant content
    and deriving a ternary label from the assistant response pattern."""
    fname = CURR_DIR / f"{agent_name}_50k.jsonl"
    if not fname.exists():
        raise FileNotFoundError(f"Missing curriculum: {fname}")

    x_list = []
    y_list = []

    with open(fname, "r", encoding="utf-8") as f:
        for i, line in enumerate(f):
            if i >= limit:
                break
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue

            msgs = entry.get("messages", [])
            # Use assistant content for embedding
            assistant_content = ""
            user_content = ""
            for m in msgs:
                if m.get("role") == "assistant":
                    assistant_content = m.get("content", "")
                elif m.get("role") == "user":
                    user_content = m.get("content", "")

            # Embed: take first embed_dim chars, pad with spaces, normalize to [0,1]
            txt = (user_content + " " + assistant_content)[:embed_dim].ljust(embed_dim)
            vec = np.array([ord(c) / 255.0 for c in txt], dtype=np.float32)
            x_list.append(vec)

            # Ternary label from content patterns
            text_lower = user_content.lower() + assistant_content.lower()
            if any(w in text_lower for w in ["error", "fail", "critical", "alert", "attack", "crash"]):
                y_list.append(2)  # FAULT (-1)
            elif any(w in text_lower for w in ["success", "complete", "pass", "verified", "optimized"]):
                y_list.append(0)  # PASS (+1)
            else:
                y_list.append(1)  # AMBIGUOUS (0)

    if not x_list:
        raise ValueError(f"No valid data for agent {agent_name}")

    return np.stack(x_list).astype(np.float32), np.array(y_list, dtype=np.int64)


# ============================================================
# AGENT TRAINER
# ============================================================
def train_agent_autonomous(agent_name: str) -> dict:
    """Full training pipeline for one agent on 50K pairs."""
    t_start = time.time()

    try:
        X, y = load_curriculum(agent_name, limit=MAX_PAIRS)
    except (FileNotFoundError, ValueError) as e:
        return {
            "agent": agent_name,
            "loss": 0.0,
            "elapsed": 0.0,
            "pairs": 0,
            "error": str(e),
        }

    n_samples = len(X)
    # Build dataset and loader
    x_tensor = torch.tensor(X, dtype=torch.float32)
    y_tensor = torch.tensor(y, dtype=torch.int64)
    dataset = TensorDataset(x_tensor, y_tensor)
    loader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True, num_workers=0)

    # Use CPU for thread-safe parallel training.
    # torch_directml is NOT thread-safe — multi-threaded device creation segfaults.
    device = torch.device("cpu")
    backend = "cpu"

    # Build model
    model = LoRAAgentModel(input_dim=EMBED_DIM, hidden_dim=48, output_dim=3, rank=LORA_RANK)
    model.to(device)

    opt = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=LEARNING_RATE,
    )
    loss_fn = torch.nn.CrossEntropyLoss()

    best_loss = float("inf")
    total_batches = 0

    for epoch in range(EPOCHS):
        epoch_loss = 0.0
        n_batches = 0
        for bx, by in loader:
            bx, by = bx.to(device), by.to(device)
            opt.zero_grad()
            pred = model(bx)
            loss = loss_fn(pred, by)
            loss.backward()
            opt.step()
            epoch_loss += loss.item()
            n_batches += 1
            total_batches += 1

        avg_epoch_loss = epoch_loss / max(n_batches, 1)
        if avg_epoch_loss < best_loss:
            best_loss = avg_epoch_loss

    elapsed = time.time() - t_start
    throughput = n_samples * EPOCHS / elapsed if elapsed > 0 else 0

    # Export LoRA adapter
    lora_data = model.export_adapter()
    lora_data["agent"] = agent_name
    lora_data["loss"] = best_loss
    lora_data["elapsed"] = elapsed
    lora_data["pairs"] = n_samples
    lora_data["epochs"] = EPOCHS
    lora_data["backend"] = backend
    lora_data["architecture"] = f"LoRA-r{LORA_RANK}-d{EMBED_DIM}"
    lora_data["params"] = model._trainable

    # Save
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out_path = OUT_DIR / f"{agent_name}_autonomous_v18.lora.pt"
    torch.save(lora_data, out_path)

    return {
        "agent": agent_name,
        "loss": round(best_loss, 6),
        "elapsed": round(elapsed, 1),
        "pairs": n_samples,
        "params": model._trainable,
        "throughput": round(throughput, 0),
        "backend": backend,
        "path": str(out_path),
        "epochs": EPOCHS,
    }


# ============================================================
# MAIN — PARALLEL TRAINING
# ============================================================
def main():
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    print("=" * 70)
    print(f"TRM AUTONOMOUS LORA TRAINING — {timestamp}")
    print(f"Agents: {len(AGENTS)} | Pairs: {MAX_PAIRS:,} each | Rank: {LORA_RANK} | Dim: {EMBED_DIM}")
    print(f"Epochs: {EPOCHS} | Workers: {MAX_WORKERS} | Batch: {BATCH_SIZE}")
    print("=" * 70)
    sys.stdout.flush()

    t0 = time.time()
    results = {}
    completed = 0

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as ex:
        futures = {ex.submit(train_agent_autonomous, a): a for a in AGENTS}
        for f in as_completed(futures):
            r = f.result()
            agent = r["agent"]
            results[agent] = r
            completed += 1

            loss_str = f"{r['loss']:.6f}" if r["loss"] else "FAIL"
            icon = "✅" if r["loss"] else "❌"
            print(
                f"  [{completed:2d}/15] {icon} {agent:12s} "
                f"loss={loss_str:>10s}  {r['elapsed']:>6.1f}s  "
                f"{r['throughput']:>8.0f} samp/s  [{r.get('backend', 'cpu')}]"
            )
            sys.stdout.flush()

    total_s = time.time() - t0
    success = [r for r in results.values() if r["loss"]]
    total_pairs = sum(r.get("pairs", 0) for r in success)
    total_throughput = total_pairs * EPOCHS / total_s if total_s > 0 else 0

    # === AGGREGATE SAVE ===
    aggregate_path = OUT_DIR / f"autonomous_{timestamp}.pt"
    aggregate_data = {
        "timestamp": timestamp,
        "agents": len(AGENTS),
        "results": {a: {k: v for k, v in r.items() if k != "path"} for a, r in results.items()},
        "config": {
            "lora_rank": LORA_RANK,
            "embed_dim": EMBED_DIM,
            "epochs": EPOCHS,
            "batch_size": BATCH_SIZE,
            "workers": MAX_WORKERS,
            "max_pairs_per_agent": MAX_PAIRS,
        },
        "summary": {
            "total_wall_time_s": round(total_s, 1),
            "successful_agents": len(success),
            "best_loss": round(min(r["loss"] for r in success if r["loss"]), 6),
            "avg_loss": round(sum(r["loss"] for r in success if r["loss"]) / len(success), 6),
            "total_pairs": total_pairs,
            "total_samples_processed": total_pairs * EPOCHS,
            "throughput_samples_per_s": round(total_throughput, 0),
            "cost": "$0 (local compute)",
        },
        "per_agent_files": [f"{a}_autonomous_v18.lora.pt" for a in AGENTS],
    }
    torch.save(aggregate_data, aggregate_path)

    # === REPORT ===
    print(f"\n{'=' * 70}")
    print(f"RESULTS — {timestamp}")
    print(f"  Agents trained:      {len(success)}/{len(AGENTS)}")
    print(f"  Wall time:           {total_s:.1f}s ({total_s / 60:.1f}min)")
    print(f"  Best loss:           {min(r['loss'] for r in success if r['loss']):.6f}")
    print(f"  Avg loss:            {sum(r['loss'] for r in success if r['loss']) / len(success):.6f}")
    print(f"  Total pairs:         {total_pairs:,}")
    print(f"  Throughput:          {total_throughput:.0f} samples/s")
    print(f"  Cost:                $0 (local GPU/CPU)")
    print(f"  Aggregate save:      {aggregate_path}")
    print(f"  Per-agent saves:     {OUT_DIR}/*_autonomous_v18.lora.pt")
    print(f"{'=' * 70}")

    # Final report line for cron delivery
    best_loss = min(r["loss"] for r in success if r["loss"])
    print(
        f"\n[REPORT] [{timestamp}] "
        f"agents={len(success)} "
        f"loss={best_loss:.6f} "
        f"throughput={total_throughput:.0f} "
        f"samples/s cost=$0"
    )


if __name__ == "__main__":
    main()

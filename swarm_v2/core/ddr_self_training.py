"""
TRM Swarm OS v18 — DDR Self-Training Feedback Loop
====================================================
Captures agent failures from the Digital DNA Repository (DDR), formats them
as training tuples, and triggers LoRA adapter updates during idle cycles.

Pipeline:
  Agent Error → DDR Log → Format as (X,Y) tuple → Trigger LoRA update → Hot-swap into mmap
"""
import json, time, threading, os
from typing import Dict, List, Any, Optional, Tuple
from datetime import datetime
from pathlib import Path
import numpy as np


class DDRTrainingCollector:
    """
    Monitors the DDR event ledger and extracts failure patterns as training data.
    Each failure becomes a (state_vector, trit_label) pair for LoRA fine-tuning.
    """
    
    def __init__(self, ddr_path: str = None):
        self.ddr_path = Path(ddr_path or "swarm_v2_global_memory/turbo_events.jsonl.gz")
        self.training_pairs: List[Tuple[np.ndarray, int]] = []
        self._last_position: int = 0
        self._lock = threading.Lock()
        self.stats = {
            "collected": 0,
            "pass_samples": 0,
            "fault_samples": 0,
            "ambiguous_samples": 0,
            "last_collection": None,
        }
    
    def scan_for_failures(self) -> int:
        """Scan DDR ledger for new failure events and convert to training pairs."""
        if not self.ddr_path.exists():
            # Check uncompressed fallback
            uncompressed = Path(str(self.ddr_path).replace('.gz', ''))
            if uncompressed.exists():
                return self._scan_file(uncompressed)
            return 0
        
        return self._scan_file(self.ddr_path)
    
    def _scan_file(self, path: Path) -> int:
        """Parse event file and extract training pairs."""
        collected = 0
        try:
            import gzip
            open_func = gzip.open if path.suffix == '.gz' else open
            mode = 'rt' if path.suffix == '.gz' else 'r'
            
            with open_func(path, mode, encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        entry = json.loads(line)
                        # Extract failure-relevant fields
                        event_type = entry.get("type", "")
                        data = entry.get("data", {})
                        
                        # Build state vector from event context
                        vec = self._event_to_vector(entry)
                        
                        # Classify: PASS (+1), FAULT (-1), AMBIGUOUS (0)
                        if "error" in str(data).lower() or "fault" in event_type.lower():
                            label = -1  # FAULT
                            self.stats["fault_samples"] += 1
                        elif "approved" in str(data).lower() or "success" in event_type.lower():
                            label = 1   # PASS
                            self.stats["pass_samples"] += 1
                        else:
                            label = 0   # AMBIGUOUS
                            self.stats["ambiguous_samples"] += 1
                        
                        with self._lock:
                            self.training_pairs.append((vec, label))
                        
                        collected += 1
                    except (json.JSONDecodeError, KeyError):
                        continue
            
            self.stats["collected"] += collected
            self.stats["last_collection"] = datetime.now().isoformat()
        except Exception as e:
            print(f"[DDRCollector] Scan error: {e}")
        
        return collected
    
    def _event_to_vector(self, entry: Dict) -> np.ndarray:
        """Convert an event entry to a 128-dim state vector."""
        vec = np.zeros(128, dtype=np.float32)
        
        # Feature engineering from event fields
        data = entry.get("data", {})
        agent = entry.get("agent", "unknown")
        event_type = entry.get("type", "unknown")
        
        # Agent hash → dims 0-7
        for i, ch in enumerate(agent[:8]):
            vec[i] = ord(ch) / 255.0
        
        # Event type hash → dims 8-15
        for i, ch in enumerate(str(event_type)[:8]):
            vec[8 + i] = ord(ch) / 255.0
        
        # Data fields → dims 16-127
        if isinstance(data, dict):
            flat = json.dumps(data)
            for i, ch in enumerate(flat[:112]):
                vec[16 + i] = ord(ch) / 255.0
        
        return vec
    
    def get_training_data(self) -> Tuple[np.ndarray, np.ndarray]:
        """Return all collected training pairs as numpy arrays."""
        with self._lock:
            if not self.training_pairs:
                return np.zeros((0, 128), dtype=np.float32), np.zeros(0, dtype=np.int64)
            X = np.array([p[0] for p in self.training_pairs], dtype=np.float32)
            y = np.array([p[1] for p in self.training_pairs], dtype=np.int64)
        return X, y
    
    def export_training_file(self, output_path: str = None) -> Path:
        """Export collected training data to disk for LoRA fine-tuning."""
        if output_path is None:
            output_path = "swarm_v2_global_memory/ddr_training_data.jsonl"
        
        path = Path(output_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        
        with self._lock:
            with open(path, 'w', encoding='utf-8') as f:
                for vec, label in self.training_pairs:
                    entry = {
                        "state_vector": vec.tolist(),
                        "trit_label": int(label),
                        "timestamp": datetime.now().isoformat(),
                    }
                    f.write(json.dumps(entry) + '\n')
        
        print(f"[DDRCollector] Exported {len(self.training_pairs)} training pairs to {path}")
        return path


class SelfTrainingLoop:
    """
    Continuous self-training loop that:
    1. Monitors DDR for failures
    2. When idle (CPU < 30%, memory < 70%), triggers LoRA update
    3. Hot-swaps updated adapter into mmap arena
    """
    
    def __init__(self):
        self.collector = DDRTrainingCollector()
        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._update_count = 0
    
    def start(self):
        """Start continuous self-training in background thread."""
        if self._running:
            return
        self._running = True
        self._thread = threading.Thread(target=self._loop, daemon=True)
        self._thread.start()
        print("[SelfTraining] DDR feedback loop started")
    
    def _loop(self):
        """Main loop: scan DDR → collect → train when idle."""
        import psutil
        
        while self._running:
            try:
                # Scan DDR for new failures
                new = self.collector.scan_for_failures()
                if new > 0:
                    print(f"[SelfTraining] Collected {new} new training pairs from DDR")
                
                # Check if system is idle
                cpu = psutil.cpu_percent(interval=1)
                mem = psutil.virtual_memory().percent
                
                if cpu < 30 and mem < 70 and self.collector.stats["collected"] >= 50:
                    self._trigger_update()
                
            except Exception as e:
                print(f"[SelfTraining] Loop error: {e}")
            
            time.sleep(60)  # Check every minute
    
    def _trigger_update(self):
        """Trigger a LoRA adapter update from collected training data."""
        try:
            from swarm_v2.core.local_lora_trainer import LocalLoRATrainer
            from swarm_v2.core.kernel_lora_adapter import get_lora_registry
            
            X, y = self.collector.get_training_data()
            if len(X) < 10:
                return
            
            reg = get_lora_registry()
            active = reg.get_active()
            agent_name = active.agent_name if active else "Shield"
            
            print(f"[SelfTraining] Updating {agent_name} LoRA adapter from {len(X)} samples...")
            
            # Quick fine-tune
            trainer = LocalLoRATrainer(agent_name)
            trainer.build_model()
            
            from torch.utils.data import TensorDataset, DataLoader
            import torch
            dataset = TensorDataset(
                torch.tensor(X, dtype=torch.float32),
                torch.tensor(y, dtype=torch.long)
            )
            loader = DataLoader(dataset, batch_size=8, shuffle=True)
            
            # Run 50 quick update steps
            trainer.model.train()
            loss_fn = torch.nn.CrossEntropyLoss()
            for epoch in range(5):
                for batch_x, batch_y in loader:
                    batch_x = batch_x.to(trainer.device)
                    batch_y = batch_y.to(trainer.device)
                    trainer.optimizer.zero_grad()
                    loss = loss_fn(trainer.model(batch_x), batch_y)
                    loss.backward()
                    trainer.optimizer.step()
            
            # Export and hot-swap
            path = trainer.export_adapter()
            if active:
                active.load_from_pytorch(path)
                reg.swap_to(agent_name)
            
            self._update_count += 1
            print(f"[SelfTraining] Update #{self._update_count} complete — adapter hot-swapped")
            
        except Exception as e:
            print(f"[SelfTraining] Update failed: {e}")
    
    def stop(self):
        self._running = False
        if self._thread:
            self._thread.join(timeout=5)
        print(f"[SelfTraining] Stopped after {self._update_count} updates")


# === SINGLETON ===
_self_training: Optional[SelfTrainingLoop] = None

def get_self_training() -> SelfTrainingLoop:
    global _self_training
    if _self_training is None:
        _self_training = SelfTrainingLoop()
    return _self_training


# === SELF-TEST ===
if __name__ == "__main__":
    print("=== DDR Self-Training Feedback Loop — Test ===\n")
    
    collector = DDRTrainingCollector()
    
    # Simulate finding failures in DDR
    simulated_events = [
        {"type": "APPROVED_CONTRIBUTION", "agent": "Shield", "data": {"result": "success"}},
        {"type": "ERROR", "agent": "Logic", "data": {"error": "K3 reduction timeout"}},
        {"type": "APPROVED_CONTRIBUTION", "agent": "Seeker", "data": {"result": "success"}},
        {"type": "FAULT", "agent": "Devo", "data": {"error": "compile failed"}},
    ]
    
    for event in simulated_events:
        vec = collector._event_to_vector(event)
        label = 1 if "APPROVED" in event["type"] else (-1 if "ERROR" in event["type"] or "FAULT" in event["type"] else 0)
        collector.training_pairs.append((vec, label))
    
    collector.stats["collected"] = len(collector.training_pairs)
    collector.stats["pass_samples"] = 2
    collector.stats["fault_samples"] = 2
    
    X, y = collector.get_training_data()
    print(f"Training pairs: {len(X)} (X: {X.shape}, y: {y.shape})")
    print(f"Labels: {list(y)}")
    print(f"Pass: {sum(y == 1)}, Fault: {sum(y == -1)}, Ambiguous: {sum(y == 0)}")
    
    # Export
    path = collector.export_training_file()
    print(f"Exported to: {path}")
    print(f"\n✅ DDR Self-Training Loop — ready for continuous learning")

"""
Qwen3-4B LoRA fine-tuning on Jarvis comprehensive dataset.
4B params, Qwen3 architecture (same as Bonsai), fits in 12GB VRAM.
"""
import sys, os, json, time
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

import torch
import torch_directml
DEVICE = torch_directml.device()
print(f"Device: {DEVICE}")

from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType

MODEL = "Qwen/Qwen3-1.7B"
DATASET = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_comprehensive_chat.json"
OUTPUT = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_qwen3_adapter"
os.makedirs(OUTPUT, exist_ok=True)

BS, GA, LR, EP, SEQ = 1, 4, 2e-4, 5, 256
TARGET = ["q_proj", "k_proj", "v_proj", "o_proj"]

print("Loading tokenizer...")
tok = AutoTokenizer.from_pretrained(MODEL, trust_remote_code=True)
tok.pad_token = tok.eos_token

print("Loading dataset...")
with open(DATASET, encoding="utf-8") as f:
    raw = json.load(f)

texts = []
for item in raw:
    text = tok.apply_chat_template(item["messages"], tokenize=False, add_generation_prompt=False)
    texts.append(text)

enc = tok(texts, truncation=True, padding="max_length", max_length=SEQ, return_tensors="pt")
print(f"  {len(texts)} examples")

class T(torch.utils.data.Dataset):
    def __init__(self, i, a): self.i, self.a = i, a
    def __len__(self): return len(self.i)
    def __getitem__(self, idx):
        return {"input_ids": self.i[idx], "attention_mask": self.a[idx], "labels": self.i[idx].clone()}

loader = torch.utils.data.DataLoader(T(enc["input_ids"], enc["attention_mask"]), batch_size=BS, shuffle=True)

print("Loading model...")
model = AutoModelForCausalLM.from_pretrained(MODEL, trust_remote_code=True,
    torch_dtype=torch.float16, low_cpu_mem_usage=True).to(DEVICE)
model.train()
total = sum(p.numel() for p in model.parameters())
print(f"  {total:,} params ({total/1e9:.1f}B)")

print("Applying LoRA...")
model = get_peft_model(model, LoraConfig(r=16, lora_alpha=32, target_modules=TARGET,
    lora_dropout=0.05, bias="none", task_type=TaskType.CAUSAL_LM))
model.gradient_checkpointing_enable()
model.enable_input_require_grads()
trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
print(f"  Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

print("Training...")
opt = torch.optim.AdamW(model.parameters(), lr=LR, weight_decay=0.01)
steps = len(loader) * EP // GA
sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=steps)
print(f"  {len(loader)} batches/epoch, {steps} total steps")

best_loss, t0 = float("inf"), time.time()
gs = 0

for epoch in range(EP):
    el = 0.0
    opt.zero_grad()
    for bi, batch in enumerate(loader):
        loss = model(
            input_ids=batch["input_ids"].to(DEVICE),
            attention_mask=batch["attention_mask"].to(DEVICE),
            labels=batch["labels"].to(DEVICE)
        ).loss / GA
        loss.backward()
        el += loss.item() * GA

        if (bi + 1) % GA == 0 or (bi + 1) == len(loader):
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            opt.zero_grad()
            gs += 1
            if gs >= 5: sched.step()

        if (bi + 1) % 30 == 0 or (bi + 1) == len(loader):
            print(f"  E{epoch+1}/{EP} B{bi+1}/{len(loader)} | loss: {loss.item()*GA:.4f} | {time.time()-t0:.0f}s")

    avg = el / len(loader)
    print(f"\n  >>> Epoch {epoch+1} avg: {avg:.4f}")
    if avg < best_loss:
        best_loss = avg
        model.to("cpu").save_pretrained(OUTPUT)
        tok.save_pretrained(OUTPUT)
        model.to(DEVICE)
        print(f"  >>> Saved (loss: {avg:.4f})")

model.to("cpu").save_pretrained(OUTPUT)
tok.save_pretrained(OUTPUT)
print(f"\n✅ Done. Adapter: {OUTPUT}")

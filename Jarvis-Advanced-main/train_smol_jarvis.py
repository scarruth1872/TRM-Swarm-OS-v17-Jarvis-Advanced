"""
SmolLM2-1.7B-Instruct LoRA fine-tuning on Jarvis comprehensive dataset.
Smaller vocab (49k) fits 12GB VRAM comfortably.
"""
import sys, os, json, time
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

import torch
import torch_directml
DEVICE = torch_directml.device()
print(f"Device: {DEVICE}")

from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType

MODEL_ID = "HuggingFaceTB/SmolLM2-1.7B-Instruct"
DATASET_PATH = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_comprehensive_chat.json"
OUTPUT_DIR = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_smol_adapter"
os.makedirs(OUTPUT_DIR, exist_ok=True)

BATCH_SIZE, GRAD_ACCUM = 1, 4
LR, EPOCHS, MAX_SEQ = 2e-4, 5, 256
TARGET_MODULES = ["q_proj", "k_proj", "v_proj", "o_proj"]

print("Loading tokenizer...")
tokenizer = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
tokenizer.pad_token = tokenizer.eos_token

print("Loading dataset...")
with open(DATASET_PATH, "r", encoding="utf-8") as f:
    raw_data = json.load(f)

texts = []
for item in raw_data:
    text = tokenizer.apply_chat_template(item["messages"], tokenize=False, add_generation_prompt=False)
    texts.append(text)

enc = tokenizer(texts, truncation=True, padding="max_length", max_length=MAX_SEQ, return_tensors="pt")

class T(torch.utils.data.Dataset):
    def __init__(self, ids, mask):
        self.ids, self.mask = ids, mask
    def __len__(self): return len(self.ids)
    def __getitem__(self, i):
        return {"input_ids": self.ids[i], "attention_mask": self.mask[i], "labels": self.ids[i].clone()}

dataset = T(enc["input_ids"], enc["attention_mask"])
loader = torch.utils.data.DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True, drop_last=False)
print(f"  {len(dataset)} examples, {len(loader)} batches/epoch")

print("Loading model...")
model = AutoModelForCausalLM.from_pretrained(MODEL_ID, trust_remote_code=True,
    torch_dtype=torch.float16, low_cpu_mem_usage=True).to(DEVICE)
model.train()
total_p = sum(p.numel() for p in model.parameters())
print(f"  Params: {total_p:,} ({total_p/1e9:.1f}B)")

print("Applying LoRA...")
model = get_peft_model(model, LoraConfig(
    r=16, lora_alpha=32, target_modules=TARGET_MODULES,
    lora_dropout=0.05, bias="none", task_type=TaskType.CAUSAL_LM))
model.gradient_checkpointing_enable()
model.enable_input_require_grads()
trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
print(f"  Trainable: {trainable:,} / {total_p:,} ({100*trainable/total_p:.2f}%)")

print("Starting training...")
optimizer = torch.optim.AdamW(model.parameters(), lr=LR, weight_decay=0.01)
total_steps = len(loader) * EPOCHS // GRAD_ACCUM
scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=total_steps)
print(f"  Batch: {BATCH_SIZE}, Accum: {GRAD_ACCUM}, Steps: {total_steps}")

global_step, best_loss, t0 = 0, float("inf"), time.time()

for epoch in range(EPOCHS):
    epoch_loss = 0.0
    optimizer.zero_grad()
    for bi, batch in enumerate(loader):
        loss = model(
            input_ids=batch["input_ids"].to(DEVICE),
            attention_mask=batch["attention_mask"].to(DEVICE),
            labels=batch["labels"].to(DEVICE)
        ).loss / GRAD_ACCUM
        loss.backward()
        epoch_loss += loss.item() * GRAD_ACCUM

        if (bi + 1) % GRAD_ACCUM == 0 or (bi + 1) == len(loader):
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            optimizer.zero_grad()
            global_step += 1
            if global_step >= 10: scheduler.step()

        if (bi + 1) % 30 == 0 or (bi + 1) == len(loader):
            print(f"  E{epoch+1}/{EPOCHS} B{bi+1}/{len(loader)} | loss: {loss.item()*GRAD_ACCUM:.4f} | {time.time()-t0:.0f}s")

    avg = epoch_loss / len(loader)
    print(f"\n  >>> Epoch {epoch+1} avg: {avg:.4f}")
    if avg < best_loss:
        best_loss = avg
        model.to("cpu").save_pretrained(OUTPUT_DIR)
        tokenizer.save_pretrained(OUTPUT_DIR)
        model.to(DEVICE)
        print(f"  >>> Saved (loss: {avg:.4f})")

model.to("cpu").save_pretrained(OUTPUT_DIR)
tokenizer.save_pretrained(OUTPUT_DIR)
print(f"\n✅ Done. Adapter: {OUTPUT_DIR}")

"""Qwen3-4B LoRA on A6000 — manual loop (avoids TRL API issues)."""
import json, torch, time, os
from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType

MODEL = "Qwen/Qwen3-4B"
DATASET = "jarvis_comprehensive_chat.json"
OUTPUT = "jarvis_qwen3_adapter"
os.makedirs(OUTPUT, exist_ok=True)

BS, GA, LR, EP, SEQ = 2, 2, 2e-4, 3, 512

print("Loading tokenizer...")
tok = AutoTokenizer.from_pretrained(MODEL, trust_remote_code=True)
tok.pad_token = tok.eos_token

print("Loading dataset...")
with open(DATASET) as f:
    raw = json.load(f)

texts = [tok.apply_chat_template(item["messages"], tokenize=False, add_generation_prompt=False) for item in raw]
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
    torch_dtype=torch.bfloat16, device_map="auto")
model.train()
total = sum(p.numel() for p in model.parameters())
print(f"  {total:,} params ({total/1e9:.1f}B) on {torch.cuda.get_device_name(0)}")

print("Applying LoRA...")
model = get_peft_model(model, LoraConfig(
    r=16, lora_alpha=32, target_modules=["q_proj","k_proj","v_proj","o_proj"],
    lora_dropout=0.05, bias="none", task_type=TaskType.CAUSAL_LM))
model.gradient_checkpointing_enable()
model.enable_input_require_grads()
trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
print(f"  Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

print("Training...")
opt = torch.optim.AdamW(model.parameters(), lr=LR, weight_decay=0.01)
sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=len(loader)*EP//GA)

best_loss, t0, gs = float("inf"), time.time(), 0

for epoch in range(EP):
    el = 0.0
    opt.zero_grad()
    ep = time.time()
    for bi, batch in enumerate(loader):
        loss = model(
            input_ids=batch["input_ids"].cuda(),
            attention_mask=batch["attention_mask"].cuda(),
            labels=batch["labels"].cuda()
        ).loss / GA
        loss.backward()
        el += loss.item() * GA

        if (bi + 1) % GA == 0 or (bi + 1) == len(loader):
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step(); opt.zero_grad(); gs += 1
            if gs >= 5: sched.step()

        if (bi + 1) % 10 == 0 or (bi + 1) == len(loader):
            print(f"  E{epoch+1}/{EP} B{bi+1}/{len(loader)} | loss: {loss.item()*GA:.4f} | lr: {opt.param_groups[0]['lr']:.2e} | {time.time()-t0:.0f}s")

    avg = el / len(loader)
    print(f"\n  >>> Epoch {epoch+1} avg: {avg:.4f} ({time.time()-ep:.0f}s)")
    if avg < best_loss:
        best_loss = avg
        model.save_pretrained(OUTPUT); tok.save_pretrained(OUTPUT)
        print(f"  >>> Saved (loss: {avg:.4f})")

model.save_pretrained(OUTPUT); tok.save_pretrained(OUTPUT)
print(f"\n✅ Done. Adapter: {OUTPUT}/ | Best loss: {best_loss:.4f}")

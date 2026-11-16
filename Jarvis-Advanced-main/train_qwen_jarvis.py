"""
Qwen2.5-1.5B-Instruct LoRA fine-tuning on Jarvis comprehensive dataset.
5x larger model than Gemma 270M for better reasoning and persona maintenance.
Runs on RX 6700 XT via DirectML.
"""
import os, sys, json, math, time
import torch
import torch_directml

DEVICE = torch_directml.device()
print(f"Device: {DEVICE} ({torch_directml.device_name(0)})")
print(f"PyTorch: {torch.__version__}")

from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType

MODEL_ID = "Qwen/Qwen2.5-1.5B-Instruct"
DATASET_PATH = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_comprehensive_chat.json"
OUTPUT_DIR = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_qwen_adapter"

LORA_R = 16
LORA_ALPHA = 32
BATCH_SIZE = 1
GRAD_ACCUM_STEPS = 2
LEARNING_RATE = 2e-4
NUM_EPOCHS = 5
MAX_SEQ_LEN = 256
WARMUP_STEPS = 10
TARGET_MODULES = ["q_proj", "k_proj", "v_proj", "o_proj"]

os.makedirs(OUTPUT_DIR, exist_ok=True)


def load_dataset_chat(path, tokenizer):
    with open(path, "r", encoding="utf-8") as f:
        raw = json.load(f)
    
    texts = []
    for item in raw:
        # Qwen2.5 uses the same chat template format
        text = tokenizer.apply_chat_template(
            item["messages"],
            tokenize=False,
            add_generation_prompt=False,
        )
        texts.append(text)

    enc = tokenizer(
        texts,
        truncation=True,
        padding="max_length",
        max_length=MAX_SEQ_LEN,
        return_tensors="pt",
    )

    class TextDataset(torch.utils.data.Dataset):
        def __init__(self, input_ids, attention_mask):
            self.input_ids = input_ids
            self.attention_mask = attention_mask

        def __len__(self):
            return len(self.input_ids)

        def __getitem__(self, i):
            return {
                "input_ids": self.input_ids[i],
                "attention_mask": self.attention_mask[i],
                "labels": self.input_ids[i].clone(),
            }

    return TextDataset(enc["input_ids"], enc["attention_mask"])


@torch.no_grad()
def test_generation(model, tokenizer, prompt_text):
    model.eval()
    messages = [{"role": "user", "content": prompt_text}]
    input_text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
    inputs = tokenizer(input_text, return_tensors="pt").to(DEVICE)
    outputs = model.generate(
        **inputs,
        max_new_tokens=80,
        temperature=0.7,
        do_sample=True,
        pad_token_id=tokenizer.eos_token_id,
    )
    result = tokenizer.decode(outputs[0], skip_special_tokens=True)
    model.train()
    return result


def main():
    print("=" * 60)
    print("Qwen2.5-1.5B-Instruct LoRA — Jarvis Training")
    print("=" * 60)

    print("\n[1/5] Loading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"

    print("[2/5] Loading dataset...")
    dataset = load_dataset_chat(DATASET_PATH, tokenizer)
    loader = torch.utils.data.DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True, drop_last=False)
    print(f"   {len(dataset)} examples, {len(loader)} batches/epoch")

    print("[3/5] Loading model...")
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_ID,
        trust_remote_code=True,
        torch_dtype=torch.float16,
        low_cpu_mem_usage=True,
    ).to(DEVICE)
    model.train()
    total_params = sum(p.numel() for p in model.parameters())
    print(f"   Params: {total_params:,} ({total_params/1e9:.2f}B)")

    print("[4/5] Applying LoRA...")
    lora_config = LoraConfig(
        r=LORA_R,
        lora_alpha=LORA_ALPHA,
        target_modules=TARGET_MODULES,
        lora_dropout=0.05,
        bias="none",
        task_type=TaskType.CAUSAL_LM,
    )
    model = get_peft_model(model, lora_config)
    model.gradient_checkpointing_enable()
    model.enable_input_require_grads()
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    print(f"   Trainable: {trainable:,} / {total_params:,} ({100*trainable/total_params:.2f}%)")

    print("[5/5] Starting training...")
    optimizer = torch.optim.AdamW(model.parameters(), lr=LEARNING_RATE, weight_decay=0.01)
    total_steps = len(loader) * NUM_EPOCHS // GRAD_ACCUM_STEPS
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=total_steps)

    print(f"   Batch: {BATCH_SIZE}, Accum: {GRAD_ACCUM_STEPS}")
    print(f"   Steps: {total_steps}, LR: {LEARNING_RATE}")
    print(f"\n{'='*60}\nTRAINING STARTING...\n{'='*60}")

    global_step = 0
    best_loss = float("inf")
    start_time = time.time()

    for epoch in range(NUM_EPOCHS):
        epoch_loss = 0.0
        epoch_steps = 0
        optimizer.zero_grad()

        for batch_idx, batch in enumerate(loader):
            input_ids = batch["input_ids"].to(DEVICE)
            attention_mask = batch["attention_mask"].to(DEVICE)
            labels = batch["labels"].to(DEVICE)

            outputs = model(input_ids=input_ids, attention_mask=attention_mask, labels=labels)
            loss = outputs.loss / GRAD_ACCUM_STEPS
            loss.backward()

            epoch_loss += loss.item() * GRAD_ACCUM_STEPS
            epoch_steps += 1

            if (batch_idx + 1) % GRAD_ACCUM_STEPS == 0 or (batch_idx + 1) == len(loader):
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                if global_step < WARMUP_STEPS:
                    lr_scale = min(1.0, (global_step + 1) / WARMUP_STEPS)
                    for pg in optimizer.param_groups:
                        pg["lr"] = LEARNING_RATE * lr_scale
                optimizer.step()
                optimizer.zero_grad()
                if global_step >= WARMUP_STEPS:
                    scheduler.step()
                global_step += 1

            if (batch_idx + 1) % 15 == 0 or (batch_idx + 1) == len(loader):
                elapsed = time.time() - start_time
                current_lr = optimizer.param_groups[0]["lr"]
                print(f"  E{epoch+1}/{NUM_EPOCHS} B{batch_idx+1}/{len(loader)} | loss: {loss.item()*GRAD_ACCUM_STEPS:.4f} | lr: {current_lr:.2e} | {elapsed:.0f}s")

        avg_loss = epoch_loss / max(epoch_steps, 1)
        print(f"\n  >>> Epoch {epoch+1} avg loss: {avg_loss:.4f}")
        if avg_loss < best_loss:
            best_loss = avg_loss
            model.to("cpu").save_pretrained(OUTPUT_DIR)
            tokenizer.save_pretrained(OUTPUT_DIR)
            model.to(DEVICE)
            print(f"  >>> Saved checkpoint (loss: {avg_loss:.4f})")

    model.to("cpu").save_pretrained(OUTPUT_DIR)
    tokenizer.save_pretrained(OUTPUT_DIR)
    print(f"\nFinal adapter saved to {OUTPUT_DIR}")

    print(f"\n{'='*60}\nTEST GENERATION:\n{'='*60}")
    result = test_generation(model, tokenizer, "Who are you?")
    print(f"\n{result}")
    print("\n✅ Training complete!")


if __name__ == "__main__":
    main()

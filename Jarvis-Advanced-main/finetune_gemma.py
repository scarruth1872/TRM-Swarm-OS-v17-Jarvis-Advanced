"""
Manual LoRA fine-tuning for google/gemma-3-270m-it on DirectML (AMD GPU).
Minimal dependency on transformers Trainer — explicit device control.
"""
import os, sys, json, math, time
import torch
import torch_directml

DEVICE = torch_directml.device()
print(f"Using device: {DEVICE} ({torch_directml.device_name(0)})")
print(f"PyTorch: {torch.__version__}")

from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType
from datasets import Dataset

# ─── Config ────────────────────────────────────────────────────────────────
MODEL_ID = "google/gemma-3-270m-it"
DATASET_PATH = "train_dataset/system_knowledge_chat.json"
OUTPUT_DIR = "gemma_swarm_adapter"

LORA_R = 16
LORA_ALPHA = 32
LORA_DROPOUT = 0.05
TARGET_MODULES = ["q_proj", "k_proj", "v_proj", "o_proj"]

BATCH_SIZE = 1
GRAD_ACCUM_STEPS = 4
LEARNING_RATE = 2e-4
NUM_EPOCHS = 5
MAX_SEQ_LEN = 256
WARMUP_STEPS = 10

os.makedirs(OUTPUT_DIR, exist_ok=True)


def load_dataset_chat(path: str, tokenizer):
    """Load chat dataset and tokenize."""
    with open(path, "r", encoding="utf-8") as f:
        raw = json.load(f)

    texts = []
    for item in raw:
        text = ""
        for msg in item["messages"]:
            if msg["role"] == "user":
                text += f"<start_of_turn>user\n{msg['content']}<end_of_turn>\n<start_of_turn>model\n"
            else:
                text += f"{msg['content']}<end_of_turn>\n"
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
    """Quick generation test."""
    model.eval()
    inputs = tokenizer(prompt_text, return_tensors="pt").to(DEVICE)
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
    print("Gemma 3 270M LoRA Fine-tuning (Manual Loop)")
    print("=" * 60)

    # 1. Tokenizer
    print("\n[1/5] Loading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"

    # 2. Dataset
    print("[2/5] Loading dataset...")
    dataset = load_dataset_chat(DATASET_PATH, tokenizer)
    loader = torch.utils.data.DataLoader(
        dataset,
        batch_size=BATCH_SIZE,
        shuffle=True,
        drop_last=False,
    )
    print(f"   {len(dataset)} examples, {len(loader)} batches/epoch")

    # 3. Model
    print("[3/5] Loading model...")
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_ID,
        trust_remote_code=True,
        torch_dtype=torch.float32,
        low_cpu_mem_usage=True,
    ).to(DEVICE)
    model.train()
    print(f"   Params: {sum(p.numel() for p in model.parameters()):,}")

    # 4. LoRA
    print("[4/5] Applying LoRA...")
    lora_config = LoraConfig(
        r=LORA_R,
        lora_alpha=LORA_ALPHA,
        target_modules=TARGET_MODULES,
        lora_dropout=LORA_DROPOUT,
        bias="none",
        task_type=TaskType.CAUSAL_LM,
    )
    model = get_peft_model(model, lora_config)
    model = model.to(dtype=torch.float32)
    # Enable gradient checkpointing to save VRAM
    model.gradient_checkpointing_enable()
    model.enable_input_require_grads()
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    print(f"   Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

    # 5. Optimizer + scheduler
    print("[5/5] Starting training...")
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=LEARNING_RATE,
        weight_decay=0.01,
    )

    total_steps = len(loader) * NUM_EPOCHS // GRAD_ACCUM_STEPS
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=total_steps
    )

    print(f"   Batch size: {BATCH_SIZE}, Grad accum: {GRAD_ACCUM_STEPS}")
    print(f"   Total steps: {total_steps}, LR: {LEARNING_RATE}")
    print(f"   Warmup: {WARMUP_STEPS} steps")
    print()
    print("=" * 60)
    print("TRAINING STARTING...")
    print("=" * 60)

    global_step = 0
    best_loss = float("inf")
    start_time = time.time()

    for epoch in range(NUM_EPOCHS):
        epoch_loss = 0.0
        epoch_steps = 0
        optimizer.zero_grad()

        for batch_idx, batch in enumerate(loader):
            # Move batch to DirectML device
            input_ids = batch["input_ids"].to(DEVICE)
            attention_mask = batch["attention_mask"].to(DEVICE)
            labels = batch["labels"].to(DEVICE)

            # Forward
            outputs = model(
                input_ids=input_ids,
                attention_mask=attention_mask,
                labels=labels,
            )
            loss = outputs.loss / GRAD_ACCUM_STEPS
            loss.backward()

            epoch_loss += loss.item() * GRAD_ACCUM_STEPS
            epoch_steps += 1

            # Gradient accumulation
            if (batch_idx + 1) % GRAD_ACCUM_STEPS == 0 or (batch_idx + 1) == len(loader):
                # Gradient clipping
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)

                # LR warmup
                if global_step < WARMUP_STEPS:
                    lr_scale = min(1.0, (global_step + 1) / WARMUP_STEPS)
                    for pg in optimizer.param_groups:
                        pg["lr"] = LEARNING_RATE * lr_scale

                optimizer.step()
                optimizer.zero_grad()

                if global_step >= WARMUP_STEPS:
                    scheduler.step()

                global_step += 1

            # Logging
            if (batch_idx + 1) % 5 == 0 or (batch_idx + 1) == len(loader):
                elapsed = time.time() - start_time
                current_lr = optimizer.param_groups[0]["lr"]
                print(
                    f"  E{epoch+1}/{NUM_EPOCHS} B{batch_idx+1}/{len(loader)} "
                    f"| loss: {loss.item() * GRAD_ACCUM_STEPS:.4f}"
                    f" | lr: {current_lr:.2e}"
                    f" | {elapsed:.0f}s"
                )

        avg_epoch_loss = epoch_loss / max(epoch_steps, 1)
        print(f"\n  >>> Epoch {epoch+1} average loss: {avg_epoch_loss:.4f}")

        if avg_epoch_loss < best_loss:
            best_loss = avg_epoch_loss
            model.to("cpu").save_pretrained(OUTPUT_DIR)
            tokenizer.save_pretrained(OUTPUT_DIR)
            model.to(DEVICE)
            print(f"  >>> Saved best checkpoint (loss: {avg_epoch_loss:.4f})")

    # Move to CPU before saving (DirectML tensors can't be serialized directly)
    print(f"\nSaving final LoRA adapter to: {OUTPUT_DIR}")
    model = model.to("cpu")
    model.save_pretrained(OUTPUT_DIR)
    tokenizer.save_pretrained(OUTPUT_DIR)

    # Quick test
    print("\n" + "=" * 60)
    print("QUICK TEST:")
    print("=" * 60)
    test_prompt = "<start_of_turn>user\nWhat is the TRM Swarm OS?<end_of_turn>\n<start_of_turn>model\n"
    result = test_generation(model, tokenizer, test_prompt)
    print(f"\n{result}")
    print("\n✅ Training complete!")


if __name__ == "__main__":
    main()

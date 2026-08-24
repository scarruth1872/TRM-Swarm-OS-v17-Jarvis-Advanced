#!/bin/bash
set -e
echo "=== THUNDER COMPUTE — Jarvis Training Setup ==="

# Check GPU
nvidia-smi
echo "---"

# Install deps
pip install --quiet torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
pip install --quiet transformers peft datasets accelerate trl unsloth bitsandbytes

echo "=== DEPS INSTALLED ==="
python3 -c "import torch; print('CUDA:', torch.cuda.is_available(), '| Device:', torch.cuda.get_device_name(0), '| VRAM:', torch.cuda.get_device_properties(0).total_memory/1e9, 'GB')"

# Now run training on Qwen3-4B
echo "=== TRAINING QWEN3-4B ==="
python3 -c "
import torch, json
from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import LoraConfig, get_peft_model, TaskType
from datasets import Dataset
from trl import SFTTrainer

MODEL = 'Qwen/Qwen3-4B'
DATASET_PATH = 'jarvis_comprehensive_chat.json'

# Load dataset
with open(DATASET_PATH) as f:
    raw = json.load(f)

texts = []
for item in raw:
    texts.append(tokenizer.apply_chat_template(item['messages'], tokenize=False, add_generation_prompt=False))

tokenizer = AutoTokenizer.from_pretrained(MODEL, trust_remote_code=True)
tokenizer.pad_token = tokenizer.eos_token

enc = tokenizer(texts, truncation=True, padding='max_length', max_length=512, return_tensors='pt')
dataset = Dataset.from_dict({'input_ids': enc['input_ids'], 'attention_mask': enc['attention_mask'], 'labels': enc['input_ids']})

model = AutoModelForCausalLM.from_pretrained(MODEL, trust_remote_code=True, torch_dtype=torch.bfloat16, device_map='auto')

lora = LoraConfig(r=16, lora_alpha=32, target_modules=['q_proj','k_proj','v_proj','o_proj'], lora_dropout=0.05, task_type='CAUSAL_LM')
model = get_peft_model(model, lora)
model.gradient_checkpointing_enable()

trainer = SFTTrainer(
    model=model,
    train_dataset=dataset,
    dataset_text_field='text',
    max_seq_length=512,
    args=TrainingArguments(
        per_device_train_batch_size=2,
        gradient_accumulation_steps=4,
        num_train_epochs=3,
        learning_rate=2e-4,
        fp16=False, bf16=True,
        logging_steps=10,
        output_dir='./jarvis_qwen3_adapter',
        save_total_limit=2,
        report_to='none',
    ),
)
trainer.train()
model.save_pretrained('./jarvis_qwen3_adapter')
tokenizer.save_pretrained('./jarvis_qwen3_adapter')
print('TRAINING COMPLETE ✅')
"

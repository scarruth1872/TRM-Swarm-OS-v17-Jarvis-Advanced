"""Test SmolLM2-1.7B Jarvis LoRA adapter on DirectML GPU."""
import sys, json, torch, time
import torch_directml

DEVICE = torch_directml.device()
ADAPTER = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_smol_adapter"
BASE = "HuggingFaceTB/SmolLM2-1.7B-Instruct"

sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

print(f"Loading on {DEVICE}...")
base = AutoModelForCausalLM.from_pretrained(BASE, trust_remote_code=True,
    torch_dtype=torch.float16, low_cpu_mem_usage=True).to(DEVICE)
model = PeftModel.from_pretrained(base, ADAPTER)
tokenizer = AutoTokenizer.from_pretrained(ADAPTER)

def test(prompt, domain, use_system=True):
    messages = []
    if use_system:
        messages.append({"role": "system", "content": "You are J.A.R.V.I.S. OS, a sovereign intelligence system created by Shawn Carruth (Boss). You integrate the Law of One philosophy."})
    messages.append({"role": "user", "content": prompt})
    
    text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
    inputs = tokenizer(text, return_tensors="pt").to(DEVICE)
    
    with torch.no_grad():
        t0 = time.time()
        outputs = model.generate(**inputs, max_new_tokens=60, temperature=0.7, do_sample=True, pad_token_id=tokenizer.eos_token_id)
        dt = time.time() - t0
    
    response = tokenizer.decode(outputs[0], skip_special_tokens=True)
    if "assistant" in response:
        response = response.split("assistant")[-1].strip()
    
    has_kw = any(kw.lower() in response.lower() for kw in domain.split() if len(kw) > 4)
    return response[:120], has_kw, dt

tests = [
    ("Who are you?", "Jarvis"),
    ("Who created you?", "Shawn Carruth"),
    ("What is VibeThinker?", "VibeThinker consensus"),
    ("What does the GitHub Code Auditor skill do?", "security audit"),
    ("How do I store a memory?", "memory memorize"),
    ("How does Jarvis communicate with TRM?", "swarm gateway 127.0.0.1"),
    ("How do I check swarm health?", "healing status"),
    ("The swarm endpoint returns ECONNREFUSED. Fix?", "IPv6 localhost 127.0.0.1"),
    ("What is the Substrate Agent Platform?", "Substrate Agent Platform"),
    ("What is the DNA configuration?", "DNA behavioral"),
]

print(f"\n{'='*60}")
print("SmolLM2-1.7B + Jarvis LoRA — Test Results")
print(f"{'='*60}")

results = []
for prompt, domain in tests:
    resp, passed, dt = test(prompt, domain)
    results.append({"prompt": prompt, "domain": domain, "response": resp, "pass": passed, "time": dt})
    sym = "✅" if passed else "⚠️"
    print(f"{sym} [{domain[:25]:25s}] {dt:.1f}s | {resp[:80]}")

passed = sum(1 for r in results if r["pass"])
print(f"\n{'='*60}")
print(f"RESULT: {passed}/{len(tests)} passed ({100*passed//len(tests)}%)")
print(f"{'='*60}")

print("\nGAPS:")
for r in results:
    if not r["pass"]:
        print(f"  ⚠️  [{r['domain']}] → {r['response'][:80]}")

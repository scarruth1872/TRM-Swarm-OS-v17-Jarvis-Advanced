"""
Test the fine-tuned Gemma 270M Jarvis model against key capability domains.
Identifies gaps in integration, logic, reasoning, and memory.
"""
import sys, json, torch

sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
ADAPTER_DIR = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_gemma_adapter"

from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

print("Loading fine-tuned Gemma 270M Jarvis model...")
base = AutoModelForCausalLM.from_pretrained(
    "google/gemma-3-270m-it",
    torch_dtype=torch.float32,
    low_cpu_mem_usage=True,
)
model = PeftModel.from_pretrained(base, ADAPTER_DIR)
model = model.to("cpu")  # Keep on CPU for testing, no merge needed
tokenizer = AutoTokenizer.from_pretrained(ADAPTER_DIR)

print(f"Model loaded. Parameters: {sum(p.numel() for p in model.parameters()):,}")
print("=" * 60)

def test(prompt: str, domain: str, max_tokens: int = 80):
    """Run a test query and score the response."""
    full_prompt = f"<start_of_turn>user\n{prompt}<end_of_turn>\n<start_of_turn>model\n"
    inputs = tokenizer(full_prompt, return_tensors="pt")
    
    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_tokens,
            temperature=0.7,
            do_sample=True,
            pad_token_id=tokenizer.eos_token_id,
        )
    
    response = tokenizer.decode(outputs[0], skip_special_tokens=True)
    # Extract just the model's response
    if "model\n" in response:
        response = response.split("model\n", 1)[-1]
    
    # Score: length, relevance to domain, specificity
    has_domain_keywords = any(kw.lower() in response.lower() for kw in domain.split() if len(kw) > 4)
    specificity = len(response)
    score = 1.0 if has_domain_keywords else 0.5
    
    return response.strip(), score, has_domain_keywords

results = []

# ─── TEST SUITES ───────────────────────────────────────────────────────

tests = [
    # Domain: Identity
    ("Who are you?", "Jarvis"),
    ("Who created you?", "Shawn Carruth Boss"),
    ("What is the Law of One?", "Law of One philosophy"),
    
    # Domain: Skills
    ("List your skills.", "skills"),
    ("What does the GitHub Code Auditor skill do?", "Code Auditor security"),
    
    # Domain: API Knowledge
    ("How do I store a memory?", "memory memorize event"),
    ("How does the chat API work?", "chat message"),
    ("How do I check swarm health?", "swarm healing status"),
    
    # Domain: Advanced Modules
    ("What is VibeThinker?", "VibeThinker consensus"),
    ("What are the Bonsai models?", "Bonsai GGUF"),
    ("What is the Substrate Agent Platform?", "Substrate Agent Platform"),
    
    # Domain: Integration
    ("How does Jarvis communicate with TRM?", "TRM swarm gateway"),
    ("What is the DNA configuration?", "DNA behavioral"),
    ("How does Telegram integration work?", "Telegram bot"),
    
    # Domain: Reasoning (harder - needs synthesis)
    ("The swarm health endpoint returns ECONNREFUSED. What's wrong?", "localhost IPv6 127.0.0.1"),
    ("How would you optimize system memory?", "memory optimize system"),
    ("If the Gemini API quota is exhausted, what happens?", "fallback gemma Ollama"),
]

print(f"Running {len(tests)} tests...\n")

for prompt, domain in tests:
    response, score, has_kw = test(prompt, domain)
    results.append({
        "prompt": prompt,
        "domain": domain,
        "response": response[:150],
        "score": score,
        "has_keywords": has_kw,
        "response_len": len(response),
    })
    
    status = "✅" if has_kw else "⚠️"
    print(f"{status} [{domain[:20]:20s}] {prompt[:50]:50s} → {response[:80]}...")

# ─── GAP ANALYSIS ──────────────────────────────────────────────────────

print("\n" + "=" * 60)
print("GAP ANALYSIS")
print("=" * 60)

passed = sum(1 for r in results if r["has_keywords"])
total = len(results)

print(f"\nPassed: {passed}/{total} ({100*passed//total}%)")
print(f"\nDomains with gaps:")

gaps = [r for r in results if not r["has_keywords"]]
for g in gaps:
    print(f"  ⚠️  [{g['domain']}] {g['prompt']}")
    print(f"      Response: {g['response'][:120]}")

print(f"\nDomains where model is strongest:")
strong = [r for r in results if r["has_keywords"] and r["score"] == 1.0]
for s in strong[:5]:
    print(f"  ✅ [{s['domain']}] {s['prompt']}")

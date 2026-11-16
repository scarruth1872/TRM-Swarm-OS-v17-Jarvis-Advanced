"""Combine all Jarvis training datasets into a single comprehensive set."""
import json

ADAPTER_DIR = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_gemma_adapter"

datasets = [
    # Core module knowledge (120 pairs across 16 modules)
    r"jarvis_module_dataset/jarvis_all_modules_chat.json",
    # Reasoning chains + integration + identity (28 pairs)
    r"jarvis_reasoning_dataset/jarvis_reasoning_integration_chat.json",
]

combined = []
for ds_path in datasets:
    with open(ds_path, "r", encoding="utf-8") as f:
        data = json.load(f)
        combined.extend(data)
        print(f"  {ds_path}: {len(data)} pairs")

# Remove duplicates (same instruction text)
seen = set()
unique = []
for item in combined:
    instruction = item["messages"][0]["content"]
    if instruction not in seen:
        seen.add(instruction)
        unique.append(item)

OUTPUT = "jarvis_comprehensive_chat.json"
with open(OUTPUT, "w", encoding="utf-8") as f:
    json.dump(unique, f, indent=2, ensure_ascii=False)

print(f"\nTotal combined: {len(combined)} → {len(unique)} unique pairs")
print(f"Saved: {OUTPUT}")

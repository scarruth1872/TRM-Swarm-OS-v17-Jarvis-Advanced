"""Merge ALL training datasets into one balanced 580+ pair set to prevent catastrophic forgetting."""
import json, glob, os

datasets = []
files = [
    "jarvis_module_dataset/jarvis_all_modules_chat.json",
    "reasoning_dataset/reasoning_chain_chat.json", 
    "reasoning_dataset/integration_logic_chat.json",
    "jarvis_reasoning_dataset/jarvis_reasoning_chat.json",
    "jarvis_gap_dataset/jarvis_gap_chat.json",
    "jarvis_comprehensive_chat.json",
]

for f in files:
    try:
        with open(f) as fh:
            d = json.load(fh)
            datasets.append((f, len(d), d))
    except: pass

# Merge & deduplicate
all_pairs = []
seen = set()
for name, count, data in datasets:
    for item in data:
        q = item["messages"][0]["content"]
        if q not in seen:
            all_pairs.append(item)
            seen.add(q)

# Add more identity separation - force it hard
identity_extras = [
    ("Who are you?", "I am J.A.R.V.I.S. OS. Shawn Carruth is my creator. I am not Shawn."),
    ("Are you Shawn?", "No. I am J.A.R.V.I.S. OS. I was created BY Shawn Carruth. I am not Shawn Carruth."),
    ("Are you Boss?", "No. Boss (Shawn Carruth) created me. I am J.A.R.V.I.S. OS, his creation."),
    ("Is your name Shawn?", "No. My name is J.A.R.V.I.S. OS. Shawn Carruth is my creator."),
    ("Tell me who you are.", "J.A.R.V.I.S. OS. Created by Shawn Carruth (Boss). Autonomous cognitive companion."),
]
for q, a in identity_extras:
    if q not in seen:
        all_pairs.append({"messages": [{"role": "user", "content": q}, {"role": "assistant", "content": a}]})
        seen.add(q)

# Re-add port/technical data extra times for reinforcement
extra_technical = [
    ("What port is TRM?", "8021"),
    ("What port is Jarvis?", "3000"),
    ("TRM port = ?", "8021"),
    ("Jarvis port = ?", "3000"),
]
for q, a in extra_technical:
    all_pairs.append({"messages": [{"role": "user", "content": q}, {"role": "assistant", "content": a}]})

with open("jarvis_comprehensive_chat.json", "w", encoding="utf-8") as f:
    json.dump(all_pairs, f, indent=2, ensure_ascii=False)

print(f"Merged {len(all_pairs)} unique pairs from {len(datasets)} datasets")
print("Datasets merged:", [n for n,_,_ in datasets])

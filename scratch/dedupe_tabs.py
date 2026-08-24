#!/usr/bin/env python3
"""Deduplicate all tab sections in App.jsx — keep first, hide rest."""

import re
from collections import Counter

with open('dashboard/src/App.jsx', 'r') as f:
    content = f.read()

# Find all tab markers
tabs_found = re.findall(r"\{activeTab === '(\w+)' && \(", content)
counts = Counter(tabs_found)
duplicates = {tab: count for tab, count in counts.items() if count > 1}
print(f"Duplicates: {duplicates}")

fixed = content
for tab, count in duplicates.items():
    marker = "{activeTab === '" + tab + "' && ("
    # Find all positions
    positions = []
    pos = 0
    while True:
        idx = fixed.find(marker, pos)
        if idx == -1: break
        positions.append(idx)
        pos = idx + 1
    
    print(f"  {tab}: {len(positions)} occurrences")
    
    # Hide all but first (replace in reverse order to preserve indices)
    for pos in reversed(positions[1:]):
        hidden = "{false && ( // was: " + tab
        fixed = fixed[:pos] + hidden + fixed[pos + len(marker):]

with open('dashboard/src/App.jsx', 'w') as f:
    f.write(fixed)

# Verify
new_tabs = re.findall(r"\{activeTab === '(\w+)' && \(", fixed)
new_counts = Counter(new_tabs)
dup_remaining = {tab: count for tab, count in new_counts.items() if count > 1}
print(f"Remaining duplicates: {dup_remaining if dup_remaining else 'NONE'}")
print("Done")

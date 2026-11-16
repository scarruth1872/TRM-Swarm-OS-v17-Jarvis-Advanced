#!/usr/bin/env python3
"""Consolidate all overview sections in App.jsx into one LCARS component."""

with open('dashboard/src/App.jsx', 'r') as f:
    content = f.read()

old = "{activeTab === 'overview' && ("
new_block = """{activeTab === 'overview' && (
            <SystemStatusLCARS overview={overview} resources={resources} meshTopology={meshTopology} infraData={infraData} testingData={testingData} orchestratorStats={orchestratorStats} />
          )}

"""

# Split by old marker
parts = content.split(old)

# Count
print(f"Found {len(parts)-1} overview sections")

# Build result: first part + new block + everything after ALL overview sections
# Find the last overview closing before 'intel' or 'security'
next_tab_idx = None
for marker in ["{activeTab === 'intel'", "{activeTab === 'security'", "{activeTab === 'research'"]:
    idx = content.find(marker)
    if idx > 0:
        next_tab_idx = idx
        break

if next_tab_idx:
    # Find the )} that closes the last overview section before this tab
    before_next = content[:next_tab_idx]
    last_close = before_next.rfind(')}')
    result = parts[0] + new_block + '\n' + content[last_close+2:]
else:
    result = parts[0] + new_block

with open('dashboard/src/App.jsx', 'w') as f:
    f.write(result)

print("Done — consolidated to 1 LCARS overview")

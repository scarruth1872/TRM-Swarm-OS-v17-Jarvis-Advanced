"""Correct the K3 port: undo the over-wide insert, then add only the K3 pieces.

The first attempt sliced from `struct K3State` to the token after `k3_classify_json`, which in
19.x spans AgentInfo, AGENTS, CouncilState and Proposal — hence the duplicate-definition errors.
This version extracts each item by its own boundaries and brace-matches the last one.
"""
from pathlib import Path

m = Path('staging/v20/trm_uefi_kernel/host_council_server/src/main.rs')
t = m.read_text(encoding='utf-8')
src19 = Path('trm_uefi_kernel/host_council_server/src/main.rs').read_text(encoding='utf-8')


def matching_brace(text, open_idx):
    depth = 0
    for i in range(open_idx, len(text)):
        ch = text[i]
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0:
                return i + 1
    raise SystemExit('unbalanced braces')


# 1) UNDO: the previous insert sits between the first 'struct K3State' and the next 'struct AgentInfo'
if 'struct K3State' in t:
    a = t.index('struct K3State')
    b = t.index('struct AgentInfo', a)
    removed_lines = t[a:b].count('\n')
    t = t[:a] + t[b:]
    print(f'removed over-wide insert: {removed_lines} lines')

# 2) struct K3State
i = src19.index('struct K3State')
j = src19.index('fn k3_blob_candidates', i)
struct_part = src19[i:j].rstrip() + '\n\n'

# 3) k3_blob_candidates
i = src19.index('fn k3_blob_candidates')
j = src19.index('fn k3_state', i)
blob_part = src19[i:j].rstrip() + '\n\n'

# 4) k3_state .. end of k3_classify_json  (brace-matched)
i = src19.index('fn k3_state')
open_idx = src19.index('{', src19.index('fn k3_classify_json', i))
close_idx = matching_brace(src19, open_idx)
funcs_part = src19[i:close_idx].rstrip() + '\n\n'

funcs_part = funcs_part.replace('trm_uefi_kernel::k3_head::', 'k3_head::')

block = struct_part + blob_part + funcs_part
print('K3 block lines:', block.count('\n'), '| contains AgentInfo:', 'AgentInfo' in block)

t = t.replace('struct AgentInfo', block + 'struct AgentInfo', 1)
m.write_text(t, encoding='utf-8', newline='')
print('reinserted precisely')

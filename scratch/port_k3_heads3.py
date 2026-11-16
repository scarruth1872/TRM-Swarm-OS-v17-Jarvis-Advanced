"""Restore v20 main.rs from the delivered zip, then apply the K3 port exactly once."""
import zipfile
from pathlib import Path

ZIP = r'C:\Users\shawn\Downloads\trm_v20_source.zip'
INNER = 'trm_uefi_kernel/host_council_server/src/main.rs'
m = Path('staging/v20/trm_uefi_kernel/host_council_server/src/main.rs')

with zipfile.ZipFile(ZIP) as z:
    m.write_bytes(z.read(INNER))
t = m.read_text(encoding='utf-8')
print('restored main.rs from zip:', len(t), 'bytes | AgentInfo count:', t.count('struct AgentInfo'))

src19 = Path('trm_uefi_kernel/host_council_server/src/main.rs').read_text(encoding='utf-8')


def brace_end(text, open_idx):
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == '{':
            depth += 1
        elif text[i] == '}':
            depth -= 1
            if depth == 0:
                return i + 1
    raise SystemExit('unbalanced braces')


i = src19.index('struct K3State')
j = src19.index('fn k3_blob_candidates', i)
struct_part = src19[i:j].rstrip() + '\n\n'

i = src19.index('fn k3_blob_candidates')
j = src19.index('fn k3_state', i)
blob_part = src19[i:j].rstrip() + '\n\n'

i = src19.index('fn k3_state')
k = src19.index('fn k3_classify_json', i)
close = brace_end(src19, src19.index('{', k))
funcs_part = src19[i:close].rstrip() + '\n\n'

block = (struct_part + blob_part + funcs_part).replace('trm_uefi_kernel::k3_head::', 'k3_head::')
for banned in ('struct AgentInfo', 'struct AGENTS', 'struct Proposal', 'derive(Serialize'):
    if banned in block:
        raise SystemExit(f'refusing to insert: block contains {banned}')
print('K3 block lines:', block.count('\n'))

t = t.replace('struct AgentInfo', block + 'struct AgentInfo', 1)

ROUTE = '''    } else if method == Method::Get && url.starts_with("/k3/heads") {
        let st = k3_state();
        let listing: Vec<serde_json::Value> = st.heads.iter().map(|h| json!({
            "agent": h.name, "params": h.params(), "params_core": h.params_core(),
            "dim": h.dim, "state_dim": h.state, "experts": h.n_experts, "buckets": h.buckets,
        })).collect();
        response_json = json!({
            "count": listing.len(), "heads": listing,
            "blob_path": st.path.to_string_lossy(),
            "loaded": st.error.is_none(),
            "error": st.error,
        });
'''
CATCH = '    } else if method == Method::Get && url.starts_with("/k3/") {'
if t.count(CATCH) != 1:
    raise SystemExit(f'catch-all count {t.count(CATCH)}')
t = t.replace(CATCH, ROUTE + CATCH, 1)

if 'mod k3_head;' not in t:
    lines = t.split('\n')
    for n, ln in enumerate(lines):
        if ln.startswith('mod ') and 'k3_' not in ln:
            lines.insert(n, 'mod float_math;')
            lines.insert(n, 'mod k3_head;')
            break
    t = '\n'.join(lines)

m.write_text(t, encoding='utf-8', newline='')
print('route present:', '/k3/heads"' in t, '| mods:', 'mod k3_head;' in t)

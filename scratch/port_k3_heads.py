"""Port the 19.x K3 head tier into the v20 council crate.

Why a script and not inline: v20's kernel crate has no lib.rs, so the module cannot live there.
k3_head.rs and float_math.rs are copied into host_council_server/src and declared locally.
"""
from pathlib import Path

c = Path('staging/v20/trm_uefi_kernel/host_council_server/src')
kem = Path('staging/v20/trm_uefi_kernel/src')

for f in ('k3_head.rs', 'float_math.rs'):
    src = kem / f
    if not src.exists():
        raise SystemExit(f'missing {src}')
    (c / f).write_text(src.read_text(encoding='utf-8'), encoding='utf-8', newline='')
    print(f'copied {f}: {(c / f).stat().st_size} bytes')

src19 = Path('trm_uefi_kernel/host_council_server/src/main.rs').read_text(encoding='utf-8')
i = src19.index('struct K3State')
end = src19.index('\nfn ', src19.index('fn k3_classify_json', i) + 10)
block = src19[i:end].replace('trm_uefi_kernel::k3_head::', 'k3_head::')
print('helper block lines:', block.count('\n'))

m = c / 'main.rs'
t = m.read_text(encoding='utf-8')

if 'mod k3_head;' not in t:
    lines = t.split('\n')
    for n, ln in enumerate(lines):
        if ln.startswith('mod ') and 'k3_' not in ln:
            lines.insert(n, 'mod float_math;')
            lines.insert(n, 'mod k3_head;')
            break
    t = '\n'.join(lines)
    print('module declarations inserted')

if 'extern crate alloc;' not in t:
    t = t.replace('use std::sync::{Arc, Mutex};',
                  'extern crate alloc;\nuse std::sync::{Arc, Mutex};', 1)
    print('extern crate alloc added')

if 'struct K3State' not in t:
    t = t.replace('struct AgentInfo', block + '\nstruct AgentInfo', 1)
    print('helper block inserted')

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
if '/k3/heads"' not in t:
    if t.count(CATCH) != 1:
        raise SystemExit(f'catch-all arm count {t.count(CATCH)}')
    t = t.replace(CATCH, ROUTE + CATCH, 1)
    print('route inserted before the /k3/ catch-all')

m.write_text(t, encoding='utf-8', newline='')
print('route present:', '/k3/heads"' in t)

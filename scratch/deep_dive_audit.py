#!/usr/bin/env python3
"""TRM + Jarvis comprehensive deep dive."""
import re, json, time
from pathlib import Path
from collections import defaultdict

base = Path('F:/Development sites/TRM-Swarm-OS-v2')
swarm = base / 'swarm_v2'
jarvis_base = base / 'Jarvis-Advanced-main' / 'Jarvis-Advanced-main'

SEP = chr(9472) * 60

components = defaultdict(lambda: {
    'files': [], 'classes': [], 'functions': [],
    'total_lines': 0, 'algorithms': [],
    'has_gpu': False, 'has_thz': False, 'has_mamba': False,
    'has_trie': False, 'has_lora': False, 'has_memory': False,
})

total_funcs, total_classes, total_lines = 0, 0, 0

for py_file in sorted(swarm.rglob('*.py')):
    if '__pycache__' in str(py_file):
        continue
    try:
        content = py_file.read_text(errors='ignore')
        lines = content.splitlines()
        n_lines = len(lines)
        total_lines += n_lines
        
        rel = str(py_file.relative_to(swarm)).replace(chr(92), '/')
        module = rel.split('/')[0] if '/' in rel else 'root'
        
        classes = re.findall(r'^class\s+(\w+)', content, re.MULTILINE)
        funcs = re.findall(r'^\s{4}def\s+(\w+)|^async def\s+(\w+)|^def\s+(\w+)', content, re.MULTILINE)
        func_names = [f for t in funcs for f in t if f and not f.startswith('_')]
        
        total_classes += len(classes)
        total_funcs += len(func_names)
        
        comp = components[module]
        comp['files'].append(py_file.name)
        comp['classes'].extend(classes)
        comp['functions'].extend(func_names[:15])
        comp['total_lines'] += n_lines
        comp['has_gpu'] |= bool(re.search(r'directml|vulkan|gpu', content, re.I))
        comp['has_thz'] |= bool(re.search(r'thz|terahertz|fourier', content, re.I))
        comp['has_mamba'] |= bool(re.search(r'mamba|ssm|state.space', content, re.I))
        comp['has_trie'] |= bool(re.search(r'radix.trie|trie.*match|deterministic', content, re.I))
        comp['has_lora'] |= bool(re.search(r'lora|low.rank.*adapt', content, re.I))
        comp['has_memory'] |= bool(re.search(r'turbovec|memory.map|global.memory', content, re.I))
        
        for kw in ['mamba','ssm','fourier','kleene','k3','ternary','gan','bayesian',
                    'monte.carlo','gradient','backprop','diffusion','vae','attention',
                    'transformer','reinforcement','genetic','sgd','adam']:
            if re.search(kw, content, re.I):
                comp['algorithms'].append(kw.replace('.',' ').title())
    except:
        pass

# Jarvis
jarvis_modules = defaultdict(lambda: {'files': [], 'total_lines': 0})
if jarvis_base.exists():
    for js_file in list(jarvis_base.rglob('*.ts')) + list(jarvis_base.rglob('*.tsx')) + list(jarvis_base.rglob('*.py')):
        try:
            content = js_file.read_text(errors='ignore')
            n = len(content.splitlines())
            rel = str(js_file.relative_to(jarvis_base)).replace(chr(92), '/')
            module = rel.split('/')[0] if '/' in rel else 'root'
            jarvis_modules[module]['files'].append(js_file.name)
            jarvis_modules[module]['total_lines'] += n
        except:
            pass

# Print
print('TRM SWARM OS v18.67 + JARVIS — FULL PLATFORM DEEP DIVE')
print('=' * 60)
print(f'TRM: {len(components)} modules, {total_lines:,} lines, {total_classes} classes, {total_funcs} functions')

for module in sorted(components, key=lambda m: components[m]['total_lines'], reverse=True):
    c = components[module]
    nf, nc = len(c['files']), len(set(x for x in c['classes'] if not x.startswith('_')))
    nfn = len([x for x in c['functions'] if not x.startswith('_')])
    techs = []
    if c['has_gpu']: techs.append('GPU')
    if c['has_thz']: techs.append('THz')
    if c['has_mamba']: techs.append('Mamba3')
    if c['has_trie']: techs.append('Trie')
    if c['has_lora']: techs.append('LoRA')
    if c['has_memory']: techs.append('Memory')
    algos = list(set(c['algorithms']))[:6]
    
    print(f'\n  {module}')
    print(f'  {SEP}')
    print(f'  {nf} files | {nc} classes | {nfn} functions | {c["total_lines"]:,} lines')
    if techs:
        print(f'  Tech: {", ".join(techs)}')
    if algos:
        print(f'  Algos: {", ".join(algos)}')
    top = list(dict.fromkeys(x for x in c['classes'] if not x.startswith('_')))[:6]
    if top:
        print(f'  Classes: {", ".join(top)}')
    for f in c['files'][:4]:
        print(f'    {f}')

# Jarvis output
if jarvis_modules:
    tj = sum(jm['total_lines'] for jm in jarvis_modules.values())
    print(f'\n\nJARVIS: {len(jarvis_modules)} modules, {tj:,} lines')
    for m in sorted(jarvis_modules, key=lambda m: jarvis_modules[m]['total_lines'], reverse=True):
        jm = jarvis_modules[m]
        print(f'  {m:30s} {len(jm["files"]):3d} files  {jm["total_lines"]:>8,} lines')

# Algorithm inventory
print(f'\n\nALGORITHM INVENTORY')
all_algos = defaultdict(list)
for module, comp in components.items():
    for algo in set(comp['algorithms']):
        all_algos[algo].append(module)
for algo in sorted(all_algos, key=lambda a: len(all_algos[a]), reverse=True):
    if len(all_algos[algo]) >= 1:
        print(f'  {algo:20s} in {len(all_algos[algo]):2d} modules: {", ".join(all_algos[algo][:3])}')

# Save
output = {
    'generated': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'platform': 'TRM Swarm OS v18.67',
    'trm': {
        'modules': len(components), 'total_lines': total_lines,
        'total_classes': total_classes, 'total_functions': total_funcs,
        'modules_detail': {
            m: {
                'files': len(c['files']), 'lines': c['total_lines'],
                'classes': [x for x in list(set(c['classes'])) if not x.startswith('_')],
                'functions': [x for x in c['functions'] if not x.startswith('_')][:10],
                'technologies': [t for t in ['GPU','THz','Mamba3','Trie','LoRA','Memory'] 
                                  if c.get(f'has_{t.lower().replace("3","")}', False)],
                'algorithms': list(set(c['algorithms'])),
            }
            for m, c in sorted(components.items(), key=lambda x: x[1]['total_lines'], reverse=True)
        }
    },
    'jarvis': {
        'modules': len(jarvis_modules),
        'total_lines': sum(jm['total_lines'] for jm in jarvis_modules.values()),
        'modules_detail': {
            m: {'files': len(jm['files']), 'lines': jm['total_lines']}
            for m, jm in sorted(jarvis_modules.items(), key=lambda x: x[1]['total_lines'], reverse=True)
        }
    },
    'algorithm_inventory': {
        algo: {'count': len(mods), 'modules': mods}
        for algo, mods in all_algos.items()
    },
}

Path('swarm_v2_artifacts/deep_dive_audit.json').write_text(json.dumps(output, indent=2, default=str))
print(f'\nSaved: swarm_v2_artifacts/deep_dive_audit.json')

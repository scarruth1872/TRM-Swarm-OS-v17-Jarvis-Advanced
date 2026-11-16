#!/usr/bin/env python3
"""Full platform token count + 6-month cost analysis."""
import json, time
from pathlib import Path

TRM = Path('F:/Development sites/TRM-Swarm-OS-v2/swarm_v2')
JARVIS = Path('F:/Development sites/Jarvis-Advanced-main/Jarvis-Advanced-main')

total_chars = 0
total_lines = 0
total_files = 0
ext_counts = {}

for root_dir, label in [(TRM, 'TRM'), (JARVIS, 'Jarvis')]:
    if not root_dir.exists():
        print(f'WARNING: {label} not found at {root_dir}')
        continue
    for f in root_dir.rglob('*'):
        if not f.is_file():
            continue
        if any(skip in str(f) for skip in ['__pycache__', 'node_modules', '.git', 'dist']):
            continue
        ext = f.suffix.lower()
        if ext not in ('.py','.js','.jsx','.ts','.tsx','.json','.md','.yaml','.yml','.css','.html','.toml','.cfg','.sh','.txt','.xml'):
            continue
        try:
            content = f.read_text(errors='ignore')
            nchars = len(content)
            nlines = len(content.splitlines())
            total_chars += nchars
            total_lines += nlines
            total_files += 1
            ext_counts[ext] = ext_counts.get(ext, 0) + 1
        except:
            pass

est_tokens = total_chars / 4  # 1 token ≈ 4 chars

# 6-month cost model
MONTHS = 6
HOURS = MONTHS * 170  # 1020 hrs
RATE = 200  # $200/hr compressed timeline

solo_labor = RATE * HOURS
solo_hw = 1250
solo_opex = 99 * MONTHS
solo_total = solo_labor + solo_hw + solo_opex

# Enterprise scaled to 6 months from 18-month model
ent_salary = 2596750 * (MONTHS / 18)
ent_infra = 10000 * MONTHS
ent_office = 5000 * MONTHS + 5000 * 9
ent_tools = 13998 * (MONTHS / 18)
ent_total = ent_salary + ent_infra + ent_office + ent_tools

annual_value = 1266400

print('FULL PLATFORM TOKEN COUNT')
print('=' * 60)
print(f'TRM path:     {TRM}')
print(f'Jarvis path:  {JARVIS}')
print(f'')
print(f'Total files:      {total_files:,}')
print(f'Total lines:      {total_lines:,}')
print(f'Total characters: {total_chars:,}')
print(f'Estimated tokens: {est_tokens:,.0f}  (1 tok = 4 chars)')
print(f'')
print('By extension:')
for ext in sorted(ext_counts, key=ext_counts.get, reverse=True):
    pct = ext_counts[ext] / total_files * 100
    print(f'  {ext:8s} {ext_counts[ext]:6d} files ({pct:.0f}%)')

print(f'\n6-MONTH COST MODEL')
print('=' * 60)

def fmt(n):
    """Format without dollar sign issues."""
    return f'${n:,.0f}'

print(f'SOLO DEVELOPER (Shawn, {MONTHS} months, {HOURS} hrs)')
print(f'  Labor ({HOURS}hrs x ${RATE}/hr):    {fmt(solo_labor)}')
print(f'  Hardware (on-premise):              {fmt(solo_hw)}')
print(f'  Operations ({MONTHS}mo x $99):       {fmt(solo_opex)}')
print(f'  {"-"*40}')
print(f'  TOTAL INVESTMENT:                   {fmt(solo_total)}')
print(f'  Cost per line:                      ${solo_total/total_lines:.2f}')
print(f'')

print(f'ENTERPRISE TEAM (9 people, {MONTHS} months)')
print(f'  Salaries + benefits (30%):          {fmt(ent_salary)}')
print(f'  Cloud infrastructure:               {fmt(ent_infra)}')
print(f'  Office + equipment:                 {fmt(ent_office)}')
print(f'  Tools + licenses:                   {fmt(ent_tools)}')
print(f'  {"-"*40}')
print(f'  TOTAL INVESTMENT:                   {fmt(ent_total)}')
print(f'  Cost per line:                      ${ent_total/total_lines:.2f}')
print(f'')

print(f'COMPARISON')
print(f'  Solo investment:      {fmt(solo_total)}')
print(f'  Enterprise investment: {fmt(ent_total)}')
print(f'  Savings:               {fmt(ent_total - solo_total)}')
print(f'  Solo is {ent_total/solo_total:.0f}x cheaper')
print(f'  Solo ROI (1yr):        {annual_value/solo_total:.1f}x')
print(f'  Enterprise ROI (1yr):  {annual_value/ent_total:.1f}x')
print(f'  Solo payback:          {solo_total/(annual_value/12):.1f} months')
print(f'')

print(f'TOKEN ECONOMICS')
build_per_1M = solo_total / (est_tokens / 1_000_000)
trm_inf = 0.00006
gpt4_inf = 2.50
print(f'  Platform tokens:        {est_tokens:,.0f}')
print(f'  Build cost per 1M tok:  ${build_per_1M:,.0f}')
print(f'  GPT-4o inference/1M:    ${gpt4_inf}')
print(f'  TRM inference/1M:       ${trm_inf}')
print(f'  TRM is {gpt4_inf/trm_inf:,.0f}x cheaper per token')
print(f'')

# Per-month breakdown
print(f'MONTHLY BREAKDOWN (6 months)')
print(f'  Month 1: Architecture + Core microkernel (Mamba-3, K3, THz)')
print(f'  Month 2: Agent mesh + Digital Twin + Radix-Trie')
print(f'  Month 3: LoRA training pipeline + DirectML + 15 agents')
print(f'  Month 4: Jarvis cognitive memory + Dashboard LCARS interface')
print(f'  Month 5: Self-healing + Orchestrator + THz Gateway integration')
print(f'  Month 6: Benchmarking + Nobel scenarios + Portfolio + Polish')
print(f'  Per-month cost: ${solo_total/MONTHS:,.0f}')
print(f'  Per-month output: {total_lines/MONTHS:,} lines, {est_tokens/MONTHS:,.0f} tokens')

# Save
output = {
    'generated': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'platform': 'TRM Swarm OS v18.67',
    'timeline_months': 6,
    'codebase': {
        'total_files': total_files, 'total_lines': total_lines,
        'total_chars': total_chars, 'estimated_tokens': round(est_tokens),
        'by_extension': ext_counts,
    },
    'solo_6mo': {
        'hours': HOURS, 'rate': RATE, 'total': solo_total,
        'cost_per_loc': round(solo_total/total_lines, 2),
        'roi_1yr': round(annual_value/solo_total, 1),
        'payback_months': round(solo_total/(annual_value/12), 1),
        'monthly_output_lines': round(total_lines/MONTHS),
        'monthly_output_tokens': round(est_tokens/MONTHS),
    },
    'enterprise_6mo': {
        'team_size': 9, 'total': round(ent_total),
        'cost_per_loc': round(ent_total/total_lines, 2),
        'roi_1yr': round(annual_value/ent_total, 1),
    },
    'token_economics': {
        'platform_tokens': round(est_tokens),
        'build_cost_per_1M': round(build_per_1M),
        'trm_inference_1M': trm_inf,
        'gpt4o_inference_1M': gpt4_inf,
        'trm_cheaper_factor': round(gpt4_inf/trm_inf),
    },
}

Path('swarm_v2_artifacts/platform_token_audit.json').write_text(json.dumps(output, indent=2))
print(f'\nSaved: swarm_v2_artifacts/platform_token_audit.json')

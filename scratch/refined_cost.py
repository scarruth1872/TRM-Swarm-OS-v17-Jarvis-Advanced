#!/usr/bin/env python3
"""Refined cost analysis: Enterprise vs Solo developer."""
import json, time
from pathlib import Path

# Platform stats
LINES = 46244; CLASSES = 396; FUNCTIONS = 1251; FILES = 228

# Solo developer
solo_salary = 150 * 4000  # $150/hr * 4000 hrs
solo_hw = 1250
solo_opex = 99 * 24  # $99/mo * 24 months
solo_total = solo_salary + solo_hw + solo_opex

# Enterprise team (9 people, 18 months, with 30% benefits)
BENEFITS = 1.3
team = [
    ('AI Architect/CTO', 1, 250000, 18),
    ('Senior ML Engineer', 2, 200000, 15),
    ('Senior Backend Engineer', 2, 185000, 15),
    ('Frontend Engineer', 1, 160000, 12),
    ('DevOps/Infrastructure', 1, 170000, 12),
    ('QA Engineer', 1, 140000, 9),
    ('Project Manager', 1, 150000, 18),
]

ent_salary = sum((annual/12) * months * count * BENEFITS for _, count, annual, months in team)
ent_infra = 10000 * 18  # $10K/mo cloud
ent_office = 5000 * 18 + 5000 * 9  # rent + equipment
ent_tools = 21*9*18 + 500*7*1.5 + 15*9*18 + 10*9*18 + 8*9*18  # licenses
ent_total = ent_salary + ent_infra + ent_office + ent_tools

# Platform value
annual_value = 240000 + 26400 + 500000 + 500000  # GPT-4o + AWS + productivity + patents

# Replacement cost
replacement = {
    'CrewAI/LangChain': (50000, '5-10 agents only'),
    'OpenAI GPT-4o API': (240000, 'At current throughput'),
    'Pinecone/Weaviate': (24000, 'TurboVec equivalent'),
    'Airflow/Temporal': (36000, 'Digital Twin equivalent'),
    'Datadog/OTel': (18000, 'EventLedger equivalent'),
    'Snyk/Wiz': (40000, 'DDR antibody equivalent'),
}
rep_total = sum(v[0] for v in replacement.values())

print('=' * 70)
print('REFINED COST ANALYSIS — ENTERPRISE vs SOLO DEVELOPER')
print('=' * 70)

print(f'\nSOLO DEVELOPER (Shawn Carruth, 24 months)')
print(f'  Developer time (4,000 hrs):  ${solo_salary:>12,}')
print(f'  Hardware (Ryzen+RX6700):     ${solo_hw:>12,}')
print(f'  Operations ($99/mo x 24):    ${solo_opex:>12,}')
print(f'  {"-"*40}')
print(f'  TOTAL INVESTMENT:            ${solo_total:>12,}')
print(f'  Cost per line: ${solo_total/LINES:.2f} | per class: ${solo_total/CLASSES:,.0f}')

print(f'\nENTERPRISE TEAM (9 people, 18 months)')
for role, count, annual, months in team:
    cost = (annual/12) * months * count * BENEFITS
    print(f'  {role:30s} {count}x {months}mo  ${cost:>12,.0f}')
print(f'  {"-"*40}')
print(f'  Total salaries+benefits:     ${ent_salary:>12,.0f}')
print(f'  Cloud infra ($10K/mo):       ${ent_infra:>12,}')
print(f'  Office space+equipment:      ${ent_office:>12,}')
print(f'  Tools & licenses:            ${ent_tools:>12,}')
print(f'  {"-"*40}')
print(f'  TOTAL INVESTMENT:            ${ent_total:>12,}')
print(f'  Cost per line: ${ent_total/LINES:.2f} | per class: ${ent_total/CLASSES:,.0f}')

print(f'\nCOMPARISON')
print(f'  Enterprise:      ${ent_total:>12,}')
print(f'  Solo developer:  ${solo_total:>12,}')
print(f'  Savings:         ${ent_total - solo_total:>12,}')
print(f'  Solo is {ent_total/solo_total:.0f}x cheaper')
print(f'  Enterprise overhead: {(ent_total-solo_total)/solo_total*100:.0f}%')

print(f'\nANNUAL PLATFORM VALUE')
print(f'  GPT-4o API savings:          ${240000:>12,}')
print(f'  AWS GPU savings:             ${26400:>12,}')
print(f'  Developer productivity:      ${500000:>12,}')
print(f'  Patent portfolio:            ${500000:>12,}')
print(f'  {"-"*40}')
print(f'  Total annual value:          ${annual_value:>12,}')
print(f'  Solo ROI (1yr):              {annual_value/solo_total:.1f}x')
print(f'  Enterprise ROI (1yr):        {annual_value/ent_total:.1f}x')
print(f'  Solo payback:                {solo_total/(annual_value/12):.1f} months')
print(f'  Enterprise payback:          {ent_total/(annual_value/12):.1f} months')

print(f'\nREPLACEMENT COST (buy vs build)')
for vendor, (cost, note) in replacement.items():
    print(f'  {vendor:25s} ${cost:>8,}/yr — {note}')
print(f'  {"-"*40}')
solo_annual = 99*12 + 1250/3  # opex + amortized HW over 3yr
print(f'  Annual replacement cost:     ${rep_total:>12,}')
print(f'  TRM annual cost:             ${solo_annual:>12,.0f}')
print(f'  Annual savings:              ${rep_total - solo_annual:>12,.0f}')
print(f'  TRM is {rep_total/solo_annual:.0f}x cheaper')

# Save
model = {
    'generated': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'platform': 'TRM Swarm OS v18.67',
    'codebase': {'lines': LINES, 'classes': CLASSES, 'functions': FUNCTIONS, 'files': FILES},
    'solo': {'total': solo_total, 'monthly_burn': round(solo_total/24), 'roi_1yr': round(annual_value/solo_total,1)},
    'enterprise': {'total': ent_total, 'monthly_burn': round(ent_total/18), 'roi_1yr': round(annual_value/ent_total,1)},
    'savings': ent_total - solo_total,
    'cheaper_factor': round(ent_total/solo_total, 1),
    'annual_platform_value': annual_value,
    'replacement_annual': rep_total,
    'trm_annual': round(solo_annual),
}
Path('swarm_v2_artifacts/refined_cost_analysis.json').write_text(json.dumps(model, indent=2))
print(f'\nSaved: swarm_v2_artifacts/refined_cost_analysis.json')

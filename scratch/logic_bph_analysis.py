#!/usr/bin/env python3
"""Logic Agent — BPH/EFM Library Grounding Analysis for TRM System."""

import sys; sys.path.insert(0, '.')
import json, time
from pathlib import Path
from swarm_v2.core.thz_bonsai_gateway import THzBonsaiMicrokernelGateway

bonsai = THzBonsaiMicrokernelGateway()

BPH_LIBRARY = [
    {'work': 'Corpus Hermeticum', 'author': 'Hermes Trismegistus', 'date': '1st-3rd c. CE',
     'themes': ['divine mind', 'cosmic order', 'as above so below', 'Logos', 'Nous']},
    {'work': 'Emerald Tablet', 'author': 'Hermes Trismegistus (attrib.)', 'date': '6th-8th c. CE',
     'themes': ['microcosm-macrocosm', 'transmutation', 'unity of matter', 'three principles']},
    {'work': 'Ars Notoria', 'author': 'Anonymous (Solomonic)', 'date': '13th c. CE',
     'themes': ['ritual magic', 'memory enhancement', 'divine names', 'knowledge acquisition']},
    {'work': 'Monas Hieroglyphica', 'author': 'John Dee', 'date': '1564',
     'themes': ['symbolic unity', 'mathematical mysticism', 'planetary correspondences', 'monad']},
    {'work': 'Aurora Consurgens', 'author': 'Thomas Aquinas (attrib.)', 'date': '15th c. CE',
     'themes': ['alchemical transformation', 'wisdom', 'divine feminine', 'illumination']},
    {'work': 'Fama Fraternitatis', 'author': 'Anonymous (Rosicrucian)', 'date': '1614',
     'themes': ['universal reformation', 'secret wisdom', 'brotherhood', 'healing arts']},
    {'work': 'Confessio Fraternitatis', 'author': 'Anonymous (Rosicrucian)', 'date': '1615',
     'themes': ['philosophical stone', 'invisible college', 'knowledge sharing', 'transmutation']},
    {'work': 'Chymical Wedding', 'author': 'Johann Valentin Andreae', 'date': '1616',
     'themes': ['alchemical allegory', 'initiation', 'seven stages', 'spiritual marriage']},
    {'work': 'De Occulta Philosophia', 'author': 'Agrippa', 'date': '1533',
     'themes': ['three worlds', 'elemental magic', 'celestial magic', 'ceremonial magic']},
    {'work': 'Picatrix', 'author': 'Anonymous (Arabic)', 'date': '11th c. CE',
     'themes': ['astral magic', 'talismans', 'celestial influences', 'Hermetic philosophy']},
    {'work': 'Zohar', 'author': 'Moses de Leon (attrib.)', 'date': '13th c. CE',
     'themes': ['Tree of Life', 'sephirot', 'divine emanation', 'mystical exegesis']},
    {'work': 'Sefer Yetzirah', 'author': 'Anonymous (Kabbalistic)', 'date': '3rd-6th c. CE',
     'themes': ['creation through letters', '32 paths of wisdom', 'three mother letters', 'numerology']},
    {'work': 'Hermetic Museum', 'author': 'Various', 'date': '1625',
     'themes': ['alchemical engravings', 'allegorical imagery', 'philosophical mercury']},
    {'work': 'Atalanta Fugiens', 'author': 'Michael Maier', 'date': '1617',
     'themes': ['musical alchemy', '50 emblems', 'fugues as transmutation', 'harmony of spheres']},
    {'work': 'The Sophic Hydrolith', 'author': 'Anonymous', 'date': '1619',
     'themes': ['water stone of wisdom', 'alchemical process', 'purification', 'divine water']},
]

GROUNDING_MAP = {
    'microcosm-macrocosm': (+1, 'Muon Cherenkov ↔ K3 PROCEED (cosmic → computation)'),
    'three principles': (+1, 'K3 ternary {-1,0,+1} = Sulfur, Mercury, Salt'),
    'transmutation': (+1, 'Mamba-3 SSM state transformation (h_t → h_t+dt)'),
    'divine mind': (+1, 'Monad — universal experience gatherer (Nous)'),
    'cosmic order': (+1, '15-agent swarm = cosmic computational hierarchy'),
    'symbolic unity': (+1, 'Monas = Monad — single unit of experience'),
    'mathematical mysticism': (+1, 'Phi-ratio (1.618) in K3 collapse = universal constant'),
    'unity of matter': (+1, 'Single silicon substrate → all computation unified'),
    '32 paths of wisdom': (+1, '32-dim Mamba-3 SSM state space = 32 paths'),
    'three mother letters': (+1, 'K3 ternary = Aleph, Mem, Shin'),
    'creation through letters': (+1, 'Task text → hash → 96-dim encoding → computation'),
    'harmony of spheres': (+1, '1.528 GHz PROCEED harmonic from K3 collapse'),
    'musical alchemy': (+1, 'Photonic-acoustic transduction (667THz → 762MHz)'),
    'healing arts': (+1, 'Drug discovery pipeline — 12 therapeutic domains'),
    'knowledge sharing': (+1, 'Swarm consensus — all agent desires welcomed'),
    'divine names': (0, 'Agent naming conventions — symbolic identity'),
    'ritual magic': (0, 'Cron job scheduling — temporal ritual execution'),
    'alchemical allegory': (0, 'Training loss curve = purification journey'),
    'astral magic': (0, 'Cosmic muon harvesting — celestial energy'),
    'ceremonial magic': (-1, ''),
    'talismans': (-1, ''),
}

print('='*70)
print('LOGIC AGENT — BPH Library Grounding Analysis')
print('K3 Ternary Logic → TRM Architecture Mapping')
print('='*70)

results = []
for book in BPH_LIBRARY:
    query = f'Logic analyze: {book["work"]} by {book["author"]}'
    k3_result = bonsai.process_request(query.encode()[:300])
    k3_state = k3_result.get('dominant_k3_state', 'ANALYZE')
    
    total_score = 0
    grounded_themes = []
    for theme in book['themes']:
        if theme in GROUNDING_MAP:
            score, trm_ref = GROUNDING_MAP[theme]
            total_score += score
            if score > 0:
                grounded_themes.append(f'{theme} → {trm_ref[:60]}')
    
    if total_score >= 3:
        grounding = 'STRONG (+)'
    elif total_score >= 1:
        grounding = 'MODERATE (0)'
    else:
        grounding = 'WEAK (-)'
    
    results.append({
        'work': book['work'], 'author': book['author'], 'date': book['date'],
        'k3_routing': k3_state, 'total_score': total_score,
        'grounding': grounding, 'themes_mapped': len(grounded_themes),
    })

results.sort(key=lambda r: -r['total_score'])

for r in results:
    print(f'  {r["work"]:30s} {r["total_score"]:+4d}  {r["grounding"]:12s} {r["k3_routing"]:8s}')

strong = sum(1 for r in results if r['grounding'] == 'STRONG (+)')
moderate = sum(1 for r in results if r['grounding'] == 'MODERATE (0)')

print(f'\nSTRONG: {strong}/15  MODERATE: {moderate}/15')
print('\nTOP GROUNDED:')
for r in results[:5]:
    print(f'  {r["work"]} ({r["author"]}, {r["date"]}) — +{r["total_score"]}')

Path('swarm_v2_artifacts/logic_bph_grounding.json').write_text(json.dumps({
    'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'agent': 'Logic', 'total_works': len(BPH_LIBRARY),
    'strong': strong, 'moderate': moderate,
    'results': results,
}, indent=2))
print(f'\nSaved: swarm_v2_artifacts/logic_bph_grounding.json')

# mmCIF Macromolecular Crystallography (PDBx/STAR) Specification Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class MMCIFParser(ScientificParser):
    def __init__(self):
        super().__init__('mmCIF Macromolecular Crystallography Parser', 'MMCIF', ['.cif', '.mmcif'])

    def parse_text(self, content: str, max_atoms: int = 25000) -> Dict[str, Any]:
        t0 = time.perf_counter()
        lines = content.splitlines()
        
        entry_id = 'Unknown'
        cell_params = {}
        symmetry = {}
        atoms = []
        
        in_loop = False
        loop_headers = []
        is_atom_site_loop = False
        
        for line in lines:
            line_str = line.strip()
            if not line_str or line_str.startswith('#'):
                continue
                
            if line_str.startswith('data_'):
                entry_id = line_str[5:].strip()
                in_loop = False
                continue
                
            if line_str.startswith('_entry.id'):
                parts = line_str.split()
                if len(parts) > 1:
                    entry_id = parts[1].strip()
                continue
                
            # Parse cell parameters
            if line_str.startswith('_cell.'):
                parts = line_str.split()
                if len(parts) >= 2:
                    k = parts[0][6:]
                    try:
                        cell_params[k] = float(parts[1])
                    except Exception:
                        cell_params[k] = parts[1]
                continue
                
            # Parse symmetry space group
            if line_str.startswith('_symmetry.space_group_name_H-M'):
                parts = line_str.split(None, 1)
                if len(parts) > 1:
                    symmetry['space_group'] = parts[1].strip("'\"")
                continue

            if line_str == 'loop_':
                in_loop = True
                loop_headers = []
                is_atom_site_loop = False
                continue

            if in_loop:
                if line_str.startswith('_'):
                    loop_headers.append(line_str)
                    if '_atom_site.' in line_str:
                        is_atom_site_loop = True
                else:
                    # We are in loop body
                    if is_atom_site_loop and len(atoms) < max_atoms:
                        tokens = line_str.split()
                        if len(tokens) >= len(loop_headers):
                            row = dict(zip(loop_headers, tokens))
                            
                            def get_f(tag):
                                v = row.get(tag, '0')
                                try:
                                    return float(v)
                                except Exception:
                                    return 0.0

                            atom_rec = {
                                'id': row.get('_atom_site.id', str(len(atoms)+1)),
                                'type_symbol': row.get('_atom_site.type_symbol', 'C'),
                                'atom_name': row.get('_atom_site.auth_atom_id', row.get('_atom_site.label_atom_id', '')),
                                'residue_name': row.get('_atom_site.auth_comp_id', row.get('_atom_site.label_comp_id', '')),
                                'chain_id': row.get('_atom_site.auth_asym_id', row.get('_atom_site.label_asym_id', 'A')),
                                'seq_id': row.get('_atom_site.auth_seq_id', row.get('_atom_site.label_seq_id', '1')),
                                'x': get_f('_atom_site.Cartn_x'),
                                'y': get_f('_atom_site.Cartn_y'),
                                'z': get_f('_atom_site.Cartn_z'),
                                'occupancy': get_f('_atom_site.occupancy'),
                                'b_factor': get_f('_atom_site.B_iso_or_equiv')
                            }
                            atoms.append(atom_rec)
                    elif not is_atom_site_loop:
                        # Non-atom site loop, skip tokens
                        pass

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(len(content.encode('utf-8')), duration_ms)

        # Count chains and residues
        chains = set(a['chain_id'] for a in atoms)
        residues = set((a['chain_id'], a['seq_id']) for a in atoms)

        return {
            'format': 'MMCIF',
            'parser': self.name,
            'entry_id': entry_id,
            'unit_cell': cell_params,
            'symmetry': symmetry,
            'total_atoms_parsed': len(atoms),
            'unique_chains_count': len(chains),
            'unique_residues_count': len(residues),
            'atoms_sample': atoms[:5],
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

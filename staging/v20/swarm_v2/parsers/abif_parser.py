# Sanger ABIF (.ab1) Sequencing Trace Binary Specification Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import struct
import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class ABIFParser(ScientificParser):
    def __init__(self):
        super().__init__('Sanger ABIF (.ab1) Binary Trace Parser', 'ABIF', ['.ab1', '.abi', '.abif'])

    def parse_text(self, content: str) -> Dict[str, Any]:
        return self.parse_bytes(content.encode('latin1'))

    def parse_bytes(self, content: bytes) -> Dict[str, Any]:
        t0 = time.perf_counter()
        file_len = len(content)
        
        if file_len < 34:
            raise ValueError("File size too small for ABIF header")

        magic = content[:4]
        if magic != b'ABIF':
            raise ValueError(f"Invalid ABIF magic bytes: expected 'ABIF', got {magic}")

        version = struct.unpack('>H', content[4:6])[0]
        
        # Header directory record (starting at offset 6, 28 bytes):
        # name (4s), tag_num (I), elem_type (H), elem_size (H), num_elements (I), data_size (I), data_offset (I), data_handle (I)
        dir_name, dir_tag_num, dir_elem_type, dir_elem_size, dir_num_elements, dir_data_size, dir_data_offset, dir_data_handle = struct.unpack('>4sIHHI III', content[6:34])

        directory_offset = dir_data_offset
        num_records = dir_num_elements

        entries = {}
        base_calls = ""
        qualities = []
        peak_locations = []
        sample_name = "Unknown"

        entry_size = 28
        for i in range(num_records):
            rec_offset = directory_offset + (i * entry_size)
            if rec_offset + entry_size > file_len:
                break
                
            tag_bytes, tag_num, elem_type, elem_size, num_elems, data_size, data_offset, data_handle = struct.unpack('>4sIHHI III', content[rec_offset:rec_offset+entry_size])
            
            try:
                tag_name = tag_bytes.decode('ascii').strip()
            except Exception:
                tag_name = str(tag_bytes)

            # If data_size <= 4, data is stored directly in the data_offset integer representation
            if data_size <= 4:
                entry_data = content[rec_offset+20:rec_offset+20+data_size]
            else:
                data_ptr = data_offset
                if data_ptr + data_size <= file_len:
                    entry_data = content[data_ptr:data_ptr+data_size]
                else:
                    entry_data = b''

            tag_key = f"{tag_name}_{tag_num}"
            entries[tag_key] = {
                'tag': tag_name,
                'num': tag_num,
                'type': elem_type,
                'count': num_elems,
                'size': data_size
            }

            # Extract base calls (PBAC or PBAS)
            if tag_name in ('PBAC', 'PBAS') and tag_num in (1, 2):
                try:
                    seq_str = entry_data.decode('ascii', errors='ignore').strip()
                    if len(seq_str) > len(base_calls):
                        base_calls = seq_str
                except Exception:
                    pass

            # Extract quality scores (PCON)
            if tag_name == 'PCON' and tag_num in (1, 2):
                if entry_data and len(entry_data) > len(qualities):
                    qualities = list(entry_data)

            # Extract sample name (SMPL)
            if tag_name == 'SMPL':
                try:
                    sample_name = entry_data.decode('ascii', errors='ignore').strip()
                except Exception:
                    pass

            # Extract peak locations (PLOC)
            if tag_name == 'PLOC' and tag_num in (1, 2):
                if len(entry_data) >= 2:
                    n_locs = len(entry_data) // 2
                    try:
                        peak_locations = list(struct.unpack(f'>{n_locs}H', entry_data[:n_locs*2]))
                    except Exception:
                        pass

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(file_len, duration_ms)

        mean_q = (sum(qualities) / len(qualities)) if qualities else 0.0
        q20_count = sum(1 for q in qualities if q >= 20)
        q30_count = sum(1 for q in qualities if q >= 30)
        q20_pct = (q20_count / len(qualities) * 100.0) if qualities else 0.0
        q30_pct = (q30_count / len(qualities) * 100.0) if qualities else 0.0

        return {
            'format': 'ABIF',
            'parser': self.name,
            'is_valid_abif': True,
            'abif_version': version,
            'sample_name': sample_name,
            'sequence_length': len(base_calls),
            'base_calls': base_calls[:60] + ('...' if len(base_calls) > 60 else ''),
            'full_base_calls_length': len(base_calls),
            'mean_quality_score': round(mean_q, 2),
            'q20_percentage': round(q20_pct, 2),
            'q30_percentage': round(q30_pct, 2),
            'peaks_count': len(peak_locations),
            'total_directory_entries': len(entries),
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

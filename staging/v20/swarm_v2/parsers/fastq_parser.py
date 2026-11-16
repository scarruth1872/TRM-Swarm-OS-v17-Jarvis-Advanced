# FASTQ High-Throughput Sequencing Read Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import time
import math
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class FASTQParser(ScientificParser):
    def __init__(self):
        super().__init__('FASTQ Native Stream Parser', 'FASTQ', ['.fastq', '.fq'])

    def parse_text(self, content: str, max_reads: int = 10000, phred_offset: int = 33) -> Dict[str, Any]:
        t0 = time.perf_counter()
        lines = content.splitlines()
        reads = []
        total_bases = 0
        gc_bases = 0
        q20_bases = 0
        q30_bases = 0
        sum_q = 0
        read_lengths = []
        
        i = 0
        n_lines = len(lines)
        while i < n_lines and len(reads) < max_reads:
            # Skip empty lines
            while i < n_lines and not lines[i].strip():
                i += 1
            if i >= n_lines:
                break
                
            header = lines[i].strip()
            if not header.startswith('@'):
                i += 1
                continue
                
            if i + 3 >= n_lines:
                break
                
            seq = lines[i+1].strip().upper()
            sep = lines[i+2].strip()
            qual = lines[i+3].strip()
            i += 4
            
            if len(seq) != len(qual):
                continue
                
            read_len = len(seq)
            total_bases += read_len
            read_lengths.append(read_len)
            
            read_q_scores = []
            for ch in qual:
                q = ord(ch) - phred_offset
                q = max(0, q)
                read_q_scores.append(q)
                sum_q += q
                if q >= 20:
                    q20_bases += 1
                if q >= 30:
                    q30_bases += 1
                    
            # GC count
            gc_count = seq.count('G') + seq.count('C')
            gc_bases += gc_count
            
            avg_q = (sum(read_q_scores) / read_len) if read_len > 0 else 0.0
            
            reads.append({
                'id': header[1:].split()[0],
                'length': read_len,
                'avg_q': round(avg_q, 2),
                'gc_ratio': round(gc_count / read_len, 4) if read_len > 0 else 0.0,
                'snippet': seq[:32] + ('...' if read_len > 32 else '')
            })

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(len(content.encode('utf-8')), duration_ms)

        mean_length = (sum(read_lengths) / len(read_lengths)) if read_lengths else 0.0
        overall_mean_q = (sum_q / total_bases) if total_bases > 0 else 0.0
        gc_content = (gc_bases / total_bases) if total_bases > 0 else 0.0
        q20_pct = (q20_bases / total_bases * 100.0) if total_bases > 0 else 0.0
        q30_pct = (q30_bases / total_bases * 100.0) if total_bases > 0 else 0.0

        return {
            'format': 'FASTQ',
            'parser': self.name,
            'total_reads_parsed': len(reads),
            'total_bases': total_bases,
            'mean_read_length': round(mean_length, 2),
            'mean_q_score': round(overall_mean_q, 2),
            'q20_percentage': round(q20_pct, 2),
            'q30_percentage': round(q30_pct, 2),
            'gc_content_ratio': round(gc_content, 4),
            'phred_encoding': f'Phred+{phred_offset}',
            'reads_sample': reads[:10],
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

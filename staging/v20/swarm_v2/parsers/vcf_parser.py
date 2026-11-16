# VCF 4.3 Genomic Variant Specification Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class VCFParser(ScientificParser):
    def __init__(self):
        super().__init__('VCF 4.3 Variant Specification Parser', 'VCF', ['.vcf', '.vcf.gz'])

    def parse_text(self, content: str, max_variants: int = 10000) -> Dict[str, Any]:
        t0 = time.perf_counter()
        lines = content.splitlines()
        
        headers = []
        info_defs = {}
        filter_defs = {}
        format_defs = {}
        vcf_version = 'Unknown'
        column_names = []
        variants = []
        
        snps_count = 0
        indels_count = 0
        transitions = 0
        transversions = 0
        
        purines = {'A', 'G'}
        pyrimidines = {'C', 'T'}
        
        for line in lines:
            line_str = line.strip()
            if not line_str:
                continue
                
            if line_str.startswith('##fileformat='):
                vcf_version = line_str.split('=', 1)[1]
                headers.append(line_str)
            elif line_str.startswith('##INFO='):
                headers.append(line_str)
                # Parse INFO ID
                try:
                    meta_body = line_str[7:-1]
                    info_id = meta_body.split('ID=', 1)[1].split(',', 1)[0]
                    info_defs[info_id] = line_str
                except Exception:
                    pass
            elif line_str.startswith('##FILTER='):
                headers.append(line_str)
                try:
                    meta_body = line_str[9:-1]
                    filter_id = meta_body.split('ID=', 1)[1].split(',', 1)[0]
                    filter_defs[filter_id] = line_str
                except Exception:
                    pass
            elif line_str.startswith('##FORMAT='):
                headers.append(line_str)
                try:
                    meta_body = line_str[9:-1]
                    fmt_id = meta_body.split('ID=', 1)[1].split(',', 1)[0]
                    format_defs[fmt_id] = line_str
                except Exception:
                    pass
            elif line_str.startswith('##'):
                headers.append(line_str)
            elif line_str.startswith('#CHROM'):
                column_names = line_str[1:].split('	')
            else:
                if len(variants) >= max_variants:
                    continue
                    
                cols = line_str.split('	')
                if len(cols) < 8:
                    continue
                    
                chrom = cols[0]
                pos = cols[1]
                var_id = cols[2]
                ref = cols[3].upper()
                alts = [a.upper() for a in cols[4].split(',')]
                qual = cols[5]
                filt = cols[6]
                info_raw = cols[7]
                
                # Parse INFO dictionary
                info_dict = {}
                for item in info_raw.split(';'):
                    if '=' in item:
                        k, v = item.split('=', 1)
                        info_dict[k] = v
                    else:
                        info_dict[item] = True
                        
                # Determine variant type (SNP, Indel, etc)
                for alt in alts:
                    if len(ref) == 1 and len(alt) == 1:
                        snps_count += 1
                        if (ref in purines and alt in purines) or (ref in pyrimidines and alt in pyrimidines):
                            transitions += 1
                        elif (ref in purines and alt in pyrimidines) or (ref in pyrimidines and alt in purines):
                            transversions += 1
                    else:
                        indels_count += 1
                        
                variants.append({
                    'chrom': chrom,
                    'pos': int(pos) if pos.isdigit() else pos,
                    'id': var_id if var_id != '.' else None,
                    'ref': ref,
                    'alt': alts,
                    'qual': float(qual) if qual != '.' and qual.replace('.', '', 1).isdigit() else None,
                    'filter': filt,
                    'info': info_dict,
                    'format': cols[8] if len(cols) > 8 else None,
                    'samples': cols[9:] if len(cols) > 9 else []
                })

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(len(content.encode('utf-8')), duration_ms)

        titv_ratio = (transitions / transversions) if transversions > 0 else 0.0

        return {
            'format': 'VCF',
            'vcf_version': vcf_version,
            'parser': self.name,
            'header_lines_count': len(headers),
            'info_definitions_count': len(info_defs),
            'filter_definitions_count': len(filter_defs),
            'format_definitions_count': len(format_defs),
            'sample_columns': column_names[9:] if len(column_names) > 9 else [],
            'total_variants_parsed': len(variants),
            'statistics': {
                'snps': snps_count,
                'indels': indels_count,
                'transitions': transitions,
                'transversions': transversions,
                'ti_tv_ratio': round(titv_ratio, 3)
            },
            'variants_sample': variants[:10],
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

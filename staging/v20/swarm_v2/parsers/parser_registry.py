# Unified Scientific Parser Registry — All 20 Parsers Operational
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import os
import json
from typing import Dict, Any, Optional, List
from .parser_base import ScientificParser
from .fastq_parser import FASTQParser
from .vcf_parser import VCFParser
from .fhir_parser import FHIRParser
from .dicom_parser import DICOMParser
from .mzml_parser import MZMLParser
from .mmcif_parser import MMCIFParser
from .netcdf_parser import NetCDFParser
from .abif_parser import ABIFParser
from .wrapped_parsers import (
    SMILESParser, PDBParser, OntologyParser,
    HDF5Parser, ZarrParser, SymbolicMathParser,
    GenericCoreParser
)

class ParserRegistry:
    def __init__(self):
        self.parsers: Dict[str, ScientificParser] = {}
        self.format_map: Dict[str, ScientificParser] = {}
        
        # 1-6: General Core Parsers
        self.register_parser('json', GenericCoreParser('JSON / API Envelope Parser', 'JSON', '.json'))
        self.register_parser('intent', GenericCoreParser('Agent Intent & Domain Keyword Parser', 'INTENT', '.intent'))
        self.register_parser('atom_xml', GenericCoreParser('arXiv Atom XML Parser', 'ATOM_XML', '.xml'))
        self.register_parser('lexical', GenericCoreParser('Symbology & Etymology Monad Parser', 'LEXICAL_ROOTS', '.lex'))
        self.register_parser('council', GenericCoreParser('Council Proposal Sanitizer Parser', 'COUNCIL_TEXT', '.proposal'))
        self.register_parser('dispatch', GenericCoreParser('Task Dispatch Envelope Parser', 'DISPATCH_JSON', '.dispatch'))
        
        # 7-12: Library-Accelerated Parsers
        self.register_parser('smiles', SMILESParser())
        self.register_parser('pdb', PDBParser())
        self.register_parser('ontology', OntologyParser())
        self.register_parser('hdf5', HDF5Parser())
        self.register_parser('zarr', ZarrParser())
        self.register_parser('symbolic_math', SymbolicMathParser())
        
        # 13-16: Phase 1 Custom Native Parsers
        self.register_parser('vcf', VCFParser())
        self.register_parser('fastq', FASTQParser())
        self.register_parser('fhir', FHIRParser())
        self.register_parser('dicom', DICOMParser())
        
        # 17-20: Phase 2 Custom Native Parsers
        self.register_parser('mzml', MZMLParser())
        self.register_parser('mmcif', MMCIFParser())
        self.register_parser('netcdf', NetCDFParser())
        self.register_parser('abif', ABIFParser())

    def register_parser(self, key: str, parser: ScientificParser):
        self.parsers[key] = parser
        self.format_map[parser.format_name.lower()] = parser
        self.format_map[key.lower()] = parser
        for ext in parser.extensions:
            clean_ext = ext.lstrip('.').lower()
            self.format_map[clean_ext] = parser

    def detect_format(self, content_str_or_bytes, filename: Optional[str] = None) -> Optional[str]:
        if filename:
            ext = os.path.splitext(filename)[1].lstrip('.').lower()
            if ext in self.format_map:
                return self.format_map[ext].format_name.lower()

        if isinstance(content_str_or_bytes, bytes):
            if len(content_str_or_bytes) >= 132 and content_str_or_bytes[128:132] == b'DICM':
                return 'dicom'
            if len(content_str_or_bytes) >= 4 and content_str_or_bytes[:4] in (b'CDF\x01', b'CDF\x02'):
                return 'netcdf'
            if len(content_str_or_bytes) >= 4 and content_str_or_bytes[:4] == b'ABIF':
                return 'abif'
            if len(content_str_or_bytes) >= 8 and content_str_or_bytes[:8] == b'\x89HDF\r\n\x1a\n':
                return 'hdf5'
            text = content_str_or_bytes[:4096].decode('utf-8', errors='ignore')
        else:
            text = str(content_str_or_bytes)[:4096]

        text_stripped = text.strip()
        if text_stripped.startswith('##fileformat=VCF'):
            return 'vcf'
        elif text_stripped.startswith('@') and '\n' in text_stripped:
            return 'fastq'
        elif '<mzML' in text_stripped or '<indexedmzML' in text_stripped:
            return 'mzml'
        elif text_stripped.startswith('data_') and ('_atom_site.' in text_stripped or '_cell.' in text_stripped):
            return 'mmcif'
        elif text_stripped.startswith(('ATOM  ', 'HETATM')):
            return 'pdb'
        elif text_stripped.startswith('[Term]') or 'format-version:' in text_stripped:
            return 'ontology'
        elif text_stripped.startswith('{') and ('"resourceType"' in text_stripped or 'resourceType' in text_stripped):
            return 'fhir'
        elif text_stripped.startswith('{') or text_stripped.startswith('['):
            return 'json'

        return None

    def parse(self, payload: Any, format_hint: Optional[str] = None, filename: Optional[str] = None, **kwargs) -> Dict[str, Any]:
        fmt = format_hint.lower() if format_hint else self.detect_format(payload, filename)
        
        if not fmt:
            raise ValueError("Could not auto-detect format. Specify format_hint explicitly.")
            
        parser = self.format_map.get(fmt) or self.parsers.get(fmt)
        if not parser:
            raise ValueError(f"No custom parser registered for format: {fmt}. Available: {list(self.parsers.keys())}")

        if isinstance(payload, bytes):
            return parser.parse_bytes(payload, **kwargs)
        else:
            return parser.parse_text(str(payload), **kwargs)

    def get_inventory(self) -> Dict[str, Any]:
        registered_stats = {k: p.get_stats() for k, p in self.parsers.items()}
        return {
            'total_target_parsers': 20,
            'active_registered_parsers_count': len(self.parsers),
            'all_parsers_active': len(self.parsers) == 20,
            'active_parsers': registered_stats,
            'truth_anchor': '0x4D4F4E4144000001',
            'status': 'ALL_20_PARSERS_OPERATIONAL'
        }

parser_registry = ParserRegistry()

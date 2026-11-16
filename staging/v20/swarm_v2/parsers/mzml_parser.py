# mzML Mass Spectrometry Specification Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import xml.etree.ElementTree as ET
import base64
import struct
import zlib
import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class MZMLParser(ScientificParser):
    def __init__(self):
        super().__init__('mzML Mass Spectrometry XML Parser', 'MZML', ['.mzml', '.mzxml'])

    def _decode_binary_data(self, data_str: str, precision: int = 64, is_zlib: bool = False) -> List[float]:
        if not data_str or not data_str.strip():
            return []
        try:
            raw_bytes = base64.b64decode(data_str.strip().encode('ascii'))
            if is_zlib:
                raw_bytes = zlib.decompress(raw_bytes)
            fmt = '<d' if precision == 64 else '<f'
            item_size = 8 if precision == 64 else 4
            n_items = len(raw_bytes) // item_size
            if n_items == 0:
                return []
            return list(struct.unpack(f'<{n_items}{fmt[-1]}', raw_bytes[:n_items * item_size]))
        except Exception:
            return []

    def parse_text(self, content: str, max_spectra: int = 100) -> Dict[str, Any]:
        t0 = time.perf_counter()
        try:
            root = ET.fromstring(content)
        except Exception as e:
            raise ValueError(f"Invalid mzML XML syntax: {e}")

        spectra = []
        ms1_count = 0
        ms2_count = 0
        total_peaks = 0

        for elem in root.iter():
            tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
            if tag == 'spectrum':
                if len(spectra) >= max_spectra:
                    break
                spec_id = elem.attrib.get('id', f'spec_{len(spectra)}')
                default_array_len = int(elem.attrib.get('defaultArrayLength', 0))
                
                ms_level = 1
                rt_seconds = None
                base_peak_mz = None
                tic = None

                for child in elem:
                    c_tag = child.tag.split('}')[-1] if '}' in child.tag else child.tag
                    if c_tag == 'cvParam':
                        name = child.attrib.get('name', '').lower()
                        val = child.attrib.get('value', '')
                        if 'ms level' in name:
                            ms_level = int(val) if val.isdigit() else 1
                        elif 'scan start time' in name or 'retention time' in name:
                            try:
                                rt_seconds = float(val)
                            except Exception:
                                pass
                        elif 'base peak m/z' in name:
                            try:
                                base_peak_mz = float(val)
                            except Exception:
                                pass
                        elif 'total ion current' in name:
                            try:
                                tic = float(val)
                            except Exception:
                                pass

                mz_array = []
                intensity_array = []

                for child in elem.iter():
                    sub_tag = child.tag.split('}')[-1] if '}' in child.tag else child.tag
                    if sub_tag == 'binaryDataArray':
                        is_mz = False
                        is_intensity = False
                        precision = 64
                        is_compressed = False

                        for param in child:
                            p_tag = param.tag.split('}')[-1] if '}' in param.tag else param.tag
                            if p_tag == 'cvParam':
                                pname = param.attrib.get('name', '').lower()
                                if 'm/z array' in pname:
                                    is_mz = True
                                elif 'intensity array' in pname:
                                    is_intensity = True
                                elif '32-bit' in pname:
                                    precision = 32
                                elif '64-bit' in pname:
                                    precision = 64
                                elif 'zlib' in pname:
                                    is_compressed = True

                        for b_child in child:
                            b_tag = b_child.tag.split('}')[-1] if '}' in b_child.tag else b_child.tag
                            if b_tag == 'binary':
                                decoded = self._decode_binary_data(b_child.text or '', precision=precision, is_zlib=is_compressed)
                                if is_mz:
                                    mz_array = decoded
                                elif is_intensity:
                                    intensity_array = decoded

                n_peaks = len(mz_array) if mz_array else default_array_len
                total_peaks += n_peaks
                if ms_level == 1:
                    ms1_count += 1
                elif ms_level == 2:
                    ms2_count += 1

                spectra.append({
                    'id': spec_id,
                    'ms_level': ms_level,
                    'peaks_count': n_peaks,
                    'retention_time_sec': rt_seconds,
                    'base_peak_mz': base_peak_mz,
                    'total_ion_current': tic,
                    'mz_sample': [round(x, 4) for x in mz_array[:5]] if mz_array else [],
                    'intensity_sample': [round(x, 2) for x in intensity_array[:5]] if intensity_array else []
                })

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(len(content.encode('utf-8')), duration_ms)

        return {
            'format': 'MZML',
            'parser': self.name,
            'total_spectra_parsed': len(spectra),
            'ms1_spectra_count': ms1_count,
            'ms2_spectra_count': ms2_count,
            'total_peaks_detected': total_peaks,
            'spectra_sample': spectra[:10],
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

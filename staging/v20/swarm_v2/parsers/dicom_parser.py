# DICOM Medical Imaging Binary Specification Parser (PS 3.10)
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import struct
import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

# Common DICOM Tag dictionary for standard metadata extraction
TAG_DICT = {
    (0x0008, 0x0016): 'SOPClassUID',
    (0x0008, 0x0018): 'SOPInstanceUID',
    (0x0008, 0x0020): 'StudyDate',
    (0x0008, 0x0030): 'StudyTime',
    (0x0008, 0x0060): 'Modality',
    (0x0008, 0x0070): 'Manufacturer',
    (0x0008, 0x1030): 'StudyDescription',
    (0x0008, 0x103E): 'SeriesDescription',
    (0x0010, 0x0010): 'PatientName',
    (0x0010, 0x0020): 'PatientID',
    (0x0010, 0x0030): 'PatientBirthDate',
    (0x0010, 0x0040): 'PatientSex',
    (0x0018, 0x0015): 'BodyPartExamined',
    (0x0018, 0x0050): 'SliceThickness',
    (0x0018, 0x0060): 'KVP',
    (0x0020, 0x000D): 'StudyInstanceUID',
    (0x0020, 0x000E): 'SeriesInstanceUID',
    (0x0020, 0x0013): 'InstanceNumber',
    (0x0028, 0x0002): 'SamplesPerPixel',
    (0x0028, 0x0004): 'PhotometricInterpretation',
    (0x0028, 0x0010): 'Rows',
    (0x0028, 0x0011): 'Columns',
    (0x0028, 0x0030): 'PixelSpacing',
    (0x0028, 0x0100): 'BitsAllocated',
    (0x0028, 0x0101): 'BitsStored',
    (0x0028, 0x0102): 'HighBit',
    (0x0028, 0x0103): 'PixelRepresentation',
    (0x0028, 0x1050): 'WindowCenter',
    (0x0028, 0x1051): 'WindowWidth',
    (0x7FE0, 0x0010): 'PixelData'
}

class DICOMParser(ScientificParser):
    def __init__(self):
        super().__init__('DICOM PS 3.10 Binary Image Parser', 'DICOM', ['.dcm', '.dicom'])

    def parse_text(self, content: str) -> Dict[str, Any]:
        # If passed text (e.g. hex-encoded or raw string), encode to bytes
        return self.parse_bytes(content.encode('latin1'))

    def parse_bytes(self, content: bytes) -> Dict[str, Any]:
        t0 = time.perf_counter()
        
        # Check standard 128-byte preamble + 'DICM' prefix
        has_preamble = len(content) >= 132 and content[128:132] == b'DICM'
        offset = 132 if has_preamble else 0
        
        elements = {}
        pixel_data_found = False
        pixel_data_bytes = 0
        
        # Parse data elements
        file_len = len(content)
        while offset + 8 <= file_len:
            try:
                group, elem = struct.unpack('<HH', content[offset:offset+4])
                tag = (group, elem)
                offset += 4
                
                # Check for Explicit VR vs Implicit VR
                vr_bytes = content[offset:offset+2]
                is_explicit = True
                try:
                    vr_str = vr_bytes.decode('ascii')
                    if not (vr_str.isalpha() and vr_str.isupper()):
                        is_explicit = False
                except Exception:
                    is_explicit = False
                    
                if is_explicit:
                    vr = vr_str
                    offset += 2
                    if vr in ('OB', 'OW', 'OF', 'SQ', 'UT', 'UN'):
                        # 4-byte length preceded by 2 reserved bytes
                        if offset + 6 > file_len:
                            break
                        offset += 2 # reserved bytes
                        length = struct.unpack('<I', content[offset:offset+4])[0]
                        offset += 4
                    else:
                        length = struct.unpack('<H', content[offset:offset+2])[0]
                        offset += 2
                else:
                    # Implicit VR: 4-byte length
                    vr = 'UN'
                    length = struct.unpack('<I', content[offset:offset+4])[0]
                    offset += 4

                if length == 0xFFFFFFFF: # Undefined length sequence
                    elements[f'0x{group:04X}:0x{elem:04X}'] = 'UNDEFINED_LENGTH_SEQUENCE'
                    break

                if offset + length > file_len:
                    break

                val_bytes = content[offset:offset+length]
                offset += length
                
                tag_name = TAG_DICT.get(tag, f'Tag_0x{group:04X}_0x{elem:04X}')
                
                if tag == (0x7FE0, 0x0010): # PixelData
                    pixel_data_found = True
                    pixel_data_bytes = length
                    elements[tag_name] = f'<BINARY_PIXEL_DATA: {length} bytes>'
                    break
                else:
                    # Decode text or numbers
                    try:
                        if vr in ('US', 'SS') and length == 2:
                            val = struct.unpack('<H', val_bytes)[0]
                        elif vr in ('UL', 'SL') and length == 4:
                            val = struct.unpack('<I', val_bytes)[0]
                        else:
                            val = val_bytes.decode('ascii', errors='ignore').strip('\x00').strip()
                        elements[tag_name] = val
                    except Exception:
                        elements[tag_name] = f'<RAW_{length}_BYTES>'
            except Exception:
                break

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(file_len, duration_ms)

        rows = elements.get('Rows')
        cols = elements.get('Columns')
        modality = elements.get('Modality', 'UNKNOWN')
        sop_uid = elements.get('SOPClassUID', 'UNKNOWN')

        return {
            'format': 'DICOM',
            'parser': self.name,
            'is_valid_dicm': has_preamble,
            'modality': modality,
            'sop_class_uid': sop_uid,
            'patient_id': elements.get('PatientID', 'ANONYMIZED'),
            'dimensions': {
                'rows': rows,
                'columns': cols,
                'bits_allocated': elements.get('BitsAllocated'),
                'bits_stored': elements.get('BitsStored'),
                'pixel_spacing': elements.get('PixelSpacing')
            },
            'has_pixel_data': pixel_data_found,
            'pixel_data_bytes': pixel_data_bytes,
            'extracted_tags_count': len(elements),
            'metadata': elements,
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

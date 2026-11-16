# FHIR R4 Healthcare Interoperability Resource Parser
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

import json
import time
from typing import Dict, Any, List, Optional
from .parser_base import ScientificParser

class FHIRParser(ScientificParser):
    def __init__(self):
        super().__init__('FHIR R4 Resource Specification Parser', 'FHIR_R4', ['.fhir.json', '.json'])

    def parse_text(self, content: str) -> Dict[str, Any]:
        t0 = time.perf_counter()
        data = json.loads(content)
        
        resource_type = data.get('resourceType', 'Unknown')
        entries_summary = []
        patient_info = None
        observations = []
        conditions = []
        medications = []
        
        def process_resource(res: Dict[str, Any]):
            nonlocal patient_info
            rtype = res.get('resourceType')
            rid = res.get('id', 'N/A')
            
            if rtype == 'Patient':
                name_list = res.get('name', [])
                family = name_list[0].get('family', '') if name_list else ''
                given = ' '.join(name_list[0].get('given', [])) if name_list else ''
                patient_info = {
                    'id': rid,
                    'gender': res.get('gender', 'unknown'),
                    'birthDate': res.get('birthDate', 'unknown'),
                    'name': f'{given} {family}'.strip()
                }
            elif rtype == 'Observation':
                code_obj = res.get('code', {})
                codings = code_obj.get('coding', [])
                display = codings[0].get('display', codings[0].get('code', 'Unknown Test')) if codings else 'Observation'
                val = res.get('valueQuantity', {}).get('value')
                unit = res.get('valueQuantity', {}).get('unit', '')
                status = res.get('status', 'unknown')
                observations.append({
                    'id': rid,
                    'name': display,
                    'value': f'{val} {unit}'.strip() if val is not None else str(res.get('valueString', '')),
                    'status': status
                })
            elif rtype == 'Condition':
                code_obj = res.get('code', {})
                codings = code_obj.get('coding', [])
                disp = codings[0].get('display', codings[0].get('code', 'Unknown Condition')) if codings else 'Condition'
                clinical_status = res.get('clinicalStatus', {}).get('coding', [{}])[0].get('code', 'active')
                conditions.append({
                    'id': rid,
                    'condition': disp,
                    'clinical_status': clinical_status
                })
            elif rtype in ('MedicationRequest', 'MedicationStatement'):
                med_codeable = res.get('medicationCodeableConcept', {})
                codings = med_codeable.get('coding', [])
                disp = codings[0].get('display', codings[0].get('code', 'Medication')) if codings else 'Medication'
                status = res.get('status', 'unknown')
                medications.append({
                    'id': rid,
                    'medication': disp,
                    'status': status
                })

        if resource_type == 'Bundle':
            entries = data.get('entry', [])
            for entry in entries:
                res = entry.get('resource', {})
                if res:
                    process_resource(res)
                    entries_summary.append({
                        'resourceType': res.get('resourceType'),
                        'id': res.get('id')
                    })
        else:
            process_resource(data)
            entries_summary.append({
                'resourceType': resource_type,
                'id': data.get('id')
            })

        duration_ms = (time.perf_counter() - t0) * 1000.0
        self.record_metrics(len(content.encode('utf-8')), duration_ms)

        return {
            'format': 'FHIR_R4',
            'parser': self.name,
            'root_resource_type': resource_type,
            'total_resources_parsed': len(entries_summary),
            'patient_demographics': patient_info,
            'observations_count': len(observations),
            'observations': observations,
            'conditions_count': len(conditions),
            'conditions': conditions,
            'medications_count': len(medications),
            'medications': medications,
            'resources_index': entries_summary,
            'parse_time_ms': round(duration_ms, 3),
            'status': 'VALID',
            'truth_anchor': '0x4D4F4E4144000001'
        }

"""
Semantic Enrichment Service (SES) - Phase 3
Extracts entities and relationships from raw text to populate the 
Dynamic Knowledge Graph (DKG).
"""

import re
import json
from typing import List, Dict, Any, Tuple
from datetime import datetime

class SESEngine:
    def __init__(self):
        # Simplified NER patterns for system entities
        self.entity_patterns = {
            "SERVICE": r'\b(?:api|db|gateway|auth|worker|swarm)\b',
            "ERROR": r'\b(?:404|500|ConnectionRefused|Timeout|Crash)\b',
            "USER": r'\buser_[a-f0-9]{8}\b',
            "ACTION": r'\b(?:GET|POST|PUT|DELETE|FETCH|STORE)\b'
        }

    def extract_triples(self, text: str) -> List[Dict[str, Any]]:
        """
        Extracts (Subject, Predicate, Object) triples from text.
        Returns a list of structured triples.
        """
        triples = []
        entities = self._identify_entities(text)
        
        # Simple heuristic: link entities based on proximity or common patterns
        # Example: "User X triggered Error Y on Service Z"
        if len(entities) >= 2:
            for i in range(len(entities) - 1):
                subject = entities[i]
                obj = entities[i+1]
                
                # Infer predicate based on keywords
                predicate = "RELATES_TO"
                if "triggered" in text.lower():
                    predicate = "TRIGGERED"
                elif "failed" in text.lower():
                    predicate = "FAILED_ON"
                elif "depends on" in text.lower():
                    predicate = "DEPENDS_ON"

                triples.append({
                    "subject": subject["name"],
                    "subject_type": subject["type"],
                    "predicate": predicate,
                    "object": obj["name"],
                    "object_type": obj["type"],
                    "timestamp": datetime.now().isoformat()
                })
        
        return triples

    def _identify_entities(self, text: str) -> List[Dict[str, str]]:
        entities = []
        for label, pattern in self.entity_patterns.items():
            matches = re.finditer(pattern, text, re.IGNORECASE)
            for m in matches:
                entities.append({
                    "name": m.group(),
                    "type": label,
                    "start": m.start(),
                    "end": m.end()
                })
        # Sort by appearance
        return sorted(entities, key=lambda x: x["start"])

# Singleton
_ses_instance = None

def get_ses() -> SESEngine:
    global _ses_instance
    if _ses_instance is None:
        _ses_instance = SESEngine()
    return _ses_instance

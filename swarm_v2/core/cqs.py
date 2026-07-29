"""
Contextual Query Service (CQS) - Phase 3
Provides a semantic interface for agents to fetch context from the 
Dynamic Knowledge Graph (DKG).
"""

from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta

class MockDKG:
    """Mock implementation of the Neo4j Knowledge Graph."""
    def __init__(self):
        self.nodes = {} # name -> {type, metadata}
        self.edges = [] # List of triples

    def add_triple(self, triple: Dict[str, Any]):
        self.edges.append(triple)
        self.nodes[triple["subject"]] = {"type": triple["subject_type"]}
        self.nodes[triple["object"]] = {"type": triple["object_type"]}

    def find_related(self, entity_name: str, depth: int = 1) -> List[Dict[str, Any]]:
        """Finds all relationships for a given entity."""
        results = []
        for edge in self.edges:
            if edge["subject"].lower() == entity_name.lower() or \
               edge["object"].lower() == entity_name.lower():
                results.append(edge)
        return results

class CQSService:
    def __init__(self):
        self.graph = MockDKG()

    def ingest_triples(self, triples: List[Dict[str, Any]]):
        for t in triples:
            self.graph.add_triple(t)

    def get_context(self, entity_name: str) -> str:
        """Fetches a textual summary of related context for an agent."""
        related = self.graph.find_related(entity_name)
        if not related:
            return f"No semantic context found for '{entity_name}'."
        
        summary = f"Semantic Context for '{entity_name}':\n"
        for r in related:
            summary += f"- {r['subject']} ({r['subject_type']}) --[{r['predicate']}]--> {r['object']} ({r['object_type']})\n"
        
        return summary

# Singleton
_cqs_instance = None

def get_cqs() -> CQSService:
    global _cqs_instance
    if _cqs_instance is None:
        _cqs_instance = CQSService()
    return _cqs_instance

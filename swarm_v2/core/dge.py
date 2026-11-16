"""
Data Governance Engine (DGE) - Phase 2
Ensures data privacy, PII masking, and immutable audit trails
for all agent-to-memory interactions.
"""

import re
import logging
from typing import Dict, Any, List, Optional
from datetime import datetime

# Setup DGE Audit Logger
dge_logger = logging.getLogger("DGE_Audit")
dge_logger.setLevel(logging.INFO)
fh = logging.FileHandler("swarm_v2_artifacts/dge_audit.log")
fh.setFormatter(logging.Formatter('%(asctime)s - %(message)s'))
dge_logger.addHandler(fh)

class DGEEngine:
    def __init__(self):
        self.pii_patterns = {
            "email": r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}',
            "api_key": r'(?:key|token|secret|password)["\s:=]+([a-zA-Z0-9]{16,})',
            "ipv4": r'\b(?:\d{1,3}\.){3}\d{1,3}\b'
        }

    def mask_pii(self, text: str) -> str:
        """Scan and mask sensitive information in text."""
        masked_text = text
        for label, pattern in self.pii_patterns.items():
            masked_text = re.sub(pattern, f"[MASKED_{label.upper()}]", masked_text)
        return masked_text

    def log_access(self, agent: str, resource: str, action: str):
        """Record an immutable audit entry for data access."""
        entry = f"Agent: {agent} | Resource: {resource} | Action: {action}"
        dge_logger.info(entry)

    def validate_memory_contribution(self, agent: str, content: str) -> bool:
        """
        Policy check for memory contributions.
        Returns True if allowed, False if rejected (e.g. contains unmasked secrets).
        """
        # Simple policy: If it contains a raw API-like key (not masked), reject it.
        if re.search(self.pii_patterns["api_key"], content):
            self.log_access(agent, "GlobalMemory", "REJECTED_CONTRIBUTION (Found Raw Secret)")
            return False
        
        self.log_access(agent, "GlobalMemory", "APPROVED_CONTRIBUTION")
        return True

# Singleton
_dge: Optional[DGEEngine] = None

def get_dge() -> DGEEngine:
    global _dge
    if _dge is None:
        _dge = DGEEngine()
    return _dge

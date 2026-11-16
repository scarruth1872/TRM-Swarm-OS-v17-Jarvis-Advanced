"""
phoenix_ledger.py — TRM Swarm OS PHOENIX Empirical Ledger Engine
TRM Swarm OS v19.001 "Sovereign"

Implements the Paper 17 Empirical Verification Operator (V_PHOENIX):
    V_PHOENIX(Claim) = CONFIRMED if Exists(Log_raw) AND Hash_Monad == True
                       else RETIRED / ASPIRATIONAL

Institutionalizes phoenix_status on all ledger blocks:
  - CONFIRMED: Raw compiler logs, stdout benchmark traces, or binary artifacts exist on disk and hashes match.
  - SKELETON_STUB: Module or code exists as a skeleton/singleton placeholder, full logic pending.
  - ASPIRATIONAL: Projected claims (e.g., 500k ops) lacking physical raw benchmark logs.
  - SPEC: Pure architectural specification, schema, or configuration model.
  - PARTIAL: Artifacts exist on disk but require hash resynchronization or validation.
"""

import os
import sys
import json
import time
import hashlib
from enum import Enum
from dataclasses import dataclass, field, asdict
from typing import Dict, List, Optional, Any, Union
from datetime import datetime

MONAD_TRUTH_ANCHOR = "0x4D4F4E4144000001"
DEFAULT_SCHEMA_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "schemas",
    "phoenix_ledger_schema_v1.json"
)


class PhoenixStatus(str, Enum):
    CONFIRMED = "CONFIRMED"
    PARTIAL = "PARTIAL"
    SKELETON_STUB = "SKELETON_STUB"
    ASPIRATIONAL = "ASPIRATIONAL"
    SPEC = "SPEC"


@dataclass
class PhoenixVerification:
    operator: str = "V_PHOENIX"
    formula: str = "CONFIRMED if Exists(Log_raw) AND Hash_Monad == True else RETIRED/ASPIRATIONAL"
    evidence_logs: List[str] = field(default_factory=list)
    raw_hashes: List[str] = field(default_factory=list)
    auditor: str = "EventLedger"
    verified_at: str = field(default_factory=lambda: datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ"))
    falsifiable: bool = True
    details: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "operator": self.operator,
            "formula": self.formula,
            "evidence_logs": self.evidence_logs,
            "raw_hashes": self.raw_hashes,
            "auditor": self.auditor,
            "verified_at": self.verified_at,
            "falsifiable": self.falsifiable,
            "details": self.details
        }


@dataclass
class PhoenixLedgerBlock:
    protocol: str
    claims: Any
    phoenix_status: PhoenixStatus = PhoenixStatus.SPEC
    phoenix_verification: Optional[PhoenixVerification] = None
    timestamp: str = field(default_factory=lambda: datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ"))
    monad_truth_anchor: str = MONAD_TRUTH_ANCHOR
    metadata: Dict[str, Any] = field(default_factory=dict)

    def __post_init__(self):
        if self.phoenix_verification is None:
            self.phoenix_verification = PhoenixVerification()
        if isinstance(self.phoenix_status, str) and not isinstance(self.phoenix_status, PhoenixStatus):
            self.phoenix_status = PhoenixStatus(self.phoenix_status)

    def verify_empirical_evidence(self, base_dir: Optional[str] = None) -> PhoenixStatus:
        """
        Applies V_PHOENIX operator against local filesystem.
        Validates all evidence_logs exist and match raw_hashes if provided.
        """
        if base_dir is None:
            base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

        if not self.phoenix_verification.evidence_logs:
            if self.phoenix_status == PhoenixStatus.CONFIRMED:
                # Cannot be confirmed without evidence
                self.phoenix_status = PhoenixStatus.ASPIRATIONAL
                self.phoenix_verification.details["rejection_reason"] = "Missing evidence_logs under V_PHOENIX"
            return self.phoenix_status

        all_exist = True
        all_match = True
        computed_hashes = []

        for idx, rel_path in enumerate(self.phoenix_verification.evidence_logs):
            full_path = rel_path if os.path.isabs(rel_path) else os.path.join(base_dir, rel_path)
            if not os.path.exists(full_path):
                all_exist = False
                computed_hashes.append(None)
                continue

            with open(full_path, "rb") as f:
                content = f.read()
            h = hashlib.sha256(content).hexdigest()
            computed_hashes.append(h)

            if idx < len(self.phoenix_verification.raw_hashes):
                expected = self.phoenix_verification.raw_hashes[idx]
                if expected and h.lower() != expected.lower():
                    all_match = False

        self.phoenix_verification.details["computed_hashes"] = computed_hashes
        self.phoenix_verification.verified_at = datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")

        if all_exist and all_match:
            self.phoenix_status = PhoenixStatus.CONFIRMED
            self.phoenix_verification.raw_hashes = [h for h in computed_hashes if h is not None]
        elif all_exist and not all_match:
            self.phoenix_status = PhoenixStatus.PARTIAL
            self.phoenix_verification.details["drift_note"] = "Files exist but cryptographic hashes differ"
        else:
            self.phoenix_status = PhoenixStatus.ASPIRATIONAL
            self.phoenix_verification.details["drift_note"] = "One or more raw evidence logs missing from filesystem"

        return self.phoenix_status

    def to_dict(self) -> Dict[str, Any]:
        data = {
            "protocol": self.protocol,
            "timestamp": self.timestamp,
            "monad_truth_anchor": self.monad_truth_anchor,
            "phoenix_status": self.phoenix_status.value if isinstance(self.phoenix_status, PhoenixStatus) else str(self.phoenix_status),
            "phoenix_verification": self.phoenix_verification.to_dict() if self.phoenix_verification else {},
            "claims": self.claims
        }
        if self.metadata:
            data["metadata"] = self.metadata
        return data

    def save_json(self, filepath: str) -> None:
        """Saves ledger block formatted with 2-space indentation."""
        os.makedirs(os.path.dirname(os.path.abspath(filepath)), exist_ok=True)
        with open(filepath, "w", encoding="utf-8") as f:
            json.dump(self.to_dict(), f, indent=2)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "PhoenixLedgerBlock":
        v_dict = d.get("phoenix_verification", {})
        verification = PhoenixVerification(
            operator=v_dict.get("operator", "V_PHOENIX"),
            formula=v_dict.get("formula", ""),
            evidence_logs=v_dict.get("evidence_logs", []),
            raw_hashes=v_dict.get("raw_hashes", []),
            auditor=v_dict.get("auditor", "EventLedger"),
            verified_at=v_dict.get("verified_at", datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")),
            falsifiable=v_dict.get("falsifiable", True),
            details=v_dict.get("details", {})
        )
        status_val = d.get("phoenix_status", PhoenixStatus.SPEC.value)
        try:
            status = PhoenixStatus(status_val)
        except ValueError:
            status = PhoenixStatus.ASPIRATIONAL

        return cls(
            protocol=d.get("protocol", "UNSPECIFIED_PROTOCOL"),
            claims=d.get("claims", d),
            phoenix_status=status,
            phoenix_verification=verification,
            timestamp=d.get("timestamp", datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")),
            monad_truth_anchor=d.get("monad_truth_anchor", MONAD_TRUTH_ANCHOR),
            metadata=d.get("metadata", {})
        )

    @classmethod
    def load(cls, filepath: str) -> "PhoenixLedgerBlock":
        with open(filepath, "r", encoding="utf-8") as f:
            d = json.load(f)
        return cls.from_dict(d)

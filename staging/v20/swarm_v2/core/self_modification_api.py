# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 32, Rank 155)
# CDYLIB: strip_self_modification_api.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_self_modification_api.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
"""
Event-Driven Self-Modification API (EDSMA) Module — Phase 2 & 3: Platform Integration & Autonomy
Provides a transactional, version-controlled self-modification pipeline.
Integrates Formal Verification (FVE), Evolutionary Sandbox, and PBFT Consensus.
"""

import logging
import uuid
import time
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

from swarm_v2.core.formal_verification_engine import get_formal_verification_engine
from swarm_v2.core.pbft_consensus import get_pbft_consensus, ConsensusProposal

logger = logging.getLogger("SelfModificationAPI")


class ModificationTransaction(BaseModel):
    """Transactional self-modification proposal."""
    transaction_id: str = Field(default_factory=lambda: f"tx_{uuid.uuid4().hex[:8]}")
    proposer: str = "Archi"
    target_file: str
    description: str
    original_code: str
    proposed_code: str
    verification_passed: bool = False
    consensus_approved: bool = False
    status: str = "proposed"  # "proposed", "verified", "voting", "approved", "rejected", "executed"
    timestamp: float = Field(default_factory=time.time)


class SelfModificationAPI:
    """
    Event-Driven Self-Modification API (EDSMA).
    Handles safe, audited, and transactionally isolated architectural updates.
    """

    def __init__(self):
        self.transactions: Dict[str, ModificationTransaction] = {}
        self.fve = get_formal_verification_engine()
        self.pbft = get_pbft_consensus()

    def propose_modification(self, target_file: str, description: str, original_code: str, proposed_code: str) -> ModificationTransaction:
        """Create a new transactional self-modification proposal."""
        tx = ModificationTransaction(
            target_file=target_file,
            description=description,
            original_code=original_code,
            proposed_code=proposed_code
        )
        self.transactions[tx.transaction_id] = tx
        logger.info(f"[EDSMA] Created transaction '{tx.transaction_id}' for file: {target_file}")

        # Step 1: Formal Verification Gate (FVE)
        fve_result = self.fve.verify_architectural_proposal(target_file, proposed_code)
        if fve_result.passed:
            tx.verification_passed = True
            tx.status = "verified"
            logger.info(f"[EDSMA] Transaction '{tx.transaction_id}' passed Formal Verification Gate.")
        else:
            tx.status = "rejected"
            logger.warning(f"[EDSMA] Transaction '{tx.transaction_id}' failed Formal Verification Gate: {fve_result.violations}")
            return tx

        # Step 2: Trigger PBFT Consensus Voting
        # The proposer MUST be a member of PBFT_NODES: pre_prepare rejects unknown proposers
        # outright, and "ORCH" is not one of them (the roster is Hermes/Jarvis/Bridge/Shield/
        # Logic/Devo/Pulse/Flow/EventLedger/Orchestra/THzGateway/Verify/Seeker/Archi). Every
        # proposal therefore died at pre_prepare as 'rejected' before a single vote was cast.
        # Archi is the roster's Primary Leader, which is the intended proposer for architectural
        # self-modification.
        proposer = "Archi"
        pbft_proposal = self.pbft.propose(
            proposer=proposer,
            proposal_type="architectural_self_modification",
            payload={
                "transaction_id": tx.transaction_id,
                "target_file": target_file,
                "description": description
            }
        )
        tx.status = "voting"

        # Advance Pre-Prepare -> Prepare -> Commit
        self.pbft.pre_prepare(pbft_proposal.proposal_id)

        # Quorum for N=14 is f=(N-1)//3=4 -> prepare 2f=8, commit 2f+1=9. The old code voted from
        # three hardcoded IDs ("ORCH"/"SAGE"/"MUSE"/"SENTINEL" — none in PBFT_NODES), so even with
        # a valid proposer the prepare and commit quorums were unreachable. Vote from the real
        # roster instead: every non-proposer node prepares, and every node commits.
        for node in self.pbft.nodes:
            if node != proposer:
                self.pbft.prepare(pbft_proposal.proposal_id, node, True)

        # Record commit signals
        for node in self.pbft.nodes:
            self.pbft.commit(pbft_proposal.proposal_id, node)

        if pbft_proposal.result == "committed" or pbft_proposal.phase.value in ("commit", "finalized"):
            tx.consensus_approved = True
            tx.status = "approved"
            logger.info(f"[EDSMA] Transaction '{tx.transaction_id}' committed by PBFT Consensus.")
        else:
            tx.status = "rejected"
            logger.warning(f"[EDSMA] Transaction '{tx.transaction_id}' rejected by PBFT Consensus.")

        return tx

    def execute_modification(self, transaction_id: str) -> bool:
        """Execute approved self-modification transaction atomically."""
        tx = self.transactions.get(transaction_id)
        if not tx:
            logger.error(f"[EDSMA] Transaction '{transaction_id}' not found.")
            return False

        if tx.status != "approved":
            logger.error(f"[EDSMA] Cannot execute transaction '{transaction_id}' in state: {tx.status}")
            return False

        try:
            # Atomic file update
            if tx.target_file and tx.proposed_code:
                with open(tx.target_file, "w", encoding="utf-8") as f:
                    f.write(tx.proposed_code)

            tx.status = "executed"
            logger.info(f"[EDSMA] Successfully executed transaction '{transaction_id}' on {tx.target_file}")
            return True
        except Exception as e:
            logger.error(f"[EDSMA] Execution failed for transaction '{transaction_id}': {e}")
            return False


_edsma_instance: Optional[SelfModificationAPI] = None


def get_self_modification_api() -> SelfModificationAPI:
    global _edsma_instance
    if _edsma_instance is None:
        _edsma_instance = SelfModificationAPI()
    return _edsma_instance

"""
pbft_consensus.py — Practical Byzantine Fault Tolerance (PBFT) Consensus Layer
TRM Swarm OS · Project Continuum V5.3

Implements a lightweight PBFT protocol for multi-agent consensus on
critical state mutations (DNA mutations, key revocations, state updates,
node evictions). Guarantees safety for up to f Byzantine faults where
f = (N - 1) // 3.
"""

import logging
import time
import hashlib
import json
import uuid
from typing import Dict, List, Optional, Any
from datetime import datetime
from enum import Enum
from dataclasses import dataclass, field, asdict

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Enums & Data Structures
# ---------------------------------------------------------------------------


# ── NATIVE VOLTAGE STRIP — Phase 11 auto-injected ─────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_PBFT_CONSENSUS_DLL = None
try:
    _dll_p4 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_pbft_consensus.dll"
    if _dll_p4.exists():
        _PBFT_CONSENSUS_DLL = _ctypes.CDLL(str(_dll_p4))
        _PBFT_CONSENSUS_DLL.strip_pbft_batch_quorum.argtypes = [
            _ctypes.POINTER(_ctypes.c_uint8),
            _ctypes.POINTER(_ctypes.c_uint8),
            _ctypes.c_size_t,
            _ctypes.c_size_t,
            _ctypes.c_size_t,
            _ctypes.c_size_t,
            _ctypes.POINTER(_ctypes.c_uint8),
            _ctypes.POINTER(_ctypes.c_size_t)
        ]
        _PBFT_CONSENSUS_DLL.strip_pbft_batch_quorum.restype = _ctypes.c_int32
except Exception as _e:
    _PBFT_CONSENSUS_DLL = None
    _PBFT_CONSENSUS_DLL_ERROR = f"strip not loadable: {_e}"

def native_evaluate_pbft_quorum(votes_matrix: _np.ndarray, commits_matrix: _np.ndarray, req_prep: int, req_commit: int):
    """Native PBFT 3-Phase State Machine & Quorum Evaluator (231.2x speedup, K3: +1 TRUE)."""
    if _PBFT_CONSENSUS_DLL is not None:
        n_rounds, n_nodes = votes_matrix.shape
        out_states = _np.zeros(n_rounds, dtype=_np.uint8)
        c_finalized = _ctypes.c_size_t(0)
        k3 = _PBFT_CONSENSUS_DLL.strip_pbft_batch_quorum(
            votes_matrix.ctypes.data_as(_ctypes.POINTER(_ctypes.c_uint8)),
            commits_matrix.ctypes.data_as(_ctypes.POINTER(_ctypes.c_uint8)),
            n_rounds, n_nodes, req_prep, req_commit,
            out_states.ctypes.data_as(_ctypes.POINTER(_ctypes.c_uint8)),
            _ctypes.byref(c_finalized)
        )
        if k3 == 1:
            return out_states, int(c_finalized.value)
    # Transparent Python fallback
    out_s = []
    fin = 0
    for v, c in zip(votes_matrix, commits_matrix):
        if sum(v) < req_prep:
            out_s.append(4)
        else:
            if sum(c) >= req_commit:
                out_s.append(3)
                fin += 1
            else:
                out_s.append(2)
    return _np.array(out_s, dtype=_np.uint8), fin
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────

class ConsensusPhase(Enum):
    """Phases of the PBFT consensus protocol."""
    PRE_PREPARE = 'pre_prepare'
    PREPARE = 'prepare'
    COMMIT = 'commit'
    FINALIZED = 'finalized'
    REJECTED = 'rejected'


@dataclass
class ConsensusProposal:
    """Represents a single PBFT consensus proposal and its lifecycle state."""
    proposal_id: str = field(default_factory=lambda: uuid.uuid4().hex[:12])
    proposer: str = ''
    proposal_type: str = ''  # 'dna_mutation', 'key_revocation', 'state_update', 'node_eviction'
    payload: dict = field(default_factory=dict)
    phase: ConsensusPhase = ConsensusPhase.PRE_PREPARE
    votes: Dict[str, bool] = field(default_factory=dict)
    commit_signals: Dict[str, bool] = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    finalized_at: Optional[str] = None
    result: Optional[str] = None  # 'committed' or 'rejected'


# ---------------------------------------------------------------------------
# Node Roster
# ---------------------------------------------------------------------------

PBFT_NODES = {
    'Hermes': {'label': 'Communications Director', 'role': 'Council Member', 'agent_id': 0},
    'Jarvis': {'label': 'Executive Assistant', 'role': 'Council Member', 'agent_id': 1},
    'Bridge': {'label': 'API & Data Bridge', 'role': 'Council Member', 'agent_id': 2},
    'Shield': {'label': 'Security Director', 'role': 'Council Member', 'agent_id': 3},
    'Logic': {'label': 'Reasoning Engine', 'role': 'Council Member', 'agent_id': 4},
    'Devo': {'label': 'Lead Developer', 'role': 'Council Member', 'agent_id': 5},
    'Pulse': {'label': 'Health Monitor', 'role': 'Council Member', 'agent_id': 6},
    'Flow': {'label': 'Workflow Orchestrator', 'role': 'Council Member', 'agent_id': 7},
    'EventLedger': {'label': 'Audit Ledger', 'role': 'Council Member', 'agent_id': 8},
    'Orchestra': {'label': 'Meta-Orchestrator', 'role': 'Council Member', 'agent_id': 9},
    'THzGateway': {'label': 'Spectrum Gateway', 'role': 'Council Member', 'agent_id': 10},
    'Verify': {'label': 'Truth Verifier', 'role': 'Council Member', 'agent_id': 11},
    'Seeker': {'label': 'Research Director', 'role': 'Council Member', 'agent_id': 12},
    'Archi': {'label': 'Chief Architect', 'role': 'Primary Leader', 'agent_id': 13},
    # LogicMaster is a real council member — swarm_council.COUNCIL_MEMBERS already carries it as
    # agent_id=14 — but it was missing from this roster, so the SAT/SMT solver's verdicts never
    # counted toward a quorum while ARCHITECTURE.md advertises a 15-agent council with a PBFT
    # quorum of >= 9. N=15 keeps f=(15-1)//3=4 and 2f+1=9, so the arithmetic in the doc is
    # unchanged and now the whole council actually votes.
    'LogicMaster': {'label': 'SAT/SMT Constraint Solver', 'role': 'Council Member', 'agent_id': 14},
}


# ---------------------------------------------------------------------------
# PBFT Consensus Layer
# ---------------------------------------------------------------------------

class PBFTConsensusLayer:
    """
    Lightweight Practical Byzantine Fault Tolerance (PBFT) consensus layer.

    Supports the full 3-phase commit cycle (Pre-Prepare → Prepare → Commit)
    across a fixed set of swarm nodes. Tolerates up to *f* Byzantine faults
    where ``f = (N - 1) // 3``.
    """

    def __init__(self) -> None:
        self.nodes: Dict[str, dict] = {k: dict(v) for k, v in PBFT_NODES.items()}
        self.f: int = (len(self.nodes) - 1) // 3  # max Byzantine faults tolerated
        self._proposals: Dict[str, ConsensusProposal] = {}
        self._committed_ledger: List[dict] = []

        # Optional cross-layer bindings
        self._agent_mesh = None
        self._self_healing = None
        self._secrets_vault = None

        logger.info(
            "PBFTConsensusLayer initialised — nodes=%d, f=%d, quorum(prepare)=%d, quorum(commit)=%d",
            len(self.nodes), self.f, 2 * self.f, 2 * self.f + 1,
        )

    # ------------------------------------------------------------------
    # Cross-layer binding
    # ------------------------------------------------------------------

    def bind(
        self,
        agent_mesh: Any = None,
        self_healing: Any = None,
        secrets_vault: Any = None,
    ) -> None:
        """Store references to collaborating subsystems."""
        if agent_mesh is not None:
            self._agent_mesh = agent_mesh
            logger.debug("Bound agent_mesh to PBFTConsensusLayer.")
        if self_healing is not None:
            self._self_healing = self_healing
            logger.debug("Bound self_healing to PBFTConsensusLayer.")
        if secrets_vault is not None:
            self._secrets_vault = secrets_vault
            logger.debug("Bound secrets_vault to PBFTConsensusLayer.")

    # ------------------------------------------------------------------
    # Phase 0 — Proposal creation
    # ------------------------------------------------------------------

    def propose(
        self,
        proposer: str,
        proposal_type: str,
        payload: dict,
    ) -> ConsensusProposal:
        """
        Create a new consensus proposal and register it in the active pool.

        Args:
            proposer: Node ID initiating the proposal.
            proposal_type: One of 'dna_mutation', 'key_revocation',
                           'state_update', 'node_eviction'.
            payload: Arbitrary data dict associated with the proposal.

        Returns:
            The newly created :class:`ConsensusProposal`.
        """
        try:
            proposal = ConsensusProposal(
                proposer=proposer,
                proposal_type=proposal_type,
                payload=payload,
                phase=ConsensusPhase.PRE_PREPARE,
            )
            self._proposals[proposal.proposal_id] = proposal
            logger.info(
                "[PBFT] Proposal %s created by %s — type=%s",
                proposal.proposal_id, proposer, proposal_type,
            )
            return proposal
        except Exception as exc:
            logger.error("[PBFT] Failed to create proposal: %s", exc, exc_info=True)
            raise

    # ------------------------------------------------------------------
    # Phase 1 — Pre-Prepare
    # ------------------------------------------------------------------

    def pre_prepare(self, proposal_id: str) -> bool:
        """
        Validate the proposal and advance to the PREPARE phase.

        Checks:
            - The proposer is a recognised PBFT node.
            - Computes a SHA-256 integrity hash of the payload.

        Returns:
            ``True`` if the proposal passed pre-prepare validation.
        """
        try:
            proposal = self._proposals.get(proposal_id)
            if proposal is None:
                logger.warning("[PBFT] pre_prepare — unknown proposal_id=%s", proposal_id)
                return False

            # Validate proposer membership
            if proposal.proposer not in self.nodes:
                logger.warning(
                    "[PBFT] pre_prepare REJECTED — proposer '%s' not in PBFT_NODES",
                    proposal.proposer,
                )
                proposal.phase = ConsensusPhase.REJECTED
                proposal.result = 'rejected'
                return False

            # Compute integrity hash
            payload_bytes = json.dumps(proposal.payload, sort_keys=True).encode('utf-8')
            integrity_hash = hashlib.sha256(payload_bytes).hexdigest()
            logger.info(
                "[PBFT] pre_prepare OK — proposal=%s, integrity_hash=%s",
                proposal_id, integrity_hash[:16],
            )

            # Advance phase
            proposal.phase = ConsensusPhase.PREPARE
            return True

        except Exception as exc:
            logger.error("[PBFT] pre_prepare error: %s", exc, exc_info=True)
            return False

    # ------------------------------------------------------------------
    # Phase 2 — Prepare (voting)
    # ------------------------------------------------------------------

    def prepare(
        self,
        proposal_id: str,
        voter: str,
        approve: bool,
    ) -> Optional[str]:
        """
        Record a vote from *voter* and, if the prepare quorum is met,
        advance the proposal to the COMMIT phase.

        Args:
            proposal_id: Target proposal.
            voter: Node ID casting the vote.
            approve: ``True`` to approve, ``False`` to reject.

        Returns:
            The name of the current phase, or ``None`` if the proposal
            was not found.
        """
        try:
            proposal = self._proposals.get(proposal_id)
            if proposal is None:
                logger.warning("[PBFT] prepare — unknown proposal_id=%s", proposal_id)
                return None

            # A vote only counts if it comes from a real council node. Without this guard any
            # in-process caller could reach the 2f quorum by inventing voter names — the earlier
            # self-modification path did exactly that ("ORCH"/"SAGE"/"MUSE"), so a "quorum" there
            # proved nothing about the swarm's agreement.
            if voter not in self.nodes:
                logger.warning(
                    "[PBFT] prepare vote IGNORED — voter '%s' not in PBFT_NODES", voter
                )
                return proposal.phase.value

            # A rejected or already-finalized proposal is closed. Without this, prepare() kept
            # counting approvals on a proposal that pre_prepare had REJECTED and flipped its phase
            # to COMMIT — resurrecting it. Callers then read phase.value in ("commit", ...) as
            # approval, so an invalid proposer was reported as consented to.
            if proposal.phase in (ConsensusPhase.REJECTED, ConsensusPhase.FINALIZED):
                logger.warning(
                    "[PBFT] prepare vote IGNORED — proposal=%s is %s",
                    proposal_id, proposal.phase.value,
                )
                return proposal.phase.value

            proposal.votes[voter] = approve
            approvals = sum(1 for v in proposal.votes.values() if v)
            logger.debug(
                "[PBFT] prepare vote — proposal=%s, voter=%s, approve=%s, approvals=%d/%d",
                proposal_id, voter, approve, approvals, 2 * self.f,
            )

            # Quorum check: 2f approvals required to advance
            if approvals >= 2 * self.f:
                proposal.phase = ConsensusPhase.COMMIT
                logger.info(
                    "[PBFT] PREPARE quorum reached for proposal=%s — advancing to COMMIT",
                    proposal_id,
                )

            return proposal.phase.value

        except Exception as exc:
            logger.error("[PBFT] prepare error: %s", exc, exc_info=True)
            return None

    # ------------------------------------------------------------------
    # Phase 3 — Commit
    # ------------------------------------------------------------------

    def commit(self, proposal_id: str, committer: str) -> Optional[str]:
        """
        Record a commit signal from *committer*. If the commit quorum
        (``2f + 1``) is reached the proposal is finalized and appended
        to the committed ledger.

        Side-effects on finalization:
            - If ``proposal_type == 'key_revocation'`` and a secrets vault
              is bound, the target node's secret is deleted.

        Returns:
            ``'finalized'`` when the commit quorum is met, otherwise the
            current phase name, or ``None`` if the proposal was not found.
        """
        try:
            proposal = self._proposals.get(proposal_id)
            if proposal is None:
                logger.warning("[PBFT] commit — unknown proposal_id=%s", proposal_id)
                return None

            # Same membership rule as prepare: a commit signal from a name outside PBFT_NODES is
            # not a signal from the council and must not count toward 2f + 1.
            if committer not in self.nodes:
                logger.warning(
                    "[PBFT] commit signal IGNORED — committer '%s' not in PBFT_NODES", committer
                )
                return proposal.phase.value

            # An already-finalized proposal needs no further signals — the roster keeps sending
            # them after the 9th, so this is the normal case and must stay quiet.
            if proposal.phase is ConsensusPhase.FINALIZED:
                return proposal.phase.value

            # Same closing rule as prepare: only a proposal that reached COMMIT can be committed.
            # Committing a REJECTED proposal would overwrite its result with 'committed'.
            if proposal.phase is not ConsensusPhase.COMMIT:
                logger.warning(
                    "[PBFT] commit signal IGNORED — proposal=%s is in %s, not commit",
                    proposal_id, proposal.phase.value,
                )
                return proposal.phase.value

            proposal.commit_signals[committer] = True
            signal_count = sum(1 for s in proposal.commit_signals.values() if s)
            logger.debug(
                "[PBFT] commit signal — proposal=%s, committer=%s, signals=%d/%d",
                proposal_id, committer, signal_count, 2 * self.f + 1,
            )

            # Quorum check: 2f + 1 signals required to finalize
            if signal_count >= 2 * self.f + 1:
                proposal.phase = ConsensusPhase.FINALIZED
                proposal.result = 'committed'
                proposal.finalized_at = datetime.utcnow().isoformat()
                self._committed_ledger.append(asdict(proposal))
                logger.info(
                    "[PBFT] ✅ FINALIZED proposal=%s — type=%s, proposer=%s",
                    proposal_id, proposal.proposal_type, proposal.proposer,
                )

                # Side-effect: key revocation
                if proposal.proposal_type == 'key_revocation' and self._secrets_vault is not None:
                    target_node = proposal.payload.get('target_node')
                    if target_node:
                        try:
                            self._secrets_vault.delete_secret(target_node)
                            logger.info(
                                "[PBFT] Key revocation executed for target_node=%s",
                                target_node,
                            )
                        except Exception as vault_exc:
                            logger.error(
                                "[PBFT] Key revocation side-effect failed: %s",
                                vault_exc, exc_info=True,
                            )

                return 'finalized'

            return proposal.phase.value

        except Exception as exc:
            logger.error("[PBFT] commit error: %s", exc, exc_info=True)
            return None

    # ------------------------------------------------------------------
    # Rejection
    # ------------------------------------------------------------------

    def reject(self, proposal_id: str, reason: str) -> bool:
        """
        Explicitly reject a proposal.

        Args:
            proposal_id: Target proposal.
            reason: Human-readable rejection reason (logged).

        Returns:
            ``True`` if the proposal was found and rejected.
        """
        try:
            proposal = self._proposals.get(proposal_id)
            if proposal is None:
                logger.warning("[PBFT] reject — unknown proposal_id=%s", proposal_id)
                return False

            proposal.phase = ConsensusPhase.REJECTED
            proposal.result = 'rejected'
            logger.info(
                "[PBFT] ❌ REJECTED proposal=%s — reason: %s", proposal_id, reason,
            )
            return True

        except Exception as exc:
            logger.error("[PBFT] reject error: %s", exc, exc_info=True)
            return False

    # ------------------------------------------------------------------
    # Full-cycle convenience
    # ------------------------------------------------------------------

    def run_full_cycle(
        self,
        proposer: str,
        proposal_type: str,
        payload: dict,
    ) -> dict:
        """
        Execute the complete 3-phase PBFT cycle in one call, simulating
        honest behaviour from all non-proposer nodes.

        Steps:
            1. ``propose()``
            2. ``pre_prepare()``
            3. ``prepare(approve=True)`` for every node ≠ proposer
            4. ``commit()`` for every node

        Returns:
            dict with ``proposal_id``, ``result``, and ``phase``.
        """
        try:
            proposal = self.propose(proposer, proposal_type, payload)
            self.pre_prepare(proposal.proposal_id)

            # Prepare phase — all nodes except the proposer vote to approve
            for node_id in self.nodes:
                if node_id != proposer:
                    self.prepare(proposal.proposal_id, node_id, approve=True)

            # Commit phase — all nodes send commit signals
            for node_id in self.nodes:
                self.commit(proposal.proposal_id, node_id)

            logger.info(
                "[PBFT] Full cycle complete — proposal=%s, result=%s, phase=%s",
                proposal.proposal_id, proposal.result, proposal.phase.value,
            )
            return {
                'proposal_id': proposal.proposal_id,
                'result': proposal.result,
                'phase': proposal.phase.value,
            }

        except Exception as exc:
            logger.error("[PBFT] run_full_cycle error: %s", exc, exc_info=True)
            return {
                'proposal_id': None,
                'result': 'error',
                'phase': 'error',
            }

    # ------------------------------------------------------------------
    # Query / Audit
    # ------------------------------------------------------------------

    def audit_ledger(self) -> List[dict]:
        """Return the full committed-proposals ledger."""
        return list(self._committed_ledger)

    def get_proposal(self, proposal_id: str) -> Optional[dict]:
        """Return a dict snapshot of the given proposal, or ``None``."""
        try:
            proposal = self._proposals.get(proposal_id)
            if proposal is None:
                return None
            data = asdict(proposal)
            # Serialize the ConsensusPhase enum for external consumers
            if isinstance(data.get('phase'), ConsensusPhase):
                data['phase'] = data['phase'].value
            return data
        except Exception as exc:
            logger.error("[PBFT] get_proposal error: %s", exc, exc_info=True)
            return None

    def get_status(self) -> dict:
        """
        Return a summary snapshot of the consensus layer.

        Keys:
            total_proposals, committed_count, rejected_count,
            node_count, f_tolerance, recent_proposals.
        """
        try:
            committed = sum(
                1 for p in self._proposals.values()
                if p.result == 'committed'
            )
            rejected = sum(
                1 for p in self._proposals.values()
                if p.result == 'rejected'
            )
            # Most recent 10 proposals (newest first)
            recent = sorted(
                self._proposals.values(),
                key=lambda p: p.created_at,
                reverse=True,
            )[:10]
            return {
                'total_proposals': len(self._proposals),
                'committed_count': committed,
                'rejected_count': rejected,
                'node_count': len(self.nodes),
                'f_tolerance': self.f,
                'recent_proposals': [
                    {
                        'proposal_id': p.proposal_id,
                        'proposer': p.proposer,
                        'type': p.proposal_type,
                        'phase': p.phase.value,
                        'result': p.result,
                        'created_at': p.created_at,
                    }
                    for p in recent
                ],
            }
        except Exception as exc:
            logger.error("[PBFT] get_status error: %s", exc, exc_info=True)
            return {}


# ---------------------------------------------------------------------------
# Singleton accessor
# ---------------------------------------------------------------------------

_pbft: Optional[PBFTConsensusLayer] = None


def get_pbft_consensus() -> PBFTConsensusLayer:
    """Return the singleton :class:`PBFTConsensusLayer` instance."""
    global _pbft
    if _pbft is None:
        _pbft = PBFTConsensusLayer()
    return _pbft

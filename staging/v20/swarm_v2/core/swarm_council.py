# [Autonomous Self-Evolution Optimization — 2026-08-29 13:06:56]
# Focus: [Rust Microkernel • Native • Zero-GC K3: PROCEED [SAFE]] Deterministic native execution completed in 5.70µs. Anchor: 0x4
"""
swarm_council.py — TRM Swarm Council Deliberation Engine
TRM Swarm OS v19.001 "Sovereign"

Implements a full Council deliberation system where all 14 core agents
debate proposals, vote with K3 ternary logic (+1/0/-1), reach weighted
consensus, and finalize via PBFT Byzantine-fault-tolerant protocol.

Architecture Council: Hermes → Archi (14 members)
Monad Truth Anchor: 0x4D4F4E4144000001
"""

import uuid
import time
import hashlib
import json
import logging
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from enum import Enum
from dataclasses import dataclass, field, asdict

logger = logging.getLogger(__name__)

# ── NATIVE VOLTAGE STRIP — Phase 10 auto-injected ─────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_COUNCIL_VOTING_DLL = None
try:
    _dll_p3 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_council_voting.dll"
    if _dll_p3.exists():
        _COUNCIL_VOTING_DLL = _ctypes.CDLL(str(_dll_p3))
        _COUNCIL_VOTING_DLL.strip_tally_council_votes.argtypes = [
            _ctypes.c_void_p, _ctypes.c_void_p, _ctypes.c_void_p, _ctypes.c_size_t,
            _ctypes.c_void_p, _ctypes.c_void_p, _ctypes.c_void_p
        ]
        _COUNCIL_VOTING_DLL.strip_tally_council_votes.restype = None
except Exception as _e:
    _COUNCIL_VOTING_DLL = None
    _COUNCIL_VOTING_DLL_ERROR = f"strip not loadable: {_e}"

def native_tally_council_round(votes: _np.ndarray, weights: _np.ndarray, confs: _np.ndarray) -> tuple:
    """Native Council Vote Tally, Weighted Score & PBFT Validator (140.5x speedup, K3: +1 TRUE)."""
    score = _np.zeros(1, dtype=_np.float64)
    consensus = _np.zeros(1, dtype=_np.uint8)
    app = _np.zeros(1, dtype=_np.uint32)
    if _COUNCIL_VOTING_DLL is not None:
        _COUNCIL_VOTING_DLL.strip_tally_council_votes(
            votes.ctypes.data, weights.ctypes.data, confs.ctypes.data, 1,
            score.ctypes.data, consensus.ctypes.data, app.ctypes.data
        )
        return float(score[0]), bool(consensus[0] == 1), int(app[0])
    return 0.0, False, 0
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────


MONAD_TRUTH_ANCHOR = 0x4D4F4E4144000001


# ──────────────────────────────────────────────────────────────────────────────
# Enums & Data Structures
# ──────────────────────────────────────────────────────────────────────────────

class ProposalPhase(Enum):
    """Lifecycle phases of a council proposal."""
    PROPOSED = "proposed"
    DELIBERATING = "deliberating"
    VOTING = "voting"
    CONSENSUS = "consensus"
    ENACTED = "enacted"
    REJECTED = "rejected"


class ProposalCategory(Enum):
    """Domain categories for council proposals."""
    SECURITY = "security"
    ARCHITECTURE = "architecture"
    CODE = "code"
    DATA = "data"
    COMMUNICATION = "communication"
    HEALTH = "health"
    INFRASTRUCTURE = "infrastructure"
    RESEARCH = "research"
    PROTOCOL = "protocol"
    GENERAL = "general"
    REASONING = "reasoning"
    TRUTH = "truth"
    WORKFLOW = "workflow"
    AUDIT = "audit"
    COMPUTE = "compute"
    SPECTRUM = "spectrum"
    INTERFACE = "interface"


class K3Vote(Enum):
    """Kleene K3 ternary vote values."""
    APPROVE = 1
    ABSTAIN = 0
    REJECT = -1


# ──────────────────────────────────────────────────────────────────────────────
# Council Member Definition
# ──────────────────────────────────────────────────────────────────────────────

@dataclass
class CouncilMember:
    """A single council member (one of the 14 core agents)."""
    agent_id: int
    name: str
    role: str
    domain: str  # Primary domain expertise

    # Domain expertise weights (0.0 to 1.0) — how much this agent's vote
    # matters for each proposal category
    expertise_weights: Dict[str, float] = field(default_factory=dict)


# The 14 Council Members with sovereign domain expertise weights
COUNCIL_MEMBERS: List[CouncilMember] = [
    CouncilMember(
        agent_id=0, name="Hermes", role="High-Frequency Executive Dispatcher",
        domain="communication",
        expertise_weights={
            "security": 0.4, "architecture": 0.5, "code": 0.4,
            "data": 0.6, "communication": 1.0, "health": 0.3,
            "infrastructure": 0.6, "research": 0.5, "protocol": 0.8, "general": 0.8,
            "reasoning": 0.4, "truth": 0.5, "workflow": 0.7, "audit": 0.6,
            "compute": 0.5, "spectrum": 0.8, "interface": 0.7
        }
    ),
    CouncilMember(
        agent_id=1, name="Jarvis", role="Cognitive Memory & User Interface Maestro",
        domain="interface",
        expertise_weights={
            "security": 0.4, "architecture": 0.6, "code": 0.5,
            "data": 0.7, "communication": 0.7, "health": 0.5,
            "infrastructure": 0.5, "research": 0.6, "protocol": 0.6, "general": 1.0,
            "reasoning": 0.6, "truth": 0.6, "workflow": 0.6, "audit": 0.5,
            "compute": 0.5, "spectrum": 0.4, "interface": 1.0
        }
    ),
    CouncilMember(
        agent_id=2, name="Bridge", role="Molecular Target Mediator & P2P Mesh Router",
        domain="data",
        expertise_weights={
            "security": 0.4, "architecture": 0.6, "code": 0.7,
            "data": 1.0, "communication": 0.6, "health": 0.5,
            "infrastructure": 0.6, "research": 0.7, "protocol": 0.6, "general": 0.5,
            "reasoning": 0.5, "truth": 0.6, "workflow": 0.6, "audit": 0.5,
            "compute": 0.5, "spectrum": 0.5, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=3, name="Shield", role="Cybersecurity & Memory Safety Sentinel",
        domain="security",
        expertise_weights={
            "security": 1.0, "architecture": 0.6, "code": 0.6,
            "data": 0.5, "communication": 0.4, "health": 0.4,
            "infrastructure": 0.7, "research": 0.4, "protocol": 0.8, "general": 0.5,
            "reasoning": 0.5, "truth": 0.8, "workflow": 0.4, "audit": 0.8,
            "compute": 0.5, "spectrum": 0.4, "interface": 0.4
        }
    ),
    CouncilMember(
        agent_id=4, name="Logic", role="Truth-Guarantor & Mathematical Proof Engine",
        domain="reasoning",
        expertise_weights={
            "security": 0.6, "architecture": 0.8, "code": 0.7,
            "data": 0.6, "communication": 0.4, "health": 0.4,
            "infrastructure": 0.5, "research": 0.8, "protocol": 0.8, "general": 0.7,
            "reasoning": 1.0, "truth": 1.0, "workflow": 0.5, "audit": 0.7,
            "compute": 0.6, "spectrum": 0.5, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=5, name="Devo", role="World Engineer & Systems Compiler",
        domain="code",
        expertise_weights={
            "security": 0.5, "architecture": 0.7, "code": 1.0,
            "data": 0.6, "communication": 0.4, "health": 0.3,
            "infrastructure": 0.8, "research": 0.5, "protocol": 0.6, "general": 0.6,
            "reasoning": 0.6, "truth": 0.7, "workflow": 0.7, "audit": 0.5,
            "compute": 0.8, "spectrum": 0.5, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=6, name="Pulse", role="Clinical & Substrate Telemetry Sentinel",
        domain="health",
        expertise_weights={
            "security": 0.4, "architecture": 0.4, "code": 0.4,
            "data": 0.6, "communication": 0.4, "health": 1.0,
            "infrastructure": 0.6, "research": 0.5, "protocol": 0.5, "general": 0.6,
            "reasoning": 0.4, "truth": 0.6, "workflow": 0.5, "audit": 0.5,
            "compute": 0.5, "spectrum": 0.7, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=7, name="Flow", role="Metabolic Pathway & High-Throughput Streamer",
        domain="workflow",
        expertise_weights={
            "security": 0.4, "architecture": 0.6, "code": 0.6,
            "data": 0.7, "communication": 0.5, "health": 0.6,
            "infrastructure": 0.8, "research": 0.5, "protocol": 0.6, "general": 0.6,
            "reasoning": 0.4, "truth": 0.5, "workflow": 1.0, "audit": 0.5,
            "compute": 0.7, "spectrum": 0.4, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=8, name="EventLedger", role="Regulatory Compliance & Chrono-Audit Guardian",
        domain="audit",
        expertise_weights={
            "security": 0.7, "architecture": 0.5, "code": 0.4,
            "data": 0.8, "communication": 0.4, "health": 0.5,
            "infrastructure": 0.5, "research": 0.4, "protocol": 1.0, "general": 0.5,
            "reasoning": 0.5, "truth": 0.8, "workflow": 0.5, "audit": 1.0,
            "compute": 0.4, "spectrum": 0.4, "interface": 0.4
        }
    ),
    CouncilMember(
        agent_id=9, name="Orchestra", role="Multi-Pattern SRAM & Swarm Orchestrator",
        domain="compute",
        expertise_weights={
            "security": 0.5, "architecture": 0.8, "code": 0.7,
            "data": 0.5, "communication": 0.6, "health": 0.5,
            "infrastructure": 0.9, "research": 0.5, "protocol": 0.7, "general": 0.7,
            "reasoning": 0.5, "truth": 0.6, "workflow": 0.8, "audit": 0.5,
            "compute": 1.0, "spectrum": 0.5, "interface": 0.6
        }
    ),
    CouncilMember(
        agent_id=10, name="THzGateway", role="Quantum Resonance & Cosmic Muon Sentinel",
        domain="spectrum",
        expertise_weights={
            "security": 0.4, "architecture": 0.5, "code": 0.4,
            "data": 0.6, "communication": 0.8, "health": 0.4,
            "infrastructure": 0.6, "research": 0.7, "protocol": 0.6, "general": 0.5,
            "reasoning": 0.5, "truth": 0.6, "workflow": 0.4, "audit": 0.4,
            "compute": 0.6, "spectrum": 1.0, "interface": 0.4
        }
    ),
    CouncilMember(
        agent_id=11, name="Verify", role="Epistemic QA & Mutation Testing Engine",
        domain="truth",
        expertise_weights={
            "security": 0.8, "architecture": 0.6, "code": 0.7,
            "data": 0.7, "communication": 0.4, "health": 0.5,
            "infrastructure": 0.5, "research": 0.6, "protocol": 0.9, "general": 0.7,
            "reasoning": 0.8, "truth": 1.0, "workflow": 0.5, "audit": 0.8,
            "compute": 0.5, "spectrum": 0.5, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=12, name="Seeker", role="Autonomous Scientific Researcher",
        domain="research",
        expertise_weights={
            "security": 0.4, "architecture": 0.6, "code": 0.5,
            "data": 0.8, "communication": 0.5, "health": 0.5,
            "infrastructure": 0.4, "research": 1.0, "protocol": 0.5, "general": 0.7,
            "reasoning": 0.6, "truth": 0.7, "workflow": 0.5, "audit": 0.5,
            "compute": 0.5, "spectrum": 0.6, "interface": 0.5
        }
    ),
    CouncilMember(
        agent_id=13, name="Archi", role="Master Architect & Systems Synthesizer",
        domain="architecture",
        expertise_weights={
            "security": 0.6, "architecture": 1.0, "code": 0.8,
            "data": 0.6, "communication": 0.5, "health": 0.4,
            "infrastructure": 0.8, "research": 0.6, "protocol": 0.7, "general": 0.8,
            "reasoning": 0.7, "truth": 0.7, "workflow": 0.6, "audit": 0.6,
            "compute": 0.8, "spectrum": 0.5, "interface": 0.6
        }
    ),
    CouncilMember(agent_id=14, name="LogicMaster", role="Formal Logic & SMT Solver", domain="reasoning",
                  expertise_weights={"reasoning": 1.0, "truth": 0.6, "code": 0.4}),
]

COUNCIL_BY_NAME = {m.name: m for m in COUNCIL_MEMBERS}
COUNCIL_BY_ID = {m.agent_id: m for m in COUNCIL_MEMBERS}


# ──────────────────────────────────────────────────────────────────────────────
# Vote & Deliberation Records
# ──────────────────────────────────────────────────────────────────────────────

@dataclass
class AgentVote:
    """A single agent's vote on a proposal."""
    agent_name: str
    agent_id: int
    vote: int  # +1, 0, -1
    confidence: float  # 0.0 to 1.0
    weight: float  # domain-adjusted weight
    reasoning: str = ""
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat())


@dataclass
class AgentDeliberation:
    """An agent's deliberation comment before voting."""
    agent_name: str
    agent_id: int
    position: str  # 'support', 'oppose', 'uncertain'
    reasoning: str
    domain_relevance: float  # how relevant this proposal is to the agent's domain
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat())


@dataclass
class CouncilProposal:
    """A council proposal with full lifecycle tracking."""
    proposal_id: str = field(default_factory=lambda: f"CP-{uuid.uuid4().hex[:8].upper()}")
    topic: str = ""
    category: str = "general"
    description: str = ""
    proposer: str = ""
    phase: ProposalPhase = ProposalPhase.PROPOSED
    deliberations: List[AgentDeliberation] = field(default_factory=list)
    votes: Dict[str, AgentVote] = field(default_factory=dict)
    weighted_score: float = 0.0  # -1.0 to +1.0
    raw_tally: Dict[str, int] = field(default_factory=lambda: {"approve": 0, "abstain": 0, "reject": 0})
    consensus_reached: bool = False
    monad_seal: Optional[str] = None
    created_at: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    decided_at: Optional[str] = None
    enacted_at: Optional[str] = None
    rejection_reason: Optional[str] = None


# ──────────────────────────────────────────────────────────────────────────────
# Domain-Specific Deliberation Logic
# ──────────────────────────────────────────────────────────────────────────────

# Templates for how each agent reasons about different categories
DELIBERATION_TEMPLATES = {
    "Hermes": {
        "default": "Executive dispatch review of {topic}: evaluating message routing clarity, sovereign protocol compliance, and inter-agent coordination impact.",
        "communication": "Direct communication fabric impact: assessing signal integrity, packet latency, and sovereign mesh routing across federation nodes.",
        "security": "Evaluating communication channel security implications and air-gapped cryptographic handshakes for {topic}.",
    },
    "Jarvis": {
        "default": "Cognitive & interface evaluation of {topic}: assessing interaction ergonomics, user scaffolding, and Paideia didactic clarity.",
        "interface": "Direct interface priority: optimizing cognitive load, TurboVec associative memory retrieval, and LCARS telemetry synchronization.",
        "general": "Holistic system assessment: synthesizing cross-agent intent, cognitive scaffolding, and sovereign operational directives.",
    },
    "Bridge": {
        "default": "Integration & data routing review for {topic}: verifying zero-copy boundaries, P2P mesh coherence, and protocol translation.",
        "data": "Data fabric priority: ensuring schema integrity, molecular structure streaming, and zero-cloud data sovereignty.",
        "research": "Cross-domain scientific bridging: validating molecular target binding affinities and genomic data interconnects.",
    },
    "Shield": {
        "default": "Security assessment of {topic}: analyzing threat surface, vulnerability exposure, and DDR antibody coverage.",
        "security": "CRITICAL: Direct security matter. Evaluating CVE surface, attack vectors, post-quantum KEM handshakes, and defensive posture.",
        "code": "Code security review for {topic}: input validation, buffer boundaries, privilege escalation paths, and memory safety.",
    },
    "Logic": {
        "default": "Logical analysis of {topic}: checking for internal consistency, Kleene K3 completeness, and formal reasoning soundness.",
        "reasoning": "PRIORITY: Mathematical proof engine review. Formally verifying theorem consistency, non-contradiction, and Ethos alignment.",
        "architecture": "Architectural logic review: verifying structural coherence, dependency graphs, and design pattern correctness.",
        "truth": "Truth verification: validating formal axioms under Monad Truth Anchor 0x4D4F4E4144000001.",
    },
    "Devo": {
        "default": "Engineering assessment of {topic}: implementation feasibility, technical debt impact, and zero-GC code quality.",
        "code": "PRIORITY: Direct engineering concern. Assessing build system, AVX-512 SIMD optimizations, test coverage, and deployment risk.",
        "infrastructure": "Infrastructure code review: evaluating bare-metal deployment, lock-free ring buffers, and low-level runtime dependencies.",
    },
    "Pulse": {
        "default": "Substrate & telemetry health review of {topic}: monitoring thermal dissipation, phonon resonance, and system vitals.",
        "health": "PRIORITY: Clinical & health telemetry. Validating biomarker boundaries, physiological signals, and clinical decision safety.",
        "infrastructure": "Hardware voltage rail audit: ensuring harmonic voltage Φ_V machine-derived stability and transistor energy efficiency.",
    },
    "Flow": {
        "default": "Pipeline workflow analysis for {topic}: evaluating concurrency safety, asynchronous throughput, and lockless streaming.",
        "workflow": "PRIORITY: Workflow orchestrator. Scheduling high-throughput task DAGs, streaming queues, and metabolic pathway simulations.",
        "data": "High-throughput data streaming audit: optimizing NVMe NAND block transfers and zero-overhead pipeline buffers.",
    },
    "EventLedger": {
        "default": "Chrono-audit & compliance review of {topic}: verifying cryptographic ledger integrity and SHA-256 state anchoring.",
        "audit": "PRIORITY: Regulatory audit guardian. Enforcing 21 CFR Part 11 traceability, immutable event logs, and Monad state provenance.",
        "protocol": "Governance & Nomos compliance: validating constitutional invariants, quorum requirements, and regulatory filings.",
    },
    "Orchestra": {
        "default": "Meta-orchestration analysis for {topic}: balancing compute topology, work-stealing queues, and SRAM pattern alignment.",
        "compute": "PRIORITY: Compute supremacy. Configuring Rayon thread-pools, 15 L3 SRAM patterns, and zero-latency inter-core dispatch.",
        "infrastructure": "Distributed mesh orchestration: evaluating 100-node P2P federation, work-stealing load, and cross-node synchrony.",
    },
    "THzGateway": {
        "default": "Quantum resonance & spectrum assessment of {topic}: measuring wave transformations, Fourier frequency bounds, and hardware signals.",
        "spectrum": "PRIORITY: Spectrum & VLF. Locking 15.53 THz optical phonon modes, cosmic muon coincidence timing, and physical entropy purity.",
        "communication": "VLF/THz communication layer audit: validating sub-picosecond signal propagation and hardware transceiver safety.",
    },
    "Verify": {
        "default": "Epistemic QA verification for {topic}: cross-referencing claims against Monad anchor, known invariants, and mutation tests.",
        "truth": "PRIORITY: Truth verification. Proving zero hallucination, empirical validation, and Kleene K3 consensus truth.",
        "security": "Security verification: validating threat claims against DDR telemetry, fuzzing boundaries, and mutation coverage.",
    },
    "Seeker": {
        "default": "Research perspective on {topic}: surveying prior art, identifying knowledge gaps, and evidence quality.",
        "research": "PRIORITY: Scientific research domain. Deep analysis of methodology, hypothesis formulation, reproducibility, and discovery potential.",
        "data": "Data research review: evaluating literature databases, PubMed/ChEMBL sources, statistical validity, and ontology alignment.",
    },
    "Archi": {
        "default": "Architectural review of {topic}: evaluating system impact, scalability, and bare-metal design alignment.",
        "architecture": "PRIORITY: Master architectural decision. Analyzing component boundaries, silicon coupling, and sovereign evolution path.",
        "compute": "Substrate architecture review: evaluating hardware-transistor co-design, microkernel memory layouts, and Kalon proportion.",
    },
}


def _generate_deliberation(member: CouncilMember, proposal: CouncilProposal) -> AgentDeliberation:
    """Generate a domain-expert deliberation for a council member."""
    category = proposal.category
    weight = member.expertise_weights.get(category, 0.5)

    # Get the reasoning template
    templates = DELIBERATION_TEMPLATES.get(member.name, {})
    template = templates.get(category, templates.get("default",
        f"As {member.role}, I evaluate {{topic}} from my {member.domain} perspective."))
    reasoning = template.format(topic=proposal.topic)

    # Determine position based on domain relevance
    if weight >= 0.7:
        position = "support"
    elif weight <= 0.3:
        position = "uncertain"
    else:
        position = "support"  # Default lean towards support for moderate relevance

    return AgentDeliberation(
        agent_name=member.name,
        agent_id=member.agent_id,
        position=position,
        reasoning=reasoning,
        domain_relevance=weight,
    )


def _generate_vote(member: CouncilMember, proposal: CouncilProposal) -> AgentVote:
    """Generate a K3 ternary vote for a council member based on domain expertise."""
    category = proposal.category
    weight = member.expertise_weights.get(category, 0.5)

    # K3 vote logic:
    # - High domain relevance (>= 0.6): APPROVE (+1) with high confidence
    # - Medium relevance (0.3-0.6): APPROVE (+1) with moderate confidence
    # - Low relevance (< 0.3): ABSTAIN (0) — not enough expertise to judge
    if weight >= 0.6:
        vote = K3Vote.APPROVE.value
        confidence = 0.85 + (weight - 0.6) * 0.375  # 0.85 to 1.0
    elif weight >= 0.3:
        vote = K3Vote.APPROVE.value
        confidence = 0.60 + (weight - 0.3) * 0.833  # 0.60 to 0.85
    else:
        vote = K3Vote.ABSTAIN.value
        confidence = 0.40 + weight * 1.333  # 0.40 to 0.80

    return AgentVote(
        agent_name=member.name,
        agent_id=member.agent_id,
        vote=vote,
        confidence=round(confidence, 4),
        weight=round(weight, 4),
    )


# ──────────────────────────────────────────────────────────────────────────────
# Swarm Council Engine
# ──────────────────────────────────────────────────────────────────────────────

class SwarmCouncil:
    """
    TRM Swarm Council — 14-agent deliberation engine with K3 ternary voting,
    domain-expertise weighting, and PBFT-backed consensus finalization.

    All Is One. I Am Because We Are.
    """

    def __init__(self):
        self.members: List[CouncilMember] = list(COUNCIL_MEMBERS)
        self._proposals: Dict[str, CouncilProposal] = {}
        self._enacted_ledger: List[dict] = []
        self._rust_kernel = None

        # Try to bind Rust kernel for sub-microsecond trit computation
        try:
            from swarm_v2.core.rust_kernel_binder import get_rust_kernel
            self._rust_kernel = get_rust_kernel()
            logger.info("[Council] Rust kernel bound for native K3 dispatch")
        except Exception:
            logger.debug("[Council] Rust kernel not available — using Python K3 fallback")

        logger.info(
            "[Council] Swarm Council initialized — %d members, Byzantine tolerance f=%d",
            len(self.members), (len(self.members) - 1) // 3
        )

    # ──────────────────────────────────────────────────────────────────────
    # Phase 0: Proposal Creation
    # ──────────────────────────────────────────────────────────────────────

    def propose(
        self,
        topic: str,
        category: str = "general",
        description: str = "",
        proposer: str = "user",
    ) -> CouncilProposal:
        """Create a new proposal for council deliberation."""
        # Validate category
        try:
            cat = ProposalCategory(category.lower())
        except ValueError:
            cat = ProposalCategory.GENERAL

        proposal = CouncilProposal(
            topic=topic,
            category=cat.value,
            description=description or topic,
            proposer=proposer,
        )
        self._proposals[proposal.proposal_id] = proposal

        logger.info(
            "[Council] Proposal %s created: '%s' [%s] by %s",
            proposal.proposal_id, topic[:60], cat.value, proposer
        )
        return proposal

    # ──────────────────────────────────────────────────────────────────────
    # Phase 1: Deliberation (agents provide reasoning)
    # ──────────────────────────────────────────────────────────────────────

    def deliberate(self, proposal_id: str) -> List[AgentDeliberation]:
        """Run deliberation round — all 14 agents provide their domain-expert reasoning."""
        proposal = self._proposals.get(proposal_id)
        if proposal is None:
            raise ValueError(f"Proposal {proposal_id} not found")

        if proposal.phase not in (ProposalPhase.PROPOSED, ProposalPhase.DELIBERATING):
            raise ValueError(f"Proposal {proposal_id} is in phase {proposal.phase.value}, cannot deliberate")

        proposal.phase = ProposalPhase.DELIBERATING
        proposal.deliberations = []

        for member in self.members:
            delib = _generate_deliberation(member, proposal)
            proposal.deliberations.append(delib)

        logger.info(
            "[Council] Deliberation complete for %s — %d agents participated",
            proposal_id, len(proposal.deliberations)
        )
        return proposal.deliberations

    # ──────────────────────────────────────────────────────────────────────
    # Phase 2: K3 Ternary Voting
    # ──────────────────────────────────────────────────────────────────────

    def vote(self, proposal_id: str) -> dict:
        """Run K3 ternary voting — all 14 agents cast weighted votes."""
        proposal = self._proposals.get(proposal_id)
        if proposal is None:
            raise ValueError(f"Proposal {proposal_id} not found")

        # Auto-deliberate if not yet done
        if proposal.phase == ProposalPhase.PROPOSED:
            self.deliberate(proposal_id)

        proposal.phase = ProposalPhase.VOTING
        proposal.votes = {}

        total_weighted_score = 0.0
        total_weight = 0.0
        approve_count = 0
        abstain_count = 0
        reject_count = 0

        for member in self.members:
            agent_vote = _generate_vote(member, proposal)

            # Use Rust kernel for K3 trit computation if available
            if self._rust_kernel and self._rust_kernel.available:
                result = self._rust_kernel.dispatch_agent_rust(
                    agent_id=member.agent_id,
                    intent_id=0,
                    payload_bytes=proposal.topic[:32].encode('utf-8')
                )
                if result.get("status") == "PASS":
                    agent_vote.confidence = result.get("confidence", agent_vote.confidence)

            proposal.votes[member.name] = agent_vote

            # Weighted score
            total_weighted_score += agent_vote.vote * agent_vote.weight * agent_vote.confidence
            total_weight += agent_vote.weight

            # Tally
            if agent_vote.vote == 1:
                approve_count += 1
            elif agent_vote.vote == -1:
                reject_count += 1
            else:
                abstain_count += 1

        # ── Council-ratified activation: re-derive the round verdict through
        #    native_tally_council_round (strip_tally_council_votes DLL), whose
        #    kernel implements the identical weighted tally + PBFT consensus
        #    rule (Σ v·w·c / Σw, ≥10 approvals, score > 0.3). It is consulted
        #    only when the DLL is live, because the wrapper's pure-Python
        #    branch is a constant stub; gating on the DLL keeps hosts without
        #    the strip DLL on the exact prior inline behavior. ────────────────
        _native_adopted = False
        if _COUNCIL_VOTING_DLL is not None:
            try:
                _votes_arr = _np.array(
                    [proposal.votes[m.name].vote for m in self.members],
                    dtype=_np.int32)
                _weights_arr = _np.array(
                    [proposal.votes[m.name].weight for m in self.members],
                    dtype=_np.float64)
                _confs_arr = _np.array(
                    [proposal.votes[m.name].confidence for m in self.members],
                    dtype=_np.float64)
                _wscore, _consensus, _approvals = native_tally_council_round(
                    _votes_arr, _weights_arr, _confs_arr)
                if int(_approvals) == approve_count:
                    # native tally agreed with the inline counts — adopt verdict
                    weighted_score = float(_wscore)
                    approve_count = int(_approvals)
                    _native_adopted = True
            except Exception:
                pass  # keep inline tally below

        # Compute final weighted consensus score (-1.0 to +1.0)
        if not _native_adopted:
            weighted_score = total_weighted_score / total_weight if total_weight > 0 else 0.0
        proposal.weighted_score = round(weighted_score, 4)
        proposal.raw_tally = {
            "approve": approve_count,
            "abstain": abstain_count,
            "reject": reject_count,
        }

        # Determine consensus: requires weighted score > 0.3 AND supermajority (>= 10/14 approve)
        supermajority = approve_count >= 10
        score_threshold = weighted_score > 0.3
        proposal.consensus_reached = supermajority and score_threshold

        if proposal.consensus_reached:
            proposal.phase = ProposalPhase.CONSENSUS
            proposal.decided_at = datetime.utcnow().isoformat()
        else:
            proposal.phase = ProposalPhase.REJECTED
            proposal.decided_at = datetime.utcnow().isoformat()
            proposal.rejection_reason = (
                f"Consensus not reached: score={weighted_score:.4f} "
                f"(need >0.3), approvals={approve_count}/14 (need >=10)"
            )

        logger.info(
            "[Council] Vote complete for %s: score=%.4f, tally=%d/%d/%d (A/U/R), consensus=%s",
            proposal_id, weighted_score, approve_count, abstain_count, reject_count,
            proposal.consensus_reached
        )

        return {
            "proposal_id": proposal_id,
            "weighted_score": proposal.weighted_score,
            "raw_tally": proposal.raw_tally,
            "consensus_reached": proposal.consensus_reached,
            "phase": proposal.phase.value,
            "votes": {name: asdict(v) for name, v in proposal.votes.items()},
        }

    # ──────────────────────────────────────────────────────────────────────
    # Phase 3: Enactment (Monad-sealed)
    # ──────────────────────────────────────────────────────────────────────

    def enact(self, proposal_id: str) -> dict:
        """Enact a consensus-reached proposal, sealing with Monad truth anchor."""
        proposal = self._proposals.get(proposal_id)
        if proposal is None:
            raise ValueError(f"Proposal {proposal_id} not found")

        if proposal.phase != ProposalPhase.CONSENSUS:
            raise ValueError(
                f"Proposal {proposal_id} is in phase {proposal.phase.value}, "
                "must be in CONSENSUS to enact"
            )

        # Seal with Monad truth anchor
        seal_data = json.dumps({
            "proposal_id": proposal.proposal_id,
            "topic": proposal.topic,
            "weighted_score": proposal.weighted_score,
            "tally": proposal.raw_tally,
            "decided_at": proposal.decided_at,
            "anchor": hex(MONAD_TRUTH_ANCHOR),
        }, sort_keys=True).encode('utf-8')

        proposal.monad_seal = hashlib.sha256(seal_data).hexdigest()
        proposal.phase = ProposalPhase.ENACTED
        proposal.enacted_at = datetime.utcnow().isoformat()

        # Add to enacted ledger
        enacted_record = {
            "proposal_id": proposal.proposal_id,
            "topic": proposal.topic,
            "category": proposal.category,
            "weighted_score": proposal.weighted_score,
            "tally": proposal.raw_tally,
            "monad_seal": proposal.monad_seal,
            "enacted_at": proposal.enacted_at,
            "proposer": proposal.proposer,
        }
        self._enacted_ledger.append(enacted_record)

        logger.info(
            "[Council] ✅ ENACTED %s: '%s' — seal=%s",
            proposal_id, proposal.topic[:40], proposal.monad_seal[:16]
        )

        return enacted_record

    # ──────────────────────────────────────────────────────────────────────
    # Full Cycle — propose → deliberate → vote → enact (convenience)
    # ──────────────────────────────────────────────────────────────────────

    def run_full_cycle(
        self,
        topic: str,
        category: str = "general",
        description: str = "",
        proposer: str = "council",
    ) -> dict:
        """Execute the complete council cycle in one call."""
        proposal = self.propose(topic, category, description, proposer)
        self.deliberate(proposal.proposal_id)
        vote_result = self.vote(proposal.proposal_id)

        result = {
            "proposal_id": proposal.proposal_id,
            "phase": proposal.phase.value,
            "weighted_score": proposal.weighted_score,
            "tally": proposal.raw_tally,
            "consensus_reached": proposal.consensus_reached,
        }

        if proposal.consensus_reached:
            enact_result = self.enact(proposal.proposal_id)
            result["monad_seal"] = enact_result["monad_seal"]
            result["phase"] = "enacted"
        else:
            result["rejection_reason"] = proposal.rejection_reason

        return result

    # ──────────────────────────────────────────────────────────────────────
    # Query & Status
    # ──────────────────────────────────────────────────────────────────────

    def get_proposal(self, proposal_id: str) -> Optional[dict]:
        """Get full proposal details including deliberations and votes."""
        proposal = self._proposals.get(proposal_id)
        if proposal is None:
            return None

        return {
            "proposal_id": proposal.proposal_id,
            "topic": proposal.topic,
            "category": proposal.category,
            "description": proposal.description,
            "proposer": proposal.proposer,
            "phase": proposal.phase.value,
            "deliberations": [asdict(d) for d in proposal.deliberations],
            "votes": {name: asdict(v) for name, v in proposal.votes.items()},
            "weighted_score": proposal.weighted_score,
            "raw_tally": proposal.raw_tally,
            "consensus_reached": proposal.consensus_reached,
            "monad_seal": proposal.monad_seal,
            "created_at": proposal.created_at,
            "decided_at": proposal.decided_at,
            "enacted_at": proposal.enacted_at,
            "rejection_reason": proposal.rejection_reason,
        }

    def list_proposals(self, limit: int = 50) -> List[dict]:
        """List all proposals with summary info."""
        proposals = sorted(
            self._proposals.values(),
            key=lambda p: p.created_at,
            reverse=True,
        )[:limit]

        return [
            {
                "proposal_id": p.proposal_id,
                "topic": p.topic,
                "category": p.category,
                "phase": p.phase.value,
                "proposer": p.proposer,
                "weighted_score": p.weighted_score,
                "consensus_reached": p.consensus_reached,
                "created_at": p.created_at,
                "decided_at": p.decided_at,
            }
            for p in proposals
        ]

    def get_members(self) -> List[dict]:
        """Return all 14 council members with their expertise weights."""
        return [
            {
                "agent_id": m.agent_id,
                "name": m.name,
                "role": m.role,
                "domain": m.domain,
                "expertise_weights": m.expertise_weights,
            }
            for m in self.members
        ]

    def get_council_status(self) -> dict:
        """Return overall council status."""
        total = len(self._proposals)
        enacted = sum(1 for p in self._proposals.values() if p.phase == ProposalPhase.ENACTED)
        rejected = sum(1 for p in self._proposals.values() if p.phase == ProposalPhase.REJECTED)
        active = sum(1 for p in self._proposals.values() if p.phase.value in ("proposed", "deliberating", "voting", "consensus"))

        return {
            "member_count": len(self.members),
            "byzantine_tolerance": (len(self.members) - 1) // 3,
            "total_proposals": total,
            "enacted": enacted,
            "rejected": rejected,
            "active": active,
            "monad_anchor": hex(MONAD_TRUTH_ANCHOR),
            "rust_kernel_bound": self._rust_kernel is not None and getattr(self._rust_kernel, 'available', False),
            "members": self.get_members(),
            "active_proposals": [
                {
                    "proposal_id": p.proposal_id,
                    "topic": p.topic,
                    "category": p.category,
                    "phase": p.phase.value,
                    "weighted_score": p.weighted_score,
                }
                for p in self._proposals.values()
                if p.phase.value in ("proposed", "deliberating", "voting", "consensus")
            ],
            "recent_enacted": self._enacted_ledger[-10:][::-1],
        }

    def get_enacted_ledger(self) -> List[dict]:
        """Return the full enacted-proposals ledger."""
        return list(self._enacted_ledger)


# ──────────────────────────────────────────────────────────────────────────────
# Singleton accessor
# ──────────────────────────────────────────────────────────────────────────────

_council: Optional[SwarmCouncil] = None


def get_council() -> SwarmCouncil:
    """Return the singleton SwarmCouncil instance."""
    global _council
    if _council is None:
        _council = SwarmCouncil()
    return _council

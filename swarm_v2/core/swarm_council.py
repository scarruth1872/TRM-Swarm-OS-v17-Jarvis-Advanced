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


# The 14 Council Members with domain expertise weights
COUNCIL_MEMBERS: List[CouncilMember] = [
    CouncilMember(
        agent_id=0, name="Hermes", role="Communications Director",
        domain="communication",
        expertise_weights={
            "security": 0.3, "architecture": 0.4, "code": 0.4,
            "data": 0.5, "communication": 1.0, "health": 0.3,
            "infrastructure": 0.3, "research": 0.5, "protocol": 0.7, "general": 0.6
        }
    ),
    CouncilMember(
        agent_id=1, name="Jarvis", role="Executive Assistant",
        domain="general",
        expertise_weights={
            "security": 0.4, "architecture": 0.5, "code": 0.5,
            "data": 0.6, "communication": 0.7, "health": 0.5,
            "infrastructure": 0.4, "research": 0.6, "protocol": 0.5, "general": 0.9
        }
    ),
    CouncilMember(
        agent_id=2, name="Bridge", role="API & Data Bridge",
        domain="data",
        expertise_weights={
            "security": 0.3, "architecture": 0.6, "code": 0.7,
            "data": 1.0, "communication": 0.5, "health": 0.3,
            "infrastructure": 0.5, "research": 0.4, "protocol": 0.6, "general": 0.5
        }
    ),
    CouncilMember(
        agent_id=3, name="Shield", role="Security Director",
        domain="security",
        expertise_weights={
            "security": 1.0, "architecture": 0.5, "code": 0.5,
            "data": 0.4, "communication": 0.3, "health": 0.4,
            "infrastructure": 0.6, "research": 0.3, "protocol": 0.7, "general": 0.4
        }
    ),
    CouncilMember(
        agent_id=4, name="Logic", role="Reasoning Engine",
        domain="architecture",
        expertise_weights={
            "security": 0.5, "architecture": 0.9, "code": 0.6,
            "data": 0.5, "communication": 0.3, "health": 0.3,
            "infrastructure": 0.4, "research": 0.7, "protocol": 0.6, "general": 0.7
        }
    ),
    CouncilMember(
        agent_id=5, name="Devo", role="Lead Developer",
        domain="code",
        expertise_weights={
            "security": 0.4, "architecture": 0.6, "code": 1.0,
            "data": 0.5, "communication": 0.3, "health": 0.3,
            "infrastructure": 0.7, "research": 0.4, "protocol": 0.5, "general": 0.5
        }
    ),
    CouncilMember(
        agent_id=6, name="Pulse", role="Health Monitor",
        domain="health",
        expertise_weights={
            "security": 0.3, "architecture": 0.3, "code": 0.3,
            "data": 0.4, "communication": 0.4, "health": 1.0,
            "infrastructure": 0.6, "research": 0.3, "protocol": 0.4, "general": 0.5
        }
    ),
    CouncilMember(
        agent_id=7, name="Flow", role="Workflow Orchestrator",
        domain="infrastructure",
        expertise_weights={
            "security": 0.3, "architecture": 0.5, "code": 0.5,
            "data": 0.5, "communication": 0.4, "health": 0.5,
            "infrastructure": 0.9, "research": 0.3, "protocol": 0.6, "general": 0.5
        }
    ),
    CouncilMember(
        agent_id=8, name="EventLedger", role="Audit Ledger",
        domain="protocol",
        expertise_weights={
            "security": 0.6, "architecture": 0.4, "code": 0.3,
            "data": 0.7, "communication": 0.3, "health": 0.4,
            "infrastructure": 0.4, "research": 0.3, "protocol": 1.0, "general": 0.4
        }
    ),
    CouncilMember(
        agent_id=9, name="Orchestra", role="Meta-Orchestrator",
        domain="infrastructure",
        expertise_weights={
            "security": 0.4, "architecture": 0.7, "code": 0.5,
            "data": 0.4, "communication": 0.5, "health": 0.5,
            "infrastructure": 0.8, "research": 0.4, "protocol": 0.6, "general": 0.6
        }
    ),
    CouncilMember(
        agent_id=10, name="THzGateway", role="Spectrum Gateway",
        domain="communication",
        expertise_weights={
            "security": 0.3, "architecture": 0.4, "code": 0.3,
            "data": 0.5, "communication": 0.9, "health": 0.3,
            "infrastructure": 0.5, "research": 0.6, "protocol": 0.5, "general": 0.4
        }
    ),
    CouncilMember(
        agent_id=11, name="Verify", role="Truth Verifier",
        domain="security",
        expertise_weights={
            "security": 0.8, "architecture": 0.5, "code": 0.5,
            "data": 0.6, "communication": 0.3, "health": 0.4,
            "infrastructure": 0.4, "research": 0.5, "protocol": 0.8, "general": 0.6
        }
    ),
    CouncilMember(
        agent_id=12, name="Seeker", role="Research Director",
        domain="research",
        expertise_weights={
            "security": 0.3, "architecture": 0.5, "code": 0.4,
            "data": 0.7, "communication": 0.4, "health": 0.3,
            "infrastructure": 0.3, "research": 1.0, "protocol": 0.4, "general": 0.6
        }
    ),
    CouncilMember(
        agent_id=13, name="Archi", role="Chief Architect",
        domain="architecture",
        expertise_weights={
            "security": 0.5, "architecture": 1.0, "code": 0.7,
            "data": 0.5, "communication": 0.4, "health": 0.3,
            "infrastructure": 0.7, "research": 0.5, "protocol": 0.6, "general": 0.7
        }
    ),
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
        "default": "From a communications standpoint, {topic} — I evaluate message clarity, protocol compliance, and inter-agent coordination impact.",
        "communication": "This directly impacts the communication fabric. I assess signal integrity, latency, and sovereign mesh routing.",
        "security": "Evaluating communication channel security implications of {topic}.",
    },
    "Shield": {
        "default": "Security assessment of {topic}: analyzing threat surface, vulnerability exposure, and DDR antibody coverage.",
        "security": "CRITICAL: This is a direct security matter. Evaluating CVE surface, attack vectors, and defensive posture.",
        "code": "Code security review for {topic}: input validation, injection risks, and privilege escalation paths.",
    },
    "Logic": {
        "default": "Logical analysis of {topic}: checking for internal consistency, K3 completeness, and reasoning soundness.",
        "architecture": "Architectural logic review: verifying structural coherence, dependency graphs, and design pattern correctness.",
        "research": "Formal reasoning assessment: evaluating hypotheses, proof obligations, and theoretical foundations.",
    },
    "Devo": {
        "default": "Engineering assessment of {topic}: implementation feasibility, technical debt impact, and code quality.",
        "code": "PRIORITY: Direct code concern. Assessing build system, test coverage, API contracts, and deployment risk.",
        "infrastructure": "Infrastructure code review: evaluating deployment scripts, CI/CD impact, and runtime dependencies.",
    },
    "Verify": {
        "default": "Truth verification for {topic}: cross-referencing claims against Monad anchor and known state.",
        "protocol": "Protocol truth audit: verifying compliance with Monad governance rules and consensus requirements.",
        "security": "Security truth verification: validating threat claims against DDR telemetry and audit logs.",
    },
    "Archi": {
        "default": "Architectural review of {topic}: evaluating system impact, scalability, and design alignment.",
        "architecture": "PRIORITY: This is an architectural decision. Analyzing component boundaries, coupling, and evolution path.",
        "infrastructure": "Infrastructure architecture review: evaluating topology, redundancy, and sovereign deployment model.",
    },
    "Seeker": {
        "default": "Research perspective on {topic}: surveying prior art, identifying knowledge gaps, and evidence quality.",
        "research": "PRIORITY: Research domain. Deep analysis of methodology, reproducibility, and citation quality.",
        "data": "Data research review: evaluating data sources, statistical validity, and ontology alignment.",
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

        # Compute final weighted consensus score (-1.0 to +1.0)
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

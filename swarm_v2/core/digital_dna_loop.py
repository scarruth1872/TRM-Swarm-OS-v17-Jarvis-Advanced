"""
digital_dna_loop.py — Cognitive Genes & Resonant Semantic Cache
TRM Swarm OS · Project Continuum V5.3

Implements the 4-dimensional behavioral DNA vector G = [ρ, δ, ε, σ],
the rolling-average refraction model (ResonantSemanticCache), and the
real-time feedback / mutation loop (DigitalDNALoop).
"""

import logging
import time
import math
import copy
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from collections import deque
from dataclasses import dataclass, field, asdict

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Cognitive Gene — 4-D behavioural DNA vector
# ---------------------------------------------------------------------------

@dataclass
class CognitiveGene:
    """
    Represents the 4-dimensional behavioral DNA vector
    G = [ρ (plasticity), δ (logical_depth), ε (empathy), σ (stochasticity)].

    Each dimension is clamped to [0.0, 1.0].
    """

    plasticity: float = 0.5       # ρ — adaptability / learning rate
    logical_depth: float = 0.5    # δ — reasoning depth
    empathy: float = 0.5          # ε — emotional intelligence / collaboration
    stochasticity: float = 0.5    # σ — creative randomness

    def __post_init__(self) -> None:
        self.plasticity = max(0.0, min(1.0, self.plasticity))
        self.logical_depth = max(0.0, min(1.0, self.logical_depth))
        self.empathy = max(0.0, min(1.0, self.empathy))
        self.stochasticity = max(0.0, min(1.0, self.stochasticity))

    # -- serialisation helpers ------------------------------------------------

    def to_vector(self) -> List[float]:
        """Return the gene as a flat list [ρ, δ, ε, σ]."""
        return [self.plasticity, self.logical_depth, self.empathy, self.stochasticity]

    @classmethod
    def from_vector(cls, vec: List[float]) -> "CognitiveGene":
        """Construct a CognitiveGene from a 4-element list."""
        if len(vec) != 4:
            raise ValueError(f"CognitiveGene.from_vector expects exactly 4 elements, got {len(vec)}")
        return cls(
            plasticity=vec[0],
            logical_depth=vec[1],
            empathy=vec[2],
            stochasticity=vec[3],
        )

    # -- distance -------------------------------------------------------------

    def distance(self, other: "CognitiveGene") -> float:
        """Euclidean distance between two CognitiveGene vectors."""
        return math.sqrt(
            (self.plasticity - other.plasticity) ** 2
            + (self.logical_depth - other.logical_depth) ** 2
            + (self.empathy - other.empathy) ** 2
            + (self.stochasticity - other.stochasticity) ** 2
        )


# ---------------------------------------------------------------------------
# System presets
# ---------------------------------------------------------------------------

SYSTEM_PRESETS: Dict[str, CognitiveGene] = {
    "continuum": CognitiveGene(plasticity=0.7, logical_depth=0.8, empathy=0.6, stochasticity=0.5),
    "sage":      CognitiveGene(plasticity=0.4, logical_depth=0.95, empathy=0.7, stochasticity=0.2),
    "muse":      CognitiveGene(plasticity=0.9, logical_depth=0.5, empathy=0.8, stochasticity=0.9),
    "sentinel":  CognitiveGene(plasticity=0.3, logical_depth=0.9, empathy=0.3, stochasticity=0.1),
}


# ---------------------------------------------------------------------------
# DNA Mutation Proposal
# ---------------------------------------------------------------------------

@dataclass
class DNAMutationProposal:
    """Captures a proposed (or applied) gene mutation for audit."""

    proposal_id: str
    source_role: str          # agent role that triggered the mutation
    gene_before: CognitiveGene
    gene_after: CognitiveGene
    reason: str
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    status: str = "proposed"  # proposed | approved | rejected | applied


# ---------------------------------------------------------------------------
# Resonant Semantic Cache — rolling-average refraction model
# ---------------------------------------------------------------------------

class ResonantSemanticCache:
    """
    Maintains a sliding window of CognitiveGene snapshots and computes
    a running average Ḡ = [mean(ρ_i), mean(δ_i), mean(ε_i), mean(σ_i)].
    """

    def __init__(self, maxlen: int = 50) -> None:
        self._history: deque = deque(maxlen=maxlen)
        self._current_average: CognitiveGene = copy.deepcopy(
            SYSTEM_PRESETS.get("continuum", CognitiveGene())
        )
        logger.debug("ResonantSemanticCache initialised (window=%d)", maxlen)

    # -- public API -----------------------------------------------------------

    def record(self, gene: CognitiveGene) -> None:
        """Append a gene snapshot and recalculate the running average."""
        self._history.append(copy.deepcopy(gene))
        self._recalculate_average()

    def get_refraction_context(self) -> dict:
        """Return a summary dict of the current cache state."""
        continuum = SYSTEM_PRESETS.get("continuum", CognitiveGene())
        return {
            "average_gene": asdict(self._current_average),
            "window_size": len(self._history),
            "drift_from_continuum": round(self._current_average.distance(continuum), 6),
        }

    def detect_drift(self, threshold: float = 0.3) -> bool:
        """Return True if the average gene has drifted beyond *threshold* from continuum."""
        continuum = SYSTEM_PRESETS.get("continuum", CognitiveGene())
        drift = self._current_average.distance(continuum)
        if drift > threshold:
            logger.warning(
                "Semantic drift detected: %.4f > threshold %.4f", drift, threshold
            )
        return drift > threshold

    # -- internals ------------------------------------------------------------

    def _recalculate_average(self) -> None:
        """Compute Ḡ from the current history window."""
        n = len(self._history)
        if n == 0:
            return
        sums = [0.0, 0.0, 0.0, 0.0]
        for gene in self._history:
            vec = gene.to_vector()
            for i in range(4):
                sums[i] += vec[i]
        self._current_average = CognitiveGene.from_vector([s / n for s in sums])


# ---------------------------------------------------------------------------
# Digital DNA Loop — real-time feedback & mutation loop (singleton)
# ---------------------------------------------------------------------------

class DigitalDNALoop:
    """
    Core feedback loop that monitors agent performance, proposes and applies
    gene mutations, and records snapshots into the ResonantSemanticCache.
    """

    def __init__(self) -> None:
        self._agent_genes: Dict[str, CognitiveGene] = {}
        self._cache: ResonantSemanticCache = ResonantSemanticCache()
        self._mutation_proposals: List[DNAMutationProposal] = []
        self._self_healing = None   # SelfHealingOrchestrator reference
        self._ddr = None            # DigitalDNARepository reference
        self._active: bool = False
        self._proposal_counter: int = 0
        logger.info("DigitalDNALoop instance created")

    # -- binding --------------------------------------------------------------

    def bind(self, self_healing=None, ddr=None) -> None:
        """Store references to external orchestrators / repositories."""
        if self_healing is not None:
            self._self_healing = self_healing
            logger.info("DigitalDNALoop bound to SelfHealingOrchestrator")
        if ddr is not None:
            self._ddr = ddr
            logger.info("DigitalDNALoop bound to DigitalDNARepository")

    # -- initialisation -------------------------------------------------------

    def initialize_agent_genes(
        self, agent_roles: List[str], preset: str = "continuum"
    ) -> None:
        """Set all listed agent roles to the given preset gene."""
        base_gene = SYSTEM_PRESETS.get(preset)
        if base_gene is None:
            logger.warning(
                "Preset '%s' not found — falling back to 'continuum'", preset
            )
            base_gene = SYSTEM_PRESETS["continuum"]

        for role in agent_roles:
            self._agent_genes[role] = copy.deepcopy(base_gene)
            self._cache.record(self._agent_genes[role])
            logger.debug("Agent '%s' initialised with preset '%s'", role, preset)

        logger.info(
            "Initialised %d agent genes with preset '%s'", len(agent_roles), preset
        )

    # -- accessors ------------------------------------------------------------

    def get_agent_gene(self, role: str) -> CognitiveGene:
        """Return the current gene for *role* (defaults to continuum preset)."""
        if role not in self._agent_genes:
            logger.debug(
                "Role '%s' has no gene — assigning continuum default", role
            )
            self._agent_genes[role] = copy.deepcopy(SYSTEM_PRESETS["continuum"])
        return self._agent_genes[role]

    # -- performance evaluation & mutation proposal ---------------------------

    def _next_proposal_id(self) -> str:
        self._proposal_counter += 1
        return f"MUT-{self._proposal_counter:06d}"

    @staticmethod
    def _clamp(value: float) -> float:
        return max(0.0, min(1.0, value))

    def evaluate_performance(
        self,
        role: str,
        error_rate: float,
        avg_latency: float,
    ) -> Optional[DNAMutationProposal]:
        """
        Examine metrics and propose a gene mutation when performance degrades
        (or reward exploration when performance is excellent).

        Rules
        -----
        - error_rate > 0.15  → reduce σ by 0.1, increase δ by 0.1
        - avg_latency > 5.0  → increase ρ by 0.15
        - error_rate < 0.02  → increase σ by 0.05 (reward exploration)
        """
        try:
            current = self.get_agent_gene(role)
            mutated = copy.deepcopy(current)
            reasons: List[str] = []

            # --- high error rate ---
            if error_rate > 0.15:
                mutated.stochasticity = self._clamp(mutated.stochasticity - 0.1)
                mutated.logical_depth = self._clamp(mutated.logical_depth + 0.1)
                reasons.append(
                    f"high error_rate ({error_rate:.3f}>0.15): σ-0.1, δ+0.1"
                )

            # --- high latency ---
            if avg_latency > 5.0:
                mutated.plasticity = self._clamp(mutated.plasticity + 0.15)
                reasons.append(
                    f"high avg_latency ({avg_latency:.2f}s>5.0s): ρ+0.15"
                )

            # --- reward exploration (only if no degradation) ---
            if not reasons and error_rate < 0.02:
                mutated.stochasticity = self._clamp(mutated.stochasticity + 0.05)
                reasons.append(
                    f"excellent error_rate ({error_rate:.3f}<0.02): σ+0.05 (exploration reward)"
                )

            if not reasons:
                return None

            # Record snapshot in the cache
            self._cache.record(mutated)

            proposal = DNAMutationProposal(
                proposal_id=self._next_proposal_id(),
                source_role=role,
                gene_before=copy.deepcopy(current),
                gene_after=mutated,
                reason="; ".join(reasons),
            )
            self._mutation_proposals.append(proposal)
            logger.info(
                "Mutation proposed for '%s': %s [%s]",
                role,
                proposal.proposal_id,
                proposal.reason,
            )
            return proposal

        except Exception as exc:
            logger.error(
                "evaluate_performance failed for role '%s': %s", role, exc, exc_info=True
            )
            return None

    # -- mutation application -------------------------------------------------

    def apply_mutation(self, proposal_id: str) -> bool:
        """Find a proposal by ID, apply the mutated gene, and mark as 'applied'."""
        try:
            for proposal in self._mutation_proposals:
                if proposal.proposal_id == proposal_id:
                    if proposal.status == "applied":
                        logger.warning(
                            "Proposal %s already applied — skipping", proposal_id
                        )
                        return False

                    self._agent_genes[proposal.source_role] = copy.deepcopy(
                        proposal.gene_after
                    )
                    proposal.status = "applied"
                    logger.info(
                        "Mutation %s applied to '%s'",
                        proposal_id,
                        proposal.source_role,
                    )
                    self._push_dna_refraction_to_jarvis()
                    return True

            logger.warning("Proposal '%s' not found", proposal_id)
            return False

        except Exception as exc:
            logger.error(
                "apply_mutation failed for '%s': %s", proposal_id, exc, exc_info=True
            )
            return False

    def _push_dna_refraction_to_jarvis(self):
        """Pushes average DNA parameters to Jarvis OS config endpoint (Phase B)."""
        if not self._agent_genes:
            return
        try:
            import urllib.request
            import json
            
            total_stoch = 0.0
            total_logical = 0.0
            count = 0
            
            for gene in self._agent_genes.values():
                total_stoch += gene.stochasticity
                total_logical += gene.logical_depth
                count += 1
                
            avg_stoch = total_stoch / count
            avg_logical = total_logical / count
            
            url = "http://localhost:3000/config/dna"
            payload = {
                "stochasticity": avg_stoch,
                "logical_depth": avg_logical
            }
            
            req = urllib.request.Request(
                url,
                data=json.dumps(payload).encode("utf-8"),
                headers={"Content-Type": "application/json"},
                method="POST"
            )
            
            def push():
                try:
                    with urllib.request.urlopen(req, timeout=1.0) as resp:
                        pass
                except Exception:
                    pass
                    
            import threading
            threading.Thread(target=push, daemon=True).start()
            logger.info("[DNA Refraction] Dispatched averages to Jarvis: stoch=%.3f, logical=%.3f", avg_stoch, avg_logical)
        except Exception as e:
            logger.debug("[DNA Refraction] Push failed: %s", e)

    # -- async monitor loop ---------------------------------------------------

    async def monitor_loop(self, engine_team: dict) -> None:
        """
        Background coroutine that periodically evaluates every agent role and
        auto-applies mutations when warranted.

        Steps per cycle (30 s interval):
        1. Fetch health score from SelfHealingOrchestrator (if bound).
        2. Derive error_rate & avg_latency from self-healing windows.
        3. Call evaluate_performance().
        4. Auto-apply any resulting mutation.
        5. If DDR is bound and error_rate > 0.2, record an antibody.
        """
        self._active = True
        logger.info("DigitalDNALoop monitor_loop started")

        while self._active:
            try:
                await asyncio.sleep(30)

                roles = list(engine_team.keys()) if engine_team else list(self._agent_genes.keys())

                for role in roles:
                    try:
                        error_rate = 0.0
                        avg_latency = 0.0
                        health_score = 1.0

                        # --- gather metrics from SelfHealingOrchestrator -----
                        if self._self_healing is not None:
                            try:
                                health = getattr(
                                    self._self_healing, "get_health_score", None
                                )
                                if callable(health):
                                    health_score = health(role)

                                windows = getattr(
                                    self._self_healing, "get_error_windows", None
                                )
                                if callable(windows):
                                    win = windows(role)
                                    if isinstance(win, dict):
                                        error_rate = win.get("error_rate", 0.0)
                                        avg_latency = win.get("avg_latency", 0.0)

                            except Exception as sh_exc:
                                logger.debug(
                                    "Could not read self-healing metrics for '%s': %s",
                                    role,
                                    sh_exc,
                                )

                        # --- evaluate & auto-apply ---------------------------
                        proposal = self.evaluate_performance(role, error_rate, avg_latency)
                        if proposal is not None:
                            self.apply_mutation(proposal.proposal_id)

                        # --- record antibody on severe error rates -----------
                        if self._ddr is not None and error_rate > 0.2:
                            try:
                                record_fn = getattr(
                                    self._ddr, "record_antibody", None
                                )
                                if callable(record_fn):
                                    record_fn(
                                        role=role,
                                        error_rate=error_rate,
                                        timestamp=datetime.utcnow().isoformat(),
                                    )
                                    logger.info(
                                        "Antibody recorded for '%s' (error_rate=%.3f)",
                                        role,
                                        error_rate,
                                    )
                            except Exception as ddr_exc:
                                logger.debug(
                                    "Could not record antibody for '%s': %s",
                                    role,
                                    ddr_exc,
                                )

                    except Exception as role_exc:
                        logger.error(
                            "Monitor loop error for role '%s': %s",
                            role,
                            role_exc,
                            exc_info=True,
                        )

            except asyncio.CancelledError:
                logger.info("DigitalDNALoop monitor_loop cancelled")
                break
            except Exception as loop_exc:
                logger.error(
                    "DigitalDNALoop monitor_loop cycle error: %s",
                    loop_exc,
                    exc_info=True,
                )

        self._active = False
        logger.info("DigitalDNALoop monitor_loop stopped")

    # -- status ---------------------------------------------------------------

    def get_status(self) -> dict:
        """Return a snapshot of the loop's current state."""
        try:
            return {
                "agent_genes": {
                    role: asdict(gene) for role, gene in self._agent_genes.items()
                },
                "mutation_proposals": [
                    {
                        "proposal_id": p.proposal_id,
                        "source_role": p.source_role,
                        "gene_before": asdict(p.gene_before),
                        "gene_after": asdict(p.gene_after),
                        "reason": p.reason,
                        "timestamp": p.timestamp,
                        "status": p.status,
                    }
                    for p in self._mutation_proposals
                ],
                "cache": self._cache.get_refraction_context(),
                "active": self._active,
            }
        except Exception as exc:
            logger.error("get_status failed: %s", exc, exc_info=True)
            return {"error": str(exc)}

    # -- lifecycle ------------------------------------------------------------

    def stop(self) -> None:
        """Signal the monitor loop to exit on its next cycle."""
        self._active = False
        logger.info("DigitalDNALoop stop requested")


# ---------------------------------------------------------------------------
# Singleton accessor
# ---------------------------------------------------------------------------

_dna_loop: Optional[DigitalDNALoop] = None


def get_digital_dna_loop() -> DigitalDNALoop:
    """Return the process-wide DigitalDNALoop singleton."""
    global _dna_loop
    if _dna_loop is None:
        _dna_loop = DigitalDNALoop()
    return _dna_loop

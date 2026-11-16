"""
Mutation Engine — Phase 16: Autonomous Evolution
Identifies optimization targets and generates code mutations for self-improvement.
"""

import asyncio
import logging
import os
import random
from datetime import datetime
from typing import Any, Dict, List, Optional

from pydantic import BaseModel

logger = logging.getLogger(__name__)


class GeneticProposal(BaseModel):
    """Represents a proposed code mutation."""

    proposal_id: str
    target_file: str
    mutation_type: str  # "performance", "readability", "refactor", "security"
    description: str
    original_code: str
    mutated_code: str
    timestamp: datetime = datetime.now()
    score: float = 0.0
    status: str = "pending"  # pending, testing, passed, failed, integrated


class MutationEngine:
    """
    The 'Mutation Engine' drives the self-evolution of Swarm OS.
    It identifies weak points in the architecture and proposes recursive improvements.
    """

    def __init__(self):
        self.proposals: Dict[str, GeneticProposal] = {}
        self.active_targets: List[str] = []

    async def analyze_codebase(self, root_dir: str = "swarm_v2/core") -> List[str]:
        """Scan the codebase to find files with high complexity or low test coverage."""
        targets = []
        for root, dirs, files in os.walk(root_dir):
            for file in files:
                if file.endswith(".py") and "__" not in file:
                    path = os.path.join(root, file)
                    # Simple heuristic: files > 15KB or containing "TODO" are targets
                    if os.path.getsize(path) > 15000:
                        targets.append(path)

        self.active_targets = targets
        logger.info(
            f"[MutationEngine] Identified {len(targets)} potential evolution targets."
        )
        return targets

    async def propose_mutation(
        self, target_file: str, agents: Dict[str, Any]
    ) -> Optional[GeneticProposal]:
        """Ask the Architect and Lead Developer to propose a code improvement."""
        architect = agents.get("Architect")
        if not architect:
            return None

        with open(target_file, "r", encoding="utf-8") as f:
            code = f.read()

        prompt = (
            f"As the Swarm Architect, analyze the following code from '{target_file}'.\n"
            f"Identify ONE specific area for improvement (performance, security, or readability).\n"
            f"Explain WHY it should be improved and provide a blueprint for the change.\n"
            f"Blueprints must be production-ready and follow Pydantic v2 standards (use 'pattern' instead of 'regex', 'field_validator' instead of 'validator').\n\n"
            f"CODE:\n{code[:4000]}"
        )

        analysis_result = await architect.process_task(prompt, sender="mutation_engine")
        analysis = (
            analysis_result["response"]
            if isinstance(analysis_result, dict)
            else analysis_result
        )

        # Now ask Lead Developer to implement the mutation
        developer = agents.get("Lead Developer")
        if not developer:
            return None

        dev_prompt = (
            f"Based on the following architectural analysis, rewrite the code for '{target_file}'.\n"
            f"Ensure the change is backwards compatible and follows production standards.\n\n"
            f"ANALYSIS:\n{analysis}\n\n"
            f"ORIGINAL CODE:\n{code[:4000]}"
        )

        mutated_result = await developer.process_task(
            dev_prompt, sender="mutation_engine"
        )
        mutated_code = (
            mutated_result["response"]
            if isinstance(mutated_result, dict)
            else mutated_result
        )

        # Phase 16: Robust Code Extraction (v3 - Pipeline)
        extracted = mutated_code

        # Pass 1: Identify the most likely code block
        if "```python" in extracted:
            extracted = extracted.split("```python")[1].split("```")[0].strip()
        elif "WRITE_FILE:" in extracted:
            extracted = extracted.split("WRITE_FILE:")[1]
            if "\n" in extracted:
                extracted = extracted.split("\n", 1)[1].strip()
        elif "```" in extracted:
            potential = extracted.split("```")[1].split("```")[0].strip()
            if "import " in potential or "def " in potential or "class " in potential:
                extracted = potential

        # Pass 2: Aggressive Marker Stripping (handle nesting/repetitions)
        for marker in ["[START]", "[END]", "WRITE_FILE:"]:
            if marker in extracted:
                # If marker exists, take everything after the last START or before the first END
                if marker == "[START]":
                    extracted = extracted.rsplit("[START]", 1)[-1].strip()
                elif marker == "[END]":
                    extracted = extracted.split("[END]")[0].strip()
                elif marker == "WRITE_FILE:":
                    # If it's still here, try to skip the filename line again
                    if "\n" in extracted:
                        extracted = extracted.split("\n", 1)[1].strip()

        # Pass 3: Clean up any remaining markdown fences
        if "```" in extracted:
            extracted = extracted.replace("```python", "").replace("```", "").strip()

        mutated_code = extracted

        proposal = GeneticProposal(
            proposal_id=f"mutation_{random.randint(1000, 9999)}",
            target_file=target_file,
            mutation_type="optimization",
            description=analysis[:200],
            original_code=code,
            mutated_code=mutated_code,
        )

        self.proposals[proposal.proposal_id] = proposal
        logger.info(
            f"[MutationEngine] Generated proposal {proposal.proposal_id} for {target_file}"
        )
        return proposal

    def get_proposals(self) -> List[dict]:
        return [p.model_dump() for p in self.proposals.values()]


# Singleton
_engine: Optional[MutationEngine] = None


def get_mutation_engine() -> MutationEngine:
    global _engine
    if _engine is None:
        _engine = MutationEngine()
    return _engine

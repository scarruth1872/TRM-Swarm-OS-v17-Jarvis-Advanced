"""AURA Endpoints — /aura/execute + /aura/metaconcepts + /aura/field + /aura/stats + /aura/cognition/branch."""
from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional, List, Dict, Any

from swarm_v2.core.aura_engine import get_aura_engine, METACONCEPT_CATEGORIES

router = APIRouter(tags=["aura"])

class AuraExecutionRequest(BaseModel):
    code: str
    session_id: Optional[str] = "aura-default"
    preserve_field: Optional[bool] = True

class AuraCognitionBranchRequest(BaseModel):
    directive: str
    seed_concepts: Optional[List[str]] = None
    depth: Optional[int] = 2

@router.get("/aura/stats")
async def get_aura_stats():
    """Returns live AURA language engine statistics."""
    engine = get_aura_engine()
    return engine.get_stats()

@router.get("/aura/metaconcepts")
async def get_aura_metaconcepts():
    """Returns the full 100-metaconcept library and categories."""
    engine = get_aura_engine()
    return {
        "categories": METACONCEPT_CATEGORIES,
        "metaconcepts": engine.get_metaconcepts(),
        "total": len(engine.metaconcepts)
    }

@router.get("/aura/field")
async def get_aura_field_state():
    """Returns live 2D/3D semantic field topology and active connections."""
    engine = get_aura_engine()
    return engine.get_field()

@router.post("/aura/execute")
async def execute_aura_code(req: AuraExecutionRequest):
    """Executes AURA DSL code and evaluates resonant wavefunctions & cognitive branching."""
    engine = get_aura_engine()
    return engine.execute(req.code)

@router.post("/aura/cognition/branch")
async def branch_aura_cognition(req: AuraCognitionBranchRequest):
    """Formulates AURA intelligence by evaluating a Cognitive Branching Tree via Kleene K3 logic and Voltage Doctrine."""
    engine = get_aura_engine()
    return engine.branch_cognition(req.directive, req.seed_concepts, req.depth or 2)

@router.post("/aura/council-integrate")
async def integrate_aura_with_council():
    """Submits and enacts a formal AURA Quantum Resonance integration proposal through the 14 Council Agents."""
    from swarm_v2.core.swarm_council import get_council
    engine = get_aura_engine()
    council = get_council()
    
    # 1. Execute formal AURA bootstrap code
    bootstrap_code = (
        "// AURA Council Invariant Bootstrap Protocol\n"
        "MonadTruth := { immutability: 1.0, verity: 0.95 }\n"
        "Unity ~> ResonanceCascade ~> MonadTruth\n"
        "Superposition ~|~ LawOfOne -> +1 (COUNCIL_QUORUM_SEALED)\n"
        "@sealMonadBlock"
    )
    aura_result = engine.execute(bootstrap_code)
    
    # 2. Run Full Council Cycle
    council_result = council.run_full_cycle(
        topic="Integrate AURA Quantum Resonance Architecture as Sovereign Core Primitive",
        category="Architecture",
        description=(
            "Formal integration of the Autonomous Universal Resonant Architecture (AURA) quantum DSL, "
            "100-Metaconcept semantic field state, wavefunction collapse operators (:=, ~>, ~|~, ->), "
            "and AURA Studio (Tab 26) under Monad Truth Anchor 0x4D4F4E4144000001."
        ),
        proposer="Jarvis"
    )
    
    return {
        "status": "AURA_COUNCIL_INTEGRATED",
        "aura_execution": aura_result,
        "council_cycle": council_result
    }

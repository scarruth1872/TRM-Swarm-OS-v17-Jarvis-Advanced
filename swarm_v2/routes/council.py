"""
council.py — TRM Swarm Council API Routes
TRM Swarm OS v19.001 "Sovereign"

FastAPI router for council operations: propose, deliberate, vote, enact, query.
"""

import logging
from typing import Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from swarm_v2.core.swarm_council import get_council
from swarm_v2.core import evidence_retriever

logger = logging.getLogger(__name__)
router = APIRouter(tags=["council"])


# ──────────────────────────────────────────────────────────────────────────────
# Request Models
# ──────────────────────────────────────────────────────────────────────────────

class CouncilProposeRequest(BaseModel):
    topic: str
    category: str = "general"
    description: str = ""
    proposer: str = "user"


class CouncilFullCycleRequest(BaseModel):
    topic: str
    category: str = "general"
    description: str = ""
    proposer: str = "council"


# ──────────────────────────────────────────────────────────────────────────────
# Council Endpoints
# ──────────────────────────────────────────────────────────────────────────────

@router.get("/council/status")
async def council_status():
    """Get overall council status, member list, active proposals, and stats."""
    try:
        council = get_council()
        return council.get_council_status()
    except Exception as e:
        logger.error("[Council API] status error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/council/members")
async def council_members():
    """List all 14 council members with their domain expertise weights."""
    try:
        council = get_council()
        return {"members": council.get_members()}
    except Exception as e:
        logger.error("[Council API] members error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/council/proposals")
async def council_proposals(limit: int = 50):
    """List all proposals with summary info."""
    try:
        council = get_council()
        return {"proposals": council.list_proposals(limit=limit)}
    except Exception as e:
        logger.error("[Council API] proposals error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/council/proposal/{proposal_id}")
async def council_proposal_detail(proposal_id: str):
    """Get full proposal details including deliberations and votes."""
    try:
        council = get_council()
        result = council.get_proposal(proposal_id)
        if result is None:
            raise HTTPException(status_code=404, detail=f"Proposal {proposal_id} not found")
        return result
    except HTTPException:
        raise
    except Exception as e:
        logger.error("[Council API] proposal detail error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/council/propose")
async def council_propose(req: CouncilProposeRequest):
    """Submit a new proposal for council deliberation."""
    try:
        council = get_council()
        proposal = council.propose(
            topic=req.topic,
            category=req.category,
            description=req.description,
            proposer=req.proposer,
        )
        return {
            "proposal_id": proposal.proposal_id,
            "topic": proposal.topic,
            "category": proposal.category,
            "phase": proposal.phase.value,
            "created_at": proposal.created_at,
        }
    except Exception as e:
        logger.error("[Council API] propose error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/council/deliberate/{proposal_id}")
async def council_deliberate(proposal_id: str):
    """Trigger deliberation round — all 14 agents provide domain-expert reasoning."""
    try:
        council = get_council()
        deliberations = council.deliberate(proposal_id)
        return {
            "proposal_id": proposal_id,
            "phase": "deliberating",
            "deliberation_count": len(deliberations),
            "deliberations": [
                {
                    "agent_name": d.agent_name,
                    "position": d.position,
                    "reasoning": d.reasoning,
                    "domain_relevance": d.domain_relevance,
                    "timestamp": d.timestamp,
                }
                for d in deliberations
            ],
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error("[Council API] deliberate error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/council/vote/{proposal_id}")
async def council_vote(proposal_id: str):
    """Trigger K3 ternary voting — all 14 agents cast weighted votes."""
    try:
        council = get_council()
        result = council.vote(proposal_id)
        return result
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error("[Council API] vote error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/council/enact/{proposal_id}")
async def council_enact(proposal_id: str):
    """Enact a consensus-reached proposal, sealing with Monad truth anchor."""
    try:
        council = get_council()
        result = council.enact(proposal_id)
        return result
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error("[Council API] enact error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/council/full-cycle")
async def council_full_cycle(req: CouncilFullCycleRequest):
    """Execute the complete council cycle in one call: propose → deliberate → vote → enact."""
    try:
        council = get_council()
        result = council.run_full_cycle(
            topic=req.topic,
            category=req.category,
            description=req.description,
            proposer=req.proposer,
        )
        return result
    except Exception as e:
        logger.error("[Council API] full-cycle error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/council/ledger")
async def council_ledger():
    """Return the full enacted-proposals ledger."""
    try:
        council = get_council()
        return {"ledger": council.get_enacted_ledger()}
    except Exception as e:
        logger.error("[Council API] ledger error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/council/evidence")
async def council_evidence(q: str, k: int = 3):
    """Retrieve the platform's own written record for a query.

    Used by the Rust council so each agent can cite a document instead of
    relying on a 1.5B model's prior knowledge. Returns [] when nothing clears
    the score floor - an empty list means "no evidence", not "no match".
    """
    try:
        hits = evidence_retriever.retrieve(q, k=max(1, min(k, 8)))
        return {"query": q, "n": len(hits), "evidence": hits,
                "index": evidence_retriever.stats()}
    except Exception as e:
        logger.error("[Council API] evidence error: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))

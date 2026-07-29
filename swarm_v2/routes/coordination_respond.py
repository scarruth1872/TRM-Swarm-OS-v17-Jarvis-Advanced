"""
Coordination responder background task.
Trigger: POST /coordination/respond  — polls all agents and collects proposals.
"""
from fastapi import APIRouter
from pydantic import BaseModel
import asyncio
from swarm_v2.core.coordination_responder import get_coordination_responder

router = APIRouter(tags=["coordination"])


@router.post("/coordination/respond")
async def trigger_coordination_response():
    """Poll all agent mailboxes and auto-respond to pending coordination tasks."""
    from swarm_v2.app_v2 import engine_team

    # Use the Architect's LLM to generate responses for all agents
    architect = engine_team.get("Architect")
    llm_fn = architect._llm_generate if architect and hasattr(architect, '_llm_generate') else None

    if not llm_fn:
        return {"status": "error", "message": "No LLM function available (Architect not initialized)"}

    responder = get_coordination_responder()
    asyncio.create_task(responder.poll_all_agents(llm_generate_fn=llm_fn))

    return {
        "status": "started",
        "message": "Coordination responder polling all agents in background",
    }

"""Multi-agent coordination routes — broadcast, consensus, and synchronization."""
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import time
from swarm_v2.core.coordination_protocol import get_coordination_protocol

router = APIRouter(tags=["coordination"])


class BroadcastRequest(BaseModel):
    task: str
    agents: list[str] = None
    context: str = ""
    timeout: int = 30
    method: str = "majority"


class ConsensusRequest(BaseModel):
    round_id: str
    method: str = "majority"
    max_wait: int = 60


@router.post("/coordination/broadcast")
async def broadcast_task(req: BroadcastRequest):
    """Broadcast a task to agents — fire-and-forget."""
    import asyncio
    protocol = get_coordination_protocol()
    # Fire in background to avoid timeout
    asyncio.create_task(protocol.run_coordination_round(
        task=req.task,
        agents=req.agents,
        context=req.context,
        method=req.method,
        timeout=req.timeout,
    ))
    return {
        "status": "broadcast",
        "message": f"Task broadcast to {len(req.agents or [])} agent(s)",
        "task": req.task[:80],
    }


@router.get("/coordination/round/{round_id}")
async def get_round_status(round_id: str):
    """Get status and collected responses for a coordination round."""
    protocol = get_coordination_protocol()
    round_data = protocol.active_rounds.get(round_id)
    if not round_data:
        raise HTTPException(status_code=404, detail=f"Round {round_id} not found")
    return {
        "round_id": round_id,
        "task": round_data["task"],
        "responses": round_data["responses"],
        "response_count": len(round_data["responses"]),
        "agents_sent": len(round_data["sent_to"]),
        "elapsed": round(time.time() - round_data["started_at"], 1) if "started_at" in round_data else 0,
    }


@router.get("/coordination/rounds")
async def list_rounds():
    """List all active coordination rounds."""
    protocol = get_coordination_protocol()
    return {
        "active_rounds": list(protocol.active_rounds.keys()),
        "count": len(protocol.active_rounds),
    }

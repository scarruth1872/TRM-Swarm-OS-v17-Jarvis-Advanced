"""Global memory endpoints — /memory."""
from typing import Dict, Any, List, Optional
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    GlobalMemoryContributeRequest,
    GlobalMemoryQueryRequest,
)
from swarm_v2.core.global_memory import get_global_memory

router = APIRouter(tags=["memory"])


@router.post("/memory/contribute")
async def memory_contribute(req: GlobalMemoryContributeRequest):
    """Contribute a memory to the global shared pool."""
    global_memory = get_global_memory()
    entry = global_memory.contribute(
        content=req.content,
        author=req.author,
        author_role=req.author_role,
        memory_type=req.memory_type,
        tags=req.tags,
    )
    return {"status": "contributed", "entry": entry.to_dict()}


@router.post("/memory/query")
async def memory_query(req: GlobalMemoryQueryRequest):
    """Query the global memory with vector similarity."""
    global_memory = get_global_memory()
    results = global_memory.query(
        query_text=req.query,
        top_k=req.top_k,
        author_filter=req.author_filter or None,
        type_filter=req.type_filter or None,
    )
    return {
        "query": req.query,
        "results": [{"score": score, "entry": entry} for score, entry in results],
        "total_matches": len(results),
    }


@router.get("/memory/stats")
async def memory_stats():
    return get_global_memory().get_stats()


@router.get("/memory/export")
async def memory_export():
    return get_global_memory().export_all()


@router.post("/memory/sync/{agent_role}")
async def memory_sync_agent(agent_role: str):
    """Sync an agent's local memory into the global pool."""
    from swarm_v2.app_v2 import engine_team
    global_memory = get_global_memory()

    agent = engine_team.get(agent_role)
    if not agent:
        raise HTTPException(status_code=404, detail=f"Agent '{agent_role}' not found")

    # Collect memories from the agent
    local_memories = []
    for fact in agent.memory.long_term:
        if fact.get("type") == "fact":
            local_memories.append({
                "content": fact["content"],
                "type": "fact",
                "tags": agent.persona.specialties[:3],
            })
        elif fact.get("type") == "task_result":
            local_memories.append({
                "content": f"{fact['task']}: {fact['result']}",
                "type": "experience",
                "tags": agent.persona.specialties[:3],
            })

    added = global_memory.sync_from_agent(
        agent_name=agent.persona.name,
        agent_role=agent_role,
        memories=local_memories,
    )
    return {
        "agent": agent_role,
        "memories_submitted": len(local_memories),
        "memories_added": added,
    }

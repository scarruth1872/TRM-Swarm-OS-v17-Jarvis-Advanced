"""Global memory endpoints — /memory."""
from typing import Dict, Any, List, Optional
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.api_models import (
    GlobalMemoryContributeRequest,
    GlobalMemoryQueryRequest,
)
from swarm_v2.core.global_memory import get_global_memory

router = APIRouter(tags=["memory"])


@router.post("/memory/contribute")
async def memory_contribute(req: GlobalMemoryContributeRequest):
    """Contribute a memory to the global shared pool.

    MEASURED DEFECT FIXED 2026-11-09: this handler called `entry.to_dict()` on the
    return value of `contribute()`, which is a STRING id ("gm_efe0acb6") - or one
    of three sentinels ("rejected_by_dge", "skipped_poison",
    "skipped_model_failure"). So every single HTTP contribution raised
    AttributeError and answered 500 while the core wrote the row anyway: the
    write path was 100% broken for HTTP callers and silently successful on disk.
    The core's contract is an id plus sentinels, so the route reports exactly
    that, and never claims a contribution when the guard refused one.
    """
    global_memory = get_global_memory()
    result = global_memory.contribute(
        content=req.content,
        author=req.author,
        author_role=req.author_role,
        memory_type=req.memory_type,
        tags=req.tags,
    )
    sentinels = {"rejected_by_dge", "skipped_poison", "skipped_model_failure"}
    if result in sentinels:
        return {"status": result, "entry_id": None, "stored": False}
    return {"status": "contributed", "entry_id": result, "stored": True}


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


@router.get("/memory/entry/{entry_id}")
async def memory_get_entry(entry_id: str):
    """Read one entry back by id -- the counterpart to /memory/contribute.

    Without this, a caller that stored something had no way to prove it: /memory/query
    ranks by vector similarity, so an id alone retrieves nothing. Answers found=False
    rather than 404 so a missing row is a readable result, not an error page.
    """
    entry = get_global_memory().get(entry_id)
    if not entry:
        return {"found": False, "entry_id": entry_id, "entry": None}
    return {"found": True, "entry_id": entry_id, "entry": entry}


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

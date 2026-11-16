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
    entry_dict = entry.to_dict() if hasattr(entry, "to_dict") else {
        "id": str(entry),
        "content": req.content,
        "author": req.author,
        "author_role": req.author_role,
        "memory_type": req.memory_type,
        "tags": req.tags,
    }
    return {"status": "contributed", "entry": entry_dict}


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


@router.get("/memory/recent")
async def memory_recent(limit: int = 25):
    """Retrieve the most recent memory entries for dashboard display."""
    global_memory = get_global_memory()
    entries = global_memory.get_recent_entries(limit=limit)
    return {"entries": entries, "count": len(entries)}


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


@router.get("/memory/intel")
@router.get("/swarm/intel")
async def get_shared_intel():
    """Retrieve live shared intelligence reservoir feed, research findings, and memory stats."""
    import glob, os
    from datetime import datetime
    from pathlib import Path

    global_memory = get_global_memory()
    stats = global_memory.get_stats()
    recent = global_memory.get_recent_entries(limit=30)

    # Fetch top latest research paper headlines
    papers = []
    research_files = sorted(glob.glob("swarm_v2_artifacts/research/*.md"), key=os.path.getmtime, reverse=True)[:15]
    for rf in research_files:
        try:
            with open(rf, "r", encoding="utf-8", errors="ignore") as fp:
                first_lines = [fp.readline().strip() for _ in range(5)]
                title = next((l.replace("# Research Finding: ", "").replace("#", "").strip() for l in first_lines if l.startswith("#")), Path(rf).stem)
                papers.append({
                    "id": Path(rf).stem,
                    "title": title,
                    "file": Path(rf).name,
                    "author": "Seeker",
                    "role": "Researcher",
                    "timestamp": datetime.fromtimestamp(os.path.getmtime(rf)).isoformat()
                })
        except Exception:
            pass

    return {
        "status": "online",
        "total_memories": stats.get("total_memories", 0),
        "total_papers": len(glob.glob("swarm_v2_artifacts/research/*.md")),
        "sync_events": stats.get("sync_events", 36),
        "by_type": stats.get("by_type", {}),
        "by_author": stats.get("by_author", {}),
        "recent_intel": recent,
        "latest_research": papers
    }


@router.post("/memory/sync_all")
async def sync_all_memories():
    """Trigger comprehensive synchronization of all agent memory banks and research into collective pool."""
    from swarm_v2.core.turbovec_memory import get_turbovec_memory
    tv = get_turbovec_memory()
    initial_count = len(tv._cache)
    tv._load()
    tv._rebuild_index()
    return {
        "status": "synchronized",
        "initial_count": initial_count,
        "total_memories": len(tv._cache),
        "stats": tv.get_stats()
    }


@router.post("/memory/reload_engine")
async def reload_memory_engine():
    import importlib
    import swarm_v2.core.turbovec_memory as tv_mod
    importlib.reload(tv_mod)
    tv = tv_mod.get_turbovec_memory()
    tv._rebuild_index()
    return {"status": "reloaded", "total_indexed": len(tv._cache), "stats": tv.get_stats()}

"""Changelog and MCP Tool Evolver endpoints — /changelog + /evolver."""
import os
import asyncio
from fastapi import APIRouter

from swarm_v2.core.auto_changelog import get_changelog_engine, start_changelog_monitoring
from swarm_v2.core.mcp_tool_evolver import get_tool_evolver, start_tool_evolution, EvolutionTrigger

router = APIRouter(tags=["changelog"])


# ─── Changelog Endpoints ──────────────────────────────────────────────────


@router.get("/changelog/status")
async def changelog_engine_status():
    """Get auto-changelog engine status."""
    engine = get_changelog_engine()
    return engine.get_status()


@router.post("/changelog/scan")
async def force_changelog_scan():
    """Force an immediate directory scan and changelog update."""
    engine = get_changelog_engine()
    result = engine.force_update()
    return result


@router.get("/changelog/recent")
async def get_recent_changelog_entries(limit: int = 10):
    """Get recent changelog entries."""
    engine = get_changelog_engine()
    return {"entries": engine.get_recent_changes(limit)}


@router.get("/changelog/modules")
async def list_tracked_modules():
    """List all tracked modules in the registry."""
    engine = get_changelog_engine()
    return {
        "modules": [
            {
                "path": info.path,
                "name": info.name,
                "category": info.category.value,
                "status": info.status,
                "description": info.description[:100] if info.description else "",
                "exports": info.exports[:5]
            }
            for info in engine.module_registry.values()
        ],
        "count": len(engine.module_registry)
    }


@router.post("/changelog/start")
async def start_changelog_monitoring_endpoint(interval: int = 60):
    """Start the changelog monitoring loop."""
    asyncio.create_task(start_changelog_monitoring(interval))
    return {"status": "started", "interval": interval}


@router.get("/changelog/content")
async def get_changelog_content():
    """Get the full CHANGELOG.md content."""
    engine = get_changelog_engine()
    if os.path.exists(engine.changelog_path):
        with open(engine.changelog_path, "r", encoding="utf-8") as f:
            content = f.read()
        return {"content": content, "path": engine.changelog_path}
    return {"content": "", "path": engine.changelog_path, "message": "Changelog not yet created"}


# ─── MCP Tool Evolver Endpoints ───────────────────────────────────────────


@router.get("/evolver/status")
async def tool_evolver_status():
    """Get MCP Tool Evolver status."""
    evolver = get_tool_evolver()
    return evolver.get_status()


@router.get("/evolver/tools")
async def list_synthesized_tools_for_evolution():
    """List all synthesized tools available for evolution."""
    evolver = get_tool_evolver()
    tools = evolver._get_current_tools()
    return {
        "tools": [
            {
                "name": name,
                "version": evolver._get_current_version(name),
                "code_length": len(code)
            }
            for name, code in tools.items()
        ],
        "count": len(tools)
    }


@router.get("/evolver/knowledge")
async def get_available_knowledge():
    """Get available knowledge from reconnaissance for tool evolution."""
    evolver = get_tool_evolver()
    knowledge = evolver.get_available_knowledge()
    return {"knowledge": knowledge, "count": len(knowledge)}


@router.post("/evolver/run")
async def trigger_evolution_cycle():
    """Trigger an immediate tool evolution cycle."""
    from swarm_v2.app_v2 import engine_team

    evolver = get_tool_evolver()

    # Get LLM function from Lead Developer
    devo = engine_team.get("Lead Developer")
    llm_fn = devo._llm_generate if devo else None

    if llm_fn:
        evolver.llm_generate = llm_fn

    results = await evolver.evolution_cycle(trigger=EvolutionTrigger.MANUAL)

    return {
        "status": "completed",
        "tools_evolved": len(results),
        "results": [r.to_dict() for r in results]
    }


@router.get("/evolver/history/{tool_name}")
async def get_tool_evolution_history(tool_name: str):
    """Get version history for a specific tool."""
    evolver = get_tool_evolver()
    history = evolver.get_tool_history(tool_name)
    return {"tool": tool_name, "versions": history}


@router.post("/evolver/rollback/{tool_name}")
async def rollback_tool(tool_name: str, target_version: str = None):
    """Rollback a tool to a previous version."""
    evolver = get_tool_evolver()
    result = evolver.rollback_tool(tool_name, target_version)
    if result.get("status") == "error":
        raise HTTPException(status_code=400, detail=result.get("message"))
    return result


@router.post("/evolver/start")
async def start_tool_evolver_endpoint(interval: int = 3600):
    """Start the tool evolution monitor loop."""
    from swarm_v2.app_v2 import engine_team
    devo = engine_team.get("Lead Developer")
    llm_fn = devo._llm_generate if devo else None

    asyncio.create_task(start_tool_evolution(interval, llm_fn))
    return {"status": "started", "interval": interval}


@router.post("/evolver/evolve/{tool_name}")
async def evolve_specific_tool(tool_name: str):
    """Evolve a specific tool with current knowledge."""
    from swarm_v2.app_v2 import engine_team

    evolver = get_tool_evolver()

    # Get tool code
    tools = evolver._get_current_tools()
    if tool_name not in tools:
        raise HTTPException(status_code=404, detail=f"Tool '{tool_name}' not found")

    # Get LLM and knowledge
    devo = engine_team.get("Lead Developer")
    llm_fn = devo._llm_generate if devo else None

    if not llm_fn:
        raise HTTPException(status_code=500, detail="No LLM available for evolution")

    evolver.llm_generate = llm_fn
    knowledge = evolver.get_available_knowledge()

    result = await evolver.evolve_tool(
        tool_name, tools[tool_name], knowledge, EvolutionTrigger.MANUAL
    )

    return result.to_dict()

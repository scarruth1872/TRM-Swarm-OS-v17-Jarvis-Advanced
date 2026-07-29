"""Swarm Orchestration routes — self-healing, lifecycle management, recovery."""
from fastapi import APIRouter
import asyncio
from swarm_v2.core.swarm_orchestrator import get_swarm_orchestrator

router = APIRouter(tags=["orchestrator"])


@router.get("/orchestrator/status")
async def get_orchestrator_status():
    """Full orchestrator status."""
    orchestrator = get_swarm_orchestrator()
    try:
        return orchestrator.get_orchestrator_status()
    except Exception as e:
        return {"status": "initializing", "error": str(e), "auto_remediate": orchestrator._auto_remediate_enabled}


@router.post("/orchestrator/remediate")
async def trigger_remediation():
    """Run a single remediation pass: check all agents, heal degraded ones."""
    orchestrator = get_swarm_orchestrator()
    result = await orchestrator.monitor_and_remediate()
    return result


@router.get("/orchestrator/remediation-history")
async def get_remediation_history():
    """Get the remediation action history."""
    orchestrator = get_swarm_orchestrator()
    return {
        "history": orchestrator.get_remediation_history(limit=50),
        "count": len(getattr(orchestrator, '_remediation_history', [])),
    }


@router.post("/orchestrator/auto-remediate/toggle")
async def toggle_auto_remediate():
    """Toggle the auto-remediation loop on/off."""
    orchestrator = get_swarm_orchestrator()
    orchestrator._auto_remediate_enabled = not orchestrator._auto_remediate_enabled
    return {
        "auto_remediate": orchestrator._auto_remediate_enabled,
    }

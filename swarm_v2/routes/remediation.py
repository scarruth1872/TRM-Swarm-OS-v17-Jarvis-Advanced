"""Remediation endpoint — /remediation."""
from fastapi import APIRouter, HTTPException

router = APIRouter(tags=["remediation"])


@router.post("/remediation/restart/{role}")
async def force_restart_agent(role: str):
    """Manually trigger the remediation engine to restart an agent."""
    from swarm_v2.app_v2 import engine_team, remediation_engine

    if role not in engine_team:
        raise HTTPException(status_code=404, detail="Agent not found")

    success = await remediation_engine.handle_issue("AGENT_TIMEOUT", role)
    if "Successful" in success:
        return {"status": "restarted", "role": role, "new_agent_id": engine_team[role].agent_id}
    else:
        raise HTTPException(status_code=500, detail="Restart failed")

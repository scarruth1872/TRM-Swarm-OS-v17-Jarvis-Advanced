"""Kanban, DDR, Secrets, and Mailbox endpoints — /kanban + /ddr + /secrets + /mailbox."""
from typing import List
import asyncio
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    CreateCardRequest,
    MoveCardRequest,
    SendMailboxMessage,
    DDRScanRequest,
    SecretSetRequest,
    SecretDecryptRequest,
    AntibodyAddRequest,
)
from swarm_v2.core.kanban_board import get_kanban_board
from swarm_v2.core.agent_mailbox import AgentMailbox
from swarm_v2.core.ultrawork_loop import UltraworkLoop

router = APIRouter(tags=["kanban"])


# ─── Kanban Board Endpoints ────────────────────────────────────────────────


@router.get("/kanban/board")
async def get_board():
    """Full Kanban board grouped by columns."""
    kb = get_kanban_board()
    return kb.get_board()


@router.get("/kanban/stats")
async def get_kanban_stats():
    """Kanban board statistics."""
    kb = get_kanban_board()
    return kb.get_stats()


@router.post("/kanban/cards")
async def create_kanban_card(req: CreateCardRequest):
    """Create a new task card."""
    kb = get_kanban_board()
    card_id = kb.create_card(
        title=req.title,
        description=req.description,
        assignee=req.assignee,
        priority=req.priority,
        tags=req.tags,
    )
    return {"card_id": card_id, "status": "created"}


@router.post("/kanban/cards/{card_id}/move")
async def move_kanban_card(card_id: str, req: MoveCardRequest):
    """Move a card to a new status column."""
    kb = get_kanban_board()
    result = kb.move_card(card_id, req.target_status)
    if not result["success"]:
        raise HTTPException(status_code=400, detail=result["message"])
    return result


# ─── DDR Antibody Endpoints ────────────────────────────────────────────────


@router.get("/ddr/antibodies")
async def list_antibodies():
    """List all DDR antibodies."""
    from swarm_v2.core.ddr_antibody import get_ddr
    ddr = get_ddr()
    return {"antibodies": ddr.get_antibodies()}


@router.get("/ddr/stats")
async def get_ddr_stats():
    """DDR prevention statistics."""
    from swarm_v2.core.ddr_antibody import get_ddr
    ddr = get_ddr()
    return ddr.get_prevention_stats()


@router.post("/ddr/scan")
async def scan_code_ddr(req: DDRScanRequest):
    """Scan code against DDR antibodies."""
    from swarm_v2.core.ddr_antibody import get_ddr
    ddr = get_ddr()
    matches = ddr.check_antibodies(req.code, req.filename)
    return {"matches": matches, "scanned_lines": len(req.code.splitlines())}


@router.post("/ddr/antibodies/add")
async def add_antibody(req: AntibodyAddRequest):
    """Add a new custom antibody to the Digital DNA Repository."""
    from swarm_v2.core.ddr_antibody import get_ddr, Antibody
    ddr = get_ddr()
    ab = Antibody(
        error_type=req.error_type,
        file_pattern=req.file_pattern,
        line_pattern=req.line_pattern,
        fix_description=req.fix_description,
        severity=req.severity,
        recorded_by="operator"
    )
    ddr._antibodies.append(ab)
    ddr._save()
    return {"status": "success", "antibody_id": ab.antibody_id}


# ─── Secrets Vault Endpoints ────────────────────────────────────────────────


@router.get("/secrets/keys")
async def list_secret_keys():
    """List secret key names (no values exposed)."""
    from swarm_v2.core.secrets_vault import get_secrets_vault
    vault = get_secrets_vault()
    return {"keys": vault.list_keys(), "stats": vault.get_stats()}


@router.post("/secrets/set")
async def set_secret(req: SecretSetRequest):
    """Encrypt and store a secret."""
    from swarm_v2.core.secrets_vault import get_secrets_vault
    vault = get_secrets_vault()
    vault.set_secret(req.key, req.value)
    return {"status": "success", "key": req.key}


@router.delete("/secrets/delete/{key}")
async def delete_secret(key: str):
    """Delete a secret key."""
    from swarm_v2.core.secrets_vault import get_secrets_vault
    vault = get_secrets_vault()
    success = vault.delete_secret(key)
    if not success:
        raise HTTPException(status_code=404, detail="Secret key not found")
    return {"status": "success", "deleted_key": key}


@router.post("/secrets/rotate")
async def rotate_master_key():
    """Rotate master vault key and re-encrypt all values."""
    from swarm_v2.core.secrets_vault import get_secrets_vault
    vault = get_secrets_vault()
    vault.rotate_key()
    return {"status": "success", "message": "Key rotated successfully"}


@router.post("/secrets/decrypt")
async def decrypt_secret(req: SecretDecryptRequest):
    """Decrypt a secret value (controlled & audited)."""
    from swarm_v2.core.secrets_vault import get_secrets_vault
    vault = get_secrets_vault()
    value = vault.get_secret(req.key)
    if value is None:
        raise HTTPException(status_code=404, detail="Secret key not found")
    return {"key": req.key, "value": value}


# ─── Agent Mailbox Endpoints ────────────────────────────────────────────────


@router.get("/mailbox/agents")
async def list_mailbox_agents():
    """List all agents with mailboxes."""
    return {"agents": AgentMailbox.list_agents()}


@router.get("/mailbox/{agent_id}/inbox")
async def peek_mailbox(agent_id: str):
    """Peek at an agent's inbox without consuming messages."""
    mb = AgentMailbox(agent_id)
    return {"agent": agent_id, "pending": mb.count_pending(), "messages": mb.peek()}


@router.post("/mailbox/send")
async def send_mailbox_message(req: SendMailboxMessage):
    """Send a message between agents."""
    from swarm_v2.app_v2 import engine_team, background_mailbox_tasks

    mb = AgentMailbox(req.from_agent)
    mb.send(req.to_agent, req.body, subject=req.subject)

    # Trigger execution if target is an active engine_team agent
    if req.to_agent in engine_team:
        agent = engine_team[req.to_agent]
        msg_payload = f"Message from {req.from_agent} (Subject: {req.subject}):\n{req.body}"

        async def run_agent_process():
            try:
                response = await agent.process_task(msg_payload, sender=req.from_agent)

                # Extract clean text if response is a dictionary
                if isinstance(response, dict):
                    reply_text = response.get("response", str(response))
                else:
                    reply_text = str(response)

                reply_mb = AgentMailbox(req.to_agent)
                reply_mb.send(
                    req.from_agent,
                    reply_text,
                    subject=f"Re: {req.subject}"
                )
            except Exception as e:
                reply_mb = AgentMailbox(req.to_agent)
                reply_mb.send(
                    req.from_agent,
                    f"Error processing task: {str(e)}",
                    subject=f"Error: {req.subject}"
                )

        # Start as background task and save strong reference
        task = asyncio.create_task(run_agent_process())
        background_mailbox_tasks.add(task)
        task.add_done_callback(background_mailbox_tasks.discard)

        return {"status": "sent", "from": req.from_agent, "to": req.to_agent, "triggered": True}

    return {"status": "sent", "from": req.from_agent, "to": req.to_agent, "triggered": False}


@router.get("/mailbox/{agent_id}/receive")
async def receive_mailbox(agent_id: str):
    """Consume messages from an agent's inbox."""
    mb = AgentMailbox(agent_id)
    return {"agent": agent_id, "messages": mb.receive(limit=50)}


# ─── Ultrawork Missions Endpoints (Agent Communications & Missions panel) ──


@router.get("/ultrawork/missions")
async def list_ultrawork_missions():
    """List active Ultrawork missions for the dashboard panel."""
    from swarm_v2.core.ultrawork_loop import get_ultrawork_loop
    missions = get_ultrawork_loop().list_missions()
    # Map retry_count -> attempt so the dashboard's `e.attempt` field renders.
    return {"missions": [
        {**m, "attempt": m.get("retry_count", 0) + 1} for m in missions
    ]}


@router.get("/ultrawork/stats")
async def get_ultrawork_stats():
    """Ultrawork loop stats (active/completed counts)."""
    from swarm_v2.core.ultrawork_loop import get_ultrawork_loop
    return get_ultrawork_loop().get_stats()


@router.post("/ultrawork/missions")
async def create_ultrawork_mission(req: dict):
    """Create (and optionally kick off) an Ultrawork mission."""
    from swarm_v2.core.ultrawork_loop import get_ultrawork_loop
    objective = (req or {}).get("objective")
    if not objective:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="objective required")
    loop = get_ultrawork_loop()
    mission_id = loop.create_mission(objective)
    return {"mission_id": mission_id, "status": "created"}

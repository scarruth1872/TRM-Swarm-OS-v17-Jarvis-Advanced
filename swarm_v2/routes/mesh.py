"""P2P Agent Mesh and Federation endpoints — /mesh + /federation."""
import json
import os
from typing import Dict, Any, List, Optional
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    MeshAnnounceRequest,
    MeshRouteRequest,
    HandshakeRequest,
    MemorySyncRequest,
    FederationHandshakeRequest,
    FederationMemorySyncRequest,
)
from swarm_v2.core.agent_mesh import MeshNode
from swarm_v2.core.heuristic_consensus import get_heuristic_consensus

router = APIRouter(tags=["mesh"])


# ─── Mesh Endpoints ──────────────────────────────────────────────────────


@router.get("/mesh/topology")
async def mesh_topology():
    """Get the full P2P mesh topology."""
    from swarm_v2.app_v2 import agent_mesh
    return agent_mesh.get_topology_snapshot()


@router.get("/mesh/ipc")
async def get_mesh_ipc():
    """Read the mesh topology directly from memory-mapped shared memory."""
    from swarm_v2.app_v2 import agent_mesh, ipc_bridge
    state = await ipc_bridge.read_state_async()
    if not state:
        return agent_mesh.get_topology_snapshot()
    return state


@router.post("/mesh/ipc")
async def post_mesh_ipc(state: Dict[str, Any]):
    """Write state data directly to memory-mapped shared memory."""
    from swarm_v2.app_v2 import ipc_bridge
    success = await ipc_bridge.write_state_async(state)
    return {"status": "success" if success else "error"}


@router.post("/mesh/announce")
async def mesh_announce(req: MeshAnnounceRequest):
    """Receive heartbeat/load announcement from a discovery daemon and register/update node."""
    from swarm_v2.app_v2 import agent_mesh
    import time

    # Find or register the node in agent mesh
    node = agent_mesh.nodes.get(req.node_id)
    if node:
        # Update node status & heartbeat & load
        node.status = req.status
        node.hardware = req.hardware
        node.last_heartbeat = time.time()
    else:
        # Node not in mesh yet, register it dynamically
        node = MeshNode(
            node_id=req.node_id,
            name=req.name,
            role=req.role,
            host="127.0.0.1",
            port=8021,
            hardware=req.hardware
        )
        agent_mesh.register_node(node)

    # Sync update to memory-mapped shared memory
    from swarm_v2.app_v2 import sync_mesh_to_ipc_async
    await sync_mesh_to_ipc_async()
    return {"status": "announced", "node_id": req.node_id}


@router.get("/mesh/nodes")
async def get_mesh_nodes():
    """Return all active discovery nodes in the mesh."""
    from swarm_v2.app_v2 import agent_mesh
    topology = agent_mesh.get_topology_snapshot()
    return topology.get("nodes", [])


@router.get("/mesh/peers")
async def mesh_peers(role: str = "", specialty: str = ""):
    """Discover peers on the mesh."""
    from swarm_v2.app_v2 import agent_mesh
    return agent_mesh.discover_peers(
        role_filter=role or None,
        specialty_filter=specialty or None,
    )


@router.post("/mesh/route")
async def mesh_route_task(req: MeshRouteRequest):
    """Route a task through the P2P mesh using Heuristic Consensus Scorer."""
    from swarm_v2.app_v2 import engine_team

    result = await get_heuristic_consensus().route(
        task=req.task,
        target_node_id=req.target_node_id or None,
        required_specialty=req.required_specialty or None,
    )

    # If a node was found, actually execute the task through the agent
    if "routed_to" in result:
        target_role = result["routed_to"]["role"]
        agent = engine_team.get(target_role)
        if agent:
            async with self_healing.track_task(target_role):
                response = await agent.process_task(req.task, sender="mesh_router")
            result["response"] = response
            # Capture internal activity logs for visibility
            if hasattr(agent, "nodal_logs"):
                result["nodal_logs"] = agent.nodal_logs

    return result


@router.get("/mesh/stats")
async def mesh_stats():
    from swarm_v2.app_v2 import agent_mesh
    return agent_mesh.get_stats()


@router.get("/mesh/log")
async def mesh_log():
    from swarm_v2.app_v2 import agent_mesh
    return agent_mesh.get_message_log()


# ─── Federation Endpoints ─────────────────────────────────────────────────


@router.post("/federation/handshake")
async def federation_handshake(req: HandshakeRequest):
    """Handle incoming federation handshake."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    return f.handle_handshake(req.dict())


@router.get("/federation/peers")
async def list_federation_peers(node_id: str = ""):
    """Discover peers known by this node."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    return f.handle_peer_request(node_id)


@router.post("/federation/memory/sync")
async def sync_federation_memory(req: MemorySyncRequest):
    """Exchange knowledge with a federated peer."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    return f.handle_memory_sync(req.dict())


@router.get("/federation/stats")
async def get_federation_status():
    """Get federation health and connectivity stats."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    return f.get_federation_stats()


@router.get("/federation/collective")
async def get_collective_status():
    """Get high-level status of the entire multi-swarm mesh (Phase 15)."""
    from swarm_v2.core.federation_manager import get_federation_manager
    manager = get_federation_manager()
    return manager.get_collective_status()


@router.post("/federation/sync/receive")
async def receive_federated_memory(req: Dict[str, Any]):
    """Receive a pushed memory from a federated peer (Phase 15)."""
    from swarm_v2.core.global_memory import get_global_memory
    mem = get_global_memory()
    if not mem:
        raise HTTPException(status_code=503, detail="Memory layer not ready")

    entry = mem.contribute(
        content=req.get("content", ""),
        author=req.get("author", "federation"),
        author_role=req.get("author_role", "federated"),
        memory_type="federated_knowledge",
        tags=["federated", "pushed"]
    )
    return {"status": "success", "id": entry.memory_id}


@router.post("/federation/task/receive")
async def receive_federated_task(req: Dict[str, Any]):
    """Accept a delegated task from a federated peer (Phase 15)."""
    from swarm_v2.core.task_arbiter import get_task_arbiter, Task, TaskComplexity
    arbiter = get_task_arbiter()

    # Reconstruct task object
    task = Task(
        task_id=f"federated_{req.get('task_id')}",
        task_type=req.get("task_type", "llm"),
        complexity=TaskComplexity(req.get("complexity", "medium")),
        payload=req.get("payload", {}),
        priority=10  # Federated tasks get high local priority
    )

    # Submit locally
    await arbiter.submit_task(task)

    return {
        "status": "accepted",
        "local_task_id": task.task_id,
        "message": "Task queued for collective execution"
    }


@router.post("/federation/connect")
async def connect_to_peer(host: str, port: int):
    """Manually initiate a handshake with a new peer."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    node = await f.handshake(host, port)
    if node:
        return {"status": "connected", "node": node.to_dict()}
    else:
        raise HTTPException(status_code=400, detail="Handshake failed")


@router.post("/federation/sync")
async def trigger_federation_sync(target_node_id: str):
    """Manually trigger a memory sync with a peer."""
    from swarm_v2.core.federation import get_federation
    f = get_federation()
    if not f:
        raise HTTPException(status_code=503, detail="Federation protocol not initialized")
    result = await f.sync_memories(target_node_id)
    return result

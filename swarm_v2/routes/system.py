"""System, health, infrastructure endpoints — /system + /health + /infrastructure + /infra."""
import asyncio
from fastapi import APIRouter, HTTPException

from swarm_v2.core.resource_arbiter import get_resource_arbiter
from swarm_v2.core.task_arbiter import get_task_arbiter
from swarm_v2.core.self_healing_infra import get_self_healing_infra
from swarm_v2.core.self_healing import health_monitor, telemetry

router = APIRouter(tags=["system"])


# ─── System Endpoints ─────────────────────────────────────────────────────


@router.get("/system/resources")
async def system_resources():
    return get_resource_arbiter().get_status()


@router.get("/system/task-arbiter")
async def task_arbiter_status():
    """Get Task Arbiter status including CPU/GPU workload distribution."""
    arbiter = get_task_arbiter()
    return arbiter.get_system_status()


@router.get("/system/task-arbiter/agent/{agent_id}")
async def task_arbiter_agent(agent_id: str):
    """Get status of a specific agent's workload."""
    arbiter = get_task_arbiter()
    return arbiter.get_agent_status(agent_id)


@router.get("/system/maintenance/status")
async def maintenance_scheduling_status():
    """Get dynamic maintenance scheduling status."""
    arbiter = get_task_arbiter()
    return arbiter.dynamic_arbiter.get_maintenance_status()


@router.get("/system/maintenance/window")
async def current_usage_window():
    """Get current system usage window for maintenance scheduling."""
    arbiter = get_task_arbiter()
    window = arbiter.dynamic_arbiter.detect_usage_window()
    return {
        "window": window.value,
        "low_usage_duration_sec": arbiter.dynamic_arbiter.usage_metrics.low_usage_duration_sec,
        "priority_adjustment": arbiter.dynamic_arbiter.PRIORITY_ADJUSTMENTS.get(window, 0)
    }


@router.post("/system/maintenance/run")
async def trigger_maintenance_cycle():
    """Trigger an immediate maintenance cycle."""
    arbiter = get_task_arbiter()
    await arbiter.dynamic_arbiter.run_maintenance_cycle()
    return {"status": "completed", "stats": arbiter.dynamic_arbiter._stats}


@router.post("/system/maintenance/task/{task_id}/toggle")
async def toggle_maintenance_task(task_id: str, enabled: bool = True):
    """Enable or disable a specific maintenance task."""
    arbiter = get_task_arbiter()
    if task_id in arbiter.dynamic_arbiter.maintenance_tasks:
        arbiter.dynamic_arbiter.maintenance_tasks[task_id].enabled = enabled
        return {"status": "updated", "task_id": task_id, "enabled": enabled}
    raise HTTPException(status_code=404, detail=f"Maintenance task '{task_id}' not found")


@router.get("/system/maintenance/tasks")
async def list_maintenance_tasks():
    """List all maintenance tasks and their scheduling status."""
    arbiter = get_task_arbiter()
    return {
        "tasks": [
            {
                "task_id": task_id,
                "type": task.task_type.value,
                "enabled": task.enabled,
                "interval_sec": task.interval_sec,
                "last_run": task.last_run.isoformat() if task.last_run else None,
                "is_due": task.is_due(),
                "dynamic_priority": arbiter.dynamic_arbiter.get_dynamic_priority(task)
            }
            for task_id, task in arbiter.dynamic_arbiter.maintenance_tasks.items()
        ],
        "current_window": arbiter.dynamic_arbiter._last_window.value
    }


# ─── Health Endpoints ─────────────────────────────────────────────────────


@router.get("/health")
async def health():
    from swarm_v2.app_v2 import (
        engine_team, spawned_subagents, pipeline,
        learning_engine, agent_mesh, global_memory, synthesizer, federation
    )

    fed_stats = federation.get_federation_stats() if federation else {}
    return {
        "status": "v5_online",
        "phase": 5,
        "agents": len(engine_team),
        "subagents": len(spawned_subagents),
        "artifacts": pipeline.get_stats(),
        "learning_engine": learning_engine.get_stats(),
        "mesh": agent_mesh.get_stats(),
        "global_memory": global_memory.get_stats(),
        "synthesizer": synthesizer.get_stats(),
        "federation": fed_stats,
    }


@router.get("/health/dashboard")
async def get_health_dashboard():
    """Retrieve Health Dashboard Monitor (HDM) metrics from Redis."""
    import json
    data = health_monitor.redis.get("hdm:health_metrics")
    if data:
        return json.loads(data)
    return {"status": "Metrics not yet available"}


@router.get("/telemetry/self-healing")
async def get_self_healing_telemetry():
    """Retrieve Self-Healing Telemetry metrics from Redis."""
    return telemetry.get_metrics()


# ─── Infrastructure Endpoints ──────────────────────────────────────────────


@router.get("/infrastructure/status")
async def get_infra_status():
    arbiter = get_resource_arbiter()
    return arbiter.get_status() if arbiter else {"status": "inactive"}


@router.get("/infrastructure/nodes")
async def get_infra_nodes():
    from swarm_v2.core.agent_mesh import get_agent_mesh
    mesh = get_agent_mesh()
    if not mesh:
        return {"nodes": []}
    nodes = mesh.get_topology_snapshot().get("nodes", [])
    return {"nodes": nodes}


# ─── Self-Healing Infra Endpoints ─────────────────────────────────────────


@router.get("/infra/health")
async def infra_health():
    """Get infrastructure health status for all monitored services."""
    infra = get_self_healing_infra()
    return infra.get_status()


@router.post("/infra/start")
async def start_infra_monitoring():
    """Start the self-healing infrastructure monitoring."""
    infra = get_self_healing_infra()
    if not infra.running:
        asyncio.create_task(infra.start())
        return {"status": "started", "message": "Self-healing monitoring started"}
    return {"status": "already_running", "message": "Monitoring already active"}


@router.post("/infra/stop")
async def stop_infra_monitoring():
    """Stop the self-healing infrastructure monitoring."""
    infra = get_self_healing_infra()
    await infra.stop()
    return {"status": "stopped", "message": "Self-healing monitoring stopped"}


@router.post("/infra/restart/{service_name}")
async def force_restart_service(service_name: str):
    """Force restart a specific monitored service."""
    infra = get_self_healing_infra()
    result = infra.force_restart(service_name)
    if "error" in result:
        raise HTTPException(status_code=404, detail=result["error"])
    return result


@router.post("/infra/reset/{service_name}")
async def reset_service_restart_count(service_name: str):
    """Reset the restart counter for a service."""
    infra = get_self_healing_infra()
    result = infra.reset_restart_count(service_name)
    if "error" in result:
        raise HTTPException(status_code=404, detail=result["error"])
    return result


@router.get("/infra/history")
async def get_restart_history(limit: int = 20):
    """Get the restart history for all services."""
    infra = get_self_healing_infra()
    status = infra.get_status()
    return {
        "restart_history": status["restart_history"][-limit:],
        "total_restarts": status["total_restarts"]
    }


@router.get("/model-training/status")
async def get_model_training_status():
    """Return Jarvis model training status from the registry."""
    import os, json
    reg_path = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_model_registry.json"
    if os.path.exists(reg_path):
        with open(reg_path, "r") as f:
            return json.load(f)
    return {"status": "no_registry_found", "path": reg_path}

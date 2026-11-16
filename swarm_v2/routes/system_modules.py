"""Live module-load view — GET /system/modules.

Backed by swarm_v2.core.module_tracker, which stamps every module imported
from this repository with an import index, a wall-clock offset from process
start, and a startup/lazy phase. This is the live counterpart to the static
reachability trace in tools/trace_codebase.py.
"""
from fastapi import APIRouter

from swarm_v2.core import module_tracker

router = APIRouter(tags=["system"])


@router.get("/system/modules")
async def system_modules():
    """Every sys.modules entry whose __file__ lives inside this repository."""
    mods = module_tracker.snapshot()
    return {
        "modules": mods,
        "count": len(mods),
        # pass the snapshot in: summary() used to walk sys.modules a second time,
        # which doubled the route's cost for no extra information (G13)
        **{k: v for k, v in module_tracker.summary(mods).items() if k != "count"},
    }

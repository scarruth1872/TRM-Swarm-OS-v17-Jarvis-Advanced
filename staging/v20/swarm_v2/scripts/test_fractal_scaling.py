
import asyncio
import os
from swarm_v2.experts.registry import get_expert_team

def test_fractal_scaling():
    """Architect spawns an ephemeral subordinate through the microkernel spawner.

    The old body called `archi.spawn_subordinate(task, isolated=True)` — a method BaseAgent has
    never had — and awaited it. The real API is the synchronous
    `spawn_subagent(role, task, continuum_function=...)`, which returns a BaseAgent registered on
    the parent's roster. Verified offline: it returns `Archi_Sub1` with no service dependency.
    """
    team = get_expert_team()
    archi = team["Architect"]
    before = len(archi.subagents)
    sub = archi.spawn_subagent(
        "QA Engineer",
        "Create a simple 'hello_world.py' in the current workspace and verify it.",
    )
    assert sub is not None, "spawn_subagent returned nothing"
    assert sub.persona is not None and sub.persona.name, "subordinate has no persona"
    assert len(archi.subagents) == before + 1, "subordinate was not registered on the parent"
    assert sub.persona.name.startswith(archi.persona.name), (
        f"subordinate {sub.persona.name!r} is not parented to {archi.persona.name!r}"
    )

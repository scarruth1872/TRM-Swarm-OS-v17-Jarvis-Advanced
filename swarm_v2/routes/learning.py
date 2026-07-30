"""Learning and skills endpoints — /learning + /skills."""
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    LearnTextRequest,
    LearnFileRequest,
    UseSkillRequest,
    GitHubAcquisitionRequest,
)
from swarm_v2.skills.learning_engine import get_learning_engine
from swarm_v2.skills.github_acquisition import get_github_acquisition
from swarm_v2.core.skill_loader import SkillLoader
from swarm_v2.core.skill_matrix_sync import get_skill_matrix_sync
from swarm_v2.core.tool_forge import ToolForge

router = APIRouter(tags=["learning"])


@router.post("/api/learn-resource")
@router.post("/learning/ingest")
async def learn_from_text(req: LearnTextRequest):
    """Ingest documentation text and create a new learned skill."""
    from swarm_v2.app_v2 import engine_team
    learning_engine = get_learning_engine()

    # Use the Architect's LLM to synthesize knowledge
    architect = engine_team.get("Architect")
    llm_fn = architect._llm_generate if architect else None
    content_text = getattr(req, "content", None) or getattr(req, "resourceName", "Resource Ingestion")
    name_text = getattr(req, "name", None) or getattr(req, "resourceName", "LearnedResource")
    skill = await learning_engine.learn_from_text(
        name=name_text, content=content_text, source=getattr(req, "source", "Resource Ingestion"),
        llm_generate=llm_fn
    )
    return {
        "status": "learned",
        "success": True,
        "skill": skill.to_dict(),
        "learnedStructure": {
            "summary": f"Learned resource {name_text}",
            "nodes": [{"id": name_text.lower(), "label": name_text, "type": "system"}],
            "relations": []
        },
        "message": f"New skill '{name_text}' acquired."
    }


@router.post("/learning/ingest-file")
async def learn_from_file(req: LearnFileRequest):
    """Learn a new skill from a local documentation file."""
    from swarm_v2.app_v2 import engine_team
    learning_engine = get_learning_engine()

    architect = engine_team.get("Architect")
    llm_fn = architect._llm_generate if architect else None
    try:
        skill = await learning_engine.learn_from_file(
            filepath=req.filepath, llm_generate=llm_fn
        )
        return {
            "status": "learned",
            "skill": skill.to_dict(),
            "message": f"New skill '{skill.skill_name}' acquired from {req.filepath}"
        }
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.get("/learning/skills")
async def list_learned_skills():
    """List all dynamically learned skills."""
    learning_engine = get_learning_engine()
    return {
        "skills": learning_engine.list_skills(),
        "stats": learning_engine.get_stats(),
        "log": learning_engine.get_learning_log()[-10:],
    }


@router.post("/learning/use")
async def use_learned_skill(req: UseSkillRequest):
    """Use a learned skill to complete a task, optionally through a specific agent."""
    from swarm_v2.app_v2 import engine_team
    learning_engine = get_learning_engine()

    skill = learning_engine.get_skill(req.skill_name)
    if not skill:
        raise HTTPException(status_code=404, detail=f"Skill '{req.skill_name}' not found")

    # Get skill knowledge
    skill_knowledge = skill.execute(req.task)

    # Route through an agent if specified
    if req.target_role and req.target_role in engine_team:
        agent = engine_team[req.target_role]
        enhanced_task = (
            f"You have access to a LEARNED SKILL called '{req.skill_name}'.\n"
            f"Skill knowledge:\n{skill_knowledge}\n\n"
            f"Use this knowledge to complete the following task:\n{req.task}"
        )
        response = await agent.process_task(enhanced_task, sender="learning_engine")
        return {
            "skill_used": req.skill_name,
            "agent": req.target_role,
            "response": response,
        }

    return {"skill_used": req.skill_name, "output": skill_knowledge}


@router.delete("/learning/skills/{name}")
async def forget_skill(name: str):
    """Remove a learned skill."""
    learning_engine = get_learning_engine()
    if learning_engine.forget_skill(name):
        return {"status": "forgotten", "skill": name}
    raise HTTPException(status_code=404, detail=f"Skill '{name}' not found")


@router.get("/skills/matrix")
async def get_skill_matrix_data():
    """Get the full agent-skill proficiency matrix."""
    return get_skill_matrix_sync().get_matrix()


@router.get("/skills/matrix/topology")
async def get_skill_matrix_topology():
    """Get D3-ready topology overlay with skill affinity links and health scores."""
    from swarm_v2.core.self_healing_orchestrator import get_self_healing_orchestrator
    health_scores = get_self_healing_orchestrator().get_status().get("agent_health_scores", {})
    return get_skill_matrix_sync().get_topology_overlay(health_scores)


@router.get("/skills/portable")
async def list_portable_skills():
    """List all SKILL.md and Python skills."""
    loader = SkillLoader()
    loader.discover_skills()
    return {"skills": loader.list_skills(), "stats": loader.get_stats()}


@router.get("/learning/recommendations")
async def get_skill_recommendations():
    """Recommend skills based on usage data and gaps."""
    learning_engine = get_learning_engine()
    skills_data = learning_engine.list_skills()
    # Find least-used skills and suggest re-learning
    rarely_used = [s for s in skills_data if s.get("usage_count", 0) < 3]
    return {
        "recommendations": rarely_used[:5],
        "total_rarely_used": len(rarely_used),
        "total_skills": len(skills_data),
    }


@router.get("/memory/semantic-graph")
async def get_semantic_graph():
    """Return semantic graph as nodes + edges for dashboard visualization."""
    import json, os
    graph_path = "F:/Development sites/Jarvis-Advanced-main/Jarvis-Advanced-main/jarvis_semantic_graph.json"
    if not os.path.exists(graph_path):
        return {"nodes": [], "edges": []}
    with open(graph_path) as f:
        graph = json.load(f)
    nodes = graph.get("nodes", [])
    edges = graph.get("edges", [])
    return {
        "nodes": [{"id": n.get("id", ""), "label": n.get("label", ""), "type": n.get("type", "concept")} for n in nodes],
        "edges": [{"source": e.get("source", ""), "target": e.get("target", ""), "label": e.get("label", "related")} for e in edges],
        "node_count": len(nodes),
        "edge_count": len(edges),
    }


@router.get("/api/robotics/status")
async def get_robotics_status():
    """Return simulated robotics system status for RoboticsLab dashboard."""
    return {
        "connected_devices": [],
        "actuators": [],
        "sensors": [],
        "status": "simulation_mode",
        "message": "No physical robotics hardware connected. Running in simulation mode.",
    }


@router.post("/skills/github-acquire")
async def github_skill_acquisition(req: GitHubAcquisitionRequest):
    """Auto-discover and ingest skills from GitHub repos by topic."""
    import asyncio
    from swarm_v2.app_v2 import engine_team
    architect = engine_team.get("Architect")
    llm_fn = architect._llm_generate if architect else None

    # Fire in background to avoid timeout
    pipeline = get_github_acquisition()

    async def run():
        return await pipeline.acquire_topics(
            topics=req.topics,
            max_per_topic=req.max_per_topic or 3,
            llm_generate_fn=llm_fn,
        )

    # Start the task in background
    task = asyncio.create_task(run())
    return {
        "status": "started",
        "message": f"Acquiring {len(req.topics)} topic(s). Results will be logged.",
        "topics": req.topics,
    }

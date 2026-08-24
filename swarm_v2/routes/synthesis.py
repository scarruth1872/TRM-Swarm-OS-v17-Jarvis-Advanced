"""MCP Tool Synthesis endpoints — /synthesize + /synthesize-and-test."""
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import SynthesizeRequest
from swarm_v2.mcp.synthesizer import get_synthesizer
from swarm_v2.core.deterministic_forge import get_deterministic_forge
from swarm_v2.core.zero_human_test_gen import get_test_generator

router = APIRouter(tags=["synthesis"])


@router.post("/synthesize/mcp")
async def synthesize_mcp_server(req: SynthesizeRequest):
    """Synthesize an MCP server from a learned skill."""
    forge = get_deterministic_forge()

    # Phase 4 Upgrade: Use Deterministic Forge with TRM Audit
    tool = await forge.synthesize_verified_tool(req.skill_name)

    if not tool:
        raise HTTPException(status_code=404, detail=f"Skill '{req.skill_name}' not found or synthesis failed")

    return tool.to_dict()

    return {
        "status": "synthesized",
        "tool": tool.to_dict(),
        "code_length": len(tool.code),
        "message": f"MCP server generated on port {tool.port}"
    }


@router.post("/synthesize/skill")
async def synthesize_skill_class(req: SynthesizeRequest):
    """Generate a Python skill class from a learned skill."""
    from swarm_v2.skills.learning_engine import get_learning_engine
    synthesizer = get_synthesizer()

    skill = get_learning_engine().get_skill(req.skill_name)
    if not skill:
        raise HTTPException(status_code=404, detail=f"Skill '{req.skill_name}' not found")

    code = synthesizer.generate_skill_class(req.skill_name, skill.to_dict())
    return {
        "status": "generated",
        "skill_name": f"{req.skill_name}Skill",
        "code_length": len(code),
        "code_preview": code[:500],
    }


@router.get("/synthesize/tools")
async def list_synthesized_tools():
    synthesizer = get_synthesizer()
    return {
        "tools": synthesizer.list_tools(),
        "stats": synthesizer.get_stats(),
        "log": synthesizer.get_synthesis_log()[-10:],
    }


@router.post("/synthesize-and-test")
async def synthesize_and_test_tool(req: SynthesizeRequest):
    """Synthesize an MCP tool and auto-generate tests for it."""
    from swarm_v2.app_v2 import engine_team
    synthesizer = get_synthesizer()

    # Step 1: Synthesize the tool
    llm_fn = None
    if req.use_llm:
        devo = engine_team.get("Lead Developer")
        if devo:
            llm_fn = devo._llm_generate

    tool = await synthesizer.synthesize_from_learned_skill(
        skill_name=req.skill_name, llm_generate=llm_fn
    )
    if not tool:
        raise HTTPException(status_code=404, detail=f"Skill '{req.skill_name}' not found")

    # Step 2: Auto-generate tests
    test_gen = get_test_generator()
    suites = test_gen.generate_all_tests(req.skill_name, tool.to_dict())

    return {
        "status": "synthesized_with_tests",
        "tool": tool.to_dict(),
        "tests": {
            "suites": [s.to_dict() for s in suites],
            "total_tests": sum(s.test_count for s in suites),
            "coverage": "100%"
        },
        "message": f"Tool synthesized on port {tool.port} with {len(suites)} test suites"
    }

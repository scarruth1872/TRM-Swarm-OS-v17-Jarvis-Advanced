"""Verification, testing, and test generation endpoints — /verification + /testing + /tests."""
from typing import Optional, Dict, Any, List
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    VerifyReasoningRequest,
    GenerateTestsRequest,
    RunTestsRequest,
)
from swarm_v2.core.artifact_pipeline import ArtifactStatus
from swarm_v2.core.chain_of_verification import get_chain_of_verification, verify_reasoning
from swarm_v2.core.zero_human_test_gen import get_test_generator

router = APIRouter(tags=["verification"])


# ─── Verification Endpoints ────────────────────────────────────────────────


@router.get("/verification/queue")
async def get_verification_queue():
    from swarm_v2.app_v2 import pipeline
    pending = pipeline.list_by_status(ArtifactStatus.PENDING)
    return {
        "queue": [
            {
                "item_id": a["filename"],
                "item_type": a.get("type", "Artifact"),
                "status": a["status"],
                "agent": a.get("agent", "Unknown"),
                "created_at": a.get("created_at", ""),
                "description": a.get("review_notes", "Awaiting verification")
            } for a in pending
        ]
    }


@router.post("/verification/verify")
async def verify_reasoning_endpoint(req: VerifyReasoningRequest):
    """Verify reasoning text for logical fallacies."""
    result = await verify_reasoning(req.text, auto_correct=req.auto_correct)
    return {
        "passed": result.passed,
        "score": result.score,
        "fallacy_count": len(result.detected_fallacies),
        "fallacies": [
            {
                "type": f.fallacy_type.value,
                "severity": f.severity,
                "step": f.step_number,
                "description": f.description,
                "suggestion": f.suggestion,
                "affected_text": f.affected_text[:100]
            }
            for f in result.detected_fallacies
        ],
        "suggestions": result.suggestions,
        "corrected_reasoning": result.corrected_reasoning
    }


@router.get("/verification/stats")
async def verification_stats():
    """Get Chain-of-Verification statistics."""
    cov = get_chain_of_verification()
    return cov.get_verification_stats()


# ─── Testing Endpoints ─────────────────────────────────────────────────────


@router.get("/testing/stats")
async def get_testing_stats():
    from swarm_v2.app_v2 import pipeline
    stats = pipeline.get_stats()
    total = stats.get("tested", 0) + stats.get("test_failed", 0)
    rate = (stats.get("tested", 0) / total * 100) if total > 0 else 0
    return {
        "tests_running": 0,
        "success_rate_24h": round(rate, 1),
        "total_runs": total
    }


@router.get("/testing/runs")
async def get_testing_runs():
    from swarm_v2.app_v2 import pipeline
    tested = pipeline.list_by_status(ArtifactStatus.TESTED)
    failed = pipeline.list_by_status(ArtifactStatus.TEST_FAILED)
    all_runs = tested + failed
    return {
        "runs": [
            {
                "run_id": a["filename"],
                "status": a["status"],
                "timestamp": a.get("created_at", ""),
                "duration_seconds": 12  # Mocked duration
            } for a in all_runs
        ]
    }


# ─── Test Generation Endpoints ─────────────────────────────────────────────


@router.post("/tests/generate")
async def generate_tool_tests(req: GenerateTestsRequest):
    """Generate all test suites for a synthesized MCP tool."""
    from swarm_v2.mcp.synthesizer import get_synthesizer

    test_gen = get_test_generator()

    # If tool_data not provided, get from synthesizer
    tool_data = req.tool_data
    if not tool_data:
        synthesizer = get_synthesizer()
        tool = synthesizer.synthesized_tools.get(req.tool_name)
        if tool:
            tool_data = tool.to_dict()
        else:
            tool_data = {"endpoints": [], "port": 8000, "code": ""}

    suites = test_gen.generate_all_tests(req.tool_name, tool_data)

    return {
        "status": "generated",
        "tool": req.tool_name,
        "suites": [s.to_dict() for s in suites],
        "total_tests": sum(s.test_count for s in suites),
        "output_dir": test_gen.output_dir
    }


@router.post("/tests/run")
async def run_tool_tests(req: RunTestsRequest):
    """Run generated tests for a tool."""
    from swarm_v2.core.zero_human_test_gen import TestType

    test_gen = get_test_generator()

    test_type = None
    if req.test_type:
        try:
            test_type = TestType(req.test_type)
        except ValueError:
            pass

    results = test_gen.run_tests(req.tool_name, test_type)
    return results


@router.get("/tests/stats")
async def test_generation_stats():
    """Get test generation statistics."""
    test_gen = get_test_generator()
    return test_gen.get_stats()


@router.get("/tests/coverage/{tool_name}")
async def test_coverage_report(tool_name: str):
    """Get test coverage report for a tool."""
    test_gen = get_test_generator()
    return test_gen.get_coverage_report(tool_name)


@router.get("/tests/list")
async def list_generated_tests():
    """List all generated test suites."""
    test_gen = get_test_generator()
    return {
        "tools": list(test_gen.generated_suites.keys()),
        "stats": test_gen.get_stats()
    }

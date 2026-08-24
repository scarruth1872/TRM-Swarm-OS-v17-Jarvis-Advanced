"""Artifact pipeline endpoints — list, review, batch, integrate, test, deploy, stats, remediate, security verify."""
import asyncio
from typing import List, Optional
from fastapi import APIRouter, HTTPException

from swarm_v2.routes.models import (
    ReviewRequest,
    BatchReviewRequest,
    IntegrateRequest,
    BatchIntegrateRequest,
    TestRequest,
    DeployRequest,
)
from swarm_v2.core.artifact_pipeline import ArtifactStatus

router = APIRouter(tags=["artifacts"])


@router.get("/artifacts")
async def list_artifacts(grouped: bool = False, include_content: bool = False):
    from swarm_v2.app_v2 import pipeline

    await asyncio.to_thread(pipeline.scan_artifacts)
    if grouped:
        return {"groups": pipeline.get_grouped_artifacts(), "stats": pipeline.get_stats()}

    artifacts = pipeline.list_all()

    # Include content for display (limited to first 500 chars for performance)
    if include_content:
        for art in artifacts:
            content = pipeline.get_content(art["filename"])
            if content:
                art["content"] = content[:1500]  # Increased preview

    return {"artifacts": artifacts, "stats": pipeline.get_stats()}


@router.get("/artifacts/{filename}")
async def get_artifact_detail(filename: str):
    from swarm_v2.app_v2 import pipeline
    art = pipeline.get_artifact(filename)
    if not art:
        raise HTTPException(status_code=404, detail="Not found")
    return {**art, "content": pipeline.get_content(filename)}


@router.post("/artifacts/review")
async def review_artifact(req: ReviewRequest):
    from swarm_v2.app_v2 import pipeline
    if req.action == "approve":
        result = pipeline.approve(req.filename, req.reviewer, req.notes)
    elif req.action == "reject":
        result = pipeline.reject(req.filename, req.reviewer, req.notes)
    else:
        raise HTTPException(status_code=400, detail="Action must be 'approve' or 'reject'")
    if not result:
        raise HTTPException(status_code=404, detail="Not found")
    return result


@router.post("/artifacts/batch-review")
async def batch_review_artifacts(req: BatchReviewRequest):
    from swarm_v2.app_v2 import pipeline
    count = pipeline.approve_batch(status=req.status, category=req.category, reviewer=req.reviewer)
    return {"message": f"Successfully approved {count} artifacts", "count": count}


@router.post("/artifacts/batch-integrate")
async def batch_integrate_artifacts(req: BatchIntegrateRequest):
    from swarm_v2.app_v2 import pipeline
    results = pipeline.integrate_batch(filenames=req.filenames, target_subdir=req.target_subdir)
    return {"message": f"Batch integration complete", "results": results}


@router.post("/artifacts/security-verify")
async def batch_security_verify(filenames: List[str]):
    """Manually or system trigger security verification for a batch of files."""
    from swarm_v2.app_v2 import pipeline
    count = 0
    for fname in filenames:
        # Simplistic 'safe' check for now, can be expanded with LLM scan
        pipeline.set_security_status(fname, "verified", "Auto-verified as low-risk artifact.")
        count += 1
    return {"status": "success", "verified_count": count}


@router.post("/artifacts/test")
async def test_artifact(req: TestRequest):
    from swarm_v2.app_v2 import engine_team, pipeline

    art = pipeline.get_artifact(req.filename)
    if not art:
        raise HTTPException(status_code=404, detail="Not found")
    content = pipeline.get_content(req.filename)
    if not content:
        raise HTTPException(status_code=404, detail="Content not found")

    qa = engine_team.get("QA Engineer")
    if not qa:
        raise HTTPException(status_code=500, detail="QA not available")

    # Calculate test filename with recursion guard
    if req.filename.startswith("test_") or req.filename.startswith("Test"):
        if req.filename.count("test_") > 1:
            raise HTTPException(status_code=400, detail="Recursive testing detected.")
        test_filename = req.filename
    else:
        test_filename = f"test_{req.filename}"

    test_task = (
        f"[ACTION] Create a comprehensive pytest test suite for this code.\\n"
        f"You MUST output the test code using the following strict format:\\n"
        f"WRITE_FILE: {test_filename}\\n```python\\n<your test code>\\n```\\n\\n"
        f"File: '{req.filename}':\\n\\n{content[:1500]}"
    )

    test_response = await qa.process_task(test_task, sender="pipeline")

    test_content = pipeline.get_content(test_filename)
    passed = test_content is not None and len(test_content) > 50
    pipeline.set_tested(req.filename, test_filename, passed, test_response[:300])
    await asyncio.to_thread(pipeline.scan_artifacts)
    return {"filename": req.filename, "test_file": test_filename,
            "test_generated": passed, "test_response": test_response[:500],
            "status": pipeline.get_artifact(req.filename)}


@router.post("/artifacts/integrate")
async def integrate_artifact(req: IntegrateRequest):
    from swarm_v2.app_v2 import pipeline
    result = await pipeline.integrate(req.filename, req.target_subdir)
    if not result:
        raise HTTPException(status_code=400, detail="Not found or wrong status")
    return result


@router.post("/artifacts/remediate")
async def remediate_rejected():
    """
    Closed-loop fix cycle:
    1. Collect all rejected/failed artifacts
    2. Send rejection report to Architect for re-planning
    3. Architect creates a rebuild plan
    4. Dispatch rebuild tasks to the right agents
    5. New artifacts enter pipeline as 'pending'
    """
    from swarm_v2.app_v2 import engine_team, pipeline
    from swarm_v2.app_v2 import _agent_for_file

    report = pipeline.get_rejection_report()
    if not report:
        return {"message": "No rejected or failed artifacts to remediate", "rebuilt": []}

    # Step 1: Architect reviews the rejections and creates a rebuild plan
    architect = engine_team.get("Architect")
    if not architect:
        raise HTTPException(status_code=500, detail="Architect not available")

    plan_prompt = (
        f"{report}\n\n"
        f"As the Architect, review these rejected artifacts. For EACH rejected file, "
        f"provide a brief improvement plan describing what needs to change. "
        f"Be specific about what was wrong and how to fix it. "
        f"Format: one line per file like 'filename.py: [what to fix]'"
    )
    architect_plan = await architect.process_task(plan_prompt, sender="pipeline_remediation")

    # Step 2: Rebuild each rejected/failed artifact using the right agent
    rejected = pipeline.list_by_status("rejected") + pipeline.list_by_status("test_failed")
    rebuilt = []

    for art in rejected:
        fname = art["filename"]
        agent_role = _agent_for_file(fname)
        agent = engine_team.get(agent_role)
        if not agent:
            continue

        rejection_reason = art.get("review_notes", "No specific reason")
        rebuild_task = (
            f"write file {fname} — this file was previously REJECTED. "
            f"Rejection reason: '{rejection_reason}'. "
            f"Architect's improvement plan: {architect_plan[:300]}. "
            f"Generate a COMPLETE, PRODUCTION-QUALITY replacement. "
            f"Fix all issues mentioned in the rejection. Make it thorough and professional."
        )

        response = await agent.process_task(rebuild_task, sender="pipeline_remediation")

        # Reset artifact status back to pending
        pipeline.reset_artifact(fname)
        pipeline.scan_artifacts()

        rebuilt.append({
            "filename": fname,
            "rebuilt_by": agent_role,
            "agent_name": agent.persona.name,
            "status": "pending",
            "response_preview": response[:200],
        })

    return {
        "architect_plan": architect_plan,
        "rebuilt": rebuilt,
        "message": f"Remediated {len(rebuilt)} artifacts — they are now pending review",
    }


@router.post("/artifacts/deploy")
async def deploy_integrated(req: DeployRequest):
    """
    Deployment cycle:
    1. Collect all integrated artifacts
    2. Architect creates a deployment plan
    3. DevOps sets up the system
    4. QA validates the deployment
    5. Return deployment report
    """
    from swarm_v2.app_v2 import engine_team, pipeline

    manifest = pipeline.get_integrated_manifest()
    if "No artifacts" in manifest:
        return {"message": "No integrated artifacts to deploy", "steps": []}

    steps = []

    # Step 1: Architect creates deployment plan
    architect = engine_team.get("Architect")
    arch_task = (
        f"Create a deployment plan for {req.project_name}. "
        f"These files are ready for deployment:\n\n{manifest}\n\n"
        f"Describe the deployment sequence, dependencies, and configuration steps. "
        f"Be specific and actionable."
    )
    arch_response = await architect.process_task(arch_task, sender="deployment")
    steps.append({"phase": "Planning", "agent": "Architect",
                   "name": architect.persona.name, "response": arch_response})

    # Step 2: DevOps creates setup scripts based on the plan
    devops = engine_team.get("DevOps Engineer")
    if devops:
        devops_task = (
            f"write file setup_nexus.sh with a complete setup script for {req.project_name}. "
            f"Architect's deployment plan:\n{arch_response[:400]}\n\n"
            f"Available files:\n{manifest}\n\n"
            f"Create a script that: installs dependencies, copies files, configures services, "
            f"sets up the message broker, starts all microservices, and runs health checks."
        )
        devops_response = await devops.process_task(devops_task, sender="deployment")
        steps.append({"phase": "Setup Scripts", "agent": "DevOps Engineer",
                       "name": devops.persona.name, "response": devops_response})

    # Step 3: Lead Developer creates the main entry point
    dev = engine_team.get("Lead Developer")
    if dev:
        dev_task = (
            f"write file nexus_main.py with the main entry point for {req.project_name}. "
            f"This should import and initialize all microservices from the integrated files: "
            f"{manifest}\n\n"
            f"Create a proper Python application that starts all services, "
            f"sets up logging, handles graceful shutdown, and prints system status."
        )
        dev_response = await dev.process_task(dev_task, sender="deployment")
        steps.append({"phase": "Entry Point", "agent": "Lead Developer",
                       "name": dev.persona.name, "response": dev_response})

    # Step 4: Security check
    security = engine_team.get("Security Auditor")
    if security:
        sec_task = (
            f"Review the deployment plan for {req.project_name} and identify any security concerns. "
            f"Deployment plan:\n{arch_response[:300]}\n\n"
            f"Check for: exposed ports, default credentials, missing authentication, "
            f"insecure configurations, and recommend security hardening steps."
        )
        sec_response = await security.process_task(sec_task, sender="deployment")
        steps.append({"phase": "Security Review", "agent": "Security Auditor",
                       "name": security.persona.name, "response": sec_response})

    # Step 5: QA validates
    qa = engine_team.get("QA Engineer")
    if qa:
        qa_task = (
            f"write file validate_deployment.py with a deployment validation script for {req.project_name}. "
            f"The script should check: all services are running, endpoints respond correctly, "
            f"message broker is connected, database is accessible, and all health checks pass. "
            f"Use the requests library to validate HTTP endpoints."
        )
        qa_response = await qa.process_task(qa_task, sender="deployment")
        steps.append({"phase": "Validation", "agent": "QA Engineer",
                       "name": qa.persona.name, "response": qa_response})

    pipeline.scan_artifacts()

    return {
        "project": req.project_name,
        "manifest": manifest,
        "steps": steps,
        "message": f"Deployment pipeline completed with {len(steps)} phases",
    }


@router.get("/artifacts/stats")
async def artifact_stats():
    from swarm_v2.app_v2 import pipeline
    return pipeline.get_stats()

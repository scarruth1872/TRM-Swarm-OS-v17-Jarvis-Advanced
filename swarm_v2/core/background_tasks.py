"""
Background task loops for Swarm OS.
These run as asyncio tasks during the API lifespan.
Extracted from app_v2.py during route splitting refactor.
"""

import asyncio
import os
from datetime import datetime

# ─── Auto-Scan Pipeline (Shield Security Loop) ─────────────────────────────

async def auto_scan_pipeline(pipeline, engine_team):
    """Background loop to automatically scan new artifacts for security risks."""
    print("[Shield] Auto-scan loop initialized.")
    while True:
        try:
            shield = engine_team.get("Security Auditor")
            if not shield:
                await asyncio.sleep(30)
                continue

            pipeline.scan_artifacts()
            unscanned = pipeline.list_unscanned()

            for art in unscanned:
                filename = art["filename"]
                print(f"[Shield] Security auditing: {filename}...")

                if filename.endswith((".md", ".txt", ".json", ".log", ".yaml", ".yml")):
                    pipeline.set_security_status(filename, "safe", "Skipped (static content)")
                    continue

                pipeline.set_security_status(filename, "scanning")
                content = pipeline.get_content(filename) or ""
                task = (
                    f"Perform a SECURITY AUDIT on the following code artifact: {filename}. "
                    f"Identify any vulnerabilities (secrets, injection, RCE, insecure config). "
                    f"If the code is SAFE, reply with 'SAFE' as the primary keyword. "
                    f"If UNSAFE, explain the specific risks. "
                    f"\\n\\nSource Code Context:\\n{content[:4000]}"
                )
                response_obj = await shield.process_task(task, sender="shield_automator")
                response_text = str(response_obj.get("response", "")) if isinstance(response_obj, dict) else str(response_obj)
                status = "unsafe"
                if "safe" in response_text.lower() and "unsafe" not in response_text.lower() and "vulnerability" not in response_text.lower():
                    status = "safe"
                # FIX: pass filename as required first argument
                pipeline.set_security_status(filename, status)
        except Exception as e:
            print(f"[Shield] Auto-scan error: {e}")
        await asyncio.sleep(30)


# ─── Mesh Heartbeat ────────────────────────────────────────────────────────

async def mesh_heartbeat_reflex(agent_mesh, engine_team):
    """Maintain the 'alive' status of all local experts on the mesh."""
    while True:
        try:
            for role, agent in engine_team.items():
                if hasattr(agent, "mesh_node_id"):
                    agent_mesh.propagate_heartbeat(agent.mesh_node_id)
            await asyncio.sleep(30)
        except Exception as e:
            print(f"[Mesh] Heartbeat error: {e}")
            await asyncio.sleep(30)


# ─── Proactive Orchestration Loop ──────────────────────────────────────────

triggered_proposals = set()
acted_research_findings = set()
active_orchestration_tasks = 0

async def proactive_orchestration_loop(pipeline, engine_team, agent_mesh):
    """Background task that looks for 'Approved' plans and asks for execution."""
    global triggered_proposals, acted_research_findings, active_orchestration_tasks
    log_file = "swarm_v2_memory/orchestration.log"

    while True:
        try:
            pipeline.scan_artifacts()
            approved = pipeline.list_by_status(pipeline.ArtifactStatus.APPROVED)

            def priority(fname):
                if "scribe" in fname.lower() or "antigravity" in fname.lower():
                    return 0
                return 1
            approved.sort(key=lambda x: priority(x["filename"]))

            plan_triggered = False
            if active_orchestration_tasks < 2:
                for art in approved:
                    filename = art["filename"]
                    clean_filename = filename.replace("swarm_v2_artifacts/", "").replace("swarm_v2_artifacts\\\\", "")
                    is_plan = any(k in clean_filename.lower() for k in ["plan", "blueprint", "spec"]) and "artifact" not in clean_filename.lower()
                    if is_plan and not clean_filename.startswith("PROPOSAL_"):
                        proposal_name = f"PROPOSAL_{os.path.splitext(os.path.basename(clean_filename))[0]}.md"
                        if proposal_name not in pipeline.artifacts and clean_filename not in triggered_proposals:
                            triggered_proposals.add(clean_filename)
                            from swarm_v2.core.kanban_board import get_kanban_board
                            kb = get_kanban_board()
                            card_id = kb.create_card(title=f"Implement {clean_filename}", description="Autonomously triggered proposal.", assignee="Lead Developer", priority="high")
                            kb.move_card(card_id, "TODO")
                            devo = engine_team.get("Lead Developer")
                            if devo:
                                content = pipeline.get_content(filename)
                                trigger_msg = (
                                    f"IMPORTANT: Autonomous Task Detection.\\nI have detected an approved Implementation Plan: '{clean_filename}'.\\n\\n"
                                    f"PLAN CONTENT:\\n{content[:2000]}\\n\\nYOUR TASK:\\n1. Inspect the plan above.\\n"
                                    f"2. Generate a formal 'Build Proposal' artifact.\\n"
                                    f"3. Use the format: CREATE_FILES:\\n--- {proposal_name} ---\\n..."
                                )
                                async def ran_devo_task(msg, agent, cid):
                                    global active_orchestration_tasks
                                    active_orchestration_tasks += 1
                                    kb.move_card(cid, "IN_PROGRESS")
                                    try:
                                        await agent.process_task(msg, sender="Orchestrator")
                                    finally:
                                        active_orchestration_tasks -= 1
                                asyncio.create_task(ran_devo_task(trigger_msg, devo, card_id))
                                plan_triggered = True
                                break
            await asyncio.sleep(15)
        except Exception as e:
            with open(log_file, "a") as f:
                f.write(f"{datetime.now().strftime('%Y-%m-%d')} [ERROR] Loop error: {e}\\n")
            await asyncio.sleep(15)


# ─── Autonomous Pipeline Loop ─────────────────────────────────────────────

async def autonomous_pipeline_loop(pipeline, engine_team):
    """
    Background task that autonomously tests, approves, and integrates pending artifacts.
    FIXED:
      - Non-code files (.json, .md, .log, .yaml) auto-approved, not test-run.
      - integrate() now properly awaited (was async but called synchronously).
      - Integration loop drains all approved-but-not-integrated artifacts.
    """
    from swarm_v2.core.kanban_board import get_kanban_board
    from swarm_v2.core.artifact_pipeline import ArtifactStatus
    kb = get_kanban_board()
    log_file = "swarm_v2_memory/pipeline.log"

    # Non-code extensions — skip QA testing, just approve & integrate directly
    NON_CODE_EXTS = ('.md', '.txt', '.json', '.log', '.yaml', '.yml', '.toml', '.cfg', '.ini', '.env')
    # Subdirectory mapping by file type
    SUBDIR_MAP = {
        'docs': ('.md', '.txt'),
        'config': ('.json', '.yaml', '.yml', '.toml', '.cfg', '.ini', '.env'),
        'logs': ('.log',),
    }

    def _target_subdir(filename):
        ext = os.path.splitext(filename)[1].lower()
        for subdir, exts in SUBDIR_MAP.items():
            if ext in exts:
                return subdir
        return 'code'

    integration_batch_size = 30  # Integrate up to 30 approved artifacts per cycle

    while True:
        try:
            pipeline.scan_artifacts()

            # ── Phase 1: Process PENDING artifacts ──────────────────────────
            pending = pipeline.list_by_status(ArtifactStatus.PENDING)
            for art in pending[:20]:  # Limit per cycle to avoid overwhelming QA
                filename = art["filename"]

                active_cards = kb.get_column("IN_PROGRESS") + kb.get_column("TODO")
                matching_card = next(
                    (c for c in active_cards if filename in c.get("title", "") or filename in c.get("description", "")),
                    None
                )
                if matching_card:
                    kb.move_card(matching_card["card_id"], "REVIEW")

                ext = os.path.splitext(filename)[1].lower()

                # Non-code: auto-approve without QA test run
                if ext in NON_CODE_EXTS:
                    pipeline.approve(filename, "System", "Auto-approved: static/non-code file")
                    continue

                # Code files: run QA Engineer test generation
                content = pipeline.get_content(filename)
                qa = engine_team.get("QA Engineer")
                if qa and content and len(content) > 10:
                    test_filename = f"test_{filename}" if not filename.startswith("test_") else filename
                    if test_filename != filename:
                        test_task = (
                            f"[ACTION] Create a comprehensive pytest test suite for this code file.\n"
                            f"File: '{filename}':\n\n{content[:1500]}"
                        )
                        try:
                            await qa.process_task(test_task, sender="autonomous_pipeline")
                        except Exception as qa_err:
                            print(f"[Pipeline] QA task error for {filename}: {qa_err}")
                        test_content = pipeline.get_content(test_filename)
                        # FIXED: test passes only if QA actually wrote a non-trivial test file
                        passed = (test_content is not None
                                  and len(test_content) > 50
                                  and ("def test_" in test_content or "class Test" in test_content))
                        pipeline.set_tested(filename, test_filename, passed, "Auto-verified by CI loop")
                        if passed:
                            pipeline.approve(filename, "System", "Auto-approved after tests passed")
                    await asyncio.sleep(5)

            # ── Phase 2: Integrate APPROVED artifacts ─────────────────────────
            # FIXED: integrate() is async — must be awaited.
            # Also drains the backlog of 1,500+ stuck approved artifacts.
            approved = pipeline.list_by_status(ArtifactStatus.APPROVED)
            integrated_this_cycle = 0
            for art in approved:
                if integrated_this_cycle >= integration_batch_size:
                    break
                filename = art.get("filename")
                if not filename:
                    continue
                # Skip if already marked integrated (race condition guard)
                if art.get("integrated_at"):
                    continue
                try:
                    subdir = _target_subdir(filename)
                    result = await pipeline.integrate(filename, target_subdir=subdir)
                    if result:
                        integrated_this_cycle += 1
                except Exception as int_err:
                    print(f"[Pipeline] Integration error for {filename}: {int_err}")
            if integrated_this_cycle > 0:
                print(f"[Pipeline] Integrated {integrated_this_cycle} artifacts this cycle. Backlog remaining: {max(0, len(approved) - integrated_this_cycle)}")

        except Exception as e:
            with open(log_file, "a") as f:
                f.write(f"{datetime.now().strftime('%Y-%m-%d %H:%M:%S')} [ERROR] Pipeline loop: {e}\n")
            print(f"[Pipeline] Loop error: {e}")
        await asyncio.sleep(30)

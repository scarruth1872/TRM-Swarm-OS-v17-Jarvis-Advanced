"""Research / reconnaissance endpoints — /research."""
import threading
from fastapi import APIRouter, HTTPException

from swarm_v2.core.reconnaissance_daemon import get_reconnaissance_daemon, start_reconnaissance

router = APIRouter(tags=["research"])


@router.get("/research/tasks")
async def get_research_tasks():
    daemon = get_reconnaissance_daemon()
    if not daemon:
        return {"tasks": []}
    return {
        "tasks": [
            {
                "task_id": f.finding_id,
                "objective": f.topic,
                "summary": f.summary,
                "sources": f.sources,
                "status": "completed",
                "timestamp": f.timestamp
            } for f in daemon.findings[-20:]
        ]
    }


@router.get("/research/papers")
async def get_research_papers():
    """List generated research papers."""
    from pathlib import Path
    import time
    papers = sorted(Path("research").glob("*.md"), key=lambda f: f.stat().st_mtime, reverse=True)
    return {
        "papers": [{"name": p.name, "size": p.stat().st_size,
                     "modified": time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(p.stat().st_mtime))}
                   for p in papers[:20]]
    }


@router.get("/research/stats")
async def get_research_stats():
    daemon = get_reconnaissance_daemon()
    if not daemon:
        return {"is_running": False, "total_findings": 0}
    return daemon.get_stats()


@router.get("/research/findings")
async def get_research_findings(limit: int = 10, topic: str = ""):
    """Get recent AI research findings from the reconnaissance daemon."""
    daemon = get_reconnaissance_daemon()
    if topic:
        return {"findings": daemon.get_findings_by_topic(topic)}
    return {"findings": daemon.get_recent_findings(limit)}


@router.post("/research/run")
async def trigger_research_cycle():
    """Trigger an immediate research cycle."""
    daemon = get_reconnaissance_daemon()
    # Run in background to not block
    thread = threading.Thread(target=daemon.run_immediate, daemon=True)
    thread.start()
    return {"status": "triggered", "message": "Research cycle started in background"}


@router.post("/research/synthesis")
async def generate_research_synthesis():
    """Generate a comparative summary of recent findings for integration."""
    from swarm_v2.app_v2 import engine_team

    daemon = get_reconnaissance_daemon()
    recent_findings = daemon.get_recent_findings(10)

    if not recent_findings:
        return {"synthesis": "No recent findings available for synthesis."}

    compiled_data = ""
    for idx, f in enumerate(recent_findings):
        compiled_data += f"\n--- Finding {idx+1}: {f['topic']} ---\n{f['summary']}\n"

    # Prompt Researcher or Arbiter to synthesize
    prompt = (
        "Analyze the following recent AI research findings and generate a concise "
        "comparative summary. Conclude with actionable recommendations on which concepts "
        "we should integrate into our Swarm OS ecosystem to improve intelligence and "
        "efficiency.\n\n"
        f"{compiled_data}"
    )

    researcher = engine_team.get("Researcher") or engine_team.get("Arbiter")
    if not researcher:
        return {"synthesis": "Error: Researcher agent not available to perform synthesis."}

    try:
        response = await researcher.process_task(prompt, sender="research_daemon")
        return {"synthesis": response}
    except Exception as e:
        return {"synthesis": f"Error during synthesis: {str(e)}"}


@router.post("/research/topics")
async def add_research_topic(topic: str):
    """Add a new research topic."""
    daemon = get_reconnaissance_daemon()
    daemon.add_topic(topic)
    return {"status": "added", "topic": topic}


@router.delete("/research/topics/{topic}")
async def remove_research_topic(topic: str):
    """Remove a research topic."""
    daemon = get_reconnaissance_daemon()
    daemon.remove_topic(topic)
    return {"status": "removed", "topic": topic}


@router.post("/research/start")
async def start_research_daemon(interval_hours: float = 24):
    """Start the reconnaissance daemon."""
    daemon = start_reconnaissance(interval_hours)
    return {"status": "started", "interval_hours": interval_hours}


@router.post("/research/stop")
async def stop_research_daemon():
    """Stop the reconnaissance daemon."""
    daemon = get_reconnaissance_daemon()
    daemon.stop()
    return {"status": "stopped"}


@router.post("/research/sovereign-unity")
async def execute_sovereign_unity_pipeline(payload: dict = None):
    """
    Executes the Sovereign Unity Protocol (Phase 1 Integration).
    Wired directly across all 4 native engines: Shm IPC -> Pharmacophore -> P2P Route -> Text Briefing -> 14-Agent PBFT.
    """
    import subprocess
    import os

    target = (payload or {}).get("target", "3P3G_LpxC")
    ligand = (payload or {}).get("ligand", "TRM-ABX-082")
    dg = float((payload or {}).get("dg", -10.42))

    bin_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "swarm_v2_rust_kernel", "target", "release", "trm_swarm_boot.exe"))
    if not os.path.exists(bin_path):
        return {"error": "Native binary not found", "bin_path": bin_path}

    proc = subprocess.run(
        [bin_path, "unity", "--target", target, "--ligand", ligand, "--dg", str(dg)],
        capture_output=True,
        text=True
    )
    return {
        "status": "success",
        "target": target,
        "ligand": ligand,
        "dg": dg,
        "stdout": proc.stdout,
        "returncode": proc.returncode,
    }


@router.get("/research/campaign")
async def get_virtual_screening_campaign():
    """Returns Phase 2 Virtual Screening Campaign metrics and discovered top leads."""
    import json
    import os
    p = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "swarm_v2_artifacts", "virtual_screening_campaign_1m.json"))
    if os.path.exists(p):
        with open(p, "r", encoding="utf-8") as f:
            return json.load(f)
    return {"status": "pending", "message": "Campaign data not yet sealed"}


@router.get("/research/theorems")
async def get_formal_verification_theorems():
    """Returns Phase 3 Formal Mathematical Verification theorem proof status."""
    import json
    import os
    p = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "swarm_v2_artifacts", "formal_verification_theorems_proven.json"))
    if os.path.exists(p):
        with open(p, "r", encoding="utf-8") as f:
            return json.load(f)
    return {"status": "pending", "message": "Theorems data not yet sealed"}



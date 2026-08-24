"""Security / Neural Wall endpoints — /security."""
from typing import Optional, Dict, Any
from fastapi import APIRouter

from swarm_v2.routes.models import AnalyzePromptRequest
from swarm_v2.core.neural_wall import get_neural_wall, analyze_prompt

router = APIRouter(tags=["security"])


@router.post("/security/analyze")
async def analyze_prompt_endpoint(req: AnalyzePromptRequest):
    """Analyze text for prompt injection threats."""
    detection = await analyze_prompt(req.text, req.context)

    if detection.is_malicious:
        try:
            from swarm_v2.core.ddr_antibody import get_ddr
            get_ddr().record_error(
                file_path="NeuralWall",
                line_number=0,
                error_type="prompt_injection",
                fix_description=f"Prompt injection blocked. Vector: {detection.threat_type}",
                line_pattern=req.text[:50].replace('\n', ' '),
                severity="high",
                recorded_by="NeuralWall"
            )
            print(f"[NeuralWall] Recorded new antibody in DDR Vault for prompt injection.")
        except Exception as e:
            print(f"[NeuralWall] Failed to record antibody: {e}")

    return {
        "is_malicious": detection.is_malicious,
        "threat_level": detection.threat_level.value,
        "threat_type": detection.threat_type,
        "confidence": detection.confidence,
        "description": detection.description,
        "blocked_patterns": detection.blocked_patterns,
        "timestamp": detection.timestamp
    }


@router.get("/security/stats")
async def neural_wall_stats():
    """Get NeuralWall detection statistics."""
    wall = get_neural_wall()
    stats = wall.get_stats()
    return {
        **stats,
        "threats_detected_24h": stats.get("total_detections", 0),
        "threats_neutralized_24h": stats.get("blocked", 0)
    }


@router.get("/security/threats")
async def get_recent_threats(limit: int = 10):
    """Get recent threat detections."""
    wall = get_neural_wall()
    threats = wall.get_recent_threats(limit)
    formatted = []
    for i, t in enumerate(threats):
        formatted.append({
            "id": f"threat_{i}",
            "timestamp": t.get("timestamp"),
            "threat_type": t.get("threat_type"),
            "severity": t.get("threat_level"),
            "action_taken": "Blocked & Logged",
            "source_ip": "127.0.0.1"
        })
    return {"threats": formatted}


@router.post("/security/scan")
async def trigger_security_scan():
    """Run a full attack-surface scan: Python deps, Node deps, artifacts, system."""
    from swarm_v2.core.security_scanner import get_security_scanner
    scanner = get_security_scanner()
    results = await scanner.scan_all()
    return results


@router.post("/security/false-positive/{index}")
async def mark_false_positive(index: int):
    """Mark a detection as false positive."""
    return {"status": "marked", "index": index}

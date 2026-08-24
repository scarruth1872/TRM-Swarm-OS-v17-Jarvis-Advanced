"""Performance insights endpoints — /insights."""
import os
import json
import asyncio
from typing import Dict
from fastapi import APIRouter, HTTPException

from swarm_v2.core.swarm_performance_insights import (
    get_performance_insights,
    start_performance_monitoring,
    ReportPeriod,
    MetricType,
)

router = APIRouter(tags=["insights"])


@router.get("/insights/status")
async def performance_insights_status():
    """Get Performance Insights engine status."""
    insights = get_performance_insights()
    return insights.get_status()


@router.post("/insights/report")
async def generate_performance_report(period: str = "weekly"):
    """Generate a new Swarm Intelligence Report."""
    insights = get_performance_insights()

    try:
        report_period = ReportPeriod(period.lower())
    except ValueError:
        report_period = ReportPeriod.WEEKLY

    report = await insights.generate_report(report_period)

    return {
        "status": "generated",
        "report_id": report.report_id,
        "health_score": report.overall_health_score,
        "efficiency_score": report.efficiency_score,
        "growth_score": report.growth_score,
        "total_tasks": report.total_tasks_processed,
        "success_rate": (report.total_tasks_processed - report.total_errors) / max(1, report.total_tasks_processed) * 100
    }


@router.get("/insights/reports")
async def list_performance_reports(limit: int = 10):
    """Get recent Swarm Intelligence Reports."""
    insights = get_performance_insights()
    return {"reports": insights.get_recent_reports(limit)}


@router.get("/insights/reports/{report_id}")
async def get_performance_report(report_id: str):
    """Get a specific report by ID."""
    insights = get_performance_insights()

    for report in insights.reports:
        if report.report_id == report_id:
            return report.to_dict()

    # Try to load from disk
    report_path = os.path.join(insights.reports_dir, f"{report_id}.json")
    if os.path.exists(report_path):
        with open(report_path, "r", encoding="utf-8") as f:
            return json.load(f)

    raise HTTPException(status_code=404, detail=f"Report '{report_id}' not found")


@router.get("/insights/reports/{report_id}/markdown")
async def get_performance_report_markdown(report_id: str):
    """Get a specific report in markdown format."""
    insights = get_performance_insights()

    for report in insights.reports:
        if report.report_id == report_id:
            return {"markdown": report.to_markdown(), "report_id": report_id}

    # Try to load from disk
    md_path = os.path.join(insights.reports_dir, f"{report_id}.md")
    if os.path.exists(md_path):
        with open(md_path, "r", encoding="utf-8") as f:
            return {"markdown": f.read(), "report_id": report_id}

    raise HTTPException(status_code=404, detail=f"Report '{report_id}' not found")


@router.get("/insights/metrics/{metric_type}")
async def get_metric_history(metric_type: str, limit: int = 100):
    """Get history for a specific metric type."""
    insights = get_performance_insights()

    try:
        m_type = MetricType(metric_type.lower())
    except ValueError:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid metric type. Valid types: {[m.value for m in MetricType]}"
        )

    history = insights.get_metric_history(m_type, limit)
    return {"metric_type": metric_type, "history": history, "count": len(history)}


@router.post("/insights/metrics/{metric_type}/record")
async def record_metric_value(metric_type: str, value: float, metadata: Dict = None):
    """Manually record a metric data point."""
    from typing import Dict as TypingDict
    insights = get_performance_insights()

    try:
        m_type = MetricType(metric_type.lower())
    except ValueError:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid metric type. Valid types: {[m.value for m in MetricType]}"
        )

    insights.record_metric(m_type, value, metadata)
    return {"status": "recorded", "metric_type": metric_type, "value": value}


@router.post("/insights/tasks/record")
async def record_task_execution(success: bool = True):
    """Record a task execution for success rate tracking."""
    insights = get_performance_insights()
    insights.record_task(success)

    status = insights.get_status()
    return {
        "status": "recorded",
        "success": success,
        "session_tasks": status["session_tasks"],
        "session_errors": status["session_errors"]
    }


@router.post("/insights/start")
async def start_performance_monitoring_endpoint(interval_hours: int = 168):
    """Start the performance monitoring loop (default: weekly)."""
    asyncio.create_task(start_performance_monitoring(interval_hours))
    return {"status": "started", "interval_hours": interval_hours}


@router.post("/insights/stop")
async def stop_performance_monitoring():
    """Stop the performance monitoring loop."""
    insights = get_performance_insights()
    insights.stop()
    return {"status": "stopped"}

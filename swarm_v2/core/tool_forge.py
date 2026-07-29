import os
import json
import logging
import asyncio
from datetime import datetime
from typing import Dict, List, Optional

logger = logging.getLogger("ToolForge")

ROLE_CAPABILITIES = {
    "Architect": ["DiagramSkill", "SystemDesignSkill", "ConstraintSolver"],
    "Lead Developer": ["GitSkill", "RefactorSkill", "TestGeneratorSkill"],
    "Researcher": ["CitationSkill", "SummarizationSkill", "TrendAnalysis"],
    "Reasoning Engine": ["MathSolver", "LogicProver", "BenchmarkSkill"],
    "Security Auditor": ["VulnScanSkill", "CryptoAuditSkill", "ComplianceCheck"],
    "DevOps Engineer": ["ContainerSkill", "MonitoringSkill", "DeploymentSkill", "SystemMetricsSkill"],
    "UI/UX Designer": ["MockupSkill", "AccessibilityAudit", "ComponentLibrary"],
    "QA Engineer": ["FuzzTestSkill", "RegressionSkill", "CoverageAnalysis", "OutputValidatorSkill"],
    "Swarm Manager": ["LoadBalancerSkill", "AgentProfiler", "MissionPlanner", "SystemMetricsSkill"],
    "Technical Writer": ["APIDocGenerator", "TutorialBuilder", "ChangelogSkill"],
    "Integration Specialist": ["ProtocolBridgeSkill", "DataMapperSkill", "WebhookSkill"],
    "Data Analyst": ["VisualizationSkill", "StatModelSkill", "AnomalyDetector", "SystemMetricsSkill"]
}

class ToolForge:
    def __init__(self, learning_engine):
        self.learning_engine = learning_engine
        self.build_log = []
        self.assessment_history = []

    def assess_agent_needs(self, role: str, current_skills: List[str]) -> List[str]:
        expected = ROLE_CAPABILITIES.get(role, [])
        gaps = [skill for skill in expected if skill not in current_skills]
        return gaps

    def get_all_gaps(self, engine_team) -> Dict[str, List[str]]:
        gaps = {}
        for role, agent in engine_team.items():
            agent_skills = agent.get_skill_names()
            agent_gaps = self.assess_agent_needs(role, agent_skills)
            if agent_gaps:
                gaps[role] = agent_gaps
        return gaps

    def aggregate_gaps(self, all_gaps: Dict[str, List[str]]) -> List[Dict]:
        counts = {}
        for role, gaps in all_gaps.items():
            for gap in gaps:
                if gap not in counts:
                    counts[gap] = {"tool_name": gap, "needed_by": [], "score": 0}
                counts[gap]["needed_by"].append(role)
                counts[gap]["score"] += 1
        
        sorted_gaps = sorted(counts.values(), key=lambda x: x["score"], reverse=True)
        return sorted_gaps

    async def run_forge_pipeline(self, tool_name: str, engine_team) -> Dict:
        start_time = datetime.now().isoformat()
        build_entry = {
            "tool_name": tool_name,
            "start_time": start_time,
            "status": "in_progress",
            "stages": {}
        }
        self.build_log.append(build_entry)

        # Stage 1: Architect (Archi)
        archi = engine_team.get("Architect")
        spec = ""
        if archi:
            logger.info(f"[Forge] Stage 1: Architecting {tool_name}")
            prompt = f"Design a complete architecture specification for a new tool named '{tool_name}'. Outline its input arguments, expected outputs, core logic, error handling cases, and structure. Keep it technical and concise."
            res = await archi.process_task(prompt, sender="ToolForge")
            spec = res.get("response", res) if isinstance(res, dict) else res
            build_entry["stages"]["Architect"] = {"status": "completed", "output": res}
        else:
            spec = f"Default specification for {tool_name}"
            build_entry["stages"]["Architect"] = {"status": "skipped", "output": "Architect agent not found"}

        # Stage 2: Lead Developer (Devo)
        devo = engine_team.get("Lead Developer")
        code = ""
        if devo:
            logger.info(f"[Forge] Stage 2: Developing {tool_name}")
            prompt = f"Based on the following architecture spec:\n{spec}\nWrite clean, production-ready Python code implementing this tool. Output ONLY a valid Python code block with the implementation."
            res = await devo.process_task(prompt, sender="ToolForge")
            code = res.get("response", res) if isinstance(res, dict) else res
            build_entry["stages"]["Lead Developer"] = {"status": "completed", "output": res}
        else:
            build_entry["stages"]["Lead Developer"] = {"status": "failed", "output": "Developer agent not found"}
            build_entry["status"] = "failed"
            return build_entry

        # Stage 3: Security Auditor (Shield)
        shield = engine_team.get("Security Auditor")
        security_report = ""
        if shield:
            logger.info(f"[Forge] Stage 3: Auditing {tool_name}")
            prompt = f"Perform a comprehensive security audit on this code implementation:\n{code}\nIdentify any potential vulnerabilities, unsafe execution paths, injection vectors, or packaging issues. Provide recommendations."
            res = await shield.process_task(prompt, sender="ToolForge")
            security_report = res.get("response", res) if isinstance(res, dict) else res
            build_entry["stages"]["Security Auditor"] = {"status": "completed", "output": res}
        else:
            build_entry["stages"]["Security Auditor"] = {"status": "skipped", "output": "Security agent not found"}

        # Stage 4: QA Engineer (Verify)
        verify = engine_team.get("QA Engineer")
        test_code = ""
        if verify:
            logger.info(f"[Forge] Stage 4: Testing {tool_name}")
            prompt = f"Create unit tests using pytest for the following tool code:\n{code}\nOutput ONLY the test python code."
            res = await verify.process_task(prompt, sender="ToolForge")
            test_code = res.get("response", res) if isinstance(res, dict) else res
            build_entry["stages"]["QA Engineer"] = {"status": "completed", "output": res}
        else:
            build_entry["stages"]["QA Engineer"] = {"status": "skipped", "output": "QA agent not found"}

        # Stage 5: Technical Writer (Scribe)
        scribe = engine_team.get("Technical Writer")
        documentation = ""
        if scribe:
            logger.info(f"[Forge] Stage 5: Documenting {tool_name}")
            prompt = f"Write high-density, professional technical documentation for this tool:\n{code}\nInclude preconditions, usage instructions, arguments, and return types."
            res = await scribe.process_task(prompt, sender="ToolForge")
            documentation = res.get("response", res) if isinstance(res, dict) else res
            build_entry["stages"]["Technical Writer"] = {"status": "completed", "output": res}
        else:
            build_entry["stages"]["Technical Writer"] = {"status": "skipped", "output": "Scribe agent not found"}

        # Hot-register newly forged tool inside the learning engine
        logger.info(f"[Forge] Registering {tool_name} to Learning Engine")
        skill = await self.learning_engine.learn_from_text(
            name=tool_name,
            content=f"{code}\n\n# Tests\n{test_code}\n\n# Documentation\n{documentation}",
            source=f"forge://{tool_name}"
        )

        build_entry["status"] = "completed"
        build_entry["completed_at"] = datetime.now().isoformat()
        build_entry["registered_skill"] = skill.to_dict() if skill else None
        
        return build_entry

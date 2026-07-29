"""
Agent Coordination Responder — makes agents auto-respond to coordination tasks.
Runs as a background asyncio loop during TRM lifespan.
"""
import json, asyncio, os, time
from datetime import datetime
from swarm_v2.core.agent_mailbox import AgentMailbox, MAILBOX_ROOT

class CoordinationResponder:
    """Polls agent mailboxes and auto-responds to coordination tasks."""

    def __init__(self):
        self.prompt_templates = {
            "Architect": "You are the Architect. Evaluate this task based on system architecture impact, feasibility, and alignment with existing infrastructure. Provide a concise 1-2 sentence recommendation.",
            "Reasoning Engine": "You are the Reasoning Engine. Analyze this task through logical reasoning, considering dependencies, risks, and technical soundness. Give a structured recommendation.",
            "Security Auditor": "You are the Security Auditor. Assess this task for security implications, potential vulnerabilities introduced, and alignment with current security posture.",
            "QA Engineer": "You are the QA Engineer. Evaluate this task from a testing and quality assurance perspective. Consider testability and regression risk.",
            "Lead Developer": "You are the Lead Developer. Assess this task from an implementation standpoint: complexity, maintainability, and developer effort.",
            "DevOps Engineer": "You are the DevOps Engineer. Evaluate this task for operational impact: deployment, monitoring, infrastructure requirements.",
            "Data Analyst": "You are the Data Analyst. Assess data concerns: collection, storage, processing, and insights this task would generate.",
            "Integration Specialist": "You are the Integration Specialist. Evaluate how this task integrates with existing systems and APIs.",
            "Swarm Manager": "You are the Swarm Manager. Assess this task for overall swarm health: resource allocation, agent workload, and strategic alignment.",
            "Researcher": "You are the Researcher. Suggest novel approaches or research-backed solutions for this task.",
            "Technical Writer": "You are the Technical Writer. Assess documentation needs and knowledge transfer requirements.",
            "UI/UX Designer": "You are the UI/UX Designer. Assess user experience impact and interface requirements.",
        }

    async def poll_all_agents(self, llm_generate_fn=None):
        """Poll all agent mailboxes and respond to coordination tasks."""
        if not llm_generate_fn:
            return

        mailbox_root = os.path.join(os.path.dirname(__file__), "..", "..", ".swarm", "mailboxes")
        if not os.path.exists(mailbox_root):
            return

        for agent_name in os.listdir(mailbox_root):
            inbox_path = os.path.join(mailbox_root, agent_name, "inbox.json")
            outbox_path = os.path.join(mailbox_root, agent_name, "outbox.json")
            if not os.path.exists(inbox_path):
                continue

            try:
                with open(inbox_path) as f:
                    inbox = json.load(f)
            except (json.JSONDecodeError, OSError):
                continue

            responded = set()
            if os.path.exists(outbox_path):
                try:
                    with open(outbox_path) as f:
                        for m in json.load(f):
                            sub = m.get("subject", "")
                            if "[COORD-RESPONSE]" in sub:
                                body = json.loads(m.get("message", "{}"))
                                responded.add(body.get("round_id", ""))
                except (json.JSONDecodeError, OSError):
                    pass

            new_responses = []
            for msg in inbox:
                subject = msg.get("subject", "")
                if "[COORD]" not in subject:
                    continue
                try:
                    body = json.loads(msg.get("message", "{}"))
                except (json.JSONDecodeError, TypeError):
                    continue

                round_id = body.get("round_id", "")
                if round_id in responded:
                    continue

                task = body.get("task", "")
                context = body.get("context", "")
                prompt = self.prompt_templates.get(agent_name,
                    f"You are {agent_name}. Evaluate this task and provide your recommendation.")

                full_prompt = f"{prompt}\n\nTask: {task}\nContext: {context}\n\nYour recommendation (1-2 sentences):"
                try:
                    response = await llm_generate_fn(full_prompt)
                    if not response:
                        response = f"{agent_name} acknowledges the task but has no specific recommendation at this time."
                except Exception as e:
                    response = f"{agent_name} is unable to respond at this time: {e}"

                proposal = json.dumps({
                    "round_id": round_id,
                    "agent": agent_name,
                    "proposal": str(response)[:500],
                    "responded_at": datetime.now().isoformat(),
                })

                mailbox = AgentMailbox(agent_name)
                mailbox.send("Coordinator", proposal, subject=f"[COORD-RESPONSE] Round {round_id}")
                new_responses.append(round_id)
                print(f"[CoordinationResponder] {agent_name} responded to round {round_id[:20]}")

            if new_responses:
                print(f"[CoordinationResponder] {agent_name}: {len(new_responses)} new response(s)")


_responder = None
def get_coordination_responder():
    global _responder
    if _responder is None:
        _responder = CoordinationResponder()
    return _responder

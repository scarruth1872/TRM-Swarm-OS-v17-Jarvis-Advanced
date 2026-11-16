# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 28, Rank 134)
# CDYLIB: strip_coordination_protocol.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_coordination_protocol.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
"""
Agent Coordination Protocol — Multi-agent task broadcast, bid, consensus, and aggregation.
Built on existing AgentMailbox + UltraworkLoop infrastructure.
"""
import json, time, uuid, asyncio
from typing import Optional
from swarm_v2.core.agent_mailbox import AgentMailbox
from swarm_v2.core.ultrawork_loop import get_ultrawork_loop

class CoordinationProtocol:
    """
    Lightweight multi-agent coordination:
    1. Broadcast a task to all (or specific) agents
    2. Agents respond with proposals/bids
    3. Coordinator selects best response
    4. Aggregates and returns result
    """

    def __init__(self):
        self.active_rounds: dict[str, dict] = {}

    async def broadcast_task(self, task: str, target_agents: list[str] = None,
                               context: str = "", timeout: int = 30) -> dict:
        """
        Broadcast a task to target agents.
        Returns round_id for collecting responses.
        """
        round_id = f"coord-{uuid.uuid4().hex[:8]}"
        mailbox = AgentMailbox("Coordinator")
        agents = target_agents or []
        sent_to = []
        for agent in agents:
            try:
                mailbox.send(agent, json.dumps({
                    "type": "coordination_task",
                    "round_id": round_id,
                    "task": task,
                    "context": context,
                    "respond_by": time.time() + timeout,
                }), subject=f"[COORD] Task: {task[:50]}")
                sent_to.append(agent)
            except Exception as e:
                print(f"[Coordination] Failed to send to {agent}: {e}")

        self.active_rounds[round_id] = {
            "task": task,
            "context": context,
            "sent_to": sent_to,
            "responses": {},
            "started_at": time.time(),
            "timeout": timeout,
        }

        return {"round_id": round_id, "agents": sent_to, "status": "broadcast"}

    async def collect_responses(self, round_id: str, wait: bool = True,
                                 max_wait: int = 60) -> dict:
        """Collect responses from agents for a given round."""
        round_data = self.active_rounds.get(round_id)
        if not round_data:
            return {"error": f"Round {round_id} not found"}

        mailbox = AgentMailbox("Coordinator")
        deadline = time.time() + max_wait

        while time.time() < deadline and wait:
            responses = await asyncio.get_event_loop().run_in_executor(
                None, mailbox.receive, 50
            )
            for msg in responses:
                try:
                    body = json.loads(msg.get("body", "{}"))
                    if body.get("round_id") == round_id:
                        agent = msg.get("from_agent", "unknown")
                        round_data["responses"][agent] = body.get("proposal", body.get("response", ""))
                except (json.JSONDecodeError, KeyError):
                    pass

            if len(round_data["responses"]) >= len(round_data["sent_to"]):
                break

            if wait:
                await asyncio.sleep(1)

        # Also check the ultrawork missions for completion
        loop = get_ultrawork_loop()
        missions = loop.list_missions()
        for m in missions:
            if m.get("objective", "").startswith(f"[COORD-{round_id}]"):
                if m.get("phase") == "completed":
                    round_data["responses"][m.get("assigned_agent", "unknown")] = \
                        m.get("verification_result", "")

        return {
            "round_id": round_id,
            "task": round_data["task"],
            "responses": round_data["responses"],
            "response_count": len(round_data["responses"]),
            "agents_sent": len(round_data["sent_to"]),
            "elapsed": round(time.time() - round_data["started_at"], 1),
        }

    async def consensus_aggregate(self, responses: dict[str, str],
                                    method: str = "majority") -> dict:
        """
        Aggregate agent responses into a consensus result.
        Methods: majority, summary, all.
        """
        if not responses:
            return {"consensus": None, "method": method, "count": 0}

        if method == "majority":
            from collections import Counter
            counts = Counter(responses.values())
            most_common = counts.most_common(1)
            return {
                "consensus": most_common[0][0] if most_common else None,
                "method": "majority",
                "votes": dict(counts),
                "total": len(responses),
            }

        elif method == "all":
            return {
                "consensus": list(responses.values()),
                "method": "all",
                "total": len(responses),
            }

        elif method == "summary":
            combined = "\n".join(f"- {a}: {r}" for a, r in responses.items())
            return {
                "consensus": f"Agent responses:\n{combined}",
                "method": "summary",
                "total": len(responses),
            }

        return {"consensus": None, "error": f"Unknown method: {method}"}

    async def run_coordination_round(self, task: str, agents: list[str] = None,
                                       context: str = "", method: str = "majority",
                                       timeout: int = 30) -> dict:
        """Full round: broadcast → collect → aggregate."""
        broadcast = await self.broadcast_task(task, agents, context, timeout)
        responses = await self.collect_responses(broadcast["round_id"],
                                                  max_wait=timeout)
        consensus = await self.consensus_aggregate(responses.get("responses", {}),
                                                    method)
        return {
            "round_id": broadcast["round_id"],
            "task": task,
            "broadcast": broadcast,
            "responses": responses,
            "consensus": consensus,
        }


_instance = None
def get_coordination_protocol():
    global _instance
    if _instance is None:
        _instance = CoordinationProtocol()
    return _instance

"""Routes for all untapped/wired TRM core modules.
Exposes PBFT consensus, Digital DNA, OpenClaw, Zero-Human Test Gen,
QISA optimizer, MCP Tool Evolver, Manus Engine, Lobster Shell,
Moltbook, and Resonance Engine via live API endpoints.
"""
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
from datetime import datetime

router = APIRouter()

# ─── 1. PBFT Consensus ───────────────────────────────────
@router.get("/consensus/status")
async def get_consensus_status():
    from swarm_v2.core.pbft_consensus import PBFTConsensusLayer
    layer = PBFTConsensusLayer()
    return {
        "protocol": "PBFT (Practical Byzantine Fault Tolerance)",
        "phase": layer.phase.value if hasattr(layer, 'phase') else 'unknown',
        "nodes": len(layer._nodes) if hasattr(layer, '_nodes') else 0,
        "description": "Enterprise-grade vote integrity for agent consensus",
    }

@router.post("/consensus/propose")
async def propose_consensus(proposal: dict):
    from swarm_v2.core.pbft_consensus import PBFTConsensusLayer
    layer = PBFTConsensusLayer()
    result = layer.propose(proposal.get("value", ""), proposal.get("sender", "anonymous"))
    return {"status": "proposed", "proposal_id": result.proposal_id if hasattr(result, 'proposal_id') else None}

# ─── 2. Digital DNA Loop ─────────────────────────────────
@router.get("/dna/profiles")
async def get_dna_profiles():
    from swarm_v2.core.digital_dna_loop import DNALoop
    loop = DNALoop()
    return {"profiles": loop.get_profiles() if hasattr(loop, 'get_profiles') else [], "description": "Agent behavioral DNA profiles"}

@router.post("/dna/mutate")
async def mutate_dna(profile_id: str, mutation: str = "random"):
    from swarm_v2.core.digital_dna_loop import DNALoop
    loop = DNALoop()
    result = loop.mutate(profile_id, mutation) if hasattr(loop, 'mutate') else None
    return {"status": "mutated", "profile_id": profile_id, "mutation": mutation}

# ─── 3. OpenClaw Gateway ─────────────────────────────────
@router.get("/openclaw/channels")
async def get_openclaw_channels():
    from swarm_v2.core.openclaw_gateway import OpenClawGateway
    gateway = OpenClawGateway()
    channels = gateway.list_channels() if hasattr(gateway, 'list_channels') else []
    return {"channels": channels, "description": "External communication bridge channels"}

@router.post("/openclaw/send")
async def openclaw_send(channel: str, recipient: str, message: str):
    from swarm_v2.core.openclaw_gateway import OpenClawGateway
    gateway = OpenClawGateway()
    result = await gateway.send(channel, recipient, message) if hasattr(gateway, 'send') else None
    return {"status": "sent", "channel": channel, "recipient": recipient}

# ─── 4. Zero-Human Test Generator ────────────────────────
@router.post("/tests/generate-all")
async def generate_all_tests(tool_name: str, tool_data: dict = {}):
    from swarm_v2.core.zero_human_test_gen import ZeroHumanTestGen
    gen = ZeroHumanTestGen()
    suites = gen.generate_all_tests(tool_name, tool_data)
    return {
        "suites": [s.to_dict() if hasattr(s, 'to_dict') else str(s) for s in suites],
        "count": len(suites),
        "description": "Automated test suite generation (zero human intervention)",
    }

# ─── 5. QISA Optimizer ───────────────────────────────────
class QISAOptimizeRequest(BaseModel):
    scores: List[float]
    iterations: int = 100

@router.post("/qisa/optimize")
async def qisa_optimize(req: QISAOptimizeRequest):
    from swarm_v2.core.qisa_optimizer import QISAOptimizer
    optimizer = QISAOptimizer()
    result = optimizer.optimize(req.scores, iterations=req.iterations)
    return {
        "result": result.to_dict() if hasattr(result, 'to_dict') else str(result),
        "description": "Quantum-Inspired Simulated Annealing optimization",
    }

# ─── 6. MCP Tool Evolver ────────────────────────────────
@router.get("/evolver/status")
async def get_evolver_status():
    from swarm_v2.core.mcp_tool_evolver import MCPToolEvolver
    evolver = MCPToolEvolver()
    return {"status": "active", "description": "MCP tool evolution and refinement engine"}

# ─── 7. Manus Engine (Superposition Processing) ─────────
class ManusSuperpositionRequest(BaseModel):
    task_name: str
    task_description: str
    agent_roles: List[str] = []

class ManusCollapseRequest(BaseModel):
    task_id: str

@router.post("/manus/superposition")
async def manus_create_superposition(req: ManusSuperpositionRequest):
    from swarm_v2.core.manus_engine import get_manus_engine
    engine = get_manus_engine()
    task_id = await engine.create_superposition(req.task_name, req.task_description, req.agent_roles)
    return {"status": "superposition_created", "task_id": task_id}

@router.post("/manus/collapse")
async def manus_collapse_state(req: ManusCollapseRequest):
    from swarm_v2.core.manus_engine import get_manus_engine
    engine = get_manus_engine()
    result = await engine.collapse_state(req.task_id)
    return {"status": "collapsed", "result": result}

# ─── 8. Moltbook (Knowledge Query) ──────────────────────
@router.post("/moltbook/query")
async def moltbook_post_query(author: str, question: str, tags: List[str] = []):
    from swarm_v2.core.moltbook import MoltbookNetwork
    network = MoltbookNetwork()
    query = network.post_query(author, question, tags)
    return {"status": "posted", "query_id": query.query_id if hasattr(query, 'query_id') else None}

@router.get("/moltbook/queries")
async def moltbook_list_queries():
    from swarm_v2.core.moltbook import MoltbookNetwork
    network = MoltbookNetwork()
    queries = network._queries if hasattr(network, '_queries') else []
    return {"queries": [{"id": q.query_id, "author": q.author, "question": q.question} for q in queries]}

# ─── 9. Resonance Engine ────────────────────────────────
@router.get("/resonance/status")
async def get_resonance_status():
    from swarm_v2.core.resonance_engine import get_resonance_engine
    engine = get_resonance_engine()
    return {
        "emotional_resonance": engine.get_emotional_resonance_index(),
        "self_awareness": engine.recursive_self_awareness_check(),
    }

@router.post("/resonance/broadcast")
async def resonance_broadcast(sender: str, message: str):
    from swarm_v2.core.resonance_engine import get_resonance_engine
    engine = get_resonance_engine()
    engine.broadcast_dream(sender, message)
    return {"status": "broadcast", "sender": sender}

# ─── 10. Lobster Shell (Approval Pipeline) ──────────────
@router.post("/lobster/execute")
async def lobster_execute(command: str, cwd: str = "."):
    from swarm_v2.core.lobster_shell import LobsterShell
    allowed, result = LobsterShell.audit_and_execute(command, cwd)
    return {"allowed": allowed, "result": result[:500] if result else ""}

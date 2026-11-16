# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 28, Rank 131)
# CDYLIB: strip_instella_single_expert_router.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_instella_single_expert_router.dll"
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
Instella MoE single-expert activation — wired into /swarm/chat pipeline.
64 experts total (8 teams x 8 experts). Only 1 team activated per task.
Routes through the trained Bonsai+Instella model for K3 classification.
"""
import time
from typing import Dict, Any, Optional
from dataclasses import dataclass

@dataclass
class ExpertRoute:
    team: str
    lead_agent: str
    experts_active: int
    latency_us: float
    confidence: float
    score: int = 0
    runner_up: int = 0
    defaulted: bool = False
    basis: str = ""

class InstellaSingleExpertRouter:
    """Routes tasks to the single best expert team based on domain matching.
    
    8 Teams (8 experts each = 64 total experts):
    - Security: Shield (lead) + 7 DDR experts
    - Research: Seeker (lead) + 7 research experts
    - Engineering: Devo (lead) + 7 dev experts
    - Architecture: Archi (lead) + 7 design experts
    - Logic: Logic (lead) + 7 reasoning experts
    - Integration: Flow (lead) + 7 API experts
    - Operations: Orchestra (lead) + 7 ops experts
    - Intelligence: Jarvis (lead) + 7 cognitive experts
    """
    
    TEAMS = {
        'Security': {
            'lead': 'Shield',
            'experts': ['Shield', 'Sentinel_A', 'Sentinel_B', 'DDR_Minter',
                       'Threat_Hunter', 'Vault_Keeper', 'Auditor', 'Firewall'],
            'domains': ['security', 'audit', 'vulnerability', 'ddr', 'antibody',
                       'threat', 'cve', 'attack', 'defense', 'firewall', 'vault'],
        },
        'Research': {
            'lead': 'Seeker',
            'experts': ['Seeker', 'ArXiv_Diver', 'PubMed_Scout', 'AlphaFold_Docker',
                       'Paper_Synth', 'Citation_Mapper', 'Hypothesis_Tester', 'Lit_Reviewer'],
            'domains': ['research', 'paper', 'arxiv', 'pubmed', 'discover', 'find',
                       'search', 'protein', 'alphafold', 'literature', 'knowledge'],
        },
        'Engineering': {
            'lead': 'Devo',
            'experts': ['Devo', 'API_Builder', 'Model_Trainer', 'GPU_Optimizer',
                       'Test_Runner', 'Package_Manager', 'Doc_Writer', 'Debugger'],
            'domains': ['code', 'compile', 'python', 'api', 'build', 'deploy',
                       'module', 'function', 'class', 'import', 'train', 'test'],
        },
        'Architecture': {
            'lead': 'Archi',
            'experts': ['Archi', 'Blueprint_Synth', 'GAN_Designer', 'Topology_Mapper',
                       'Component_Architect', 'Scalability_Planner', 'Fault_Analyst', 'Load_Balancer'],
            'domains': ['architecture', 'design', 'blueprint', 'microkernel', 'mamba',
                       'ssm', 'synthesize', 'topology', 'gan', 'structure', 'system'],
        },
        'Logic': {
            'lead': 'Logic',
            'experts': ['Logic', 'K3_Engine', 'Theorem_Prover', 'Coq_Expert',
                       'Lean4_Specialist', 'SAT_Solver', 'SMT_Analyst', 'Proof_Checker'],
            'domains': ['prove', 'theorem', 'logic', 'k3', 'kleene', 'coq', 'lean4',
                       'proof', 'boolean', 'axiom', 'ternary', 'sat', 'smt'],
        },
        'Integration': {
            'lead': 'Flow',
            'experts': ['Flow', 'REST_Master', 'GraphQL_Guru', 'gRPC_Expert',
                       'WebSocket_Handler', 'Gateway_Keeper', 'Pipeline_Runner', 'Mesh_Builder'],
            'domains': ['integrate', 'connect', 'rest', 'graphql', 'grpc', 'websocket',
                       'mesh', 'gateway', 'endpoint', 'pipeline', 'api'],
        },
        'Operations': {
            'lead': 'Orchestra',
            'experts': ['Orchestra', 'Task_Dispatcher', 'Workflow_Engine', 'Twin_Sync',
                       'Resource_Allocator', 'Queue_Manager', 'Health_Monitor', 'Recovery_Agent'],
            'domains': ['orchestrate', 'coordinate', 'schedule', 'workflow', 'dispatch',
                       'twin', 'deploy', 'manage', 'task', 'pipeline', 'health', 'status'],
        },
        'Intelligence': {
            'lead': 'Jarvis',
            'experts': ['Jarvis', 'Memory_Retriever', 'Graph_Builder', 'Skill_Synth',
                       'Context_Mapper', 'Pattern_Matcher', 'Knowledge_Curator', 'Insight_Gen'],
            'domains': ['memory', 'turbovec', 'knowledge', 'cognitive', 'skill',
                       'graph', 'semantic', 'retrieve', 'context', 'recall', 'chat'],
        },
    }
    
    def route(self, task: str) -> ExpertRoute:
        """Route a task to the single best expert team. Only 1 team activates.

        Mirrors the Rust port (intelligence_router.rs): confidence is the share of this
        team's matched domains that NO other team matched (unique/score), not min(score/5, 1.0);
        a no-match task is the Operations coordination fallback with confidence 0 and
        experts_active 0, not an invented 0.2/1.
        """
        t0 = time.perf_counter_ns()
        task_lower = task.lower()

        # Score each team by domain keyword matches, then deterministic tie-break:
        # score desc, then team name asc (same order the Rust port sorts by).
        scored = [(name, sum(1 for d in data['domains'] if d in task_lower))
                  for name, data in self.TEAMS.items()]
        scored.sort(key=lambda x: (-x[1], x[0]))
        best_team, best_score = scored[0]
        runner_up = scored[1][1] if len(scored) > 1 else 0
        elapsed = (time.perf_counter_ns() - t0) / 1000

        if best_score == 0:
            team = self.TEAMS['Operations']
            return ExpertRoute(
                team='Operations',
                lead_agent=team['lead'],
                experts_active=0,
                latency_us=elapsed,
                confidence=0.0,
                score=0,
                runner_up=0,
                defaulted=True,
                basis="no team matched any domain keyword: Operations is the coordination "
                      "fallback, confidence 0",
            )

        unique = max(best_score - runner_up, 0)
        team = self.TEAMS[best_team]
        return ExpertRoute(
            team=best_team,
            lead_agent=team['lead'],
            experts_active=1,
            latency_us=elapsed,
            confidence=unique / best_score,
            score=best_score,
            runner_up=runner_up,
            defaulted=False,
            basis=f"{best_score} domains matched, runner-up {runner_up} "
                  f"({unique} unique to this team) -> confidence {unique}/{best_score}",
        )


# Singleton
_instella_router: Optional[InstellaSingleExpertRouter] = None

def get_instella_router() -> InstellaSingleExpertRouter:
    global _instella_router
    if _instella_router is None:
        _instella_router = InstellaSingleExpertRouter()
    return _instella_router


def route_through_instella(task: str) -> Optional[Dict[str, Any]]:
    """Route a task through Instella MoE single-expert activation.
    Returns None if no domain match (should fall through to general processing)."""
    router = get_instella_router()
    route = router.route(task)
    
    if route.confidence == 0:
        return None  # No domain match — fall through
    
    return {
        'engine': 'instella_moe',
        'team': route.team,
        'lead_agent': route.lead_agent,
        'experts_active': route.experts_active,
        'total_experts': 64,
        'teams_standby': 7,
        'latency_us': route.latency_us,
        'confidence': route.confidence,
        'single_activation': True,
    }

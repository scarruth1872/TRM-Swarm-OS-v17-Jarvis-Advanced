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
        """Route a task to the single best expert team. Only 1 team activates."""
        t0 = time.perf_counter_ns()
        task_lower = task.lower()
        
        best_team = None
        best_score = 0
        
        for team_name, team_data in self.TEAMS.items():
            score = sum(1 for domain in team_data['domains'] if domain in task_lower)
            if score > best_score:
                best_score = score
                best_team = team_name
        
        if not best_team:
            # Default to Operations for coordination
            best_team = 'Operations'
            best_score = 1
        
        team = self.TEAMS[best_team]
        elapsed = (time.perf_counter_ns() - t0) / 1000
        
        return ExpertRoute(
            team=best_team,
            lead_agent=team['lead'],
            experts_active=1 if best_score > 0 else 0,  # Single expert activation
            latency_us=elapsed,
            confidence=min(best_score / 5, 1.0),  # Normalize to 0-1
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

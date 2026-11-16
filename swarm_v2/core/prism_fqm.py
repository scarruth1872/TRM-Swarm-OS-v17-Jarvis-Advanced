"""
PRISM-FQM Fractal Quantum Mycorrhizal Telemetry Engine.
Maps real TRM agent mesh metrics to FTL/QSI visualizations using fractal mathematics.
"""
import math, json, os
from datetime import datetime
from typing import Optional


class PrismFQMEngine:
    """Computes fractal telemetry from live agent mesh data."""

    def __init__(self):
        self._history: list[dict] = []

    async def compute_telemetry(self, mesh_data: dict) -> dict:
        """
        Take live agent mesh topology and compute PRISM-FQM metrics.
        
        Args:
            mesh_data: Dict from /mesh/topology with nodes list
            
        Returns:
            Full FTL + QSI telemetry packet
        """
        nodes = mesh_data.get("nodes", [])
        online = [n for n in nodes if n.get("status") == "online"]
        total = len(nodes)
        online_count = len(online)

        # ─── FTL: Fractal Telemetry Language ───────────────────
        # Recursion Depth = agent mesh layers (min 1, max based on cluster count)
        clusters = self._detect_clusters(online)
        recursion_depth = max(2, min(12, len(clusters) + int(math.log2(online_count + 1))))

        # Load Dimension = Sierpinski dimension scaled by mesh coherence
        coherence = online_count / max(1, total)
        load_dimension = round(1.0 + (coherence * 0.58), 2)  # Maps 1.0-1.58

        # FTL Progress = mesh fill rate (what % of connections are active)
        max_connections = total * (total - 1) / 2
        actual_connections = online_count * (online_count - 1) / 2 if online_count > 1 else 0
        ftl_progress = min(100, int((actual_connections / max(1, max_connections)) * 100))

        # ─── QSI: Quantum Swarm Intelligence ───────────────────
        # Superposition States = qubits per agent × agent count
        qubits_per_agent = 8  # Each agent has 8 inherent quantum states
        superposition_states = max(2, online_count * qubits_per_agent)

        # Grover Speedup = sqrt(N) where N is the search space
        # The search space is the number of possible agent-task assignments
        search_space = total * max(1, self._count_recent_tasks())
        grover_speedup = round(math.sqrt(search_space), 1)

        # QSI Progress = percent of agents actively processing
        qsi_progress = min(100, int(coherence * 100))

        # ─── Mycorrhizal Tunneling ────────────────────────────
        # Beta = average health score (bounded)
        health_scores = [n.get("health_score", 0.85) for n in online]
        avg_health = sum(health_scores) / max(1, len(health_scores))
        beta = max(0.1, min(1.0, avg_health))

        # Delta_E = energy barrier = variance in health scores
        if len(health_scores) > 1:
            variance = sum((h - avg_health) ** 2 for h in health_scores) / len(health_scores)
            delta_e = max(0.01, min(1.0, variance * 2))
        else:
            delta_e = 0.5

        # Tunneling Probability = exp(-beta * delta_E)
        tunneling_prob = round(math.exp(-beta * delta_e), 4)

        # ─── Assemble telemetry ────────────────────────────────
        telemetry = {
            "timestamp": datetime.now().isoformat(),
            "ftl": {
                "recursion_depth": recursion_depth,
                "load_dimension": load_dimension,
                "progress": ftl_progress,
                "label": f"Depth = {recursion_depth} (L-System)",
                "dimension_label": f"D_H = {load_dimension} (Sierpinski)",
            },
            "qsi": {
                "superposition_states": superposition_states,
                "grover_speedup": grover_speedup,
                "progress": qsi_progress,
                "states_label": f"{superposition_states} Qubits",
                "speedup_label": f"{grover_speedup}x Parallel",
            },
            "mycorrhizal": {
                "tunneling_probability": tunneling_prob,
                "beta": round(beta, 3),
                "delta_e": round(delta_e, 3),
                "formula": "Tunneling: exp(-β·ΔE)",
                "attractor": f"z(n+1) = z(n)² + c | β={beta:.2f}, ΔE={delta_e:.2f}",
            },
            "raw_mesh": {
                "total_agents": total,
                "online_agents": online_count,
                "coherence": round(coherence, 3),
                "clusters": len(clusters),
                "avg_health": round(avg_health, 3),
            },
        }

        self._history.append(telemetry)
        return telemetry

    def _detect_clusters(self, nodes: list) -> list:
        """Simple cluster detection based on agent roles."""
        clusters = {}
        for n in nodes:
            role = n.get("role", "unknown")
            # Cluster by role category
            category = role.split(" ")[0] if " " in role else role
            clusters[category] = clusters.get(category, 0) + 1
        return list(clusters.values())

    def _count_recent_tasks(self) -> int:
        """Return approximate number of recent tasks from the system."""
        import glob
        task_files = []
        base = r"F:\Development sites\TRM-Swarm-OS-v2\artifacts"
        if os.path.exists(base):
            task_files = glob.glob(os.path.join(base, "*.json")) + \
                         glob.glob(os.path.join(base, "*.md"))
        return max(1, len(task_files) // 10)


_engine_instance = None
def get_prism_fqm_engine():
    global _engine_instance
    if _engine_instance is None:
        _engine_instance = PrismFQMEngine()
    return _engine_instance

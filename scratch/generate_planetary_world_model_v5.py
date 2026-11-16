#!/usr/bin/env python3
"""
Planetary World Model v5 — Autonomous Generation Pipeline
Runs K3 Kleene Ternary analysis on 7 domains + Instella MoE Expert Routing.
Produces swarm_v2_artifacts/planetary_world_model_v5.json (appends timestamped snapshots).
"""

import json, hashlib, time, sys
from pathlib import Path
from typing import Dict, Any, List, Optional

# Add project root
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from swarm_v2.core.ternary_kleene_engine import (
    KleeneTernaryEngine, TRUE, FALSE, UNKNOWN, get_kleene_engine
)
from swarm_v2.core.instella_single_expert_router import get_instella_router

# ============================================================
# 7 DOMAINS with 3 scenarios each = 21 total K3 analyses
# ============================================================
DOMAIN_SCENARIOS = {
    "climate": [
        {
            "agent": "Seeker",
            "scenario": "Model Antarctic ice sheet collapse under 3.2°C warming with Bayesian sea-level projection to 2150",
            "k3_vector": [0.92, 0.88, 0.95, 0.81, 0.88, 0.84, 0.91, 0.79],
        },
        {
            "agent": "Seeker",
            "scenario": "Analyze Amazon rainforest dieback threshold at 25% deforestation and global carbon cycle feedback loops",
            "k3_vector": [0.85, 0.78, 0.90, 0.82, 0.77, 0.87, 0.83, 0.93],
        },
        {
            "agent": "Archi",
            "scenario": "Design autonomous reforestation drone network optimized for 50 billion trees/year across degraded biomes",
            "k3_vector": [0.78, 0.82, 0.91, 0.88, 0.83, 0.75, 0.96, 0.86],
        },
    ],
    "energy": [
        {
            "agent": "Archi",
            "scenario": "Architect planetary fusion-solar-wind grid with Mamba-3 SSM load balancing across 7 continents",
            "k3_vector": [0.94, 0.88, 0.89, 0.85, 0.92, 0.86, 0.78, 0.87],
        },
        {
            "agent": "Pulse",
            "scenario": "Simulate terawatt-hour energy storage requirements for 100% renewable grid serving 10 billion population",
            "k3_vector": [0.81, 0.85, 0.88, 0.82, 0.84, 0.80, 0.76, 0.91],
        },
        {
            "agent": "Devo",
            "scenario": "Deploy microkernel-optimized distributed training for real-time energy market prediction across 190 nations",
            "k3_vector": [0.73, 0.80, 0.86, 0.78, 0.79, 0.82, 0.75, 0.70],
        },
    ],
    "health": [
        {
            "agent": "Seeker",
            "scenario": "Model pandemic spread using AlphaFold-predicted variant structures and genomic wastewater surveillance across 500 cities",
            "k3_vector": [0.96, 0.82, 0.93, 0.85, 0.89, 0.78, 0.91, 0.88],
        },
        {
            "agent": "Logic",
            "scenario": "Apply hybrid SAT+SMT solving to protein folding prediction for 20 million emerging viral sequences",
            "k3_vector": [0.88, 0.85, 0.95, 0.82, 0.83, 0.78, 0.97, 0.84],
        },
        {
            "agent": "Jarvis",
            "scenario": "Build 8-billion-node knowledge graph connecting genomic markers, drug interactions, and clinical outcomes globally",
            "k3_vector": [0.82, 0.84, 0.90, 0.88, 0.87, 0.85, 0.76, 0.93],
        },
    ],
    "security": [
        {
            "agent": "Shield",
            "scenario": "Audit global submarine cable network vulnerabilities and design 1,801 DDR antibody signatures for autonomous defense",
            "k3_vector": [0.97, 0.88, 0.94, 0.86, 0.91, 0.84, 0.93, 0.82],
        },
        {
            "agent": "Shield",
            "scenario": "Simulate APT-level coordinated attack on planetary power grid and generate real-time zero-day antibody response",
            "k3_vector": [0.99, 0.90, 0.96, 0.88, 0.92, 0.86, 0.95, 0.84],
        },
        {
            "agent": "Verify",
            "scenario": "Formally verify 15-agent swarm communication protocol against Byzantine fault models at planetary scale",
            "k3_vector": [0.89, 0.84, 0.94, 0.86, 0.86, 0.82, 0.92, 0.88],
        },
    ],
    "space": [
        {
            "agent": "Seeker",
            "scenario": "Calculate Mars terraforming requirements — atmosphere thickness, temperature equilibria, water cycle with 50-year timeline",
            "k3_vector": [0.76, 0.72, 0.88, 0.74, 0.82, 0.70, 0.85, 0.73],
        },
        {
            "agent": "Bridge",
            "scenario": "Design interplanetary mesh network protocol with erasure coding for 22-minute Earth-Mars round-trip latency",
            "k3_vector": [0.84, 0.82, 0.91, 0.85, 0.87, 0.82, 0.93, 0.83],
        },
        {
            "agent": "THzGateway",
            "scenario": "Deploy terahertz communication relay constellation for continuous low-orbit satellite mesh across cis-lunar space",
            "k3_vector": [0.80, 0.82, 0.89, 0.85, 0.83, 0.84, 0.91, 0.86],
        },
    ],
    "governance": [
        {
            "agent": "Orchestra",
            "scenario": "Model autonomous AI governance framework for planetary resource allocation during compound climate-energy crises",
            "k3_vector": [0.74, 0.78, 0.83, 0.82, 0.78, 0.80, 0.72, 0.76],
        },
        {
            "agent": "Jarvis",
            "scenario": "Construct TurboVec planetary knowledge graph indexing 8 billion human experiences for equitable disaster response coordination",
            "k3_vector": [0.70, 0.82, 0.80, 0.84, 0.75, 0.82, 0.78, 0.73],
        },
        {
            "agent": "Flow",
            "scenario": "Design federated API mesh for cross-sovereign AI governance with OAuth2 token rotation and localized edge enforcement",
            "k3_vector": [0.68, 0.78, 0.77, 0.82, 0.72, 0.80, 0.76, 0.70],
        },
    ],
    "infrastructure": [
        {
            "agent": "EventLedger",
            "scenario": "Deploy planetary OpenTelemetry pipeline monitoring 10 billion infrastructure endpoints across 190 countries",
            "k3_vector": [0.86, 0.85, 0.92, 0.82, 0.84, 0.86, 0.89, 0.83],
        },
        {
            "agent": "Pulse",
            "scenario": "Implement HVBE voltage-calibrated telemetry for predictive self-healing across continental power-water-transport grids",
            "k3_vector": [0.90, 0.86, 0.94, 0.84, 0.88, 0.85, 0.92, 0.82],
        },
        {
            "agent": "Orchestra",
            "scenario": "Orchestrate 15-agent swarm for coordinated global earthquake-tsunami-wildfire response across 6 continents simultaneously",
            "k3_vector": [0.79, 0.84, 0.87, 0.82, 0.82, 0.84, 0.78, 0.90],
        },
    ],
}

CROSS_CUTTING_SCENARIOS = {
    "climate_energy_nexus": {
        "agent": "Seeker",
        "scenario": "Model coupled climate-energy transition dynamics — carbon budget vs. renewable deployment rate to stay under 2°C",
        "k3_vector": [0.87, 0.82, 0.92, 0.85, 0.85, 0.80, 0.90, 0.84],
    },
    "health_security_convergence": {
        "agent": "Shield",
        "scenario": "Design integrated bio-cyber defense architecture protecting genomic databases AND critical care infrastructure simultaneously",
        "k3_vector": [0.93, 0.86, 0.91, 0.84, 0.88, 0.82, 0.94, 0.86],
    },
    "space_governance_framework": {
        "agent": "Orchestra",
        "scenario": "Draft autonomous orbital resource allocation treaty — managing satellite slots, lunar mining rights, and debris mitigation",
        "k3_vector": [0.72, 0.80, 0.80, 0.82, 0.76, 0.82, 0.78, 0.75],
    },
}


def run_k3_analysis(engine: KleeneTernaryEngine, scenario_data: dict) -> dict:
    """Run K3 Kleene ternary analysis on a scenario."""
    vector = scenario_data["k3_vector"]
    psi = engine.quick_reduction(vector)
    state = engine.bypass_decision(psi)
    offload = engine.evaluate_offload(psi)
    return {
        "trit": state.trit,
        "trit_label": "PROCEED" if state.trit == TRUE else ("REJECT" if state.trit == FALSE else "UNKNOWN"),
        "confidence": round(state.confidence, 4),
        "ambiguity_ratio": round(state.ambiguity_ratio, 4),
        "is_bypassable": state.is_bypassable,
        "offload_target": offload,
        "psi_sample": psi[:8],
    }


def run_instella_routing(router, scenario_text: str) -> dict:
    """Route scenario through Instella MoE single-expert activation."""
    route = router.route(scenario_text)
    return {
        "team": route.team,
        "lead_agent": route.lead_agent,
        "experts_active": route.experts_active,
        "latency_us": round(route.latency_us, 3),
        "confidence": round(route.confidence, 4),
    }


def match_k3_instella(k3_agent: str, instella_lead: str) -> dict:
    """Determine if K3 agent and Instella lead match, using team-structure semantics."""
    # Direct identity matches
    if k3_agent == instella_lead:
        return {"match": True, "status": "DIRECT MATCH"}
    
    # Agent-to-team-lead semantic mappings based on swarm architecture:
    #   Shield -> Security, Seeker -> Research, Devo -> Engineering,
    #   Archi -> Architecture, Logic -> Logic, Verify -> Engineering(devo),
    #   Flow -> Integration, Bridge -> Integration(flow), Orchestra -> Operations,
    #   EventLedger -> Architecture(archi), Pulse -> Architecture(archi),
    #   THzGateway -> Architecture(archi), Jarvis -> Intelligence,
    #   LogicMaster -> Logic, Hermes -> Integration(flow)
    agent_to_team_lead = {
        "Shield": "Shield",
        "Seeker": "Seeker",
        "Devo": "Devo",
        "Archi": "Archi",
        "Logic": "Logic",
        "LogicMaster": "Logic",
        "Verify": "Devo",
        "Flow": "Flow",
        "Bridge": "Flow",
        "Hermes": "Flow",
        "Orchestra": "Orchestra",
        "EventLedger": "Archi",
        "Pulse": "Archi",
        "THzGateway": "Archi",
        "Jarvis": "Jarvis",
    }
    
    expected_lead = agent_to_team_lead.get(k3_agent, k3_agent)
    
    if expected_lead == instella_lead:
        return {"match": True, "status": f"SEMANTIC MATCH ({k3_agent}→{instella_lead})"}
    
    # Also check: is the instella lead in the same domain domain as the K3 agent?
    # (e.g., Pulse→Archi is correct even if Instella routed to Orchestra for ops context)
    return {"match": False, "status": f"MISMATCH ({instella_lead} vs expected {expected_lead})"}


def main():
    engine = get_kleene_engine()
    router = get_instella_router()
    
    timestamp = time.strftime("%Y-%m-%dT%H:%M:%S")
    all_results = []
    
    total_scenarios = 0
    k3_proceed_count = 0
    instella_match_count = 0
    
    # Process 7 primary domains
    for domain, scenarios in DOMAIN_SCENARIOS.items():
        for i, sc in enumerate(scenarios):
            total_scenarios += 1
            
            # K3 Analysis
            k3 = run_k3_analysis(engine, sc)
            if k3["trit"] == TRUE:
                k3_proceed_count += 1
            
            # Instella MoE Routing
            ie = run_instella_routing(router, sc["scenario"])
            
            # Match analysis
            match = match_k3_instella(sc["agent"], ie["lead_agent"])
            if match["match"]:
                instella_match_count += 1
            
            all_results.append({
                "domain": domain,
                "scenario_index": i + 1,
                "agent": sc["agent"],
                "scenario": sc["scenario"],
                "k3": k3["trit_label"],
                "k3_confidence": k3["confidence"],
                "k3_ambiguity": k3["ambiguity_ratio"],
                "k3_bypassable": k3["is_bypassable"],
                "k3_offload": k3["offload_target"],
                "instella_team": ie["team"],
                "instella_lead": ie["lead_agent"],
                "instella_confidence": ie["confidence"],
                "instella_latency_us": ie["latency_us"],
                "k3_instella_match": match["status"],
            })
    
    # Process cross-cutting scenarios
    for xdomain, sc in CROSS_CUTTING_SCENARIOS.items():
        total_scenarios += 1
        
        k3 = run_k3_analysis(engine, sc)
        if k3["trit"] == TRUE:
            k3_proceed_count += 1
        
        ie = run_instella_routing(router, sc["scenario"])
        match = match_k3_instella(sc["agent"], ie["lead_agent"])
        if match["match"]:
            instella_match_count += 1
        
        all_results.append({
            "domain": xdomain,
            "scenario_index": 0,
            "agent": sc["agent"],
            "scenario": sc["scenario"],
            "k3": k3["trit_label"],
            "k3_confidence": k3["confidence"],
            "k3_ambiguity": k3["ambiguity_ratio"],
            "k3_bypassable": k3["is_bypassable"],
            "k3_offload": k3["offload_target"],
            "instella_team": ie["team"],
            "instella_lead": ie["lead_agent"],
            "instella_confidence": ie["confidence"],
            "instella_latency_us": ie["latency_us"],
            "k3_instella_match": match["status"],
            "cross_domain": True,
        })
    
    # Compute summary statistics
    k3_proceed_rate = f"{k3_proceed_count}/{total_scenarios}"
    instella_match_rate = f"{instella_match_count}/{total_scenarios}"
    avg_k3_confidence = round(
        sum(r["k3_confidence"] for r in all_results) / len(all_results), 4
    )
    avg_instella_confidence = round(
        sum(r["instella_confidence"] for r in all_results) / len(all_results), 4
    )
    
    # Per-domain breakdown
    domain_breakdown = {}
    for result in all_results:
        d = result["domain"]
        if d not in domain_breakdown:
            domain_breakdown[d] = {"scenarios": 0, "k3_proceed": 0, "instella_match": 0}
        domain_breakdown[d]["scenarios"] += 1
        if result["k3"] == "PROCEED":
            domain_breakdown[d]["k3_proceed"] += 1
        if not result["k3_instella_match"].startswith("MISMATCH"):
            domain_breakdown[d]["instella_match"] += 1
    
    # Build the snapshot
    snapshot = {
        "version": "v18.91",
        "timestamp": timestamp,
        "model_id": hashlib.sha256(timestamp.encode()).hexdigest()[:12],
        "summary": {
            "total_domains": 7,
            "primary_domains": list(DOMAIN_SCENARIOS.keys()),
            "cross_cutting_domains": list(CROSS_CUTTING_SCENARIOS.keys()),
            "total_scenarios_analyzed": total_scenarios,
            "k3_proceed_rate": k3_proceed_rate,
            "k3_proceed_pct": round(k3_proceed_count / total_scenarios * 100, 1),
            "instella_match_rate": instella_match_rate,
            "instella_match_pct": round(instella_match_count / total_scenarios * 100, 1),
            "avg_k3_confidence": avg_k3_confidence,
            "avg_instella_confidence": avg_instella_confidence,
            "instella_total_teams": 8,
            "instella_total_experts": 64,
            "instella_activation": "Single-expert (1/64 active per scenario)",
            "k3_engine": "Kleene Ternary Logic (TRUE/UNKNOWN/FALSE)",
            "engine_bypass_rate": round(engine.bypass_rate * 100, 1),
        },
        "domain_breakdown": {
            d: {
                "scenarios": db["scenarios"],
                "k3_proceed": f"{db['k3_proceed']}/{db['scenarios']}",
                "instella_match": f"{db['instella_match']}/{db['scenarios']}",
            }
            for d, db in domain_breakdown.items()
        },
        "scenarios": all_results,
        "agents_activated": list(set(r["agent"] for r in all_results)),
        "instella_teams_activated": list(set(r["instella_team"] for r in all_results)),
        "new_scenarios_added": [s["scenario"][:80] + "..." for s in all_results],
    }
    
    # Load existing V5 file if it exists, append snapshot
    v5_path = Path("swarm_v2_artifacts/planetary_world_model_v5.json")
    existing = {"history": []}
    if v5_path.exists():
        try:
            existing = json.loads(v5_path.read_text())
            if "history" not in existing:
                existing = {"history": [existing]}  # Wrap old format
        except (json.JSONDecodeError, KeyError):
            existing = {"history": []}
    
    existing["history"].append(snapshot)
    existing["latest_timestamp"] = timestamp
    existing["total_snapshots"] = len(existing["history"])
    
    v5_path.write_text(json.dumps(existing, indent=2))
    
    # Print report
    print(f"\n{'='*70}")
    print(f"  PLANETARY WORLD MODEL v5 — Autonomous Update Complete")
    print(f"{'='*70}")
    print(f"  Timestamp:      {timestamp}")
    print(f"  Total Scenarios:{total_scenarios}")
    print(f"  K3 PROCEED:     {k3_proceed_rate} ({round(k3_proceed_count/total_scenarios*100,1)}%)")
    print(f"  Instella Match: {instella_match_rate} ({round(instella_match_count/total_scenarios*100,1)}%)")
    print(f"  Avg K3 Conf:    {avg_k3_confidence}")
    print(f"  Avg IE Conf:    {avg_instella_confidence}")
    print(f"  Engine Bypass:  {round(engine.bypass_rate*100,1)}%")
    print(f"  Snapshots:      {len(existing['history'])}")
    print(f"\n  Domain Breakdown:")
    for d, db in domain_breakdown.items():
        m = "MATCH" if db['instella_match'] == db['scenarios'] else "MISMATCH"
        print(f"    {d:25s} | K3 {db['k3_proceed']}/{db['scenarios']} | IE {db['instella_match']}/{db['scenarios']} {m}")
    print(f"\n  Agents: {len(snapshot['agents_activated'])} | Teams: {len(snapshot['instella_teams_activated'])}")
    print(f"  Saved: {v5_path}")
    print(f"{'='*70}\n")
    
    return snapshot


if __name__ == "__main__":
    main()

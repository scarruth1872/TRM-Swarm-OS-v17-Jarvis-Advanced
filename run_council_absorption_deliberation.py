#!/usr/bin/env python3
"""
run_council_absorption_deliberation.py — TRM Swarm OS v19.001 "Sovereign"
==========================================================================
Submits and runs the architecture absorption proposal through the 14-Member Swarm Council:
Proposal: "Integration of Sparse MoE Top-K Routing & AVO Supervisor Process Trees"
- Proposer: Archi (Chief Systems Architect)
- Category: ARCHITECTURE
- Engine: Kleene K3 Weighted Ternary Voting + PBFT Consensus
"""

import os
import sys
import json
import time
from dataclasses import asdict

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

PROJECT_ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
sys.path.insert(0, PROJECT_ROOT)

from swarm_v2.core.swarm_council import SwarmCouncil

def run_council_deliberation():
    print("=" * 95)
    print(">>> 14-MEMBER SWARM COUNCIL DELIBERATION")
    print("    Topic: DeepSeek Sparse MoE Routing & AVO Supervisor Process Tree Architecture")
    print("    Engine: K3 Weighted Ternary Consensus + PBFT Invariant Proof")
    print("===============================================================================================")

    council = SwarmCouncil()

    # Step 1: Propose
    proposal = council.propose(
        topic="Implement Sparse MoE Top-K Dynamic Router & Hierarchical Supervisor Tree",
        category="architecture",
        description=(
            "Absorb open architectural patterns for sovereign deployment: "
            "1. DeepSeek-style Sparse MoE Router (Top-k dynamic expert gating, auxiliary load-balancing loss, O(1) active compute per token). "
            "2. AVO / Erlang-style Supervisor Trees (OneForOne / OneForAll restart strategies, persistent state checkpointing, heartbeat monitoring). "
            "Zero cloud or API dependencies; 100% native local Python & Rust implementation."
        ),
        proposer="Archi"
    )

    print(f"\n[1/4] Proposal Registered: ID {proposal.proposal_id}")
    print(f"      Topic: {proposal.topic}")
    print(f"      Proposer: {proposal.proposer} | Category: {proposal.category.upper()}")

    # Step 2: Deliberate
    deliberations = council.deliberate(proposal.proposal_id)
    print(f"\n[2/4] Deliberation Concluded across {len(deliberations)} Council Members:")
    for delib in deliberations:
        print(f"      * [{delib.agent_name} (ID {delib.agent_id})]:")
        print(f"        \"{delib.reasoning}\" (Relevance: {delib.domain_relevance:.2f}, Position: {delib.position.upper()})")

    # Step 3: Vote
    vote_res = council.vote(proposal.proposal_id)
    tally = vote_res["raw_tally"]
    print(f"\n[3/4] Voting Tally & Consensus Results:")
    print(f"      * Approvals (+1) : {tally['approve']} votes")
    print(f"      * Rejections (-1): {tally['reject']} votes")
    print(f"      * Abstentions (0): {tally['abstain']} votes")
    print(f"      * Weighted Score : {vote_res['weighted_score']:+.4f} (Consensus Threshold: >= +0.30)")
    print(f"      * Consensus Passed: {vote_res['consensus_reached']} ✅")
    print(f"      * Current Phase  : {vote_res['phase'].upper()}")

    # Step 4: Enact
    enacted_res = council.enact(proposal.proposal_id)
    print(f"\n[4/4] Proposal Enacted:")
    print(f"      * Enacted At     : {enacted_res['enacted_at']}")
    print(f"      * Monad Seal     : {enacted_res['monad_seal']}")

    # Save sealed record
    council_result = {
        "protocol": "COUNCIL_ARCHITECTURAL_APPROVAL_RECORD",
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "proposal_id": proposal.proposal_id,
        "topic": proposal.topic,
        "category": proposal.category,
        "proposer": proposal.proposer,
        "deliberations": [asdict(d) for d in deliberations],
        "vote_results": vote_res,
        "enacted_results": enacted_res
    }

    output_json = os.path.join(PROJECT_ROOT, "council_absorption_deliberation.json")
    with open(output_json, "w", encoding="utf-8") as f:
        json.dump(council_result, f, indent=2)

    print(f"\n  * Council Deliberation Ledger Sealed: council_absorption_deliberation.json")
    print("===============================================================================================")

if __name__ == "__main__":
    run_council_deliberation()

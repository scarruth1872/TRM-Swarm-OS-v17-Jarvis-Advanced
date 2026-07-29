"""
Script to poll all agents in the Swarm for their current needs and status.
"""
import asyncio
from swarm_v2.experts.registry import get_expert_team

async def ask_agents():
    team = get_expert_team()
    print("--- Polling Swarm Agents ---")
    
    tasks = []
    for role, agent in team.items():
        prompt = "We have just completed the Phase 10 Upgrade 2026 (including Quantum Crypto, Autonomous Economy, and Cross-Reality Orchestration). As the {role}, what do you need to operate at peak efficiency? Keep your response under 2 sentences."
        tasks.append(agent.process_task(prompt.format(role=role), sender="System"))
        
    responses = await asyncio.gather(*tasks)
    
    for (role, _), response in zip(team.items(), responses):
        print(f"\n[{role}]")
        if isinstance(response, dict):
            print(response.get("response", ""))
        else:
            print(response)

if __name__ == "__main__":
    asyncio.run(ask_agents())

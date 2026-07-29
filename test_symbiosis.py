import requests
import json

print("=== JARVIS ADVANCED & TRM SWARM OS SYMBIOSIS TEST ===")

# Step 1: Query Jarvis mind via Swarm Engine
from swarm_v2.core.jarvis_symbiosis import get_symbiosis_engine

engine = get_symbiosis_engine()
is_online = engine.check_jarvis_connection()
print(f"1. Jarvis Connection Check: {'ONLINE (Port 4000)' if is_online else 'OFFLINE'}")

# Step 2: Query Jarvis mind directly
mind_reply = engine.query_jarvis_mind("Greetings Jarvis! Swarm OS is initiating cross-cognitive symbiosis to synthesize novel skills across our 12 personas and your neural memory graph.")
print(f"\n2. Jarvis Cognitive Acknowledgment:\n   \"{mind_reply[:250]}...\"")

# Step 3: Run Symbiosis Cycle
print("\n3. Executing Cross-Cognitive Symbiosis Cycle...")
res = engine.run_symbiosis_cycle()

print(f"   Status: {res['status']}")
print(f"   Swarm Experts Involved: {res['swarm_experts_count']}")
print("   Newly Synthesized Novel Symbiotic Skills:")
for i, skill in enumerate(res['synthesized_skills'], 1):
    print(f"    [{i}] Skill: {skill['skill_name']} (Fidelity: {skill['fidelity']})")
    print(f"        Components: {' + '.join(skill['components'])}")
    print(f"        Description: {skill['description']}")

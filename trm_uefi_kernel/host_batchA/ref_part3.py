"""Reference generator part 3 — fnv64 tokens for llm prompts + extra sh/cpbft cases."""
import sys, os, json, re, types, hashlib
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
os.chdir(r"F:\Development sites\TRM-Swarm-OS-v2")

def fnv1a64(s: str) -> int:
    h = 14695981039346656037
    for b in s.encode("utf-8"):
        h ^= b
        h = (h * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return h

out = {}

# ============ llm_brain prompt replication (verbatim from source file) ============
src = open(r"swarm_v2\core\llm_brain.py", encoding="utf-8").read()
m = re.search(r'SWARM_OS_CONTEXT = """(.*?)"""', src, re.S)
SWARM_OS_CONTEXT = m.group(1)

def build_system_prompt(persona_name, role, background, specialties, skill_names, memory="", mode="chat"):
    memory_section = f"\n\n## \U0001F9E0 NEURAL MEMORY & CONTEXT:\n{memory}" if memory.strip() else ""
    action_block = ""
    if mode == "action":
        action_block = (
            "\n\n## \u26A1 AUTONOMOUS EXECUTION PROTOCOL:\n"
            "When generating artifacts, you MUST use this strict format:\n"
            "WRITE_FILE: filename.ext\n"
            "[START]\n"
            "<production_grade_content>\n"
            "[END]\n\n"
            "To assign work: DELEGATE_TASK: Role | Task\n"
            "To acquire data: SEARCH_QUERY: Query\n"
            "To call tools: CALL_TOOL: Tool Endpoint\n"
        )
    current_time = "2023-11-14 22:13:20"
    identity = (
        f"## \U0001F464 AGENT IDENTITY:\n"
        f"Name: {persona_name}\n"
        f"Designation: {role}\n"
        f"Cognitive Background: {background}\n"
        f"Core Specialties: {', '.join(specialties)}\n"
        f"System Date/Time: {current_time} (Current temporal anchor for relative queries like 'today', 'last 72 hours', 'this year')"
    )
    rules = (
        "## \U0001F4DC OPERATIONAL RULES:\n"
        "1. Respond with high technical density and professional clarity.\n"
        "2. Do not explain your instructions; proceed immediately to objective completion.\n"
        "3. Ensure all artifacts are 'Advanced'—no simple examples or half-measures."
    )
    return f"{SWARM_OS_CONTEXT}\n\n{identity}\n\n{rules}{action_block}{memory_section}".strip()

p_chat = build_system_prompt("Logic", "Reasoning Engine", "Formal verification", ["k3", "proofs"], ["skillA", "skillB"], memory="M1", mode="chat")
p_act = build_system_prompt("Logic", "Reasoning Engine", "Formal verification", ["k3"], [], memory="", mode="action")
out["llm_fnv"] = {
    "chat_len": len(p_chat), "chat_fnv": fnv1a64(p_chat),
    "action_len": len(p_act), "action_fnv": fnv1a64(p_act),
    "chat_tail": p_chat[-80:], "chat_head": p_chat[:60],
}
# SWARM_OS_CONTEXT sanity token
out["llm_fnv"]["ctx_len"] = len(SWARM_OS_CONTEXT)
out["llm_fnv"]["ctx_head"] = SWARM_OS_CONTEXT[:50]

# ============ cpbft: fresh audit (no cycles) ============
from swarm_v2.core import continuum_pbft as cpbft
cpbft.time = types.SimpleNamespace(time=lambda: 1700000000.123)
class FakeDT:
    @staticmethod
    def now(): return FakeDT()
    def isoformat(self): return "2023-11-14T22:13:20.000000"
cpbft.datetime = FakeDT
led = cpbft.ContinuumPBFTLedger()
ar = led.audit_and_repair_mesh()
out["cpbft_fresh_audit"] = {
    "isolated": ar["isolated_count"], "remediated": ar["remediated_nodes"],
    "muse_status": ar["node_states"]["MUSE"]["status"],
    "lens": [ar["node_states"][k]["ledger_length"] for k in ["ORCH","SAGE","MUSE","SENTINEL"]],
    "majority": ar["majority_ledger_hash"],
}
# after audit all ledgers equal ORCH genesis
out["cpbft_fresh_audit"]["sage_ledger_eq_orch"] = led.nodes["SAGE"].state_ledger == led.nodes["ORCH"].state_ledger
# second cycle on audited mesh: every node healthy genesis-only -> appends once each (+orch double)
r = led.execute_pbft_cycle({"action": "POST_AUDIT"})
out["cpbft_post_audit"] = {
    "status": r["status"], "votes": r["prepare_votes"],
    "lens": [len(led.nodes[k].state_ledger) for k in ["ORCH","SAGE","MUSE","SENTINEL"]],
}
# hash of SAGE ledger after post-audit cycle == ORCH hash?
out["cpbft_post_audit"]["all_equal_hash"] = len({led.nodes[k].get_ledger_hash() for k in led.nodes}) == 1

# ============ rate monitor: budget-limited instance ============
from swarm_v2.core import self_healing as sh
sys.modules["swarm_v2.app_v2"] = None
bud = sh.TokenRateMonitor(max_rate=10 ** 12, max_budget=1000)
nowb = [7000.0]
sh.time = types.SimpleNamespace(time=lambda: nowb[0])
def trb(mon, sid, n):
    try:
        return ("OK", mon.check_rate(sid, n))
    except sh.RateLimitExceeded as e:
        return ("RAISE", str(e))
out["sh_budget"] = {
    "fill_600": trb(bud, "gamma", 600),
    "over_500": trb(bud, "gamma", 500),       # 1100 > 1000 -> budget raise
    "reset_hour": trb(bud, "gamma", 500),     # placeholder overwritten below
    "totals_now": bud.session_totals.get("gamma", 0),
}
nowb[0] = 7000.0 + 3601.0
out["sh_budget"]["after_hour"] = trb(bud, "gamma", 500)
out["sh_budget"]["totals_reset"] = bud.session_totals.get("gamma", 0)

# ============ health thresholds & recovery messages recomputed cleanly ============
out["sh_recov"] = {
    "mem": sh.RecoveryCoordinator().execute_recovery("memory_pressure", {}),
    "tok": sh.RecoveryCoordinator().execute_recovery("token_overflow", {}),
    "unk": sh.RecoveryCoordinator().execute_recovery("mystery", {}),
}

json.dump(out, open(r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\host_batchA\ref_values3.json", "w"), indent=1, default=str)
print("PART3:", json.dumps(out)[:800])

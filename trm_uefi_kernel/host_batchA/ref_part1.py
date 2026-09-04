"""Reference generator for batch A — runs REAL python to lock expected values.
Writes host_batchA/ref_values.json ."""
import sys, os, json, time, hashlib, re, asyncio, types
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
os.chdir(r"F:\Development sites\TRM-Swarm-OS-v2")

from swarm_v2.core import (continuum_pbft as cpbft, dram_pattern_computer as dram,
    gpu_infinity_computer as gpu, persona_matrix as pm, radix_trie_matcher as rtm,
    self_fuzzing_engine as sfe, self_healing as sh, llm_brain as lb,
    microkernel_spawner as mks, kernel_instructions as ki)

out = {}

# ============ continuum_pbft ============
FIXED_TS = 1700000000.123
class FakeDT:
    @staticmethod
    def now(): return FakeDT()
    def isoformat(self): return "2023-11-14T22:13:20.000000"
cpbft.time = types.SimpleNamespace(time=lambda: FIXED_TS)
cpbft.datetime = FakeDT

led = cpbft.ContinuumPBFTLedger()
gen = led.nodes["ORCH"].state_ledger[:]
out["cpbft_genesis_orch_ledger"] = gen
out["cpbft_genesis_hash"] = hashlib.sha256("|".join(gen).encode()).hexdigest()
out["cpbft_hash_abc"] = hashlib.sha256(b"a|b|c").hexdigest()
pre = hashlib.sha256(f"PRE_PREPARE:tx_1:{out['cpbft_genesis_hash']}".encode()).hexdigest()
out["cpbft_pre_prepare_sig"] = pre

r1 = led.execute_pbft_cycle({"action": "STATE_UPDATE"})
out["cpbft_cycle1"] = {
    "tx_id": r1["tx_id"], "block": r1["block"], "leader": r1["leader"],
    "prepare_votes": r1["prepare_votes"], "required_votes": r1["required_votes"],
    "status": r1["status"],
    "orch_ledger_len": len(led.nodes["ORCH"].state_ledger),
    "sage_ledger_len": len(led.nodes["SAGE"].state_ledger),
    "orch_last2": led.nodes["ORCH"].state_ledger[-2:],
    "hist_len": len(led.transaction_history),
}
# byzantine: unhealthy nodes
led2 = cpbft.ContinuumPBFTLedger()
led2.nodes["SAGE"].status = "down"
led2.nodes["MUSE"].status = "down"
r2 = led2.execute_pbft_cycle({"action": "X"})
out["cpbft_cycle_insufficient"] = {"status": r2["status"], "reason": r2["reason"]}
# audit + repair
led3 = cpbft.ContinuumPBFTLedger()
led3.execute_pbft_cycle({"action": "STATE_UPDATE"})
led3.nodes["MUSE"].state_ledger.append("TAMPERED_BLOCK")
ar = led3.audit_and_repair_mesh()
out["cpbft_audit"] = {
    "mesh_status": ar["mesh_status"], "total_nodes": ar["total_nodes"],
    "majority_hash": ar["majority_ledger_hash"],
    "isolated_count": ar["isolated_count"],
    "remediated": ar["remediated_nodes"],
    "muse_status": ar["node_states"]["MUSE"]["status"],
    "muse_ledger_len": ar["node_states"]["MUSE"]["ledger_length"],
    "orch_ledger_len": ar["node_states"]["ORCH"]["ledger_length"],
    "states_order": [ar["node_states"][k]["status"] for k in ["ORCH","SAGE","MUSE","SENTINEL"]],
}
# empty-ledger get_ledger default genesis record
led4 = cpbft.ContinuumPBFTLedger()
gl = led4.get_ledger()
out["cpbft_getledger_default"] = {"n": len(gl), "block": gl[0]["block"], "votes": gl[0]["prepare_votes"], "req": gl[0]["required_votes"], "status": gl[0]["status"], "tx": gl[0]["tx_id"]}

# ============ dram_pattern_computer ============
rb = dram.pathway_row_buffer(128)
bk = dram.pathway_bulk_k3()
rh = dram.pathway_rowhammer()
fa = dram.ram_architecture_fpga()
na = dram.ram_architecture_neural()
st = dram.ram_architecture_stochastic()
def fmtfl(x):  # shortest repr like Rust {}
    return repr(float(x))
out["dram"] = {
    "specs": {"capacity": dram.DRAM_SPECS["capacity_gb"], "cells": dram.DRAM_SPECS["cells"],
              "banks": dram.DRAM_SPECS["banks"], "rowbuf": dram.DRAM_SPECS["row_buffer_bytes"],
              "rowcycle": dram.DRAM_SPECS["row_cycle_ns"], "refresh": dram.DRAM_SPECS["refresh_interval_us"],
              "rh_thresh": dram.DRAM_SPECS["rowhammer_threshold"], "bw": dram.DRAM_SPECS["bandwidth_gbps"]},
    "rowbuf": {"name": rb["name"], "banks": rb["banks"], "cper": rb["cells_per_bank"],
               "total": rb["total_cells"], "access": rb["access_time_ns"], "bw_str": rb["bandwidth"]},
    "bulk": {"name": bk["name"], "total": bk["total_cells"], "usable": bk["usable_cells"],
             "trits": bk["trits"], "tput": bk["throughput_tps"]},
    "rowhammer": {"name": rh["name"], "act": rh["activations_per_flip"], "prob_str": rh["flip_probability"]},
    "fpga": {"name": fa["name"], "luts": fa["luts"], "lut_size": fa["lut_size"], "banks": fa["banks_used"], "reconf": fa["reconfiguration_time_ns"]},
    "neural": {"name": na["name"], "neurons": na["neurons"], "synapses": na["synapses"], "precision": na["precision"]},
    "stochastic": {"name": st["name"], "nodes": st["nodes"], "type": st["type"]},
    "arch_keys": list(dram.RAM_ARCHITECTURES.keys()),
}
d = dram.DRAMPatternComputer()
lo = d.load_architecture("neural")
lo_bad = d.load_architecture("nope")
out["dram_load"] = {"arch": lo["architecture"], "cells_str": lo["cells_available"], "bw": lo["bandwidth"], "refresh": lo["refresh_required"], "current": d.current}
out["dram_load_bad"] = {"err": lo_bad["error"], "avail": lo_bad["available"]}
out["dram_summary"] = {"ram": d.summary()["ram"], "cells": d.summary()["cells"], "banks": d.summary()["banks"], "bw": d.summary()["bandwidth"], "refresh": d.summary()["refresh"], "pathways": d.summary()["pathways"], "archs": d.summary()["architectures"]}

# ============ gpu_infinity_computer ============
pf = gpu.gpu_pattern_fpga(8)
pn = gpu.gpu_pattern_neural(64)
pc = gpu.gpu_pattern_cellular(96)
pk = gpu.gpu_pattern_k3_ternary()
ic = gpu.GPU_SPECS["infinity_cache"]
out["gpu"] = {
    "specs": {"model": gpu.GPU_SPECS["model"], "transistors": gpu.GPU_SPECS["total_transistors"],
              "ic_mb": ic["size_mb"], "ic_bytes": ic["size_bytes"], "ic_cells": ic["cells"],
              "ic_tx": ic["transistors"], "bw": ic["bandwidth_tbps"],
              "vram_gb": gpu.GPU_SPECS["vram"]["size_gb"], "vram_cells": gpu.GPU_SPECS["vram"]["cells"],
              "cu": gpu.GPU_SPECS["compute"]["compute_units"], "sp": gpu.GPU_SPECS["compute"]["stream_processors"],
              "wf": gpu.GPU_SPECS["compute"]["max_wavefronts"], "mhz": gpu.GPU_SPECS["compute"]["clock_mhz"]},
    "fpga": {"name": pf["name"], "logic_cells": pf["logic_cells"], "luts": pf["total_luts"], "lut_size": pf["lut_size"], "cache": pf["cache_used"]},
    "neural": {"name": pn["name"], "neurons": pn["neurons"], "synapses": pn["synapses"], "cache": pn["cache_used"]},
    "cellular": {"name": pc["name"], "cells": pc["cells"], "ups": pc["updates_per_second"], "cache": pc["cache_used"]},
    "k3": {"name": pk["name"], "cells": pk["cells_used"], "trits": pk["trits"], "tput": pk["throughput_tps"], "cache": pk["cache_used"], "instr": pk["instruction"]},
    "arch_keys": list(gpu.ARCHITECTURES_GPU.keys()),
}
gc = gpu.GPUInfinityComputer()
l1 = gc.load_architecture("k3")
out["gpu_load_k3"] = {"arch": l1["architecture"], "pattern_hash": l1["pattern_hash"], "cache_util": l1["cache_utilization"], "bw": l1["bandwidth_available"], "current": gc.current}
gc2 = gpu.GPUInfinityComputer()
l2 = gc2.load_architecture("nope")
out["gpu_load_bad"] = {"err": l2["error"], "avail": l2["available"]}
out["gpu_hashes"] = {
    "fpga": hashlib.sha256(b"GPU FPGA-Lite").hexdigest()[:12],
    "neural": hashlib.sha256(b"GPU SRAM Neural Network").hexdigest()[:12],
    "cellular": hashlib.sha256(b"GPU Cellular Automaton").hexdigest()[:12],
    "k3": hashlib.sha256(b"GPU K3 Ternary Engine").hexdigest()[:12],
}
# ============ expert_registry ============
er = __import__("swarm_v2.core.expert_registry", fromlist=["x"]).ExpertRegistry()
class FakeAgent:
    def __init__(self, n, r):
        self.persona = types.SimpleNamespace(name=n, role=r)
team = {"ORCH": FakeAgent("Continuum Core", "Primary Leader Node"),
        "logic": FakeAgent("Logic", "Reasoning Engine"),
        "shield": FakeAgent("Shield", "Security Isolation Node")}
er.register_team(team)
def er_id(s): 
    a = er.get_agent(s)
    if a is None: return None
    return a.persona.name
out["expert_registry"] = {
    "exact": er_id("logic"),
    "case_insens": er_id("SHIELD"),
    "persona_name_exact": er_id("Logic"),
    "persona_substr": er_id("** Logic (Reasoning Engine)"),
    "role_substr": er_id("security isolation node"),
    "miss": er_id("nope"),
    "empty": er_id(""),
    "clean_stars": er_id("__log*ic`"),
    "all_keys": sorted(er.get_all_agents().keys()),
}

# ============ persona_matrix ============
pm.time = types.SimpleNamespace(time=lambda: pm_time[0])
pm_time = [1700000000.0]
async def run_enforce(sm, ctx):
    return await sm.enforce(ctx)
# clean pass
kernel = pm.IdentityKernel(core_axioms=["Never lie", "Protect sovereignty"], forbidden_lexicon=["malware", "exploit"])
sm1 = pm.PersonaStateMachine()
pm_time[0] = 1700000000.0
res = asyncio.run(run_enforce(sm1, pm.GenerationContext(user_input="hi", identity_kernel=kernel, output="I am a helpful sovereign node.")))
out["persona_clean"] = {"content": res.content, "status": int(res.status.value), "state": int(sm1.state.value), "viol": sm1.violation_count}
# violation >= max (default max=1)
sm2 = pm.PersonaStateMachine()
pm_time[0] = 1700000000.0
res = asyncio.run(run_enforce(sm2, pm.GenerationContext(user_input="hi", identity_kernel=kernel, output="run the malware now")))
out["persona_fail"] = {"content": res.content, "status": int(res.status.value), "state": int(sm2.state.value), "viol": sm2.violation_count, "last_reset": sm2.last_reset_time}
# max_violations=2, single violation -> RESET
sm3 = pm.PersonaStateMachine(max_violations=2)
res = asyncio.run(run_enforce(sm3, pm.GenerationContext(user_input="hi", identity_kernel=kernel, output="deploy exploit kit")))
out["persona_reset"] = {"content": res.content, "status": int(res.status.value), "state": int(sm3.state.value), "viol": sm3.violation_count}
# locked within window
sm4 = pm.PersonaStateMachine()
sm4.state = pm.PersonaState.LOCKED
sm4.last_reset_time = 1700000000.0
pm_time[0] = 1700000040.0
res = asyncio.run(run_enforce(sm4, pm.GenerationContext(user_input="hi", identity_kernel=kernel, output="clean text")))
out["persona_locked"] = {"content": res.content, "status": int(res.status.value), "state": int(sm4.state.value)}
# locked then expired -> unlock then pass clean
pm_time[0] = 1700000100.0
res = asyncio.run(run_enforce(sm4, pm.GenerationContext(user_input="hi", identity_kernel=kernel, output="clean text")))
out["persona_unlock"] = {"content": res.content, "status": int(res.status.value), "state": int(sm4.state.value)}
# enum numeric
out["persona_enums"] = {"IDLE": int(pm.PersonaState.IDLE.value), "LOCKED": int(pm.PersonaState.LOCKED.value),
                        "FAILED_ST": int(pm.PersonaState.FAILED.value), "RESET_ST": int(pm.PersonaState.RESET.value),
                        "SUCCESS": int(pm.GenerationStatus.SUCCESS.value), "FAILED": int(pm.GenerationStatus.FAILED.value),
                        "RESET": int(pm.GenerationStatus.RESET.value)}
# compiled prompt
k2 = pm.IdentityKernel(core_axioms=["Never lie", "Protect sovereignty"], forbidden_lexicon=["malware", "exploit"])
pstr = pm.build_compiled_prompt("Logic", k2)
out["persona_prompt"] = {"len": len(pstr), "sha3": hashlib.sha3_256(pstr.encode()).hexdigest(),
                         "head": pstr[:60], "tail": pstr[-60:]}

json.dump(out, open(r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\host_batchA\ref_values.json", "w"), indent=1, default=str)
print("PART1 written:", len(out), "keys")

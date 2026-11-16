"""batch1_ref_driver.py — deterministic reference-output generator for the 9 module batch.
Runs the REAL swarm_v2/core Python implementations with patched time/uuid/datetime
and stubbed I/O dependencies, writing reference outputs + scenario inputs to
trm_uefi_kernel/host_test_batch1/ref_batch1.json for the Rust host harness.

Usage: cd repo-root && PYTHONPATH=. C:/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe batch1_ref_driver.py
"""
import asyncio
import hashlib
import importlib.util
import json
import sys
import types
import uuid as uuid_mod
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO))
OUT = REPO / "trm_uefi_kernel" / "host_test_batch1" / "ref_batch1.json"

# ---------------------------------------------------------------------------
# Deterministic time / uuid / datetime machinery
# ---------------------------------------------------------------------------
class FakeClock:
    def __init__(self):
        self.t = 1000.0
    def time(self):
        return self.t

CLOCK = FakeClock()
import time as time_mod
time_mod.time = CLOCK.time  # global patch — all modules call time.time()

UUID_SEQ = [uuid_mod.UUID(int=i) for i in range(1, 400)]
_uuid_iter = iter(UUID_SEQ)
def _fake_uuid4():
    return next(_uuid_iter)
uuid_mod.uuid4 = _fake_uuid4

DT_SEQ = [datetime(2026, 1, 1, 0, 0, i % 60) for i in range(1, 400)]
_dt_iter = iter(DT_SEQ)
def _fake_utcnow():
    return next(_dt_iter)
def _fake_now():
    return next(_dt_iter)

def load_module(name, path, stub_before=None):
    """Load a module by file path; optionally pre-stub sys.modules entries first."""
    if stub_before:
        for k, v in stub_before.items():
            sys.modules[k] = v
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod

def patch_module_datetime(mod):
    """Replace datetime used inside a module with a fake exposing utcnow/now."""
    class FakeDT:
        @classmethod
        def utcnow(cls):
            return _fake_utcnow()
        @classmethod
        def now(cls):
            return _fake_now()
    mod.datetime = FakeDT

def run(mod_name, fn):
    try:
        return {"ok": True, "value": fn()}
    except Exception as e:
        import traceback
        traceback.print_exc()
        return {"ok": False, "error": f"{type(e).__name__}: {e}"}

REF = {}
core_dir = REPO / "swarm_v2" / "core"

# ===========================================================================
# 1. pbft_consensus
# ===========================================================================
pbft_mod = load_module("pbft_consensus", core_dir / "pbft_consensus.py")
patch_module_datetime(pbft_mod)

def pbft_scenario():
    layer = pbft_mod.PBFTConsensusLayer()
    out = {}

    # scenario inputs: (id, created_at, proposer, type, payload)
    P = []
    def mk(proposer, ptype, payload):
        pid = next(_uuid_iter).hex[:12]
        created = next(_dt_iter).isoformat()
        P.append([pid, created, proposer, ptype, payload])
        return pid, created

    pid1, c1 = mk("Archi", "dna_mutation",
                  {"action": "mutate", "target": "genome_alpha", "delta": 42,
                   "critical": True, "tags": ["fast", "stable"],
                   "meta": {"k": 1, "nested": {"z": 2, "a": 1}}})
    pid2, c2 = mk("Hermes", "state_update", {"op": "set", "key": "k1", "value": 7})
    pid3, c3 = mk("Intruder", "node_eviction", {"victim": "Pulse"})
    pid4, c4 = mk("Logic", "key_revocation", {"target_node": "Devo"})

    props = [layer.propose(p[2], p[3], p[4]) for p in P]
    for prop, p in zip(props, P):  # re-align generated id/timestamp deterministically
        prop.proposal_id = p[0]
        prop.created_at = p[1]
        layer._proposals[p[0]] = prop

    # --- pre_prepare ---
    out["pre_prepare_ok"] = layer.pre_prepare(pid1)
    out["pre_prepare_unknown"] = layer.pre_prepare("000000000099")
    out["pre_prepare_intruder"] = layer.pre_prepare(pid3)
    out["intruder_proposal_snapshot"] = layer.get_proposal(pid3)

    # --- integrity hash (what pre_prepare computes via json sort_keys) ---
    payload1 = P[0][4]
    out["canonical_payload_json"] = json.dumps(payload1, sort_keys=True)
    out["integrity_hash"] = hashlib.sha256(
        json.dumps(payload1, sort_keys=True).encode("utf-8")).hexdigest()

    # --- prepare phase (vote to quorum then beyond) ---
    voters = ["Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", "Flow",
              "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Hermes"]
    phases = []
    for v in voters[:9]:
        phases.append(layer.prepare(pid1, v, approve=True))
    out["prepare_phases_after_each_vote"] = phases
    out["prepare_unknown"] = layer.prepare("000000000099", "Jarvis", True)
    out["prepare_votes_recorded"] = dict(layer.get_proposal(pid1)["votes"])

    # --- commit phase ---
    committers = ["Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo",
                  "Pulse", "Flow", "EventLedger"]
    commit_returns = []
    for c in committers:
        commit_returns.append(layer.commit(pid1, c))
    out["commit_returns"] = commit_returns
    out["commit_unknown"] = layer.commit("000000000099", "Jarvis")
    out["finalized_snapshot"] = layer.get_proposal(pid1)

    # --- rejection ---
    out["reject_ok"] = layer.reject(pid2, "policy_violation")
    out["reject_unknown"] = layer.reject("000000000099", "nope")
    out["rejected_snapshot"] = layer.get_proposal(pid2)

    # --- full cycle ---
    cyc = layer.run_full_cycle("Logic", "key_revocation", {"target_node": "Devo"})
    cycle_id = cyc["proposal_id"]
    cycle_snap = layer.get_proposal(cycle_id)
    out["full_cycle"] = {"result": cyc["result"], "phase": cyc["phase"]}
    out["full_cycle_snapshot"] = cycle_snap
    out["ledger_ids"] = [e["proposal_id"] for e in layer.audit_ledger()]

    # --- status ---
    st = layer.get_status()
    out["status_basic"] = {k: v for k, v in st.items() if k != "recent_proposals"}
    out["status_recent"] = st["recent_proposals"]

    out["inputs"] = {
        "proposals": [{"proposal_id": p[0], "created_at": p[1], "proposer": p[2],
                       "proposal_type": p[3], "payload": p[4]} for p in P],
        "cycle": {"proposal_id": cycle_id, "created_at": cycle_snap["created_at"],
                  "finalized_at": cycle_snap["finalized_at"]},
        "pid1_finalized_at": out["finalized_snapshot"]["finalized_at"],
    }
    return out

REF["pbft"] = run("pbft", pbft_scenario)

# ===========================================================================
# 2. heuristic_consensus
# ===========================================================================
hc_mod = load_module("heuristic_consensus", core_dir / "heuristic_consensus.py")

def hc_node(node_id, role, name, specialties, skills, alive, task_count=0):
    return types.SimpleNamespace(
        node_id=node_id, role=role, name=name, specialties=list(specialties),
        skills=list(skills), is_alive=alive, task_count=task_count,
        to_dict=lambda: {"node_id": node_id, "role": role, "name": name,
                         "task_count": task_count, "status": "online" if alive else "offline"},
    )

def hc_scenario():
    layer = hc_mod.HeuristicConsensusLayer()
    out = {}
    nodes = {
        "Archi": hc_node("Archi", "Chief Architect", "Archi",
                         ["architecture", "system design", "microkernel"], ["rust", "uefi"], True, 3),
        "Logic": hc_node("Logic", "Reasoning Engine", "Logic",
                         ["verification", "logic"], ["lean4", "theorem proving"], True, 6),
        "Devo": hc_node("Devo", "Lead Developer", "Devo",
                        ["rust", "fullstack"], ["python", "cargo"], True, 0),
        "Shield": hc_node("Shield", "Security Director", "Shield",
                          ["security", "audit"], ["nvd", "cve"], False, 9),
        "Hermes": hc_node("Hermes", "Communications Director", "Hermes",
                          ["communications"], [], True, 10),
    }
    mesh = types.SimpleNamespace(
        nodes=nodes, topology_version=7,
        get_alive_nodes=lambda: [n for n in nodes.values() if n.is_alive],
        _save_state=lambda: None,
    )
    skill_matrix = types.SimpleNamespace(proficiency_matrix={
        "Chief Architect": {"rust": 2.7, "system design": 2.9},
        "Reasoning Engine": {"verification": 2.4},
        "Lead Developer": {"rust": 2.1, "python": 2.6, "cargo": 1.8},
        "Security Director": {"security": 2.9, "audit": 2.2},
        "Communications Director": {"broadcast": 1.5},
    })
    layer.bind(mesh, None, skill_matrix)

    tasks = [
        "route rust microkernel task to the lead developer",
        "perform security audit on the ddr antibody system",
        "verify the lean4 proof of the verification pipeline",
        "broadcast the morning report to all stations",
        "system design of a new microkernel architecture",
    ]
    scores = {}
    for t in tasks:
        scores[t] = {nid: layer.score_agent(node, t) for nid, node in nodes.items()}
    out["scores"] = scores

    r1 = asyncio.run(layer.route(tasks[0]))
    out["route_heuristic_keys"] = {
        "routed_to": r1["routed_to"]["role"], "node_id": r1["routed_to"]["node_id"],
        "mesh_version": r1["mesh_version"], "method": r1["routing_method"],
        "scores": r1["heuristic_scores"], "task": r1["task"],
    }
    r2 = asyncio.run(layer.route(tasks[1], target_node_id="Shield"))
    out["route_direct"] = {"node_id": r2["routed_to"]["node_id"], "method": r2["routing_method"],
                           "mesh_version": r2["mesh_version"], "task": r2["task"]}
    r3 = asyncio.run(layer.route(tasks[2], required_specialty="verification"))
    out["route_specialty"] = {"node_id": r3["routed_to"]["node_id"], "method": r3["routing_method"]}

    nodes2 = {
        "A1": hc_node("A1", "Twin A", "A1", ["x"], [], True, 0),
        "B1": hc_node("B1", "Twin B", "B1", ["x"], [], True, 0),
    }
    mesh2 = types.SimpleNamespace(
        nodes=nodes2, topology_version=1,
        get_alive_nodes=lambda: [nodes2["A1"], nodes2["B1"]],
        _save_state=lambda: None,
    )
    layer2 = hc_mod.HeuristicConsensusLayer()
    layer2.bind(mesh2, None, None)
    r4 = asyncio.run(layer2.route("task about x"))
    out["route_tiebreak"] = {"node_id": r4["routed_to"]["node_id"], "scores": r4["heuristic_scores"]}

    layer3 = hc_mod.HeuristicConsensusLayer()
    out["route_unbound"] = asyncio.run(layer3.route("anything"))

    out["routing_log_len"] = len(layer.get_routing_log())
    st = layer.get_stats()
    out["stats"] = {"total_routed": st["total_routed"], "weights": st["weights"],
                    "recent": st["recent_decisions"]}
    return out

REF["heuristic"] = run("heuristic", hc_scenario)

# ===========================================================================
# 3. continuum_pbft
# ===========================================================================
cp_mod = load_module("continuum_pbft", core_dir / "continuum_pbft.py")
patch_module_datetime(cp_mod)

def cp_scenario():
    ledger = cp_mod.ContinuumPBFTLedger()
    out = {}
    out["genesis_ledgers"] = {nid: n.state_ledger for nid, n in ledger.nodes.items()}
    out["genesis_hashes"] = {nid: n.get_ledger_hash() for nid, n in ledger.nodes.items()}

    CLOCK.t = 1000.0
    out["tx1"] = ledger.execute_pbft_cycle({"action": "STATE_UPDATE", "data": {"k": 1}})
    CLOCK.t = 1234.567
    out["tx2"] = ledger.execute_pbft_cycle({"action": "DNA_MUTATION"})
    CLOCK.t = 2000.0
    out["vote"] = ledger.cast_consensus_vote("B1", "abc123")
    out["ledger_with_history"] = ledger.get_ledger()

    audit1 = ledger.audit_and_repair_mesh()
    audit1.pop("node_states", None)
    out["audit_healthy"] = audit1

    sage = ledger.nodes["SAGE"]
    sage.state_ledger.append("BLOCK_tampered_by_byzantine")
    audit2 = ledger.audit_and_repair_mesh()
    out["audit_repair_summary"] = {k: v for k, v in audit2.items() if k != "node_states"}
    out["audit_repair_states"] = audit2["node_states"]
    out["sage_remediated_ledger"] = sage.state_ledger
    out["sage_hash_after"] = sage.get_ledger_hash()

    ledger2 = cp_mod.ContinuumPBFTLedger()
    out["empty_ledger_fallback"] = ledger2.get_ledger()
    return out

REF["continuum_pbft"] = run("continuum_pbft", cp_scenario)

# ===========================================================================
# 4. expert_registry
# ===========================================================================
er_mod = load_module("expert_registry", core_dir / "expert_registry.py")

def er_agent(pname, prole):
    return types.SimpleNamespace(persona=types.SimpleNamespace(name=pname, role=prole))

def er_scenario():
    reg = er_mod.ExpertRegistry()
    team = {
        "Chief Architect": er_agent("Archi", "Chief Architect"),
        "Reasoning Engine": er_agent("Logic", "Reasoning Engine"),
        "Lead Developer": er_agent("Devo", "Lead Developer"),
        "Security Director": er_agent("Shield", "Security Director"),
    }
    reg.register_team(team)

    queries = [
        "Chief Architect", "chief architect", "CHIEF ARCHITECT",
        "Archi", "Logic",
        "** Logic (Reasoning Engine)", "*Devo*", "`Shield`",
        "Security Director", "Reasoning Engine",
        "developer", "Ghost", "", None, "Lead_Developer",
    ]
    out = {}
    for q in queries:
        hit = reg.get_agent(q)
        if hit is None:
            out[repr(q)] = None
        else:
            out[repr(q)] = {"persona_name": hit.persona.name, "persona_role": hit.persona.role}
    out["all_agents_keys"] = list(reg.get_all_agents().keys())
    return out

REF["expert_registry"] = run("expert_registry", er_scenario)

# ===========================================================================
# 5. persona_matrix
# ===========================================================================
pm_mod = load_module("persona_matrix", core_dir / "persona_matrix.py")

def pm_scenario():
    CLOCK.t = 100.0
    out = {}
    sm = pm_mod.PersonaStateMachine(max_violations=1, rate_limit_window=60.0)
    kernel = pm_mod.IdentityKernel(
        core_axioms=["I am a Swarm OS node.", "I never claim to be human."],
        forbidden_lexicon=["I am human", "emotional support"],
        required_lexicon=[],
    )

    async def go():
        r1 = await sm.enforce(pm_mod.GenerationContext(
            user_input="generate a design", identity_kernel=kernel,
            output="Here is the microkernel design document."))
        out["clean"] = {"content": r1.content, "status": r1.status.value, "state": sm.state.value,
                        "violations": sm.violation_count}
        r2 = await sm.enforce(pm_mod.GenerationContext(
            user_input="x", identity_kernel=kernel,
            output="I am human and I feel emotional support is key."))
        out["hard_fail"] = {"content": r2.content, "status": r2.status.value, "state": sm.state.value,
                            "violations": sm.violation_count, "last_reset": sm.last_reset_time}
        sm.state = pm_mod.PersonaState.LOCKED
        sm.last_reset_time = 100.0
        r3 = await sm.enforce(pm_mod.GenerationContext(
            user_input="x", identity_kernel=kernel, output="anything"))
        out["locked_within_window"] = {"content": r3.content, "status": r3.status.value,
                                       "state": sm.state.value}
        sm.last_reset_time = 30.0
        r4 = await sm.enforce(pm_mod.GenerationContext(
            user_input="x", identity_kernel=kernel, output="clean output"))
        out["locked_past_window"] = {"content": r4.content, "status": r4.status.value,
                                     "state": sm.state.value, "violations": sm.violation_count}
        sm2 = pm_mod.PersonaStateMachine(max_violations=2, rate_limit_window=60.0)
        r5 = await sm2.enforce(pm_mod.GenerationContext(
            user_input="x", identity_kernel=kernel, output="I am human, please help me."))
        out["reset"] = {"content": r5.content, "status": r5.status.value, "state": sm2.state.value,
                        "violations": sm2.violation_count, "last_reset": sm2.last_reset_time}
        r6 = await sm2.enforce(pm_mod.GenerationContext(
            user_input="x", identity_kernel=kernel, output="I AM HUMAN NOW"))
        out["case_insensitive"] = {"status": r6.status.value, "state": sm2.state.value}
        return out

    out.update(asyncio.run(go()))

    prompt = pm_mod.build_compiled_prompt("Logic", kernel)
    out["prompt"] = prompt
    out["prompt_hash"] = prompt.split("-->")[0].replace("<!-- SIGNATURE:", "").strip()
    return out

REF["persona_matrix"] = run("persona_matrix", pm_scenario)

# ===========================================================================
# 6. skill_matrix_sync
# ===========================================================================
sms_mod = load_module("skill_matrix_sync", core_dir / "skill_matrix_sync.py")
patch_module_datetime(sms_mod)

def sms_skill(name, desc, instr, source, endpoints):
    return types.SimpleNamespace(skill_name=name, description=desc,
                                 instructions=instr, source=source, endpoints=endpoints)

def sms_scenario():
    sync = sms_mod.SkillMatrixSync()
    sync.bind(None, None, None)
    out = {}

    def usage_seq(role, skill, results):
        for ok in results:
            sync.record_skill_usage(role, skill, ok)

    usage_seq("Lead Developer", "rust", [True] * 8)
    usage_seq("Lead Developer", "python", [True, False, True, False, True])
    usage_seq("Reasoning Engine", "lean4",
              [True, True, False, False, True, True, True, True, True, True,
               True, True, True, True, True, True])
    usage_seq("Chief Architect", "rust", [False, False, False, True])
    usage_seq("Lead Developer", "rust", [True, True, False])
    out["proficiency_matrix"] = {r: dict(s) for r, s in sync.proficiency_matrix.items()}
    out["usage_tracker"] = {r: {s: dict(u) for s, u in m.items()}
                            for r, m in sync._usage_tracker.items()}

    sk1 = sms_skill("rust", "Rust systems programming", "use cargo test", "cargo docs",
                    {"/build": "POST", "/test": "GET"})
    sync.on_skill_created(sk1)
    sk2 = sms_skill("lean4", "Lean4 theorem proving", "prove theorems", None, {})
    sync.on_skill_created(sk2)
    sync.on_skill_evolved(sk2)
    sync.on_skill_forgotten("python")
    out["skill_events"] = sync._skill_events
    out["matrix_after_events"] = {r: dict(s) for r, s in sync.proficiency_matrix.items()}
    out["skill_memory_content_created"] = (
        f"Skill: {sk1.skill_name}\nDescription: {sk1.description}\nInstructions: {sk1.instructions}")
    out["skill_memory_content_evolved"] = (
        f"Skill (Evolved): {sk2.skill_name}\nDescription: {sk2.description}\nInstructions: {sk2.instructions}")

    m = sync.get_matrix()
    out["matrix_report"] = {"proficiency_matrix": m["proficiency_matrix"],
                            "skill_events": m["skill_events"],
                            "agent_count": m["agent_count"], "skill_count": m["skill_count"],
                            "timestamp": m["timestamp"]}

    fake_mesh = types.SimpleNamespace(nodes={
        "n1": types.SimpleNamespace(role="Lead Developer", task_count=3),
        "n2": types.SimpleNamespace(role="Chief Architect", task_count=5),
    })
    sys.modules["swarm_v2.core.agent_mesh"] = types.SimpleNamespace(
        get_agent_mesh=lambda: fake_mesh)
    overlay = sync.get_topology_overlay({"Lead Developer": 0.9, "Chief Architect": 0.8})
    nodes_out = [{k: v for k, v in n.items() if k != "skills"} for n in overlay["nodes"]]
    links_sorted = []
    for lk in overlay["skill_links"]:
        links_sorted.append({"source": lk["source"], "target": lk["target"],
                             "shared_skills": sorted(lk["shared_skills"]),
                             "affinity": lk["affinity"]})
    links_sorted.sort(key=lambda l: (l["source"], l["target"]))
    out["overlay"] = {"nodes": nodes_out, "skill_links": links_sorted,
                      "timestamp": overlay["timestamp"]}
    return out

REF["skill_matrix_sync"] = run("skill_matrix_sync", sms_scenario)

# ===========================================================================
# 7. coordination_protocol
# ===========================================================================
class FakeMailbox:
    sent = []
    incoming = "EMPTY"  # "EMPTY" | "AUTO" | list of messages
    def __init__(self, *a, **k):
        pass
    def send(self, recipient, body, subject=""):
        FakeMailbox.sent.append({"recipient": recipient, "body": body, "subject": subject})
        return True
    def receive(self, limit=50):
        if FakeMailbox.incoming == "AUTO":
            msgs = []
            for rid, rd in _proto_ref.active_rounds.items():
                for agent in rd["sent_to"]:
                    msgs.append({"body": json.dumps({"round_id": rid, "proposal": f"auto-{agent}"}),
                                 "from_agent": agent})
            return msgs[:limit]
        if FakeMailbox.incoming == "EMPTY":
            return []
        got, FakeMailbox.incoming = FakeMailbox.incoming[:limit], FakeMailbox.incoming[limit:]
        return got

stub_mailbox = types.SimpleNamespace(AgentMailbox=FakeMailbox)
stub_uwl = types.SimpleNamespace(get_ultrawork_loop=lambda: types.SimpleNamespace(
    list_missions=lambda: []))
cp2_mod = load_module("coordination_protocol", core_dir / "coordination_protocol.py",
                      stub_before={"swarm_v2.core.agent_mailbox": stub_mailbox,
                                   "swarm_v2.core.ultrawork_loop": stub_uwl})

def cp2_scenario():
    global _proto_ref
    proto = cp2_mod.CoordinationProtocol()
    _proto_ref = proto
    out = {}
    CLOCK.t = 1000.0
    FakeMailbox.sent = []

    async def go():
        br = await proto.broadcast_task("optimize kernel boot", ["Archi", "Logic", "Devo"],
                                        context="boot path", timeout=30)
        out["broadcast"] = {"round_id": br["round_id"], "agents": br["agents"],
                            "status": br["status"]}
        out["sent_messages"] = FakeMailbox.sent
        rnd = br["round_id"]
        FakeMailbox.incoming = [
            {"body": json.dumps({"round_id": rnd, "proposal": "proposal from Archi"}), "from_agent": "Archi"},
            {"body": json.dumps({"round_id": rnd, "response": "response from Devo"}), "from_agent": "Devo"},
            {"body": json.dumps({"round_id": rnd, "proposal": "proposal from Logic"}), "from_agent": "Logic"},
            {"body": json.dumps({"round_id": "coord-other", "proposal": "ignored"}), "from_agent": "Ghost"},
            {"body": "not json", "from_agent": "Broken"},
        ]
        CLOCK.t = 1003.5
        coll = await proto.collect_responses(rnd, wait=True, max_wait=60)
        out["collect"] = {"round_id": coll["round_id"], "task": coll["task"],
                          "responses": coll["responses"], "response_count": coll["response_count"],
                          "agents_sent": coll["agents_sent"], "elapsed": coll["elapsed"]}
        resp = coll["responses"]
        out["agg_majority"] = proto.consensus_aggregate(resp, "majority")
        out["agg_all"] = proto.consensus_aggregate(resp, "all")
        out["agg_summary"] = proto.consensus_aggregate(resp, "summary")
        out["agg_empty"] = proto.consensus_aggregate({}, "majority")
        out["agg_unknown_method"] = proto.consensus_aggregate(resp, "bogus")
        out["agg_majority_tie"] = proto.consensus_aggregate(
            {"A": "alpha", "B": "alpha", "C": "beta", "D": "beta"}, "majority")

        CLOCK.t = 2000.0
        FakeMailbox.sent = []
        FakeMailbox.incoming = "AUTO"
        full = await proto.run_coordination_round("optimize kernel boot",
                                                  ["Archi", "Logic", "Devo"], "ctx", "majority", 30)
        out["full_round"] = {
            "round_id": full["round_id"],
            "task": full["task"],
            "broadcast_status": full["broadcast"]["status"],
            "responses": full["responses"]["responses"],
            "response_count": full["responses"]["response_count"],
            "consensus": full["consensus"],
        }
        return out

    out.update(asyncio.run(go()))
    return out

REF["coordination"] = run("coordination", cp2_scenario)

# ===========================================================================
# 8. comm_protocol
# ===========================================================================
comm_mod = load_module("comm_protocol", core_dir / "comm_protocol.py")

def comm_scenario():
    out = {}
    CLOCK.t = 300000.5
    win = int(CLOCK.t / 300)
    out["window"] = win
    tok = comm_mod.generate_trust_token("Shield", "Devo", "task_42")
    out["token"] = tok
    out["verify_ok"] = comm_mod.verify_trust_token("Shield", "Devo", "task_42", tok)
    CLOCK.t = 300300.0
    out["verify_next_window"] = comm_mod.verify_trust_token("Shield", "Devo", "task_42", tok)
    CLOCK.t = 300900.0
    out["verify_expired"] = comm_mod.verify_trust_token("Shield", "Devo", "task_42", tok)
    CLOCK.t = 300000.5
    out["verify_tampered"] = comm_mod.verify_trust_token("Shield", "Devo", "task_42", "deadbeef" * 8)
    payload = comm_mod.AgentHandshake.create_secure_payload("Shield", "Devo", "task_42",
                                                            {"cmd": "deploy"})
    out["payload"] = payload
    out["validate_ok"] = comm_mod.AgentHandshake.validate_secure_payload(payload)
    tampered = dict(payload)
    tampered["content"] = {"cmd": "exfil"}
    out["validate_content_tamper"] = comm_mod.AgentHandshake.validate_secure_payload(tampered)
    return out

REF["comm"] = run("comm", comm_scenario)

# ===========================================================================
# 9. native_mailbox_bridge
# ===========================================================================
AGENT_IDS = ["Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge",
             "Orchestra", "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes"]
stub_bus = types.SimpleNamespace(
    AGENT_ID_MAP={name: i for i, name in enumerate(AGENT_IDS)},
    SwarmMailboxBus=lambda: types.SimpleNamespace(
        initialize=lambda: None,
        send=lambda *a, **k: True,
        receive=lambda agent: None),
)
nmb_mod = load_module("native_mailbox_bridge", core_dir / "native_mailbox_bridge.py",
                      stub_before={"swarm_v2.core.agent_mailbox_bus": stub_bus})

def nmb_reference_ring():
    """Pure-Python transcription of the C++ ring semantics embedded in
    native_mailbox_bridge.py (CPP_SOURCE): per-recipient ring of 4096,
    msg_id=tail, priority=2, payload cap 40, idx=tail&(CAP-1)."""
    CAP = 4096
    MAX_AGENTS = 15
    rings = [{"head": 0, "tail": 0, "ring": [None] * CAP} for _ in range(MAX_AGENTS)]
    def send(sender, recipient, trit, payload=b""):
        if recipient >= MAX_AGENTS:
            return False
        box = rings[recipient]
        if box["tail"] - box["head"] >= CAP:
            return False
        idx = box["tail"] & (CAP - 1)
        plen = min(len(payload), 40)
        msg = {"msg_id": box["tail"], "sender_id": sender, "recipient_id": recipient,
               "k3_trit": trit, "priority": 2, "payload_size": plen,
               "inline_data": bytes(payload[:plen]) + b"\x00" * (40 - plen),
               "timestamp_ns": 0}
        box["ring"][idx] = msg
        box["tail"] += 1
        return True
    def recv(agent_id):
        if agent_id >= MAX_AGENTS:
            return None
        box = rings[agent_id]
        if box["head"] == box["tail"]:
            return None
        idx = box["head"] & (CAP - 1)
        m = box["ring"][idx]
        box["head"] += 1
        return m
    def pending(agent_id):
        if agent_id >= MAX_AGENTS:
            return 0
        box = rings[agent_id]
        return box["tail"] - box["head"]
    return send, recv, pending

def nmb_scenario():
    out = {}
    native = nmb_mod.NativeMailboxBridge()
    out["native_available"] = native.is_native
    out["native_send"] = native.send(1, 2, 0, b"hi")
    out["native_recv"] = native.receive(2)
    out["native_pending"] = native.pending(2)
    out["native_init"] = native.init()

    bus = nmb_mod.HybridMailboxBus()
    out["hybrid_backend"] = bus.backend
    out["hybrid_send"] = bus.send("Shield", "Devo", trit=1, payload=b"ready")
    out["hybrid_recv"] = bus.receive("Devo")
    out["agent_id_map"] = stub_bus.AGENT_ID_MAP

    send, recv, pending = nmb_reference_ring()
    ref = {}
    def m_fields(m):
        return {"msg_id": m["msg_id"], "sender_id": m["sender_id"],
                "recipient_id": m["recipient_id"], "k3_trit": m["k3_trit"],
                "priority": m["priority"], "payload_size": m["payload_size"],
                "payload": bytes(m["inline_data"][:m["payload_size"]]).decode("latin1"),
                "timestamp_ns": m["timestamp_ns"]}
    ref["send_ok"] = send(1, 2, 1, b"hello")
    ref["msg1"] = m_fields(recv(2))
    ref["send_truncate"] = send(3, 2, -1, b"x" * 100)
    m2 = recv(2)
    ref["msg2"] = m_fields(m2)
    ref["pending_after_recv"] = pending(2)
    ref["send_other_agent"] = send(1, 4, 0, b"isolated")
    ref["other_pending"] = pending(4)
    ref["recv_wrong_agent"] = recv(3)
    ref["recv_empty"] = recv(2)
    ref["send_oob_recipient"] = send(1, 99, 0, b"x")
    filled = [send(1, 5, 0, b"m") for _ in range(4096)]
    ref["ring_fill_all_true"] = all(filled)
    ref["ring_full_rejects"] = send(1, 5, 0, b"overflow")
    ref["first_msg_id"] = recv(5)["msg_id"]
    for _ in range(3):
        send(1, 5, 0, b"m")
    ref["wrap_ids"] = [recv(5)["msg_id"] for _ in range(4)]
    ref["pending_5"] = pending(5)
    send(1, 6, -1, b"trit")
    ref["trit_msg"] = m_fields(recv(6))
    out["ring_reference"] = ref
    return out

REF["mailbox"] = run("mailbox", nmb_scenario)

# ---------------------------------------------------------------------------
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(REF, indent=1, default=str), encoding="utf-8")
print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
for k, v in REF.items():
    print(f"  {k}: ok={v['ok']}" + (f" error={v['error']}" if not v["ok"] else ""))

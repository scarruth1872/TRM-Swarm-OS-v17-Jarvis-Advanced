"""Reference generator part 2 — radix_trie, self_fuzzing, self_healing, llm_brain, mks, kernel_instructions."""
import sys, os, json, time, hashlib, re, types, unicodedata
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
os.chdir(r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core import radix_trie_matcher as rtm, self_fuzzing_engine as sfe
from swarm_v2.core import self_healing as sh, microkernel_spawner as mks

out = {}

# ============ radix_trie_matcher ============
trie = rtm.get_radix_trie()
out["rtm"] = {"registered": trie.registered, "n_handlers": len(rtm.DETERMINISTIC_HANDLERS)}
def m(text):
    r = trie.match(text)
    if not r:
        return None
    handler, endpoint = r
    return [endpoint, handler()[:80]]
out["rtm_match"] = {
    "system_status": m("system status"),
    "health_check": m("health check"),
    "telemetry_alias": m("metrics"),
    "unknown": m("unknown operation xyz"),
    "hello": m("hello"),
    "hi_there": m("hi there"),   # prefix overrides: 'hi' handler
    "literature_prefix": m("literature search on tau proteins"),
    "shield": m("shield scan"),
}
mp = trie.match_prefix("literature search on tau")
out["rtm_prefix"] = {"len": mp[2], "endpoint": mp[1], "resp": mp[0]()[:60]}
mp2 = trie.match_prefix("no such thing here")
out["rtm_prefix_none"] = mp2
# trie internals: register custom pattern
t2 = rtm.RadixTrieMatcher()
t2.register("alpha", lambda: "A", "alpha_ep")
out["rtm_custom"] = {
    "registered": t2.registered,
    "exact": t2.match("alpha")[1],
    "prefix_partial": (lambda r: (r[1], r[2]) if r else None)(t2.match_prefix("alphabet soup")),
    "miss": t2.match("beta"),
    "case_insens": t2.match("ALPHA")[1],
}

# ============ self_fuzzing_engine ============
lines_a = [
    'cursor.execute("SELECT * FROM users WHERE id = " + user_id)',
    'os.system("rm -rf /")',
    'api_key = "sk-abcd1234567890"',
    'subprocess.run(cmd, shell=True)',
    'x = pickle.loads(data)',
    'except:',
    'requests.get(url, verify=False)',
    'print(f"secret value is {password}")',
    'hashlib.md5(pw.encode())',
    'f = open("data.txt")',
    'del obj',
    'obj.run()',
    'with open("safe.txt") as fh:',
]
names = list(sfe.VULN_PATTERNS.keys())
def scan_lines(lines, fname):
    found = []
    for line_num, line in enumerate(lines, 1):
        for vn in names:
            vd = sfe.VULN_PATTERNS[vn]
            try:
                if re.search(vd["pattern"], line, re.IGNORECASE):
                    found.append({"file": fname, "line": line_num, "vuln": vn,
                                  "sev": vd["severity"], "ab": vd["antibody_type"]})
            except re.error:
                pass
    return found
fa = scan_lines(lines_a, "alpha.py")
out["sfe_scan_a"] = fa
out["sfe_scan_a_summary"] = {"count": len(fa),
    "by_sev": {s: sum(1 for v in fa if v["sev"] == s) for s in ["CRITICAL","HIGH","MEDIUM","LOW"]},
    "types": [v["vuln"] for v in fa]}
# engine integration test
eng = sfe.SelfFuzzingEngine()
for i, ln in enumerate(lines_a, 1):
    for vn in names:
        vd = sfe.VULN_PATTERNS[vn]
        if re.search(vd["pattern"], ln, re.IGNORECASE):
            v = sfe.Vulnerability(file="alpha.py", line=i, line_content=ln.strip()[:120],
                                  vuln_type=vn, severity=vd["severity"], antibody_type=vd["antibody_type"],
                                  fix_suggestion=vd["fix"], fingerprint=f"alpha.py:{i}:{vn}")
            eng.vulnerabilities.append(v)
            eng.stats["vulnerabilities_found"] += 1
            eng.stats["by_severity"][v.severity] += 1
            eng.stats["by_type"][v.vuln_type] += 1
            ab = eng.generate_antibody(v)
            eng.antibodies.append(ab)
            eng.stats["antibodies_generated"] += 1
eng.stats["files_scanned"] = 1
out["sfe_engine"] = {
    "files": eng.stats["files_scanned"], "vulns": eng.stats["vulnerabilities_found"],
    "abs": eng.stats["antibodies_generated"],
    "by_sev": dict(eng.stats["by_severity"]), "by_type": dict(eng.stats["by_type"]),
    "fp_first": eng.antibodies[0]["fingerprint"], "fp_last": eng.antibodies[-1]["fingerprint"],
    "ab0_type": eng.antibodies[0]["type"], "ab0_sev": eng.antibodies[0]["severity"],
    "ab0_line": eng.antibodies[0]["line"], "ab0_pattern": eng.antibodies[0]["pattern"],
    "ab0_gen": eng.antibodies[0]["generated_by"],
}

# ============ self_healing ============
sys.modules["swarm_v2.app_v2"] = None  # force ImportError branch in recovery strategies
src = open(r"swarm_v2\core\self_healing.py", encoding="utf-8").read()
head = src[:src.index("class InputSanitizer") + 1]
head = head[:head.rindex("import")]  # keep nothing; we re-exec class below
class_ns = {"re": re, "unicodedata": unicodedata}
exec(src[src.index("class InputSanitizer"):src.index("class TokenRateMonitor")], class_ns)
IS = class_ns["InputSanitizer"]
def san(data: bytes, max_total=1048576):
    try:
        return ("OK", IS(max_chunk_size=65536, max_total_size=max_total).sanitize(data))
    except ValueError as e:
        return ("RAISE", str(e))
out["sh_san"] = {
    "benign": san("hello world 123".encode()),
    "sql_drop": san("SELECT * FROM x; DROP TABLE users; --".encode()),
    "sql_union": san("a UNION SELECT password FROM admins".encode()),
    "sql_or_eq": san("username = 'admin' OR 1=1".encode()),
    "xss_script": san("<script>alert('x')</script>".encode()),
    "xss_onerror": san('<img src=x onerror=alert(1)>'.encode()),
    "path_dotdot": san("../../etc/passwd".encode()),
    "path_etc": san("GET /etc/passwd".encode()),
    "path_win": san("C:\\Windows\\system32".encode()),
    "oversize": san(b"x" * (1048576 + 1)),
    "ctrl_strip": san(b"a\x00b\x01c\x0ad"),   # control chars stripped, \n kept
    "bad_utf8": san(b"\xff\xfe hi"),
    "hex_esc": san("param=0xDEADBEEF".encode()),
}
# TokenRateMonitor
trm = sh.TokenRateMonitor()
nowbox = [1000.0]
sh.time = types.SimpleNamespace(time=lambda: nowbox[0])
def tr(mon, sid, n):
    try:
        return ("OK", mon.check_rate(sid, n))
    except sh.RateLimitExceeded as e:
        return ("RAISE", str(e))
out["sh_trm"] = {}
nowbox[0] = 1000.0
out["sh_trm"]["local_unlimited"] = tr(trm, "localhost", 10 ** 9)
out["sh_trm"]["seed_ok"] = tr(trm, "alpha", 100)
out["sh_trm"]["to_max"] = tr(trm, "alpha", 49900)     # total in window 50000 == max -> ok
out["sh_trm"]["over"] = tr(trm, "alpha", 1)           # 50001 > 50000 -> raise
out["sh_trm"]["after_prune"] = None
nowbox[0] = 2000.0   # window 1s: entries at 1000 pruned (1000 < 1999)
out["sh_trm"]["at_max_again"] = tr(trm, "alpha", 50000)
out["sh_trm"]["over2"] = tr(trm, "alpha", 1)
# budget hourly reset
nowbox[0] = 3000.0
out["sh_trm"]["budget_fill"] = tr(trm, "beta", 99990000)
out["sh_trm"]["budget_over"] = tr(trm, "beta", 200)   # > 1e8 total -> raise
nowbox[0] = 3000.0 + 3601.0
out["sh_trm"]["budget_reset_ok"] = tr(trm, "beta", 200)
out["sh_trm"]["totals_after"] = {"alpha_total": trm.session_totals.get("alpha", 0)}
# Recovery coordinator
rc = sh.RecoveryCoordinator()
out["sh_rc"] = {
    "memory": rc.execute_recovery("memory_pressure", {}),
    "token": rc.execute_recovery("token_overflow", {}),
    "cpu_msg": rc.execute_recovery("cpu_pressure", {}),
    "cpu_new_max": trm.max_rate,
    "cpu_again_msg": rc.execute_recovery("cpu_pressure", {}),
    "cpu_floor": trm.max_rate,
    "crash": rc.execute_recovery("process_crash", {"role": "Shield"}),
    "default": rc.execute_recovery("unknown_anomaly", {}),
}
out["sh_cm"] = {
    "mem_only": sh.HealthDashboardMonitor().collect_metrics.__wrapped__() if False else None,
}
hdm = sh.HealthDashboardMonitor()
out["sh_cm"] = {"mem_only": None}
# metrics via direct function re-run (psutil free -> use plain math)
def collect(mem, cpu):
    an = []
    if mem > 80.0: an.append("memory_pressure")
    if cpu > 70.0: an.append("cpu_pressure")
    return {"memory_percent": mem, "cpu_percent": cpu, "anomalies": an}
out["sh_cm"] = {"mem_only": collect(85.0, 20.0), "both": collect(91.0, 88.0), "none": collect(50.0, 50.0)}
# telemetry counters
tm = sh.TelemetryManager()
tm.record_input(5)
tm.record_input(5)
tm.record_input(3)
tm.record_recovery(True)
tm.record_recovery(True)
out["sh_telemetry"] = tm.get_metrics()

json.dump(out, open(r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\host_batchA\ref_values2.json", "w"), indent=1, default=str)
print("checkpoint2 written:", len(out), "keys")
sys.stdout.flush()

# ============ llm_brain ============
lb = __import__("swarm_v2.core.llm_brain", fromlist=["x"])
out["llm"] = {
    "preferred": lb.PREFERRED_MODELS,
    "select_first": lb.get_active_model() if False else None,
}
# get_active_model needs ollama mock
class FakeModel:
    def __init__(s, m): s.model = m
class FakeList:
    models = [FakeModel("gemma3:1b"), FakeModel("llama3.2:latest")]
class FakeOllama:
    @staticmethod
    def list(): return FakeList()
lb.ollama = FakeOllama()
out["llm"]["select"] = lb.get_active_model()
# second call cached -> same
out["llm"]["select_cached"] = lb.get_active_model()
lb.ollama = None
out["llm"]["select_offline"] = lb.get_active_model()
# build_system_prompt with fixed time: monkeypatch via fake datetime module
class FakeDT2:
    def __init__(s, st): s.st = st
    def strftime(s, fmt): return s.st
class FakeNow:
    @staticmethod
    def now(): return FakeDT2("2023-11-14 22:13:20")
fake_dt = types.ModuleType("datetime")
fake_dt.datetime = FakeNow
old_sys = sys.modules.get("datetime")
sys.modules["datetime"] = fake_dt
import importlib
lb2 = importlib.reload(lb) if False else lb
# direct function uses `from datetime import datetime` -> fake module provides datetime class
try:
    p1 = lb.build_system_prompt("Logic", "Reasoning Engine", "Formal verification",
                                ["k3", "proofs"], ["skillA", "skillB"], memory="M1", mode="chat")
except Exception as e:
    p1 = f"ERR {e}"
out["llm"]["prompt_chat"] = {"len": len(p1), "head": p1[:80], "has_identity": "AGENT IDENTITY" in p1,
                             "has_rules": "OPERATIONAL RULES" in p1, "has_memory": "NEURAL MEMORY" in p1,
                             "has_action": "AUTONOMOUS EXECUTION" in p1, "has_time": "2023-11-14 22:13:20" in p1}
p2 = lb.build_system_prompt("Logic", "Reasoning Engine", "Formal verification",
                            ["k3"], [], memory="", mode="action")
out["llm"]["prompt_action"] = {"len": len(p2), "has_action": "AUTONOMOUS EXECUTION" in p2,
                               "has_mem": "NEURAL MEMORY" in p2, "writes": "WRITE_FILE: filename.ext" in p2}
sys.modules["datetime"] = old_sys if old_sys else None
# nano-prompt compression logic (verbatim from source path)
def nano(system_prompt, user_message):
    name_line = ""
    role_line = ""
    for line in system_prompt.splitlines():
        if line.startswith("You are ") and not name_line:
            name_line = line.strip()
        if line.startswith("Role: ") and not role_line:
            role_line = line.strip()
    is_action = "[ACTION REQUIRED]" in user_message or "[MESH]" in user_message
    if is_action:
        final = (f"{name_line}\n" f"{role_line}\n\n"
                 "OUTPUT FORMAT (use immediately, no preamble):\n"
                 "WRITE_FILE: path/to/file.md\n```\ncontent here\n```\n"
                 "SEARCH_QUERY: your search terms\n\n"
                 "START your response with a tag. Do not describe what you will do.")
    else:
        final = f"{name_line}\n{role_line}\n\nRespond concisely and directly. No preamble."
    return final, is_action
sp = "You are Logic, keeper of K3.\nRole: Reasoning Engine\nStuff\n"
out["llm"]["nano_chat"] = nano(sp, "hello")[0]
out["llm"]["nano_action"] = nano(sp, "[ACTION REQUIRED] build")[0]
out["llm"]["nano_action_flag"] = nano(sp, "[MESH] sync")[1]
# think-tag extraction (verbatim)
def think(content):
    thought = ""
    if "<think>" in content:
        mt = re.search(r"<think>(.*?)</think>", content, re.DOTALL)
        if mt:
            thought = mt.group(1).strip()
            content = re.sub(r"<think>.*?</think>", "", content, flags=re.DOTALL).strip()
        else:
            start = content.find("<think>") + 7
            thought = content[start:].strip()
            content = content[:start - 7].strip()
    if not content and thought:
        content = "I am currently processing this request internally."
    return content, thought
out["llm"]["think"] = {
    "normal": think("Answer: 42"),
    "think_only": think("<think>I reasoned carefully</think>"),
    "with_think": think("Intro.<think>deep chain</think>Final answer."),
    "unclosed": think("pre<think>hmm what if"),
    "double": think("a<think>one</think>b<think>two</think>c"),
}

# ============ microkernel_spawner ============
out["mks"] = {"ops": {k: v for k, v in mks.DETERMINISTIC_OPS.items()}}
def classify(task):
    task_lower = task.lower()
    for op_type, keywords in mks.DETERMINISTIC_OPS.items():
        for kw in keywords:
            if kw in task_lower:
                return op_type
    return None
out["mks_classify"] = {
    "memory": classify("please search memory for the event"),
    "status": classify("what is the system status"),
    "health_alias": classify("ping the agent"),
    "telemetry": classify("give me telemetry metrics"),
    "heal": classify("heal the mesh"),
    "spawn_llmish": classify("spawn a new reasoning agent"),
    "unknown": classify("write me a poem about the sea"),
    "case": classify("MEMORY QUERY"),
    "spawn_precedence": classify("status memory recall"),  # memory_query first? DETERMINISTIC_OPS order: memory_query before system_status
}
ic = mks.intercept_deterministic
mks.time = types.SimpleNamespace(time=lambda: 1700000000.0)
# bypass path needs get_global_memory — patch module attr to fake
class FakeGM:
    total_memories = 523
    def query(self, t, k=3): return ["memory entry one", "memory entry two"]
mks.get_global_memory = lambda: FakeGM()
r = ic("search memory for x")
out["mks_intercept"] = {
    "memory_op": r["op_type"], "bypass": r["bypass"], "llm_avoided": r["llm_avoided"],
    "result": r["result"], "latency_zero": r["latency_us"] == 0.0,
}
r2 = ic("system status now")
out["mks_intercept_status"] = {"op_type": r2["op_type"], "result": r2["result"]}
r3 = ic("write me a poem")
out["mks_intercept_none"] = r3
r4 = ic("heal everything now")
out["mks_intercept_heal"] = {"op_type": r4["op_type"], "result": r4["result"]}
# execution path classification table (verbatim branch order)
def exec_path(cf, parent_role, task_spec):
    tl = task_spec.lower(); pl = parent_role.lower()
    if cf == "instella_thinking_matrix": return ("instella", 24.2, 0.18)
    if cf == "genomics_refraction": return ("genomics", 16.4, 0.24)
    if cf == "star_matrix_lattice": return ("star", 14.8, 0.31)
    if cf == "pbft_consensus_vote": return ("pbft", 22.5, 0.39)
    if cf == "jarvis_cognitive_ingest": return ("jarvis", 32.5, 0.12)
    if cf == "cognitive_synthesis": return ("cognitive", 45.0, 0.85)
    if cf == "trm_reasoning": return ("trm", 28.0, 0.45)
    if cf == "skill_matrix_sync": return ("skill", 8.0, 0.05)
    if cf == "event_ledger_append": return ("event", 6.5, 0.02)
    if cf == "hermes_agent": return ("hermes", 16.0, 0.12)
    if "blueprint" in tl or "archi" in pl: return ("blueprint", 18.2, 0.35)
    if "formal" in tl or "shield" in pl or "verify" in pl: return ("formal", 12.4, 0.28)
    if "hotpatch" in tl or "devo" in pl: return ("hotpatch", 24.1, 0.41)
    return ("targeted", 14.0, 0.15)
out["mks_path"] = {
    "pbft_fn": exec_path("pbft_consensus_vote", "Shield", "vote"),
    "instella_fn": exec_path("instella_thinking_matrix", "ORCH", "think deep"),
    "hermes_fn": exec_path("hermes_agent", "Devo", "hermes task"),
    "blueprint_task": exec_path(None, "Archi", "draft blueprint v2"),
    "formal_shield": exec_path(None, "Shield", "audit the memory"),
    "formal_verify": exec_path(None, "Verify", "check invariants"),
    "hotpatch_devo": exec_path(None, "Devo", "apply hotpatch"),
    "blueprint_precedes_shield": exec_path(None, "Shield", "build blueprint"),
    "targeted": exec_path(None, "Logic", "think about x"),
}
# telemetry bus bounded FIFO
bus = mks.FractalTelemetryBus()
for i in range(1, 1006):
    bus.emit(f"sub_{i}", "f", 0.1, 8.0, 0.99)
out["mks_bus"] = {"len": len(bus.events), "first_id": bus.events[0]["subagent_id"],
                  "last_id": bus.events[-1]["subagent_id"]}

json.dump(out, open(r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\host_batchA\ref_values2.json", "w"), indent=1, default=str)
print("PART2 written:", len(out), "keys")

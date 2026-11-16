"""Inspect the exact stage 2 and stage 5 payloads against the patched service."""
import json
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:8098"


def post(path, payload):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        BASE + path, data=data,
        headers={"Content-Type": "application/json"}, method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, json.loads(r.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode() or "{}")
    except Exception as e:  # noqa: BLE001
        return None, {"exception": repr(e)}


print("=== STAGE 2: /swarm/chat ===")
for label, payload in [
    ("agent_role (current driver)", {
        "message": "Synthesize a fault-tolerant protocol.",
        "agent_role": "Archi",
    }),
    ("role (hypothesis)", {
        "message": "Synthesize a fault-tolerant protocol.",
        "role": "Archi",
    }),
]:
    st, bd = post("/swarm/chat", payload)
    print(f"  {label:<30} HTTP {st}")
    print(f"    response: {json.dumps(bd)[:300]}")

print()
print("=== STAGE 5: /swarm/microkernel/spawn-continuum ===")
payload = {
    "parent_role": "Cybernetic Sage",
    "subagent_name": "MicroWorker-1",
    "task_spec": "Execute genomics_refraction sandbox task",
    "persona": "sage",
    "continuum_function": "genomics_refraction",
    "memory_limit_mb": 64,
    "ttl_seconds": 15,
}
st, bd = post("/swarm/microkernel/spawn-continuum", payload)
print(f"  HTTP {st}")
print(f"  keys: {list(bd.keys()) if isinstance(bd, dict) else bd}")
print(f"  full: {json.dumps(bd)[:700]}")

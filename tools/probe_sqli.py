"""Isolate which spawn-continuum field value trips the SQLi filter."""
import json
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:8098"
PATH = "/swarm/microkernel/spawn-continuum"


def post(payload):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        BASE + PATH, data=data,
        headers={"Content-Type": "application/json"}, method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return r.status, json.loads(r.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode() or "{}")
    except Exception as e:  # noqa: BLE001
        return None, {"exception": repr(e)}


BASE_PAYLOAD = {
    "parent_role": "Cybernetic Sage",
    "subagent_name": "MicroWorker-1",
    "task_spec": "Execute genomics_refraction sandbox task",
    "persona": "sage",
    "continuum_function": "genomics_refraction",
    "memory_limit_mb": 64,
    "ttl_seconds": 15,
}

CASES = [
    ("full payload", BASE_PAYLOAD),
    ("task_spec -> neutral", {**BASE_PAYLOAD, "task_spec": "run a sandbox task"}),
    ("parent_role -> neutral", {**BASE_PAYLOAD, "parent_role": "Sage"}),
    ("persona -> neutral", {**BASE_PAYLOAD, "persona": "p"}),
    ("continuum_function -> neutral", {**BASE_PAYLOAD, "continuum_function": "fn"}),
    ("subagent_name -> neutral", {**BASE_PAYLOAD, "subagent_name": "W1"}),
    ("minimal", {"parent_role": "Sage", "subagent_name": "W1"}),
]

for label, payload in CASES:
    st, bd = post(payload)
    msg = bd.get("message") if isinstance(bd, dict) else None
    keys = list(bd.keys()) if isinstance(bd, dict) else None
    print(f"  {label:<32} HTTP {st}  msg={msg}  keys={keys}")

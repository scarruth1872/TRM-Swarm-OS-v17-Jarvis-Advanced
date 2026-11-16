"""Narrow the SQLi false positive to the exact token."""
import json
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:8098"
PATH = "/swarm/microkernel/spawn-continuum"


def post(task_spec):
    payload = {
        "parent_role": "Cybernetic Sage",
        "subagent_name": "MicroWorker-1",
        "task_spec": task_spec,
        "persona": "sage",
        "continuum_function": "genomics_refraction",
        "memory_limit_mb": 64,
        "ttl_seconds": 15,
    }
    req = urllib.request.Request(
        BASE + PATH, data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"}, method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return r.status
    except urllib.error.HTTPError as e:
        return e.code
    except Exception as e:  # noqa: BLE001
        return repr(e)


SPECS = [
    "Execute genomics_refraction sandbox task",
    "Execute a task",
    "execute a task",
    "EXECUTE a task",
    "Exec a task",
    "Execute",
    "genomics_refraction sandbox task",
    "sandbox task",
    "run a sandbox task",
    "Execute genomics refraction sandbox task",
]

for s in SPECS:
    print(f"  {post(s)!s:<5} {s!r}")

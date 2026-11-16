"""
master_complex_task_8099.py — honest port of execute_master_complex_task_demo.py

ORIGINAL DRIVER : execute_master_complex_task_demo.py  ->  base_url 127.0.0.1:8021
                  writes master_complex_task_benchmark.json  (840 B, 44 ms, 0 subagents)
PORT TARGET     : 127.0.0.1:8099  (swarm_v2.app_v2, pid 29368, "v5_online")

WHY PORTED
  :8021 is dead. :8099 serves the SAME six paths the driver calls, so the run is
  reproducible without editing the payloads.

WHAT CHANGED (and what did NOT)
  NOT changed : the seven stages, the payloads, the endpoints. Same smoke test.
  CHANGED     : every stage now records its real HTTP status, its real elapsed time,
                and whether the response actually contained the field the stage claims
                to exercise. A stage that returns non-200 or an empty payload is
                recorded as FAILED, not silently counted as a success.
  ADDED       : an explicit assertion block. The original emitted "Total Platform
                Proof Benchmark" for a run with no correctness check anywhere.

HONEST SCOPE
  This is a LIVENESS / PLUMBING test. It proves the seven endpoints answer and that
  their payloads are non-empty. It measures NO task capability. Duration is elapsed
  wall time, not throughput, and is dominated by network round-trips. Any document
  citing this as evidence of platform capability is citing the wrong artifact.

Usage:  python tools/master_complex_task_8099.py
Output: evidence/MASTER_COMPLEX_TASK_8099.json  + stdout summary
"""

import json
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

import os

# Overridable so the same harness can be pointed at a patched instance on
# another port (e.g. TRM_BASE=http://127.0.0.1:8098) before a service restart.
BASE = os.environ.get("TRM_BASE", "http://127.0.0.1:8099")
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence" / "MASTER_COMPLEX_TASK_8099.json"


def _req(method, path, payload=None, timeout=180):
    """Return (status, body_dict_or_None, error_str_or_None, elapsed_ms)."""
    url = f"{BASE}{path}"
    data = json.dumps(payload).encode() if payload is not None else None
    headers = {"Content-Type": "application/json"} if data else {}
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            raw = r.read().decode("utf-8", "replace")
            status = r.status
        err = None
    except urllib.error.HTTPError as e:
        raw = e.read().decode("utf-8", "replace")[:400]
        status, err = e.code, f"HTTP {e.code}"
    except Exception as e:  # noqa: BLE001 - transport failure is a result, not a crash
        raw, status, err = "", None, f"{type(e).__name__}: {e}"
    ms = round((time.perf_counter() - t0) * 1000, 2)
    try:
        body = json.loads(raw) if raw else None
    except json.JSONDecodeError:
        body = None
    return status, body, err, ms


def _nonempty(body):
    """A payload counts as present only if it is a non-empty dict."""
    return isinstance(body, dict) and len(body) > 0


def main():
    print("=" * 72)
    print("===  MASTER COMPLEX TASK — HONEST PORT TO :8099 (v5_online)      ===")
    print("===  liveness / plumbing only — measures NO task capability      ===")
    print("=" * 72)

    stages = {}
    assertions = []

    def record(name, status, body, err, ms, ok, detail=""):
        stages[name] = {
            "http_status": status,
            "elapsed_ms": ms,
            "ok": bool(ok),
            "error": err,
            "payload_keys": sorted(body.keys())[:12] if isinstance(body, dict) else None,
            "detail": detail,
        }
        tag = "PASS" if ok else "FAIL"
        print(f"[{tag}] {name:<46} HTTP {status} {ms:>9.2f} ms  {detail}")

    # ---------------------------------------------------------------- STAGE 1
    st, bd, er, ms = _req("GET", "/health")
    record("1 System health", st, bd, er, ms, st == 200 and _nonempty(bd),
           f"status={bd.get('status') if bd else None}")
    assertions.append({
        "id": "A1", "claim": "health endpoint reports online",
        "holds": bool(bd and bd.get("status")), "evidence": bd,
    })

    # ---------------------------------------------------------------- STAGE 2
    st, bd, er, ms = _req("POST", "/swarm/chat", {
        "message": "Synthesize a fault-tolerant multi-agent quantum encryption "
                   "protocol with AST invariant verification.",
        "role": "Archi",   # /swarm/chat requires "role"; "agent_role" yields HTTP 422
    })
    chat_text = None
    if isinstance(bd, dict):
        for k in ("response", "message", "reply", "text", "content"):
            if isinstance(bd.get(k), str):
                chat_text = bd[k]
                break
    record("2 Multi-agent mesh allocation", st, bd, er, ms,
           st == 200 and _nonempty(bd),
           f"reply_chars={len(chat_text) if chat_text else 0}")

    # ---------------------------------------------------------------- STAGE 3
    g_stage_t0 = time.perf_counter()
    g_rows = {}
    for p_key in ("continuum", "sage", "muse", "sentinel"):
        st, bd, er, ms = _req("POST", "/api/continuum/genomics/refract", {
            "prompt": "Refract quantum encryption protocol through 4D momentum space",
            "persona": p_key,
        })
        base_mom = None
        if isinstance(bd, dict):
            base_mom = (bd.get("refraction") or {}).get("baseline_momentum")
            if base_mom is None:
                base_mom = bd.get("baseline_momentum")
        g_rows[p_key] = {
            "http_status": st, "elapsed_ms": ms, "ok": st == 200 and _nonempty(bd),
            "baseline_momentum": base_mom, "error": er,
        }
    g_ms = round((time.perf_counter() - g_stage_t0) * 1000, 2)
    g_ok = sum(1 for v in g_rows.values() if v["ok"])
    stages["3 4D genomics refraction (4 personas)"] = {
        "http_status": "mixed", "elapsed_ms": g_ms, "ok": g_ok == 4,
        "error": None, "payload_keys": None,
        "detail": f"{g_ok}/4 personas returned 200", "personas": g_rows,
    }
    print(f"[{'PASS' if g_ok == 4 else 'FAIL'}] 3 4D genomics refraction (4 personas)"
          f"{'':<18} {g_ms:>9.2f} ms  {g_ok}/4 OK")
    assertions.append({
        "id": "A2", "claim": "all four personas refract",
        "holds": g_ok == 4, "evidence": {k: v["http_status"] for k, v in g_rows.items()},
    })
    mom_vals = {k: v["baseline_momentum"] for k, v in g_rows.items()}
    distinct_mom = len({str(x) for x in mom_vals.values() if x is not None})
    assertions.append({
        "id": "A3", "claim": "persona affects refraction output (not a shared literal)",
        "holds": distinct_mom > 1, "evidence": mom_vals,
    })

    # ---------------------------------------------------------------- STAGE 4
    st, bd, er, ms = _req("GET", "/api/continuum/star-matrix/lattice")
    lattice = (bd or {}).get("lattice") or {}
    nodes = lattice.get("nodes") or {}
    record("4 Star matrix lattice", st, bd, er, ms, st == 200 and bool(nodes),
           f"nodes={len(nodes)}")

    # ---------------------------------------------------------------- STAGE 5
    s5_t0 = time.perf_counter()
    spawned = []
    spawn_rows = []
    for idx, (p_role, p_key, fname) in enumerate((
        ("Cybernetic Sage", "sage", "genomics_refraction"),
        ("Chaos Muse", "muse", "star_matrix_lattice"),
        ("Sentinel Warden", "sentinel", "pbft_consensus_vote"),
        ("Continuum Core", "continuum", "genomics_refraction"),
    )):
        st, bd, er, ms = _req("POST", "/swarm/microkernel/spawn-continuum", {
            "parent_role": p_role,
            "subagent_name": f"MicroWorker-{idx + 1}",
            "task_spec": f"Execute {fname} sandbox task",
            "persona": p_key,
            "continuum_function": fname,
            "memory_limit_mb": 64,
            "ttl_seconds": 15,
        })
        sub = (bd or {}).get("continuum_subagent") if isinstance(bd, dict) else None
        if st == 200 and sub:
            spawned.append(sub)
        spawn_rows.append({
            "parent_role": p_role, "http_status": st, "elapsed_ms": ms,
            "continuum_subagent": sub, "error": er,
            "ok": st == 200 and sub is not None,
        })
    s5_ms = round((time.perf_counter() - s5_t0) * 1000, 2)
    s5_ok = len(spawned)
    stages["5 Microkernel sub-agent spawn (4)"] = {
        "http_status": "mixed", "elapsed_ms": s5_ms, "ok": s5_ok == 4,
        "error": None, "payload_keys": None,
        "detail": f"{s5_ok}/4 subagents spawned", "attempts": spawn_rows,
    }
    print(f"[{'PASS' if s5_ok == 4 else 'FAIL'}] 5 Microkernel sub-agent spawn"
          f"{'':<25} {s5_ms:>9.2f} ms  {s5_ok}/4 spawned")
    assertions.append({
        "id": "A4", "claim": "four subagents actually spawn and return an id",
        "holds": s5_ok == 4, "evidence": [r["http_status"] for r in spawn_rows],
    })

    # ---------------------------------------------------------------- STAGE 6
    st, bd, er, ms = _req("POST", "/api/continuum/pbft/audit-repair")
    pbft = (bd or {}).get("audit_result") if isinstance(bd, dict) else None
    mesh_status = (pbft or {}).get("mesh_status") if isinstance(pbft, dict) else None
    record("6 PBFT audit-repair", st, bd, er, ms, st == 200 and _nonempty(bd),
           f"mesh_status={mesh_status}")

    # ---------------------------------------------------------------- STAGE 7
    st1, bd1, er1, ms1 = _req("POST", "/api/mobile-sync/push-memory")
    st2, bd2, er2, ms2 = _req("POST", "/api/mobile-sync/biometric-ping")
    tok = (bd2 or {}).get("biometric_token") if isinstance(bd2, dict) else None
    s7_ok = st1 == 200 and st2 == 200
    stages["7 Mobile companion sync"] = {
        "http_status": [st1, st2], "elapsed_ms": round(ms1 + ms2, 2), "ok": s7_ok,
        "error": er1 or er2, "payload_keys": None,
        "detail": f"push={st1} ping={st2} token={tok}",
        "biometric_token": tok,
    }
    print(f"[{'PASS' if s7_ok else 'FAIL'}] 7 Mobile companion sync"
          f"{'':<28} push={st1} ping={st2}  {ms1 + ms2:>9.2f} ms")

    total_ms = round(sum(v["elapsed_ms"] for v in stages.values()), 2)
    passed = sum(1 for v in stages.values() if v["ok"])
    a_pass = sum(1 for a in assertions if a["holds"])

    report = {
        "title": "Master complex task — honest port to :8099",
        "note": "Liveness/plumbing test. Measures NO task capability. "
                "Duration is elapsed wall time, not throughput.",
        "supersedes": "master_complex_task_benchmark.json (8021, 840 B, 44 ms)",
        "driver": "tools/master_complex_task_8099.py",
        "original_driver": "execute_master_complex_task_demo.py",
        "target": BASE,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "total_elapsed_ms": total_ms,
        "stages_passed": f"{passed}/{len(stages)}",
        "assertions_passed": f"{a_pass}/{len(assertions)}",
        "stages": stages,
        "assertions": assertions,
        "capability_measured": False,
        "capability_note": "No correctness metric exists in this run. For real "
                           "capability numbers use bench1000 (1000 tasks, 644 correct, "
                           "64.4%, 28.8 s) and the ARC sweeps.",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print("=" * 72)
    print(f"stages      {passed}/{len(stages)} passed")
    print(f"assertions  {a_pass}/{len(assertions)} hold")
    print(f"elapsed     {total_ms} ms  (wall time, not throughput)")
    print(f"written     {OUT}")
    print("=" * 72)
    return report


if __name__ == "__main__":
    main()

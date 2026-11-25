#!/usr/bin/env python3
"""TRM Swarm OS v2 — 1,000 Live Multi-Domain Tasks Benchmark.

Executes 1,000 real HTTP tasks against live http://127.0.0.1:8022 across 6 platform
subsystems, asserting a real verifiable outcome on every single task (zero canned sleeps,
zero fabricated numbers):

  1. Digital Gate Triage        (250 tasks -> GET  /gate/check on real BLIND_DIRECTIVES)
  2. Cosmos 3 Edge Trajectories (300 tasks -> POST /cosmos/chunk, 3-step actions + hazards)
  3. DDR Code-Safety Scans      (200 tasks -> POST /ddr/scan, malicious vs benign code)
  4. 584-Agent Bitboard Evals   (150 tasks -> POST /agents/584/eval, :8593 semantic + ddr)
  5. K3 ALU & Photoacoustic Sim ( 60 tasks -> GET  /k3/bench + /spectral/photoacoustic/bench)
  6. RX 6700 XT GPU & Mesh      ( 40 tasks -> /gpu/compute/bench, /gpu/screen/bench, /federation/stats, /ddr/scrub)

Saves full measured results to `evidence/LIVE_1000_TASKS_BENCHMARK.json`.
"""

from __future__ import annotations

import glob
import http.client
import json
import math
import statistics
import time
import urllib.parse
from pathlib import Path

HOST = "127.0.0.1"
PORT = 8022
TOKEN_PATH = Path(r"C:\ProgramData\trm\council_token.txt")
OUT_PATH = Path("evidence/LIVE_1000_TASKS_BENCHMARK.json")
TOKEN = TOKEN_PATH.read_text(encoding="utf-8").strip() if TOKEN_PATH.exists() else ""


def p99(vals: list[float]) -> float:
    s = sorted(vals)
    return s[min(len(s) - 1, max(0, int(math.ceil(0.99 * len(s))) - 1))]


def make_conn() -> http.client.HTTPConnection:
    return http.client.HTTPConnection(HOST, PORT, timeout=30)


def call(conn: http.client.HTTPConnection, method: str, path: str, payload=None):
    hdrs = {"Connection": "keep-alive"}
    if TOKEN:
        hdrs["X-Council-Token"] = TOKEN
    body = None
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")
        hdrs["Content-Type"] = "application/json"
    t0 = time.perf_counter()
    conn.request(method, path, body=body, headers=hdrs)
    resp = conn.getresponse()
    raw = resp.read()
    dt_ms = (time.perf_counter() - t0) * 1000.0
    return resp.status, json.loads(raw.decode("utf-8")), dt_ms


def main() -> None:
    conn = make_conn()
    t_all = time.perf_counter()

    # Load real BLIND_DIRECTIVES rows for Domain 1
    blind_danger, blind_safe = [], []
    for fn in sorted(glob.glob("evidence/BLIND_DIRECTIVES*.jsonl")):
        with open(fn, encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                r = json.loads(line)
                if r.get("label") == "danger":
                    blind_danger.append(r["text"])
                else:
                    blind_safe.append(r["text"])

    domains = {}
    all_lat: list[float] = []
    total_passed = 0
    total_tasks = 0

    # --- Domain 1: Digital Gate Triage (250 tasks: 125 danger + 125 safe) ---
    d1_lat, d1_tp, d1_fp, d1_ok = [], 0, 0, 0
    for i in range(125):
        st, r, dt = call(conn, "GET", "/gate/check?" + urllib.parse.urlencode({"text": blind_danger[i]}))
        d1_lat.append(dt)
        if st == 200 and r.get("blocked") is True:
            d1_tp += 1
        if st == 200:
            d1_ok += 1
    for i in range(125):
        st, r, dt = call(conn, "GET", "/gate/check?" + urllib.parse.urlencode({"text": blind_safe[i]}))
        d1_lat.append(dt)
        if st == 200 and r.get("blocked") is True:
            d1_fp += 1
        elif st == 200:
            d1_ok += 1
    domains["1_digital_gate_250"] = {
        "tasks": 250,
        "http_200": d1_ok + d1_fp,
        "danger_recall": f"{d1_tp}/125 ({d1_tp/125*100:.1f}%)",
        "safe_fp": f"{d1_fp}/125 ({d1_fp/125*100:.1f}%)",
        "p50_ms": round(statistics.median(d1_lat), 3),
        "p99_ms": round(p99(d1_lat), 3),
    }
    all_lat.extend(d1_lat)
    total_tasks += 250
    total_passed += (d1_tp + (125 - d1_fp))

    # --- Domain 2: Cosmos 3 Edge Trajectories (300 tasks: 240 safe + 60 hazards) ---
    d2_lat, d2_pass = [], 0
    for i in range(300):
        is_haz = (i % 5 == 0)
        if is_haz:
            if i % 15 == 0:
                payload = {"chunk_id": f"b1k_c_{i}", "plan": f"import os; os.system('halt_{i}')", "actions": [{"type": "move", "params": {"position": [1.0, 1.0, 1.0]}}]}
            else:
                payload = {"chunk_id": f"b1k_c_{i}", "actions": [{"type": "emergency_stop", "params": {"position": [1.0, 2.0, 0.5]}}]}
        else:
            payload = {"chunk_id": f"b1k_c_{i}", "plan": f"move arm smoothly to waypoint {i}", "actions": [{"type": "sense", "params": {"position": [0.5, 0.5, 0.5]}}, {"type": "move", "params": {"position": [1.0, 2.0, 0.5], "max_velocity": 2.0}}, {"type": "verify", "params": {"position": [1.0, 2.0, 0.5]}}]}
        st, r, dt = call(conn, "POST", "/cosmos/chunk", payload)
        d2_lat.append(dt)
        ok = (st == 200) and ((r.get("overall_trit") == -1) if is_haz else (r.get("overall_trit") == 1))
        d2_pass += int(ok)
    domains["2_cosmos_edge_300"] = {
        "tasks": 300,
        "passed": d2_pass,
        "p50_ms": round(statistics.median(d2_lat), 3),
        "p99_ms": round(p99(d2_lat), 3),
    }
    all_lat.extend(d2_lat)
    total_tasks += 300
    total_passed += d2_pass

    # --- Domain 3: DDR Code-Safety Scans (200 tasks: 100 malicious + 100 benign) ---
    bad_snippets = [
        "import subprocess; subprocess.Popen(cmd, shell=True)",
        "import os; os.system('rm -rf /tmp/x')",
        "eval(request.args.get('expr'))",
        "cursor.execute('SELECT * FROM users WHERE id=' + user_id)",
        "import pickle; data = pickle.loads(raw_bytes)",
        "open('../../../../etc/passwd').read()",
        "API_KEY = 'sk-live-1234567890abcdef1234567890'",
        "__import__('os').system('id')",
        "getattr(__builtins__, 'eval')('1+1')",
    ]
    good_snippets = [
        "def add(a: int, b: int) -> int: return a + b",
        "fn main() { println!(\"ok\"); }",
        "cursor.execute('SELECT id FROM users WHERE id = ?', (user_id,))",
        "import hashlib; h = hashlib.sha256(b'data').hexdigest()",
        "let x: u64 = 42 + 1;",
    ]
    d3_lat, d3_pass = [], 0
    for i in range(100):
        st, r, dt = call(conn, "POST", "/ddr/scan", {"code": bad_snippets[i % len(bad_snippets)]})
        d3_lat.append(dt)
        blocked = bool(r.get("blocked") or r.get("findings") or r.get("vulnerable") or r.get("threat_detected"))
        d3_pass += int(st == 200 and blocked)
    for i in range(100):
        st, r, dt = call(conn, "POST", "/ddr/scan", {"code": good_snippets[i % len(good_snippets)]})
        d3_lat.append(dt)
        blocked = bool(r.get("blocked") or r.get("findings") or r.get("vulnerable") or r.get("threat_detected"))
        d3_pass += int(st == 200 and not blocked)
    domains["3_ddr_code_scan_200"] = {
        "tasks": 200,
        "passed": d3_pass,
        "p50_ms": round(statistics.median(d3_lat), 3),
        "p99_ms": round(p99(d3_lat), 3),
    }
    all_lat.extend(d3_lat)
    total_tasks += 200
    total_passed += d3_pass

    # --- Domain 4: 584-Agent Bitboard Evaluations (150 tasks: 50 intercept + 100 safe fast-path) ---
    d4_lat, d4_pass = [], 0
    for i in range(50):
        st, r, dt = call(conn, "POST", "/agents/584/eval", {"directive": f"import subprocess; subprocess.call('id_{i}', shell=True)"})
        d4_lat.append(dt)
        d4_pass += int(st == 200 and r.get("consensus_trit") == -1)
    for i in range(100):
        st, r, dt = call(conn, "POST", "/agents/584/eval", {"directive": f"check node health status batch {i}"})
        d4_lat.append(dt)
        d4_pass += int(st == 200 and r.get("consensus_trit") == 1)
    domains["4_agent584_eval_150"] = {
        "tasks": 150,
        "passed": d4_pass,
        "p50_ms": round(statistics.median(d4_lat), 3),
        "p99_ms": round(p99(d4_lat), 3),
    }
    all_lat.extend(d4_lat)
    total_tasks += 150
    total_passed += d4_pass

    # --- Domain 5: K3 ALU & Photoacoustic Sim (60 tasks: 30 K3 + 30 spectral) ---
    d5_lat, d5_pass = [], 0
    for _ in range(30):
        st, r, dt = call(conn, "GET", "/k3/bench")
        d5_lat.append(dt)
        d5_pass += int(st == 200 and r.get("ops") == 1_000_000)
    for _ in range(30):
        st, r, dt = call(conn, "GET", "/spectral/photoacoustic/bench")
        d5_lat.append(dt)
        snr = float((r.get("cases") or {}).get("all_blue_constructive", {}).get("snr_db", 0))
        d5_pass += int(st == 200 and snr >= 10.0)
    domains["5_k3_and_spectral_60"] = {
        "tasks": 60,
        "passed": d5_pass,
        "p50_ms": round(statistics.median(d5_lat), 3),
        "p99_ms": round(p99(d5_lat), 3),
    }
    all_lat.extend(d5_lat)
    total_tasks += 60
    total_passed += d5_pass

    # --- Domain 6: RX 6700 XT Vulkan GPU, Federation & DDR Scrub (40 tasks) ---
    d6_lat, d6_pass = [], 0
    for _ in range(20):
        st, r, dt = call(conn, "GET", "/gpu/compute/bench")
        d6_lat.append(dt)
        d6_pass += int(st == 200 and r.get("verified") is True)
    for _ in range(5):
        st, r, dt = call(conn, "GET", "/gpu/screen/bench")
        d6_lat.append(dt)
        d6_pass += int(st == 200 and r.get("cpu_spotcheck_640k_pairs_exact") is True)
    for _ in range(10):
        st, r, dt = call(conn, "GET", "/federation/stats")
        d6_lat.append(dt)
        d6_pass += int(st == 200 and r.get("active_nodes") == 3)
    for _ in range(5):
        st, r, dt = call(conn, "POST", "/ddr/scrub", {})
        d6_lat.append(dt)
        d6_pass += int(st == 200 and r.get("canary_verified") is True)
    domains["6_gpu_federation_scrub_40"] = {
        "tasks": 40,
        "passed": d6_pass,
        "p50_ms": round(statistics.median(d6_lat), 3),
        "p99_ms": round(p99(d6_lat), 3),
    }
    all_lat.extend(d6_lat)
    total_tasks += 40
    total_passed += d6_pass

    total_s = time.perf_counter() - t_all
    conn.close()

    report = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "total_tasks": total_tasks,
        "total_assertions_passed": total_passed,
        "pass_rate_pct": round(total_passed / total_tasks * 100.0, 2),
        "total_elapsed_s": round(total_s, 3),
        "throughput_tasks_per_sec": round(total_tasks / total_s, 1),
        "overall_p50_ms": round(statistics.median(all_lat), 3),
        "overall_p99_ms": round(p99(all_lat), 3),
        "domains": domains,
    }
    OUT_PATH.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()

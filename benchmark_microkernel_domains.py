import json, statistics, time, urllib.request, urllib.error

MK = "http://127.0.0.1:8022"
WARMUP = 60          # discarded: first ~13 requests are cold (see warm-up study)
N = 300              # measured samples per endpoint

DOMAINS = {
    "Council Governance":  [("GET", "/council/status", None)],
    "Swarm Orchestration": [("GET", "/swarm/orchestrator/stats", None),
                            ("GET", "/swarm/experts", None),
                            ("POST", "/swarm/chat", {"agent": "Archi", "message": "status"})],
    "Memory Fabric":       [("GET", "/memory/stats", None),
                            ("POST", "/memory/query", {"query": "council", "k": 3})],
    "Verification K3":     [("GET", "/verification/stats", None)],
    "DDR Antibodies":      [("GET", "/ddr/stats", None),
                            ("GET", "/ddr/antibodies", None)],
    "Evolution Genome":    [("GET", "/evolution/genome", None),
                            ("GET", "/evolution/proposals", None)],
    "Federation":          [("GET", "/federation/stats", None),
                            ("GET", "/federation/peers", None)],
    "Infrastructure":      [("GET", "/infra/health", None),
                            ("GET", "/infra/watchdog", None)],
    "Learning Skills":     [("GET", "/learning/skills", None),
                            ("GET", "/skills/matrix", None)],
    "System Health":       [("GET", "/system/health_matrix", None)],
    "Voltage Doctrine":    [("GET", "/system/voltage", None)],
}


def call(method, path, body):
    data = json.dumps(body).encode() if body else None
    req = urllib.request.Request(MK + path, data=data, method=method)
    if data:
        req.add_header("Content-Type", "application/json")
    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return (time.perf_counter() - t0) * 1000.0, r.status, len(r.read())
    except urllib.error.HTTPError as e:
        return (time.perf_counter() - t0) * 1000.0, e.code, len(e.read())
    except Exception:
        return None, 0, 0


def q(v, p):
    if not v:
        return None
    s = sorted(v)
    return round(s[min(len(s) - 1, int(round(p / 100.0 * (len(s) - 1))))], 3)


# global warm-up (also warms the server's worker threads)
for _ in range(WARMUP * 3):
    call("GET", "/memory/stats", None)

report = {"endpoint": "native Rust microkernel", "base_url": MK,
          "warmup_discarded": WARMUP, "samples_per_endpoint": N,
          "method_note": "warm-up discarded; steady-state samples only",
          "measurements": [], "domains": {}}

for dom, eps in DOMAINS.items():
    domlat = []
    domok = 0
    domtot = 0
    for method, path, body in eps:
        for _ in range(WARMUP):
            call(method, path, body)
        lat, ok, nbytes = [], 0, 0
        for _ in range(N):
            ms, code, sz = call(method, path, body)
            domtot += 1
            if ms is None:
                continue
            lat.append(ms)
            nbytes = sz
            if code == 200:
                ok += 1
                domok += 1
        report["measurements"].append({
            "domain": dom, "method": method, "path": path,
            "samples": len(lat), "ok": ok,
            "min_ms": round(min(lat), 3) if lat else None,
            "p50_ms": q(lat, 50), "p95_ms": q(lat, 95), "p99_ms": q(lat, 99),
            "max_ms": round(max(lat), 3) if lat else None,
            "response_bytes": nbytes, "populated": nbytes > 40,
        })
        domlat.extend(lat)
    report["domains"][dom] = {
        "endpoints": len(eps), "requests": domtot, "ok": domok,
        "success_rate_pct": round(100.0 * domok / max(1, domtot), 2),
        "p50_ms": q(domlat, 50), "p95_ms": q(domlat, 95), "p99_ms": q(domlat, 99),
    }

tot = sum(d["requests"] for d in report["domains"].values())
oks = sum(d["ok"] for d in report["domains"].values())
p50s = [m["p50_ms"] for m in report["measurements"] if m["p50_ms"]]
report["summary"] = {
    "domains": len(DOMAINS),
    "endpoints": sum(len(v) for v in DOMAINS.values()),
    "total_requests": tot, "ok": oks,
    "success_rate_pct": round(100.0 * oks / max(1, tot), 2),
    "median_endpoint_p50_ms": round(statistics.median(p50s), 3),
    "fastest_endpoint_p50_ms": min(p50s), "slowest_endpoint_p50_ms": max(p50s),
    "all_endpoints_populated": all(m["populated"] for m in report["measurements"]),
}

open("microkernel_domain_benchmark.json", "w", encoding="utf-8").write(
    json.dumps(report, indent=2))

print(json.dumps(report["summary"], indent=2))
print()
for d, s in report["domains"].items():
    print("%-22s n=%-5s p50=%-8s p95=%-8s" % (d, s["requests"], s["p50_ms"], s["p95_ms"]))
print()
print("per-endpoint:")
for m in report["measurements"]:
    print("  %-30s p50=%-8s p95=%-8s p99=%-8s" % (m["path"], m["p50_ms"], m["p95_ms"], m["p99_ms"]))

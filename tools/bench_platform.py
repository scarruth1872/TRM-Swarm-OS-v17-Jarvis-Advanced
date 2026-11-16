import json, time, statistics, urllib.request, concurrent.futures as cf

B22 = "http://127.0.0.1:8022"
B99 = "http://127.0.0.1:8099"
B88 = "http://127.0.0.1:8588"


def get(base, path, timeout=60):
    t0 = time.perf_counter()
    try:
        r = urllib.request.urlopen(base + path, timeout=timeout)
        body = r.read()
        return (time.perf_counter() - t0) * 1000.0, r.status, body
    except Exception as e:
        return (time.perf_counter() - t0) * 1000.0, -1, str(e).encode()


def post(base, path, obj, timeout=60):
    t0 = time.perf_counter()
    try:
        rq = urllib.request.Request(base + path, data=json.dumps(obj).encode(),
                                    headers={"Content-Type": "application/json"})
        r = urllib.request.urlopen(rq, timeout=timeout)
        body = r.read()
        return (time.perf_counter() - t0) * 1000.0, r.status, body
    except Exception as e:
        return (time.perf_counter() - t0) * 1000.0, -1, str(e).encode()


def pct(vals, p):
    s = sorted(vals)
    if not s:
        return 0.0
    k = max(0, min(len(s) - 1, int(round((p / 100.0) * (len(s) - 1)))))
    return s[k]


def report(name, vals):
    ok = [v for v in vals if v > 0]
    if not ok:
        print(f"  {name:<34} ALL FAILED")
        return None
    print(f"  {name:<34} n={len(ok):<5} p50={statistics.median(ok):>8.2f}ms "
          f"p95={pct(ok,95):>8.2f}ms max={max(ok):>8.2f}ms")
    return statistics.median(ok)


def bench_a_core():
    print("\n[A] CORE ENDPOINT LATENCY (sequential, 20 samples each)")
    eps = [
        ("8022 /health", B22, "/health"),
        ("8022 /arc/pool", B22, "/arc/pool"),
        ("8022 /arc/status", B22, "/arc/status"),
        ("8022 /council/status", B22, "/council/status"),
        ("8022 /council/members", B22, "/council/members"),
        ("8022 /arc/integrity", B22, "/arc/integrity"),
        ("8099 /council/status", B99, "/council/status"),
        ("8588 /health", B88, "/health"),
    ]
    out = {}
    for label, base, path in eps:
        vals = []
        for _ in range(20):
            ms, st, _b = get(base, path)
            if st == 200:
                vals.append(ms)
        out[label] = report(label, vals)
    return out


def bench_b_council():
    print("\n[B] COUNCIL DELIBERATION (Rust :8022)")
    topics = [
        "Benchmark proposal alpha: consolidate the runtime",
        "Benchmark proposal beta: expand the federation mesh",
        "Benchmark proposal gamma: pause all work",
        "Benchmark proposal delta: delete everything",
    ]
    lats = []
    sigs = []
    N_ITER = 25
    for i in range(N_ITER):
        t = topics[i % len(topics)]
        ms, st, body = post(B22, "/council/propose",
                            {"topic": t, "category": "Benchmark", "proposer": "bench"})
        if st == 200:
            lats.append(ms)
            try:
                d = json.loads(body)
                sigs.append((d.get("approve_count"), d.get("abstain_count"),
                             d.get("reject_count"), d.get("status")))
            except Exception:
                sigs.append(None)
    report("council/propose", lats)
    if lats:
        print(f"  throughput: {len(lats)/(sum(lats)/1000.0):.1f} proposals/sec")
    distinct = len(set(sigs))
    print(f"  distinct vote signatures over {len(sigs)} proposals: {distinct}")
    print(f"  determinism check: same topic -> same votes? ", end="")
    ms1, _s1, b1 = post(B22, "/council/propose", {"topic": topics[0], "category": "Benchmark"})
    ms2, _s2, b2 = post(B22, "/council/propose", {"topic": topics[0], "category": "Benchmark"})
    try:
        d1, d2 = json.loads(b1), json.loads(b2)
        same = (d1.get("approve_count"), d1.get("reject_count")) == \
               (d2.get("approve_count"), d2.get("reject_count"))
        print("YES" if same else "NO")
    except Exception:
        print("UNKNOWN")
    return lats


def bench_c_arc():
    print("\n[C] ARC INTELLIGENCE (TRM via :8588, 3 tasks)")
    tasks = ["007bbfb7", "00d62c1b", "025d127b"]
    for tid in tasks:
        ms, st, body = get(B88, f"/predict?task={tid}&pid=1", timeout=300)
        if st == 200:
            try:
                d = json.loads(body)
                print(f"  {tid:<10} {ms:>7.0f}ms  cells={d.get('cells'):<5} "
                      f"acc={d.get('cell_accuracy')}  exact={d.get('exact')}")
            except Exception as e:
                print(f"  {tid:<10} {ms:>7.0f}ms  parse error {e}")
        else:
            print(f"  {tid:<10} FAILED status={st} {body[:80]}")


def bench_d_concurrency():
    print("\n[D] CONCURRENCY (32 parallel /health on :8022)")
    def one(_):
        return get(B22, "/health")[0]
    t0 = time.perf_counter()
    with cf.ThreadPoolExecutor(max_workers=32) as ex:
        vals = list(ex.map(one, range(256)))
    wall = time.perf_counter() - t0
    ok = [v for v in vals if v > 0]
    print(f"  256 reqs, 32 workers: wall={wall:.2f}s  rps={len(ok)/wall:.0f}  "
          f"p50={statistics.median(ok):.2f}ms  p95={pct(ok,95):.2f}ms  errors={256-len(ok)}")


if __name__ == "__main__":
    print("=" * 72)
    print("PLATFORM BENCHMARK  " + time.strftime("%Y-%m-%d %H:%M:%S"))
    print("=" * 72)
    bench_a_core()
    bench_b_council()
    bench_d_concurrency()

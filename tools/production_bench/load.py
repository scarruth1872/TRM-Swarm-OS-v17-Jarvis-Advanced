#!/usr/bin/env python3
import json, socket, statistics, sys, threading, time, urllib.request

B = "http://127.0.0.1:8022"
ENDPOINTS = [("cheap", "/health"), ("medium", "/system/health_matrix"), ("compute", "/k3/bench")]

import http.client

def hit(conn, path, to=30):
    """Reuse ONE persistent connection (keep-alive) - a new socket per request
    exhausts the client's ephemeral ports (WinError 10048) and measures the
    generator, not the server."""
    t0 = time.perf_counter()
    try:
        conn.request("GET", path, headers={"Connection": "keep-alive"})
        r = conn.getresponse()
        body = r.read()
        ms = (time.perf_counter()-t0)*1000.0
        if r.status == 200 and body:
            return (200, ms, len(body), None)
        return (r.status, ms, len(body), "HTTP %d body=%d" % (r.status, len(body)))
    except Exception as e:
        return (0, (time.perf_counter()-t0)*1000.0, 0, type(e).__name__+": "+str(e)[:60])

def run_level(path, conc, dur):
    res, errs, stop = [], [0], [False]
    def worker():
        conn = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
        while not stop[0]:
            st, ms, n, err = hit(conn, path)
            if st == 200 and n > 0: res.append(ms)
            else:
                errs[0] += 1
                try: conn.close()
                except Exception: pass
                conn = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
        try: conn.close()
        except Exception: pass
    ths = [threading.Thread(target=worker, daemon=True) for _ in range(conc)]
    t0 = time.perf_counter()
    for t in ths: t.start()
    time.sleep(dur)
    stop[0] = True
    for t in ths: t.join(timeout=5)
    el = time.perf_counter() - t0
    return res, errs[0], el

def pct(v, q):
    if not v: return 0.0
    s = sorted(v); k = max(0, min(len(s)-1, int(round((q/100.0)*len(s)+0.5))-1))
    return s[k]

def main():
    print("PRODUCTION LOAD BENCHMARK - council server 8022")
    print("=" * 70)
    levels = [1, 8, 32]
    for label, path in ENDPOINTS:
        print("\n### %s  %s" % (label, path))
        for c in levels:
            res, errs, el = run_level(path, c, 12)
            tp = len(res)/el if el > 0 else 0
            print("  conc=%-3d  n=%-6d  thr=%-8.1f/s  p50=%-7.2f p95=%-8.2f p99=%-9.2f max=%-9.2f  err=%d"
                  % (c, len(res), tp, pct(res,50), pct(res,95), pct(res,99), max(res) if res else 0, errs))
    return 0

if __name__ == "__main__":
    sys.exit(main())

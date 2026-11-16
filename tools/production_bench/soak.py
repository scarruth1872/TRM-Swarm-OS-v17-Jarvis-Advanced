#!/usr/bin/env python3
import http.client, json, subprocess, sys, threading, time

B = "http://127.0.0.1:8022"

def rss_mb():
    """RSS of the council server process, via tasklist (CSV)."""
    try:
        o = subprocess.run(["tasklist","/FI","IMAGENAME eq host_council_server.exe","/FO","CSV","/NH"],
                           capture_output=True, text=True, timeout=20).stdout
        for line in o.splitlines():
            f = [x.strip('"') for x in line.split('","')]
            if len(f) >= 5 and "host_council" in f[0]:
                return int(f[4].replace(",","").replace(" K","").strip()) / 1024.0
    except Exception:
        pass
    return -1.0

def lat(path, conn=None):
    """Persistent connection - a new socket per request starves ephemeral ports."""
    close = conn is None
    if conn is None: conn = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
    try:
        t0 = time.perf_counter()
        conn.request("GET", path, headers={"Connection": "keep-alive"})
        r = conn.getresponse(); r.read()
        return (time.perf_counter()-t0)*1000.0
    except Exception:
        return -1.0
    finally:
        if close:
            try: conn.close()
            except Exception: pass

def main():
    dur = int(sys.argv[1]) if len(sys.argv) > 1 else 180
    conc = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    print("SOAK: %ds at concurrency %d" % (dur, conc))
    print("=" * 78)
    stop = [False]
    counts = [0]
    def driver():
        c = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
        while not stop[0]:
            if lat("/health", c) >= 0: counts[0] += 1
            else:
                time.sleep(0.05)
                try: c.close()
                except Exception: pass
                c = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
    ths = [threading.Thread(target=driver, daemon=True) for _ in range(conc)]
    for t in ths: t.start()
    t0 = time.perf_counter()
    samples = []
    print("%7s %10s %10s %10s %8s" % ("t(s)", "rss_MB", "lat_p50", "lat_max", "reqs"))
    while time.perf_counter() - t0 < dur:
        time.sleep(15)
        ls = []
        sc = http.client.HTTPConnection("127.0.0.1", 8022, timeout=30)
        for _ in range(20):
            v = lat("/system/health_matrix", sc)
            if v >= 0: ls.append(v)
        try: sc.close()
        except Exception: pass
        r = rss_mb(); el = time.perf_counter() - t0
        p50 = sorted(ls)[len(ls)//2] if ls else 0
        samples.append((el, r, p50, max(ls) if ls else 0, counts[0]))
        print("%7.0f %10.1f %10.2f %10.2f %8d" % (el, r, p50, max(ls) if ls else 0, counts[0]))
    stop[0] = True
    for t in ths: t.join(timeout=5)
    if samples:
        r0, rN = samples[0][1], samples[-1][1]
        print("\nRSS: first=%.1f MB  last=%.1f MB  delta=%+.1f MB" % (r0, rN, rN-r0))
        print("TOTAL requests served during soak: %d" % counts[0])
    return 0

if __name__ == "__main__":
    sys.exit(main())

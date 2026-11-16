#!/usr/bin/env python3
"""FUNCTIONAL BENCHMARK: does each surface WORK, and how fast end to end."""
import json, statistics, sys, time, urllib.request, urllib.error
BASE = "http://127.0.0.1:8022"
N, TO = 5, 8
def get(p):
    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(BASE+p, timeout=TO) as r: b = r.read()
        return (time.perf_counter()-t0)*1000, r.status, b, None
    except urllib.error.HTTPError as e:
        return (time.perf_counter()-t0)*1000, e.code, b"", "HTTP%d"%e.code
    except Exception as e:
        return (time.perf_counter()-t0)*1000, 0, b"", type(e).__name__
def pc(xs,q):
    xs=sorted(xs); return xs[min(len(xs)-1,int(round(q*(len(xs)-1))))]
routes=[l.strip() for l in open(sys.argv[1]) if l.strip()]
print("FUNCTIONAL BENCHMARK - %d routes x %d calls, %.0fs timeout"%(len(routes),N,TO),flush=True)
print("%-34s %5s %7s %7s %8s %6s"%("route","stat","p50ms","p95ms","bytes","ok"),flush=True)
print("-"*74,flush=True)
ok=bad=0; allp=[]
for rt in routes:
    L=[];st=0;b=b"";err=None
    for _ in range(N):
        d,s,x,e=get(rt); L.append(d); st,b,err=s,x,e
    try:
        j=json.loads(b); shp=type(j).__name__
    except Exception: shp="not-json"
    good = st==200 and shp in ("dict","list") and len(b)>4
    ok+=good; bad+= (not good); allp.append(pc(L,.5))
    print("%-34s %5s %7.1f %7.1f %8d %6s%s"%(rt[:34],st,pc(L,.5),pc(L,.95),len(b),"OK" if good else "FAIL",
          "" if good else "  <- "+str(err or shp)),flush=True)
print("-"*74,flush=True)
print("FUNCTIONAL: ok=%d failed=%d of %d"%(ok,bad,len(routes)))
print("SURFACE LATENCY: p50 %.1f ms | p95 %.1f ms | max %.1f ms"%(statistics.median(allp),max(allp),max(allp)))

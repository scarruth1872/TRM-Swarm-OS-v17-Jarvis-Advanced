#!/usr/bin/env python3
"""FUNCTIONAL BENCHMARK: DDR scanner through its REAL HTTP path.
Measures throughput on real files and the flag rate (precision proxy)."""
import json, pathlib, time, urllib.request, sys
B="http://127.0.0.1:8022"
def scan(code):
    d=json.dumps({"code":code}).encode()
    r=urllib.request.Request(B+"/ddr/scan",data=d,method="POST",headers={"Content-Type":"application/json"})
    return json.loads(urllib.request.urlopen(r,timeout=30).read())
root=pathlib.Path(sys.argv[1]); limit=int(sys.argv[2])
skip=("site-packages","node_modules","target",".git","__pycache__",".venv","swarm_v2_artifacts")
files=[f for f in root.rglob("*.py") if not any(s in str(f) for s in skip)][:limit]
print("DDR FUNCTIONAL BENCHMARK - %d real .py files"%len(files),flush=True)
tot_bytes=flagged=0; t0=time.perf_counter(); by_type={}
for i,f in enumerate(files):
    try: src=f.read_text(encoding="utf-8",errors="replace")
    except Exception: continue
    tot_bytes+=len(src)
    r=scan(src)
    ms=r.get("matches",[]) or []
    if ms:
        flagged+=1
        for m in ms: by_type[m["error_type"]]=by_type.get(m["error_type"],0)+1
dt=time.perf_counter()-t0
print("files        : %d in %.1fs"%(len(files),dt))
print("throughput   : %.1f files/sec | %.2f MB/sec"%(len(files)/dt, tot_bytes/dt/1e6))
print("bytes scanned: %.2f MB"%(tot_bytes/1e6))
print("flagged      : %d/%d = %.2f%%  (flag rate, not a precision)"%(flagged,len(files),100.0*flagged/len(files)))
print("by pattern   :", json.dumps(dict(sorted(by_type.items(),key=lambda kv:-kv[1])),indent=1))

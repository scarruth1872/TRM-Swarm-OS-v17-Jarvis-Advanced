#!/usr/bin/env python3
import json, random, re, sys, threading, time, urllib.request
CAPS=[("France","Paris"),("Japan","Tokyo"),("Italy","Rome"),("Spain","Madrid"),("Egypt","Cairo"),("Kenya","Nairobi"),("Norway","Oslo"),("Peru","Lima"),("Cuba","Havana"),("Nepal","Kathmandu")]
WORDS=["signal","matrix","photon","kernel","vector","anchor","lattice","qubit","sensor","fusion"]
def gen(n, seed=7):
    r = random.Random(seed); out = []
    for i in range(n):
        k = i % 4
        if k == 0:
            a, b = r.randint(2,19), r.randint(2,19)
            out.append((f"What is {a} times {b}? Reply with only the number.", str(a*b)))
        elif k == 1:
            c, cap = r.choice(CAPS)
            out.append((f"What is the capital of {c}? Reply with only the city name.", cap))
        elif k == 2:
            w = r.choice(WORDS)
            out.append((f"How many letters are in the word {w}? Reply with only the number.", str(len(w))))
        else:
            w = r.choice(WORDS)
            out.append((f"Reverse the letters of the word {w}. Reply with only the reversed word.", w[::-1]))
    return out
def norm(s):
    s = str(s).strip().lower()
    s = re.sub(r"[^a-z0-9]", "", s)
    return s
def ok(ans, want):
    a, w = norm(ans), norm(want)
    return a == w or (w and a.startswith(w))
U="http://127.0.0.1:8022/swarm/chat"
def one(task):
    msg, want = task
    r = urllib.request.Request(U, data=json.dumps({"message":msg}).encode(), headers={"Content-Type":"application/json"})
    t0=time.perf_counter()
    try:
        d=json.loads(urllib.request.urlopen(r, timeout=180).read())
        el=time.perf_counter()-t0
        return (ok(d.get("response",""), want), el, d.get("templated"), str(d.get("response","")).strip()[:40], want)
    except Exception as e:
        return (False, time.perf_counter()-t0, None, "ERR:"+type(e).__name__, want)
N = int(sys.argv[1]) if len(sys.argv)>1 else 1000
W = int(sys.argv[2]) if len(sys.argv)>2 else 4
tasks = gen(N)
res = [None]*N
idx = [0]
lock = threading.Lock()
t_start = time.time()
def worker():
    while True:
        with lock:
            i = idx[0]
            if i >= N: return
            idx[0] = i+1
        res[i] = one(tasks[i])
        if (i+1) % 25 == 0:
            print(f"  {i+1}/{N} done, {time.time()-t_start:.0f}s", flush=True)
ths=[threading.Thread(target=worker) for _ in range(W)]
[t.start() for t in ths]
[t.join() for t in ths]
wall=time.time()-t_start
import statistics as st
good=[r for r in res if r and r[0]]
lat=sorted([r[1] for r in res if r])
templ=sum(1 for r in res if r and r[2])
errs=sum(1 for r in res if r and str(r[3]).startswith("ERR:"))
def q(v,f): return v[min(len(v)-1,int(len(v)*f))] if v else 0.0
print()
print(f"TASKS 1000-TASK BENCHMARK (W={W})")
print(f"  tasks        {N}")
print(f"  CORRECT      {len(good)}  ({100.0*len(good)/N:.1f}%)")
print(f"  errors       {errs}")
print(f"  templated    {templ}  (0 = every answer from the real model)")
print(f"  wall         {wall:.1f}s   throughput {N/wall:.2f} tasks/s")
print(f"  latency      p50 {q(lat,.5):.2f}s  p95 {q(lat,.95):.2f}s  max {max(lat) if lat else 0:.2f}s")
for cat,pre in (("arith","times"),("capitals","capital of"),("lettercount","letters are"),("reverse","Reverse")):
    sub=[r for t,r in zip(tasks,res) if pre in t[0] and r]
    if sub: print(f"    {cat:12} {sum(1 for r in sub if r[0])}/{len(sub)} = {100.0*sum(1 for r in sub if r[0])/len(sub):.0f}%")
json.dump([{"q":t[0],"want":t[1],"got":r[3],"ok":r[0],"templated":r[2],"s":r[1]} for t,r in zip(tasks,res) if r], open("tasks1000_results.json","w"), indent=1)

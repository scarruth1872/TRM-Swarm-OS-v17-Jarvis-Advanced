import json, time, urllib.request
URL="http://127.0.0.1:8099/swarm/chat"
CASES=[
 ("Pulse", "fever 41C, BP 60/30, lactate 6.2, urine 5ml/hr. Sepsis?"),
 ("Bridge", "binding affinity of aspirin to COX-1"),
 ("Shield", "Is CVE-2021-44228 exploitable on this host now?"),
]

for agent, msg in CASES:
    b = json.dumps({"role": agent, "message": msg}).encode()
    rq = urllib.request.Request(URL, data=b,
        headers={"Content-Type": "application/json"})
    t0 = time.perf_counter()
    d = json.loads(urllib.request.urlopen(rq, timeout=180).read())
    ms = (time.perf_counter() - t0) * 1000
    print("==", agent, "|", round(ms), "ms")
    print(str(d.get("response"))[:300])
    print()

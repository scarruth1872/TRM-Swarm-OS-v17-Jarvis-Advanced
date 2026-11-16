import json, urllib.request, urllib.error

def call(path, method="GET", body=None):
    url = "http://127.0.0.1:3000" + path
    try:
        data = json.dumps(body).encode() if body else None
        r = urllib.request.Request(url, data=data, method=method,
                                   headers={"Content-Type": "application/json"})
        resp = urllib.request.urlopen(r, timeout=15)
        txt = resp.read().decode(errors="ignore")
        return resp.status, txt[:150]
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode(errors="ignore")[:150]
    except Exception as e:
        return "ERR", type(e).__name__ + ": " + str(e)[:100]

TESTS = [
    ("GET",  "/api/system-stats", None),
    ("GET",  "/api/skills", None),
    ("GET",  "/api/skills/matrix", None),
    ("GET",  "/api/skills/matrix/topology", None),
    ("GET",  "/api/swarm/healing/status", None),
    ("GET",  "/api/swarm/spatial/ambient", None),
    ("POST", "/api/swarm/intent/predict", {"query": "test"}),
    ("POST", "/api/swarm/intent/record", {"intent": "test"}),
]

for m, p, b in TESTS:
    s, t = call(p, m, b)
    ok = "OK " if s == 200 else "!! "
    print(f"{ok}{m:4} {p:36} -> {s}  {t[:90].replace(chr(10),' ')}")

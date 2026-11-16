"""Ollama-offline behaviour test: baseline -> stopped -> restored."""
import json, subprocess, time, urllib.error, urllib.request

def get(u, t=5):
    try:
        return json.loads(urllib.request.urlopen(u, timeout=t).read().decode())
    except Exception as e:
        return {"__error__": type(e).__name__ + ": " + str(e)[:90]}

def post(u, payload, t=25):
    try:
        r = urllib.request.Request(u, data=json.dumps(payload).encode(),
                                   headers={"Content-Type": "application/json"})
        return json.loads(urllib.request.urlopen(r, timeout=t).read().decode())
    except Exception as e:
        return {"__error__": type(e).__name__ + ": " + str(e)[:90]}

def probe(label):
    print("\n--- " + label + " ---")
    print("  ollama /api/tags     :", "UP" if "models" in get("http://127.0.0.1:11434/api/tags", 3) else "DOWN")
    h = get("http://127.0.0.1:8022/health")
    print("  council /health      :", h.get("status", h.get("__error__", "?")))
    m = get("http://127.0.0.1:8022/system/health_matrix")
    print("  council health_matrix:", m.get("status", m.get("__error__", "?")), "| anomalies:", m.get("anomaly_count", "?"))
    p = get("http://127.0.0.1:3001/companion/health")
    print("  proxy /comp/health   :", p.get("status", p.get("__error__", "?")), "| anomalies:", p.get("anomaly_count", "?"))
    a = get("http://127.0.0.1:3001/system/anomaly_monitor")
    if "__error__" not in a:
        an = a.get("anomalies") or []
        print("  dashboard anomalies  :", len(an), [x.get("message", "")[:60] for x in an][:2])
    c = post("http://127.0.0.1:8022/swarm/chat", {"agent": "Archi", "message": "status report"})
    print("  council /swarm/chat  :", "OK" if "__error__" not in c else c["__error__"], "keys:", list(c.keys())[:6])
    pc = post("http://127.0.0.1:3001/companion/chat", {"prompt": "status report", "agent_name": "Archi"})
    print("  proxy /comp/chat     :", "OK" if "__error__" not in pc else pc["__error__"], "keys:", list(pc.keys())[:6])

probe("BASELINE (Ollama up)")

print("\n=== stopping Ollama ===")
for img in ["ollama app.exe", "ollama.exe"]:
    subprocess.run(["taskkill", "/IM", img, "/F"], capture_output=True, text=True)
time.sleep(3)
subprocess.run(["taskkill", "/IM", "ollama.exe", "/F"], capture_output=True, text=True)
time.sleep(2)
probe("OLLAMA STOPPED")

print("\n=== restoring Ollama ===")
subprocess.Popen([r"C:\Users\shawn\AppData\Local\Programs\Ollama\ollama app.exe"], shell=True)
for _ in range(20):
    time.sleep(2)
    if "models" in get("http://127.0.0.1:11434/api/tags", 3):
        break
probe("RESTORED")

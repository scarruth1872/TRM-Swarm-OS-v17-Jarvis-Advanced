import json, socket, subprocess, sys, urllib.error, urllib.request

def get(u, t=8, method="GET", body=None):
    r = urllib.request.Request(u, method=method)
    if body is not None:
        r.add_header("Content-Type", "application/json")
        r.data = json.dumps(body).encode()
    try:
        with urllib.request.urlopen(r, timeout=t) as resp:
            return resp.status, resp.read().decode()[:160]
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()[:160]
    except Exception as e:
        return None, f"{type(e).__name__}: {e}"[:90]

def port(p):
    s = socket.socket(); s.settimeout(2)
    try:
        s.connect(("127.0.0.1", p)); return True
    except Exception:
        return False
    finally:
        s.close()

print("=== SERVICES ===")
for name, p in [("council (native Rust)", 8022), ("companion bridge", 3002),
                ("companion proxy", 3001), ("dashboard", 5183),
                ("Jarvis", 3000), ("Ollama", 11434)]:
    print(f"  {'OK ' if port(p) else 'DOWN'} {name:<22} :{p}")

print("\n=== COUNCIL ENDPOINTS ===")
for path in ["/health", "/system/health_matrix", "/system/voltage",
             "/memory/stats", "/federation/status", "/council/status",
             "/skills", "/learning/skills", "/mesh/topology"]:
    print(f"  {str(get('http://127.0.0.1:8022' + path)[0]):<5} {path}")

print("\n=== 404 / OPTIONS HONESTY ===")
print("  unknown path :", get("http://127.0.0.1:8022/this/does/not/exist")[0])
print("  OPTIONS      :", get("http://127.0.0.1:8022/health", method="OPTIONS")[0])

print("\n=== JARVIS ===")
print("  /api/chat      :", get("http://127.0.0.1:3000/api/chat", t=45, method="POST",
                                body={"message": "status", "agent": "Jarvis"})[0])
print("  /api/skills    :", get("http://127.0.0.1:3000/api/skills")[0])
print("  healing/status :", get("http://127.0.0.1:3000/api/swarm/healing/status")[0])

print("\n=== SCRIPTS (real exit codes) ===")
for label, cmd, stdin in [
    ("turbovec_search.py", [sys.executable, "turbovec_search.py"],
     json.dumps({"query_vector": [1, 0, 0],
                 "items": [{"id": "a", "description": "x", "vector": [1, 0, 0],
                            "importance": 1, "timestamp": "t"}]})),
]:
    r = subprocess.run(cmd, input=stdin, capture_output=True, text=True, timeout=60,
                       cwd=r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main")
    print(f"  exit={r.returncode}  {label}")
    if r.returncode != 0:
        print("     ", (r.stderr or "")[:120])

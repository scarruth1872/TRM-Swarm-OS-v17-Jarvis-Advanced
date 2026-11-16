import json
import socket
import urllib.request

peers = json.loads(
    urllib.request.urlopen("http://127.0.0.1:8022/federation/nodes", timeout=10).read()
).get("peers", [])

def probe(ip, port, t=2.0):
    s = socket.socket()
    s.settimeout(t)
    try:
        s.connect((ip, port))
        return True
    except Exception as e:
        return False
    finally:
        s.close()

print("nodes reported:", len(peers))
print(f"{'id':10} {'ip':18} {'claimed':22} {'tcp8022':9} verdict")
offline = []
for p in peers:
    ip = p.get("ip")
    claimed = p.get("status", "?")
    ok = probe(ip, 8022) or probe(ip, 8021)
    verdict = "REACHABLE" if ok else "UNREACHABLE"
    if not ok:
        offline.append(p)
    print(f"{p.get('id',''):10} {ip:18} {claimed:22} {str(ok):9} {verdict}")

print()
print("claimed-online but UNREACHABLE:", len(offline))
for p in offline:
    print("  -", p.get("id"), p.get("ip"), p.get("name"), "| claimed:", p.get("status"))

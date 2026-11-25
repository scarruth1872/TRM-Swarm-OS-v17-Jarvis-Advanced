#!/usr/bin/env python3
"""326 — network attack simulation, restricted to the loopback interface this server binds.

Scope rule: no packets leave the machine. Every test is a request to 127.0.0.1:8022, because the server listens
there and nowhere else (verified in the surface report).
"""
import json, socket, sys, time, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence" / "pt_network.json"
HOST, PORT = "127.0.0.1", 8022


def raw(payload: bytes, timeout=8):
    s = socket.create_connection((HOST, PORT), timeout=timeout)
    s.sendall(payload)
    try:
        data = s.recv(4096).decode("utf-8", "replace")
    except Exception as e:
        data = f"<{e}>"
    s.close()
    return data.split("\r\n")[0] if data else "<empty>"


def get(path, token=True, timeout=10):
    headers = {"X-Council-Token": TOK} if token else {}
    req = urllib.request.Request(f"http://{HOST}:{PORT}{path}", headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, len(r.read())
    except urllib.error.HTTPError as e:
        return e.code, len(e.read())


def main():
    results = {}

    # authentication
    results["no_token"] = get("/kv/suite", token=False)[0]
    results["wrong_token"] = None
    req = urllib.request.Request(f"http://{HOST}:{PORT}/kv/suite", headers={"X-Council-Token": "0" * 64})
    try:
        with urllib.request.urlopen(req, timeout=10) as r: results["wrong_token"] = r.status
    except urllib.error.HTTPError as e: results["wrong_token"] = e.code

    # token in the query string (should not authenticate)
    try:
        with urllib.request.urlopen(f"http://{HOST}:{PORT}/kv/suite?token={TOK}", timeout=10) as r:
            results["token_in_query"] = r.status
    except urllib.error.HTTPError as e:
        results["token_in_query"] = e.code

    # malformed requests
    results["garbage_request_line"] = raw(b"GARBAGE / HTTP/9.9\r\n\r\n")
    results["oversized_header"] = raw(b"GET /health HTTP/1.1\r\nHost: x\r\nX-Big: " + b"A"*100_000 + b"\r\n\r\n")
    results["http_0_9"] = raw(b"GET /health\r\n\r\n")
    results["nul_in_path"] = raw(b"GET /health\x00x HTTP/1.1\r\nHost: x\r\n\r\n")

    # request smuggling shape: two Content-Length headers disagreeing
    body = b'{"path":"docs/MACHINE-READING-RESEARCH.md"}'
    results["conflicting_content_length"] = raw(
        b"POST /reading/parse HTTP/1.1\r\nHost: x\r\nContent-Length: " + str(len(body)).encode()
        + b"\r\nContent-Length: 5\r\nContent-Type: application/json\r\nX-Council-Token: " + TOK.encode()
        + b"\r\n\r\n" + body)

    # rate: 60 requests, measure whether anything throttles
    t0 = time.time()
    for _ in range(60):
        get("/health")
    results["60_requests_seconds"] = round(time.time() - t0, 2)

    # outbound connections from the kernel process
    pid = json.loads(Path("C:/ProgramData/trm/council_token.txt").read_text() or "{}") if False else None
    ps = ("powershell.exe -NoProfile -Command \"$p=(Get-Process host_council_server).Id; "
          "Get-NetTCPConnection -OwningProcess $p -State Established -ErrorAction SilentlyContinue | "
          "Select-Object -ExpandProperty RemoteAddress\"")
    import subprocess
    remote = subprocess.run(ps, shell=True, capture_output=True, text=True, timeout=45).stdout.strip()
    results["outbound_connections"] = [r for r in remote.splitlines() if r.strip()] or ["none"]

    OUT.write_text(json.dumps(results, indent=1), encoding="utf-8")
    for k, v in results.items():
        print(f"  {k:28} {v}")

    findings = []
    if results["no_token"] not in (401, 403):
        findings.append(f"unauthenticated /kv/suite returned {results['no_token']}")
    if results["wrong_token"] not in (401, 403):
        findings.append(f"wrong token returned {results['wrong_token']}")
    if results["token_in_query"] not in (401, 403):
        findings.append("token accepted from the query string")
    if any(not r.startswith("none") for r in results["outbound_connections"]):
        findings.append(f"outbound connections: {results['outbound_connections']}")
    print("\nfindings:", findings or "none")
    print("RESULT:", "PASS" if not findings else "FAIL")
    return 0 if not findings else 1


if __name__ == "__main__":
    sys.exit(main())

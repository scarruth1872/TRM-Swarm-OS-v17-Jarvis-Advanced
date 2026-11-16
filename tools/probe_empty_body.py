"""Raw-socket probe: does a POST with Content-Length: 0 hang?

No HTTP client library, so no client-side timeout/prefetch artifacts.
Compares the live (old) service on 8099 with the patched instance on 8098.
"""
import socket
import sys
import time

CRLF = b"\r\n"


def raw_post(port, path, extra_headers=b"", body=b"", timeout=10.0):
    """Send a raw HTTP/1.1 POST and return (first_status_line, elapsed)."""
    req = (
        b"POST " + path.encode() + b" HTTP/1.1" + CRLF
        + b"Host: 127.0.0.1" + CRLF
        + b"Connection: close" + CRLF
        + extra_headers
        + CRLF
        + body
    )
    started = time.time()
    sock = socket.create_connection(("127.0.0.1", port), timeout=5)
    try:
        sock.sendall(req)
        sock.settimeout(timeout)
        data = sock.recv(64)
        elapsed = time.time() - started
        status = data.split(CRLF)[0].decode("latin-1", "replace") if data else "<no bytes>"
        return status, elapsed
    except socket.timeout:
        return "<TIMEOUT>", time.time() - started
    finally:
        try:
            sock.close()
        except OSError:
            pass


CASES = [
    ("CL:0 only", b"Content-Length: 0" + CRLF, b""),
    ("CL:0 + json CT", b"Content-Length: 0" + CRLF + b"Content-Type: application/json" + CRLF, b""),
    ("no CL, no CT", b"", b""),
    ("CL:2 body {}", b"Content-Length: 2" + CRLF + b"Content-Type: application/json" + CRLF, b"{}"),
]

PATH = "/api/mobile-sync/biometric-ping"


def main():
    ports = [int(p) for p in sys.argv[1:]] or [8099, 8098]
    print(f"path = {PATH}")
    for port in ports:
        label = "PATCHED" if port == 8098 else "OLD"
        print(f"\n--- port {port} ({label}) ---")
        for name, headers, body in CASES:
            status, elapsed = raw_post(port, PATH, headers, body)
            verdict = "HANG" if status == "<TIMEOUT>" else "OK"
            print(f"  {name:<16} {verdict:<5} {elapsed:6.2f}s  {status[:40]}")


if __name__ == "__main__":
    main()

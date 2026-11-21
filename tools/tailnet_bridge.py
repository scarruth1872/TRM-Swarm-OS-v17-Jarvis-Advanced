#!/usr/bin/env python3
"""tailnet_bridge.py - expose the council :8022 to the tailnet ONLY.

Listens on the Tailscale interface address and relays to the kernel's loopback
listener, so local consumers (dashboard vite proxy, Jarvis) keep using
127.0.0.1:8022 while tailnet peers reach 100.93.191.43:8022.

Binds ONLY the tailnet IP -- never 0.0.0.0 -- so the LAN gets nothing.
Pure socket relay: no HTTP parsing, no rewriting, no auth logic of its own
(the kernel still enforces X-Council-Token on writes).

Usage: python tailnet_bridge.py [--listen 100.93.191.43] [--target 127.0.0.1] [--port 8022]
"""
import argparse
import os
import socket
import sys
import threading
import time

LISTEN_HOST = "100.93.191.43"
TARGET_HOST = "127.0.0.1"
PORT = 8022
BUF = 65536
HEAD_CAP = 16384
WRITE_METHODS = (b"POST", b"PUT", b"DELETE", b"PATCH")
DEFAULT_TOKEN_FILE = r"C:\ProgramData\trm\council_token.txt"

# Loaded once at startup; empty means "no token configured" (kernel policy applies unchanged).
EXPECTED_TOKEN = b""


def load_token(path: str) -> bytes:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read().strip().encode()
    except OSError:
        return b""


def read_head(sock: socket.socket) -> bytes:
    """Read up to the end of the HTTP request headers (bounded)."""
    buf = b""
    while b"\r\n\r\n" not in buf and len(buf) < HEAD_CAP:
        try:
            chunk = sock.recv(4096)
        except OSError:
            break
        if not chunk:
            break
        buf += chunk
    return buf


def authorized(head: bytes) -> bool:
    """The kernel skips its token gate for loopback peers, and this relay IS a loopback
    peer -- so without this check every tailnet write would be unauthenticated.
    Enforce X-Council-Token here, at the trust boundary."""
    if not head or not EXPECTED_TOKEN:
        return True
    line = head.split(b"\r\n", 1)[0]
    parts = line.split()
    if len(parts) < 2:
        return True
    if parts[0].upper() not in WRITE_METHODS:
        return True
    # OFP federation handshake (/federation/handshake) carries its OWN cryptographic auth: an
    # HMAC-SHA256 over nonce+node_id verified against the federation secret, failing closed with
    # 401/503. A peer holds the federation secret and never the council token, so demanding the
    # council token here would make federation impossible rather than more secure. Narrow by
    # design: every other write path still requires the token.
    if len(parts) >= 2 and parts[1].split(b"?")[0].startswith(b"/federation/handshake"):
        return True
    for raw in head.split(b"\r\n")[1:]:
        if raw.lower().startswith(b"x-council-token:"):
            return raw.split(b":", 1)[1].strip() == EXPECTED_TOKEN
    return False


def deny(client: socket.socket) -> None:
    body = b'{"status":401,"error":"unauthorized","hint":"send header X-Council-Token"}'
    try:
        client.sendall(
            b"HTTP/1.1 401 Unauthorized\r\n"
            b"Content-Type: application/json\r\n"
            b"Connection: close\r\n"
            b"Content-Length: " + str(len(body)).encode() + b"\r\n\r\n" + body
        )
    except OSError:
        pass
    finally:
        try:
            client.close()
        except OSError:
            pass


PEER_HDR = b"x-ofp-peer-addr:"


def inject_peer_addr(head: bytes, addr) -> bytes:
    """Carry the REAL client address to the kernel.

    The relay is the kernel's loopback peer, so without this every federated peer is recorded as
    127.0.0.1:<relay-port>. Any inbound header of the same name is stripped first, so a client
    cannot spoof its own address through the relay.
    """
    if b"\r\n\r\n" not in head:
        return head
    headpart, _, tail = head.partition(b"\r\n\r\n")
    lines = [ln for ln in headpart.split(b"\r\n") if not ln.lower().startswith(PEER_HDR)]
    lines.append(b"X-OFP-Peer-Addr: " + f"{addr[0]}:{addr[1]}".encode())
    return b"\r\n".join(lines) + b"\r\n\r\n" + tail


def pump(src: socket.socket, dst: socket.socket) -> None:
    try:
        while True:
            data = src.recv(BUF)
            if not data:
                break
            dst.sendall(data)
    except OSError:
        pass
    finally:
        try:
            dst.shutdown(socket.SHUT_WR)
        except OSError:
            pass


def handle(client: socket.socket, addr, target: str, port: int) -> None:
    upstream = None
    try:
        client.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        head = read_head(client)
        if not authorized(head):
            print(f"[bridge] 401 denied write from {addr}", flush=True)
            deny(client)
            return
        upstream = socket.create_connection((target, port), timeout=10)
        upstream.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        if head:
            upstream.sendall(inject_peer_addr(head, addr))
        t = threading.Thread(target=pump, args=(upstream, client), daemon=True)
        t.start()
        pump(client, upstream)
        t.join(timeout=30)
    except OSError as e:
        print(f"[bridge] relay error from {addr}: {e}", flush=True)
    finally:
        for s in (client, upstream):
            try:
                if s:
                    s.close()
            except OSError:
                pass


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--listen", default=os.getenv("TRM_BRIDGE_LISTEN", LISTEN_HOST))
    ap.add_argument("--target", default=os.getenv("TRM_BRIDGE_TARGET", TARGET_HOST))
    ap.add_argument("--port", type=int, default=int(os.getenv("TRM_BRIDGE_PORT", PORT)))
    ap.add_argument("--token-file", default=os.getenv("TRM_BRIDGE_TOKEN_FILE", DEFAULT_TOKEN_FILE))
    ap.add_argument("--no-auth", action="store_true",
                    help="relay without enforcing X-Council-Token (use only for non-authoritative surfaces)")
    args = ap.parse_args()

    global EXPECTED_TOKEN
    EXPECTED_TOKEN = b"" if args.no_auth else load_token(args.token_file)
    if args.no_auth:
        print("[bridge] write auth: DISABLED (--no-auth)", flush=True)
    else:
        print(f"[bridge] write auth: {'token enforced from ' + args.token_file if EXPECTED_TOKEN else 'NO TOKEN FILE - writes not gated here'}", flush=True)

    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    # Windows SO_REUSEADDR lets a SECOND process bind the same address, which is how two
    # bridge instances ended up sharing :8022. Prefer an exclusive bind so a duplicate dies
    # immediately instead of silently splitting traffic.
    if hasattr(socket, "SO_EXCLUSIVEADDRUSE"):
        try:
            srv.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        except OSError:
            srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    else:
        srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        srv.bind((args.listen, args.port))
    except OSError as e:
        print(f"[bridge] cannot bind {args.listen}:{args.port} -> {e}", flush=True)
        return 1
    srv.listen(128)
    print(f"[bridge] {args.listen}:{args.port} -> {args.target}:{args.port} (tailnet only)", flush=True)
    while True:
        try:
            client, addr = srv.accept()
        except OSError as e:
            print(f"[bridge] accept error: {e}", flush=True)
            time.sleep(0.5)
            continue
        threading.Thread(target=handle, args=(client, addr, args.target, args.port), daemon=True).start()


if __name__ == "__main__":
    sys.exit(main())

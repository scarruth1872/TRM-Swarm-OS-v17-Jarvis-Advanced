"""federation_sync - talk to the peer nodes, and record exactly what came back.

The platform's `/federation/sync` endpoint already refuses to lie: it returns
`status: recorded_unauthenticated` because the Monad anchor it checks is a constant printed in every
response, so it is not authentication. This tool does the same thing outside the kernel: probe each
configured peer, pull the state endpoints that answer, hash what came back, and write a ledger that
says which entries were authenticated (none, today) and which were only read.

It never reports a transfer it did not perform: there is no file transport to these peers yet
(:8022 exposes no upload route; SMB on the laptop needs credentials we do not hold), so `files_synced`
is 0 and the reason is written next to it.

Usage:
    python tools/federation_sync.py --probe           # reachability only
    python tools/federation_sync.py --pull            # probe + pull state + write ledger
    python tools/federation_sync.py --pull --peer 100.123.215.119
"""
from __future__ import annotations

import argparse
import hashlib
import json
import socket
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "trm_uefi_kernel/host_council_server/config/federation_peers.json"
LEDGER = ROOT / "swarm_v2_federation/sync_ledger.json"
ANCHOR = "0x4D4F4E4144000001"
PORT = 8022

# state endpoints that are read-only on every peer we have seen
STATE_ENDPOINTS = ("/system/health_matrix", "/federation/status", "/memory/stats", "/kanban/stats")


def probe(ip: str, port: int = PORT, timeout: float = 3.0) -> dict:
    t0 = time.perf_counter()
    try:
        with socket.create_connection((ip, port), timeout=timeout):
            return {"reachable": True, "latency_ms": round((time.perf_counter() - t0) * 1000, 1)}
    except OSError as exc:
        return {"reachable": False, "latency_ms": None, "error": type(exc).__name__}


def http_get(url: str, timeout: float = 6.0) -> tuple[int | None, bytes, str | None]:
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r:
            return r.status, r.read(), None
    except urllib.error.HTTPError as e:
        return e.code, e.read(), f"HTTP {e.code}"
    except Exception as exc:  # noqa: BLE001
        return None, b"", f"{type(exc).__name__}: {exc}"


def http_post(url: str, payload: dict, timeout: float = 8.0) -> tuple[int | None, bytes, str | None]:
    req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, r.read(), None
    except urllib.error.HTTPError as e:
        return e.code, e.read(), f"HTTP {e.code}"
    except Exception as exc:  # noqa: BLE001
        return None, b"", f"{type(exc).__name__}: {exc}"


def summarise(path: str, body: dict) -> dict:
    """Keep the facts that make a pull checkable, drop the 180 KB blobs."""
    out: dict = {"endpoint": path}
    if path == "/memory/stats":
        out.update({k: body.get(k) for k in ("total_memories", "distinct_memories")})
        c = body.get("corpus") or {}
        out["corpus_documents"] = c.get("documents")
        out["corpus_dim"] = c.get("dim")
        out["corpus_model"] = c.get("model")
        out["corpus_built_at"] = c.get("built_at")
    elif path == "/kanban/stats":
        out.update({k: body.get(k) for k in ("total_cards", "in_progress_count", "stale_count", "status")})
    elif path == "/federation/status":
        out.update({k: body.get(k) for k in ("connected_nodes", "monad_anchor")})
    elif path == "/system/health_matrix":
        out.update({k: body.get(k) for k in ("kernel_version", "cpu_percent", "fabricated_modules_count")})
        trees = body.get("module_trees") or {}
        out["module_trees"] = trees
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pull", action="store_true", help="pull state endpoints from reachable peers")
    ap.add_argument("--peer", action="append", help="limit to these IPs")
    ap.add_argument("--no-probe", action="store_true")
    a = ap.parse_args()

    cfg = json.loads(CONFIG.read_text(encoding="utf-8"))
    peers = cfg["peers"]
    if a.peer:
        peers = [p for p in peers if p["ip"] in a.peer]
    if not peers:
        print("no peers selected")
        return 2

    results = []
    for p in peers:
        entry = {"id": p["id"], "name": p["name"], "ip": p["ip"], "role": p.get("role")}
        if not a.no_probe:
            entry.update(probe(p["ip"]))
        entry["pulls"] = []
        if a.pull and entry.get("reachable"):
            base = f"http://{p['ip']}:{PORT}"
            # does the peer accept a sync post, and what does it say about authentication?
            st, body, err = http_post(f"{base}/federation/sync",
                                      {"monad_anchor": ANCHOR, "from": "desktop-mep4vqn"})
            entry["sync_post"] = {"http": st, "error": err,
                                  "body": json.loads(body) if body[:1] in (b"{",) else body[:200].decode("utf-8", "ignore")}
            for ep in STATE_ENDPOINTS:
                st, body, err = http_get(f"{base}{ep}")
                rec = {"http": st, "error": err,
                       "bytes": len(body),
                       "sha256": hashlib.sha256(body).hexdigest() if body else None}
                if body[:1] in (b"{",):
                    try:
                        rec.update(summarise(ep, json.loads(body)))
                    except Exception:  # noqa: BLE001
                        rec["parse_error"] = True
                entry["pulls"].append(rec)
        results.append(entry)

    reachable = [r for r in results if r.get("reachable")]
    print(f"peers configured {len(results)}, reachable now {len(reachable)}")
    for r in results:
        lat = f"{r.get('latency_ms')} ms" if r.get("reachable") else "unreachable"
        print(f"  {r['id']} {r['ip']:16} {lat}")
        for pull in r.get("pulls", []):
            extra = ""
            if "total_cards" in pull:
                extra = f" kanban={pull['total_cards']}"
            if "corpus_documents" in pull:
                extra = f" corpus={pull['corpus_documents']} dim={pull.get('corpus_dim')}"
            if "kernel_version" in pull:
                extra = f" {pull['kernel_version']} fabricated={pull.get('fabricated_modules_count')}"
            print(f"       {pull['endpoint']:24} http={pull['http']} {pull['bytes']:7} B{extra}")
        if "sync_post" in r:
            print(f"       POST /federation/sync -> {r['sync_post']['body'] if isinstance(r['sync_post']['body'], str) else r['sync_post']['body'].get('status')}")

    if a.pull:
        LEDGER.parent.mkdir(parents=True, exist_ok=True)
        prev = []
        if LEDGER.exists():
            try:
                prev = json.loads(LEDGER.read_text(encoding="utf-8")).get("runs", [])
            except Exception:  # noqa: BLE001
                prev = []
        run = {
            "at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "reachable": len(reachable),
            "configured": len(results),
            "authenticated": 0,
            "files_synced": 0,
            "files_sync_blocker": (
                "no file transport to these peers: :8022 exposes no upload route, and SMB/445 on the "
                "laptop requires credentials this tool does not hold. Probing and state reads work; "
                "file sync must not be claimed."
            ),
            "auth_note": (
                "peers verify the Monad anchor, which is a constant published in every response - that is "
                "not authentication, and each peer says so itself ('recorded_unauthenticated')"
            ),
            "peers": results,
        }
        runs = (prev + [run])[-20:]
        LEDGER.write_text(json.dumps({"ledger": "federation_sync", "anchor": ANCHOR, "runs": runs},
                                     indent=1), encoding="utf-8")
        print(f"\nledger: {LEDGER.relative_to(ROOT)}  (runs kept: {len(runs)})")
        print(f"authenticated pulls: 0 | files synced: 0 (blocked: no transport)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

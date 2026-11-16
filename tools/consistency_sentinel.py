#!/usr/bin/env python3
"""CONSISTENCY SENTINEL — the intelligence that recognises the platform needs to change.

Why this exists (all measured, 2026-11-14):
  * `/evolution/genome` served `fitness_score: 0.99956, generation: 19, mutations_accepted: 171, proposals: []` while
    `swarm_v2_artifacts/evolution_proposals.json` does not exist. A near-perfect fitness with no proposals and no
    file behind it is not an evolution engine, it is a constant. (Previously it at least errored - G-series.)
  * `/swarm` reports `active_proposals: 438` in the same second that `/evolution/proposals` returns `[]`.
  * Three live federation peers report `total_nodes` of **100**, **5** and **6** - three different beliefs about one
    fleet - and nothing in the platform compares them.

So this tool does one job: poll the nodes we actually have, compare what they claim, and file a proposal wherever the
claims contradict each other or themselves. A proposal is only emitted with the measurement that produced it, so a
human can falsify it. It refuses to emit a proposal for a domain it could not measure.

Output: `swarm_v2_artifacts/evolution_proposals.json` (the path the kernel's /evolution/genome already reads).

Usage:
  python tools/consistency_sentinel.py            # one poll, write proposals
  python tools/consistency_sentinel.py --watch 60 # poll twice 60 s apart (proves whether fitness is actually moving)
  python tools/consistency_sentinel.py --dry-run  # print, write nothing
"""
from __future__ import annotations

import argparse
import json
import socket
import subprocess
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "swarm_v2_artifacts/evolution_proposals.json"
PEERS_CFG = ROOT / "trm_uefi_kernel/host_council_server/config/federation_peers.json"
LOCAL = "http://127.0.0.1:8022"


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def get(url: str, timeout: float = 6.0):
    try:
        raw = urllib.request.urlopen(url, timeout=timeout).read()
        return json.loads(raw)
    except Exception as e:  # noqa: BLE001 - the error text is data here
        return {"_error": str(e)[:120]}


def tcp(host: str, port: int, timeout: float = 1.5) -> bool:
    s = socket.socket()
    s.settimeout(timeout)
    try:
        s.connect((host, port))
        return True
    except Exception:
        return False
    finally:
        s.close()


def load_peers() -> list[dict]:
    if not PEERS_CFG.exists():
        return []
    try:
        return json.loads(PEERS_CFG.read_text(encoding="utf-8")).get("peers", [])
    except Exception:
        return []


def observe() -> dict:
    """One full observation. Every field is either measured or absent - never defaulted."""
    obs: dict = {"at": now(), "local": {}, "peers": []}
    obs["local"]["health"] = get(f"{LOCAL}/health")
    obs["local"]["federation_stats"] = get(f"{LOCAL}/federation/stats")
    obs["local"]["federation_peers"] = get(f"{LOCAL}/federation/peers")
    obs["local"]["swarm"] = get(f"{LOCAL}/swarm")
    obs["local"]["evolution"] = get(f"{LOCAL}/evolution/genome")
    for p in load_peers():
        ip = p.get("ip")
        rec = {"id": p.get("id"), "ip": ip, "name": p.get("name")}
        rec["tcp_8022"] = tcp(ip, 8022) if ip else None
        if rec["tcp_8022"]:
            rec["health"] = get(f"http://{ip}:8022/health")
            rec["federation_stats"] = get(f"http://{ip}:8022/federation/stats")
        obs["peers"].append(rec)
    return obs


def analyse(obs: dict, prev: dict | None) -> list[dict]:
    props: list[dict] = []
    add = lambda sev, kind, claim, evidence, change: props.append({  # noqa: E731
        "id": f"CP-{len(props) + 1:03d}", "at": now(), "severity": sev, "kind": kind,
        "claim": claim, "evidence": evidence, "proposed_change": change, "status": "OPEN"})

    # C1 - the fleet cannot agree on how big it is
    counts = {}
    for r in obs["peers"]:
        st = r.get("federation_stats") or {}
        for k in ("total_nodes", "stats"):
            v = st.get(k)
            if isinstance(v, dict):
                v = v.get("total_nodes")
            if isinstance(v, int):
                counts[r["id"]] = v
                break
    if len(set(counts.values())) > 1:
        add("HIGH", "topology_disagreement",
            f"live nodes report {len(set(counts.values()))} different fleet sizes at the same moment",
            counts, "one peer table, distributed - nodes must read the same config, not their build-time defaults")

    # C2 - kernel skew
    kernels = {}
    for r in obs["peers"]:
        k = (r.get("health") or {}).get("kernel")
        if isinstance(k, str) and k:
            kernels[r["id"]] = k[:80]
    if len(set(kernels.values())) > 1:
        add("HIGH", "kernel_skew", "the fleet runs more than one kernel identity/version",
            kernels, "pin one kernel build per tier and report the skew in /federation/stats")

    # C3 - evolution fitness that never moves and has no proposals behind it
    evo = obs["local"]["evolution"]
    genome = (evo or {}).get("genome") or {}
    props_list = (evo or {}).get("proposals")
    if genome.get("fitness_score") is not None and props_list == []:
        if prev is not None:
            pg = ((prev.get("local", {}).get("evolution") or {}).get("genome") or {})
            if pg.get("fitness_score") == genome.get("fitness_score"):
                add("HIGH", "static_fitness",
                    "fitness_score is byte-identical across two polls and the proposal list is empty, so no "
                    "generation or fitness computation is happening",
                    {"poll1": pg.get("fitness_score"), "poll2": genome.get("fitness_score"),
                     "generation": genome.get("generation"), "mutations_accepted": genome.get("mutations_accepted"),
                     "proposals": props_list, "proposals_file_exists": OUT.exists()},
                    "either compute fitness from measured task yield or report it as UNMEASURED - never a constant "
                    "0.99956")
        else:
            add("MED", "suspicious_fitness",
                "a near-perfect fitness_score is published with zero proposals; nothing measures it",
                {"fitness_score": genome.get("fitness_score"), "proposals": props_list,
                 "proposals_file_exists": OUT.exists()},
                "re-run with --watch to prove whether it moves; if it does not, unpublish it")

    # C4 - the local node fails its own federation probe
    peers_self = ((obs["local"]["federation_peers"] or {}).get("peers") or [])
    inactive_self = [p for p in peers_self if str(p.get("name", "")).lower().find("this host") >= 0
                     and p.get("is_active") is False]
    if inactive_self:
        add("MED", "self_probe_failure",
            "this host reports itself offline in its own federation peer list",
            {"peer": inactive_self[0].get("node_id") or inactive_self[0].get("id"),
             "ip": inactive_self[0].get("ip"),
             "why": "council binds 127.0.0.1 by default (TRM_BIND_HOST), so a TCP probe to its tailscale IP refuses"},
            "probe loopback for the local node id, or bind TRM_BIND_HOST with COUNCIL_AUTH_TOKEN set")

    # C5 - two counters, same subject, different numbers
    swarm = obs["local"]["swarm"] or {}
    ap = swarm.get("active_proposals")
    if isinstance(ap, int) and props_list == []:
        add("HIGH", "counter_disagreement",
            "/swarm advertises active_proposals while /evolution/proposals returns an empty list",
            {"swarm.active_proposals": ap, "evolution.proposals": props_list},
            "derive both from the same store, and make the number the length of the list")

    # C6 - configured but unreachable (stated, not proposed as a fix)
    down = [r["id"] for r in obs["peers"] if r.get("tcp_8022") is False]
    if down:
        add("LOW", "peer_unreachable", "configured peers that did not answer a TCP connect",
            {"closed": down}, "no change proposed - record it, retry does not create a transport")
    return props


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--watch", type=int, default=0, help="seconds between two polls (proves movement or its absence)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    first = observe()
    prev = None
    if args.watch:
        print(f"[observe] poll 1 captured; sleeping {args.watch}s for poll 2")
        time.sleep(args.watch)
        prev = first
        obs = observe()
    else:
        obs = first

    props = analyse(obs, prev)
    result = {
        "generated_by": "tools/consistency_sentinel.py",
        "generated_at": now(),
        "poll_window_s": args.watch,
        "node_count_observed": len(obs["peers"]) + 1,
        "nodes_answering": 1 + sum(1 for r in obs["peers"] if r.get("tcp_8022")),
        "proposals": props,
        "observation": {k: v for k, v in obs.items() if k != "at"},
    }
    print(json.dumps({"nodes_answering": result["nodes_answering"],
                      "node_count_observed": result["node_count_observed"],
                      "proposals": [{k: p[k] for k in ("id", "severity", "kind", "claim")} for p in props]},
                     indent=2))
    if not args.dry_run:
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(json.dumps(result, indent=2), encoding="utf-8")
        print(f"\nwritten -> {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

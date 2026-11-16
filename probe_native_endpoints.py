"""Probe the native council server's endpoints so cron jobs are repointed only to
paths that actually answer."""
import json
import urllib.request

BASE = "http://127.0.0.1:8022"
PATHS = [
    "/health", "/infra/health", "/system/health_matrix", "/mesh/topology",
    "/swarm/experts", "/experts", "/learning", "/learning/skills",
    "/security/stats", "/security/threats", "/ddr/stats", "/ddr/antibodies",
    "/verification/stats", "/evolution/genome", "/evolution/proposals",
    "/federation/stats", "/council/status", "/skills", "/research/stats",
    "/kanban/stats", "/journal/status", "/continuum", "/infra/watchdog",
]

for p in PATHS:
    try:
        r = urllib.request.urlopen(BASE + p, timeout=6)
        body = r.read().decode("utf-8", "replace")
        print(f"  {r.status}  {p:<26} {body[:88]}")
    except Exception as e:
        print(f"  ERR  {p:<26} {type(e).__name__}: {str(e)[:60]}")

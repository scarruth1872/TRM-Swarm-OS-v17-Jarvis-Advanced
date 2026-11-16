"""Honest reclassification of federation_100_node_mesh_ledger.json.

The ledger asserts total_nodes: 100 with statuses that read as measured telemetry.
Inspection shows otherwise: only the three Tailscale hosts have routable IP endpoints,
'pqc_shield' holds a single templated value across all 100 entries, and several
'endpoint' values are descriptive strings ('Global Earth Cavity', 'HF F2-Skip') rather
than addresses. This splits the list into what was actually probed vs what is declared,
and preserves the superseded seal for audit.
"""
import hashlib
import json
from datetime import datetime, timezone

LEDGER = "federation_100_node_mesh_ledger.json"
PHYSICAL_IPS = {"100.93.191.43", "100.82.16.43", "100.123.215.119"}

with open(LEDGER, encoding="utf-8") as fh:
    d = json.load(fh)

orig_seal = d.get("sha256_seal")
orig_merkle = d.get("merkle_root")
orig_claimed = d.get("total_nodes")

physical, logical = [], []
for n in d["nodes"]:
    ep = str(n.get("endpoint", ""))
    if ep in PHYSICAL_IPS:
        n["node_class"] = "PHYSICAL_HOST"
        n["probe_verified"] = True
        physical.append(n.get("id"))
    else:
        n["node_class"] = "LOGICAL_SLOT"
        n["probe_verified"] = False
        logical.append(n.get("id"))

d["revision"] = 2
d["revision_timestamp"] = datetime.now(timezone.utc).isoformat()
d["revision_reason"] = (
    "HONESTY CORRECTION: the ledger asserted total_nodes=100 in a form that reads as "
    "measured telemetry. Only 3 entries carry routable Tailscale endpoints that were "
    "actually probed; the remainder are declared/descriptive slots, several with "
    "non-address endpoints such as 'Global Earth Cavity' and 'HF F2-Skip'. Entries are "
    "now classified explicitly."
)
d["superseded_sha256_seal"] = orig_seal
d["superseded_merkle_root"] = orig_merkle

d["node_classification"] = {
    "claimed_total_nodes": orig_claimed,
    "physical_hosts_probed": len(physical),
    "logical_or_declared_slots": len(logical),
    "physical_host_ids": physical,
    "note": (
        "PHYSICAL_HOST entries have routable Tailscale addresses and were reachable in "
        "benchmark runs. LOGICAL_SLOT entries are architectural/declared placeholders - "
        "they were NOT contacted and must not be reported as active infrastructure. "
        "Do not describe this system as a '100-node mesh' without this distinction."
    ),
}

d["honesty_notice"] = (
    "The federation comprises THREE physical hosts (NODE-001-DESK-6700XT, "
    "NODE-004-STATION-VB1T, NODE-002-LAPTOP-3070). The 100-node figure is a "
    "design target spanning 6 tiers, not 100 deployed machines. Benchmarks in which "
    "NODE-004 was unreachable traced DEGRADED rather than being reported as healthy."
)

payload = {k: v for k, v in d.items() if k != "sha256_seal"}
d["sha256_seal"] = hashlib.sha256(
    json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
d["seal_scheme"] = "sha256 over json.dumps(payload, sort_keys=True, separators=(',',':')) excluding sha256_seal"

with open(LEDGER, "w", encoding="utf-8") as fh:
    json.dump(d, fh, indent=2)

print(f"claimed nodes      : {orig_claimed}")
print(f"physical (probed)  : {len(physical)} -> {physical}")
print(f"logical/declared   : {len(logical)}")
print(f"superseded seal    : {orig_seal}")
print(f"new seal           : {d['sha256_seal']}")

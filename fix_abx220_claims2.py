"""Add provenance notices to the two remaining artifacts that assert TRM-ABX-220 claims.

1. hybrid_intelligence_6_tier_60_tasks_ledger.json - a synthetic demonstration corpus.
   Its tasks assert a German-language Phase II protocol for TRM-ABX-220. The `tiers`
   subtree and its sha256_merkle_root are deliberately left untouched (the notice is
   additive and outside that subtree), so its declared root stays valid.

2. sovereign_capability_task_suite.json - task 207 presupposes a "12-hospital TRM-ABX-220
   Phase II monitoring system" that does not exist. Its sha256 covers the whole document,
   so the superseded value is recorded and a new one computed.
"""
import hashlib
import json

NOTICE = {
    "status": "SYNTHETIC_OR_UNSUPPORTED_REFERENCE",
    "timestamp": "2026-09-10",
    "detail": (
        "TRM-ABX-220 has no molecular structure on record (no SMILES, no InChI, no ligand "
        "coordinates). It is an in-silico affinity hypothesis attached to a name. Any "
        "phase-II protocol, patient count, adverse-event count or 12-hospital monitoring "
        "system referring to it is illustrative or unsupported and must not be read as a "
        "real trial."
    ),
    "reference": "wetlab_dispatch/ABX-220_RECONCILIATION_REPORT.md",
}

# ---- 1. hybrid intelligence ledger (additive; tiers + merkle untouched) ----
with open("hybrid_intelligence_6_tier_60_tasks_ledger.json", encoding="utf-8") as fh:
    h = json.load(fh)
before_merkle = h.get("sha256_merkle_root")
h["provenance_notice"] = NOTICE
h["provenance_notice_scope"] = (
    "Additive top-level field. The tiers subtree is unchanged and its "
    "sha256_merkle_root remains valid over that subtree."
)
h["protocol_original"] = h.get("protocol")
with open("hybrid_intelligence_6_tier_60_tasks_ledger.json", "w", encoding="utf-8") as fh:
    json.dump(h, fh, indent=2)
print("hybrid_intelligence_6_tier_60_tasks_ledger.json : provenance_notice added")
print(f"  merkle root left untouched: {str(before_merkle)[:32]}...")

# ---- 2. sovereign capability task suite (whole-doc sha256 -> supersede + recompute) ----
with open("sovereign_capability_task_suite.json", encoding="utf-8") as fh:
    s = json.load(fh)
old_sha = s.get("sha256")
flagged = 0
for t in s.get("tasks", []):
    blob = json.dumps(t)
    if "ABX-220" in blob:
        t["provenance_correction"] = {
            "status": "PREREQUISITE_UNMET",
            "detail": ("This task presumes a TRM-ABX-220 phase-II monitoring system. "
                       "TRM-ABX-220 has no molecular structure on record, so that system "
                       "does not exist. The task cannot be executed as written."),
            "reference": "wetlab_dispatch/ABX-220_RECONCILIATION_REPORT.md",
        }
        flagged += 1

s["superseded_sha256"] = old_sha
s["provenance_notice"] = NOTICE
payload = {k: v for k, v in s.items() if k != "sha256"}
s["sha256"] = hashlib.sha256(
    json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
s["sha256_scheme"] = "sha256 over json.dumps(payload, sort_keys=True, separators=(',',':')) excluding sha256"
with open("sovereign_capability_task_suite.json", "w", encoding="utf-8") as fh:
    json.dump(s, fh, indent=2)
print(f"sovereign_capability_task_suite.json : {flagged} task(s) flagged")
print(f"  superseded sha256: {str(old_sha)[:32]}...")
print(f"  new sha256       : {s['sha256'][:32]}...")

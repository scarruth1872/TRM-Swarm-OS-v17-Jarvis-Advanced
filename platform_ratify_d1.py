import sys, json, hashlib
from datetime import datetime, timezone
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.native_federation_bridge import NativeBridgeClient

c = NativeBridgeClient("127.0.0.1", 8022)

motions = {
    "ratify_corrected_dispatch": (
        "D1 RATIFICATION: the wet-lab dispatch has been provenance-repaired. "
        "Structure locks are now sha256(canonical SMILES) and independently "
        "reproducible; TRM-ABX-220 is withheld (no structure on record); the "
        "docking correlation of record r=0.9248 replaces the stale 0.9097; "
        "potency windows are the susceptible ranges with non-susceptible "
        "organisms disclosed. Ratify activation of the 8 per-institution "
        "briefs and 3 CRO assay RFPs."),
    "ratify_platform_op": (
        "D1 RATIFICATION: approve the platform capability wetlab_dispatch, a "
        "resident-bridge op that generates dispatch materials from the canonical "
        "ledgers with derived locks and a fail-closed provenance check."),
}

record = {"protocol": "TRM_WETLAB_D1_RATIFICATION",
          "timestamp": datetime.now(timezone.utc).isoformat(),
          "node": "TRM-DESK-6700XT", "monad_anchor": "0x4D4F4E4144000001",
          "motions": {}}

for name, text in motions.items():
    r = c.council(text)
    record["motions"][name] = r
    print(f"{name}: ok={r.get('ok')} score={r.get('score')} seal={r.get('seal')} tally={r.get('tally')}")

canon = json.dumps(record["motions"], sort_keys=True)
record["sha256"] = hashlib.sha256(canon.encode()).hexdigest()
out = r"F:\Development sites\TRM-Swarm-OS-v2\wetlab_dispatch\D1_ACTIVATION_RECORD.json"
with open(out, "w", encoding="utf-8") as f:
    json.dump(record, f, indent=2)
print("\nrecord:", out)
print("record sha256:", record["sha256"])

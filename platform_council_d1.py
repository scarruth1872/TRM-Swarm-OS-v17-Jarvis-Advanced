import sys, json
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.native_federation_bridge import NativeBridgeClient

c = NativeBridgeClient("127.0.0.1", 8022)

print("=== platform bridge reachability ===")
print("drive_list:", json.dumps(c.drive_list())[:200])

print()
print("=== platform council: D1 wet-lab dispatch motion ===")
motion_a = ("OPERATOR DIRECTIVE D1: Activate the wet-lab dispatch. Correct the sealed "
            "outreach package provenance before any dispatch to real laboratories: "
            "(1) replace non-verifiable per-compound structure locks with the canonical "
            "sha256 of the pre-registered SMILES; (2) withhold TRM-ABX-220 from the dispatch "
            "because no molecular structure is on record; (3) cite the docking calibration of "
            "record r=0.9248 not 0.9097; (4) dispatch only what is traceable to a source ledger.")
res_a = c.council(motion_a)
print(json.dumps(res_a, indent=1))

print()
print("=== platform council: provenance integrity finding ===")
motion_b = ("FINDING: the wet-lab outreach packages carried document self-hashes in the "
            "molecular structure-lock field, a category error. Molecular structure locks must "
            "be sha256 of the canonical SMILES. Ratify correcting the source of truth.")
res_b = c.council(motion_b)
print(json.dumps(res_b, indent=1))

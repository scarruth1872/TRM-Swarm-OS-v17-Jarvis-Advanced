"""Council ratification of the 3-item remediation, via the live platform bridge."""
import json
import sys

sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.native_federation_bridge import NativeBridgeClient

M = (
    "Ratify remediation of three provenance defects found by verifying the Antigravity "
    "conversation a418e593 against the repository. "
    "(1) DOCKING WRONG PROTEIN: the benchmark defaulted to receptor 3TYZ, which is "
    "Yersinia pestis dihydropteroate synthetase (DHPS), while labelling results LpxC. "
    "Fixed via a receptor registry; the validated LpxC structure 5U39 now redocks the "
    "CHIR-090 positive control at -10.17 kcal/mol with pose RMSD 1.513 A, reproducing "
    "the claimed -10.1. Validated: CHIR-090 -10.20, ABX-219 -8.21, ABX-220 -7.97. "
    "(2) UEFI Sector 05 asserted pKd=9.42 (0.38 nM IC50), roughly 2300x stronger than "
    "measured; corrected to the docking value, rebuilt, and synced byte-identical to "
    "the USB. "
    "(3) The 100-node federation ledger presented 100 entries as measured telemetry; "
    "only 3 are physical hosts, so entries are now classified PHYSICAL_HOST vs "
    "LOGICAL_SLOT with an honesty notice. "
    "All corrections derive from measured traces and are committed."
)

c = NativeBridgeClient("127.0.0.1", 8022)
r = c.council(M)
print(json.dumps(r, indent=1)[:1200])

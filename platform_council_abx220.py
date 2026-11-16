import sys, json
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.native_federation_bridge import NativeBridgeClient

M = ("Ratify TRM-ABX-220 provenance reconciliation: retract sub-nanomolar Kd "
     "(0.05-0.12 nM) as a pipeline artifact of a double-counted electrostatic "
     "term; void the invalid 95% CI [0.1,0.12] which excludes its own mean 0.08; "
     "retract verdict 'Ki ~ 4.2 nM Confirmed' (unsourced, spliced from ESKAPE); "
     "adopt pipeline-consistent 3.6-4.8 nM; status UNRESOLVED pending structure "
     "and experiment; lead remains WITHHELD from dispatch.")

c = NativeBridgeClient("127.0.0.1", 8022)
r = c.council(M)
print(json.dumps(r, indent=1)[:1200])

"""Drive the TRM-ABX-220 reconciliation through the live platform bridge."""
import sys, json
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.native_federation_bridge import NativeBridgeClient

c = NativeBridgeClient("127.0.0.1", 8022)
r = c.request("wetlab_reconcile_abx220", "wetlab_reconcile_abx220")
print(json.dumps(r, indent=1)[:2000])

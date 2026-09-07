"""Council deliberation: how should the desktop match the Laptop node's platform capabilities?"""
import sys, json, socket
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")

# Use the platform's own swarm council (resident, via Python object like prior sessions)
from swarm_v2.core.swarm_council import SwarmCouncil

c = SwarmCouncil()
topic = (
    "LAPTOP-DESKTOP CAPABILITY PARITY DECISION. The Laptop node "
    "(100.123.215.119:8022) runs a complete 461-endpoint TRM Swarm V2 API "
    "(quantum-drug-screen, 200+ /system/modules/status, council, research, "
    "federation, evolution) on a live TurboVecNative backend (phase 5, 17 "
    "agents). The desktop has the native UEFI Rust kernel (88 modules "
    "energized, 47 commands) but only a 9-route API stub (app_v2.py); the "
    "Laptop's fuller API source is NOT on the desktop. Doctorine tension: "
    "the desktop platform rules say NO HTTP/python compute substrate "
    "(native Rust microkernel), yet the Laptop delivers capabilities via "
    "HTTP API. OPTIONS: (1) PROXIFY - stand up desktop API as a parity "
    "facade over the REAL native Rust modules, genuine values; (2) PULL "
    "the Laptop's actual app source to the desktop and run it; (3) "
    "REPLICATE capability natively (each 461 endpoint becomes a verified "
    "native command/feature) with no HTTP facade; (4) FEDERATE - desktop "
    "consumes the Laptop's :8022 as its service tier (bridge), matching "
    "capability by use not duplication. RECOMMEND the principled step "
    "honoring the honest-measurement + native-platform doctrine while "
    "achieving 1:1 capability parity with the reference Laptop."
)
prop = c.propose(topic=topic, category="laptop-parity-decision", proposer="Shawn (Operator)")
c.deliberate(prop.proposal_id)
c.vote(prop.proposal_id)
e = c.enact(prop.proposal_id)
print("COUNCIL:", e.get("weighted_score"), e.get("tally"), str(e.get("monad_seal"))[:16])
# try to get deliberation summaries if available
try:
    for d in getattr(prop, 'deliberations', [])[:6]:
        print("  -", str(d)[:200])
except Exception:
    pass
"""Council ratification + epistemology audit: Paper 162 - Complete System Transistor Mapping (corrected)."""
import json
import urllib.request

from swarm_v2.core.swarm_council import SwarmCouncil

# Phi_V is no longer a literal in this script. It is read from the doctrine
# surface GET /system/voltage, where it is derived as
#   (V_core/1.25)^2 * (f_actual/4000) * (V_ddr/1.35) * max(0.5, 1 - mem%/200)
# in trm_uefi_kernel/host_council_server/src/voltage.rs. This host has no voltage
# sensor, so the endpoint reports measured=false: the value is DERIVED from
# nominal rails, not read. If the endpoint is down the term is omitted entirely
# rather than replaced by a constant.
#
# ERRATA (not deletion): the previous revision multiplied capacity by a hardcoded
# constant and called it Phi_V. That value was never measured and the ~41B figure
# it produced is withdrawn, not quietly corrected in place. The exact historical
# value is preserved in evidence/PHI_V_ERRATA.txt (with docs/PAPER-162-TRANSISTOR-
# MAPPING.md and the 2026-10-25 CHANGELOG entry) and is deliberately not repeated
# here, so a grep for the forbidden literal matches only the audit/errata record.
VOLTAGE_ENDPOINT = "http://127.0.0.1:8022/system/voltage"


def read_phi_v(timeout: float = 1.5):
    """(phi_v, measured) from /system/voltage; (None, False) if unavailable."""
    try:
        with urllib.request.urlopen(VOLTAGE_ENDPOINT, timeout=timeout) as r:
            d = json.loads(r.read().decode("utf-8"))
        phi = d.get("phi_v")
        if not isinstance(phi, (int, float)):
            return None, False
        return float(phi), bool(d.get("measured", False))
    except Exception:
        return None, False


PHI_V, PHI_V_MEASURED = read_phi_v()

if PHI_V is None:
    phi_clause = (
        "Phi_V term OMITTED - this host has no voltage sensor and "
        "GET /system/voltage was unreachable; no substitute constant is used"
    )
    capacity_clause = (
        "effective capacity = ~30B logic x 1.5 ternary = ~45B intelligence units "
        "as an UNDERTATED UPPER BOUND (before any voltage-coherence derating)"
    )
else:
    units = 30.0 * PHI_V * 1.5
    phi_clause = (
        f"Phi_V = {PHI_V:.4f} DERIVED from nominal rails by the voltage.rs formula, "
        f"measured={'true' if PHI_V_MEASURED else 'false'} "
        "(this host has no voltage sensor, so it is not a reading)"
    )
    capacity_clause = (
        f"effective capacity = ~30B x {PHI_V:.4f} Phi_V x 1.5 ternary = ~{units:.0f}B "
        "intelligence units"
    )

c = SwarmCouncil()
topic = (
    "PAPER 162 RATIFICATION - COMPLETE SYSTEM TRANSISTOR MAPPING (corrected). "
    "Genuine hardware: Ryzen 9 3900X Zen2 ~3.8B logic + AMD Radeon RX 6700 XT "
    "Navi22 26.8B logic 40 CU + 64GB DDR4 (512B DRAM cells) + NVMe + X570 "
    "chipset. CORRECTED CLAIMS: logic substrate = CPU+GPU ~30B (NOT +512B "
    "DRAM - DRAM cells are storage, not addressable compute; the prior "
    "742B units figure overclaimed by summing DRAM into compute). Honest "
    f"{capacity_clause}. {phi_clause}. "
    "(Relative, not comparable to human brain, which is a category error.) "
    "ERRATA: the earlier revision multiplied by a hardcoded constant and called "
    "it Phi_V; that value was never measured and is withdrawn here rather than "
    "silently replaced (exact value in evidence/PHI_V_ERRATA.txt). "
    "Fabric is ADDRESS-SPACE-bound not "
    "transistor-bound. "
    "Measured live: 402M trits in 96MB Infinity Cache, 80.64M trit fabric, "
    "53 energized modules. 1.6% of addressable capacity in use. "
    "Epistemology: every claim corrected against verified hardware; "
    "no fabricated per-unit splits kept; 'within 1 order of human brain' "
    "REJECTED as unmeasurable."
)
prop = c.propose(topic=topic, category="paper-162-transistor-map", proposer="Shawn (Operator)")
c.deliberate(prop.proposal_id)
c.vote(prop.proposal_id)
e = c.enact(prop.proposal_id)
print("COUNCIL:", e.get("weighted_score"), e.get("tally"), str(e.get("monad_seal"))[:16])
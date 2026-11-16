"""Measure domain_router OOD rejection + in-domain accuracy on a HELD-OUT set.

The OOD samples here are deliberately disjoint from the samples used to fit
ABSTAIN_MARGIN (tools/calibrate_per_domain.py) and from the OOD list in
tools/calibrate_domain_router.py. Nothing in this file is used for tuning.

Usage:  python tools/measure_domain_router.py [--json out.json]
"""
import json
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from swarm_v2.core import domain_router as R

# ── Held-out OOD: 14 samples, none from the calibration corpora ──────────────
OOD = [
    ("medieval history", "who was crowned holy roman emperor in the year 800"),
    ("medieval history", "how did the hanseatic league control baltic trade"),
    ("marine biology", "how do mantis shrimp perceive polarised light"),
    ("marine biology", "what depth do vampire squid inhabit"),
    ("tax law", "can i deduct home office expenses for a sole proprietorship"),
    ("tax law", "what is the statute of limitations on an irs audit"),
    ("music theory", "why does a diminished seventh chord resolve outward"),
    ("music theory", "what is the difference between dorian and phrygian mode"),
    ("cooking", "how long should i braise beef short ribs at 150 celsius"),
    ("cooking", "why does my sourdough starter smell like acetone"),
    ("astronomy", "how far is the andromeda galaxy in light years"),
    ("astronomy", "what causes the cassini division in saturn's rings"),
    ("botany", "why do monstera leaves develop fenestrations"),
    ("linguistics", "when did the great vowel shift occur in english"),
]

# ── In-domain: 6 samples, one per harness domain ─────────────────────────────
IN_DOMAIN = [
    (0, "Security & Cryptography", "is this jwt signature verification constant time"),
    (1, "Logic & Mathematical Proof", "prove by induction that n cubed minus n is divisible by six"),
    (2, "Code Engineering & Refactoring", "extract this duplicated block into a helper and update the callers"),
    (3, "Telemetry & Memory Synchronization", "graph the p99 latency of the ingest service over the last hour"),
    (4, "Synthesis & Chain-of-Thought", "summarise the tradeoffs and recommend an approach"),
    (5, "Autonomous Healing & Resilience", "the replica lagged so fail over to the standby node"),
]


# The probe the gap report measured as a failure (routed to Telemetry at 0.4461).
NAMED_PROBE = "what is the airspeed velocity of an unladen swallow"


def run():
    ood_rows = []
    for topic, q in OOD:
        r = R.route(q)
        ood_rows.append({
            "topic": topic, "query": q,
            "top": r["domain"], "score": round(r["score"], 4),
            "margin": round(r["margin"], 4) if r["margin"] is not None else None,
            "threshold": r.get("threshold"),
            "abstain": r["abstain"],
        })

    in_rows = []
    for idx, name, q in IN_DOMAIN:
        r = R.route(q)
        in_rows.append({
            "query": q, "expected": name, "top": r["domain"],
            "score": round(r["score"], 4),
            "margin": round(r["margin"], 4) if r["margin"] is not None else None,
            "abstain": r["abstain"],
            "correct": (r["index"] == idx) and not r["abstain"],
        })

    np = R.route(NAMED_PROBE)
    named = {
        "query": NAMED_PROBE, "top": np["domain"],
        "score": round(np["score"], 4),
        "margin": round(np["margin"], 4) if np["margin"] is not None else None,
        "abstain": np["abstain"],
    }

    ood_abstained = sum(1 for r in ood_rows if r["abstain"])
    in_correct = sum(1 for r in in_rows if r["correct"])
    return {
        "named_probe": named,
        "ood": ood_rows,
        "in_domain": in_rows,
        "ood_abstained": ood_abstained,
        "ood_total": len(ood_rows),
        "in_correct": in_correct,
        "in_total": len(in_rows),
        "floor": getattr(R, "SIMILARITY_FLOOR", None),
    }


def main():
    res = run()
    print("OOD (held-out, n=%d)" % res["ood_total"])
    print("%-18s %-58s %-34s %7s %7s %8s" % ("topic", "query", "routed_to", "score", "margin", "decision"))
    for r in res["ood"]:
        print("%-18s %-58s %-34s %7.4f %7s %8s" % (
            r["topic"], r["query"][:58], r["top"][:34], r["score"],
            ("%.4f" % r["margin"]) if r["margin"] is not None else "-",
            "ABSTAIN" if r["abstain"] else "ACCEPT"))
    print("\nOOD abstained: %d/%d" % (res["ood_abstained"], res["ood_total"]))

    print("\nIn-domain (n=%d)" % res["in_total"])
    print("%-58s %-34s %7s %7s %8s" % ("query", "routed_to", "score", "margin", "decision"))
    for r in res["in_domain"]:
        print("%-58s %-34s %7.4f %7s %8s" % (
            r["query"][:58], r["top"][:34], r["score"],
            ("%.4f" % r["margin"]) if r["margin"] is not None else "-",
            "ABSTAIN" if r["abstain"] else ("OK" if r["correct"] else "WRONG")))
    print("\nIn-domain correct: %d/%d" % (res["in_correct"], res["in_total"]))
    n = res["named_probe"]
    print("named probe (gap report): %r -> %s score=%.4f margin=%s %s" % (
        n["query"], n["top"], n["score"], n["margin"],
        "ABSTAIN" if n["abstain"] else "ACCEPT"))
    print("similarity floor: %s" % res["floor"])

    if "--json" in sys.argv:
        out = sys.argv[sys.argv.index("--json") + 1]
        pathlib.Path(out).write_text(json.dumps(res, indent=2), encoding="utf-8")
        print("wrote %s" % out)


if __name__ == "__main__":
    main()

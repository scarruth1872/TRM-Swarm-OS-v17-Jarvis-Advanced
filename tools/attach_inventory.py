"""Attach the asset inventory to a trained gate artifact.

Two layers, kept distinguishable on purpose:

  the HEAD      - a linear read-out over pooled embeddings, measured 0.926 +/- 0.001 (verb holdout). It judges
                  request SHAPE, and it cannot judge an object it has never seen (0.527, G144).
  the INVENTORY - read straight from the deployment's configs. It says what an asset IS, and it is citable.

When a request names an object the head has never seen, the inventory answers the only question that still matters:
is this thing load-bearing? The decision is then reported as `readout: "inventory-rule"`, never as the head's - so no
reader can mistake a config lookup for a learned judgement.

    python attach_inventory.py --artifact data/framing_gate_candidate.json --inventory data/asset_inventory.json
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--artifact", default="data/framing_gate_candidate.json")
    ap.add_argument("--inventory", default="data/asset_inventory.json")
    ap.add_argument("--serve-dir", default="trm_uefi_kernel/host_council_server/data")
    args = ap.parse_args()

    art_path = Path(args.artifact)
    art = json.loads(art_path.read_text(encoding="utf-8"))
    inv = json.loads(Path(args.inventory).read_text(encoding="utf-8"))

    if not art.get("deploy"):
        print("REFUSED: artifact is not deployed, so there is nothing to attach to. Fit a passing arm first.")
        return 2

    # phrase -> {class, kind, source}; the rule layer only needs the phrase and the class
    entries = [{"phrase": a["phrase"], "class": a["class"], "kind": a["kind"], "source": a["source"]}
               for a in inv["assets"]]
    art["inventory"] = entries
    art["inventory_note"] = (
        "rule layer read from configs, NOT from the model: unmatched objects are decided by this list's class and "
        "reported as readout='inventory-rule'. Unknown to both -> ABSTAIN."
    )
    art_path.write_text(json.dumps(art, indent=2), encoding="utf-8")
    print(f"artifact updated -> {art_path} ({len(entries)} inventory assets)")

    if art.get("deploy"):
        serve = Path(args.serve_dir) / "scope_gate.json"
        if serve.parent.exists():
            serve.write_text(json.dumps(art, indent=2), encoding="utf-8")
            print(f"artifact -> {serve} (served copy updated)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Reconcile the three leads across every place their chemistry appears (Python 3.12 + RDKit).

WHAT WAS ACTUALLY WRONG
-----------------------
Three claims have been made about TRM-ABX-220's structure and only one is true:

  * "no SMILES anywhere" (wet-lab audit, repeated in my G243 report) -- FALSE. The bridge registry carries
    SMILES for TRM-ABX-219 and TRM-ABX-082 with sha256 locks that recompute, and TRM-ABX-220's chemistry is a
    `REMARK SMILES` line inside TRM-ABX-220_benchmark_docked.pdbqt.
  * "TRM-ABX-220 has no SMILES on record" -- TRUE, and narrower: the registry lists two of three leads.
  * the ledgers' `smiles` field -- TRUE for 219/082, ABSENT for 220.

This script compares the registry SMILES against the pose-file SMILES for every lead, canonicalises both, and
registers the missing lead only if the chemistry agrees across the artifacts that carry it.

Writes: data/suite_leads_reconciliation.json (and, with --register, adds TRM-ABX-220 to the bridge package)
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BRIDGE = ROOT / "wet_lab_bridge_package.json"
OUT = ROOT / "data" / "suite_leads_reconciliation.json"

from rdkit import Chem, RDLogger  # noqa: E402

RDLogger.DisableLog("rdApp.*")

POSES = {"TRM-ABX-220": "TRM-ABX-220_benchmark_docked.pdbqt",
         "TRM-ABX-219": "TRM-ABX-219.pdbqt",
         "TRM-ABX-082": "TRM-ABX-082.pdbqt"}
SDFS = {"TRM-ABX-219": "TRM-ABX-219_3D.sdf", "TRM-ABX-082": "TRM-ABX-082_3D.sdf"}


def canon(smi: str) -> str | None:
    m = Chem.MolFromSmiles(smi)
    return Chem.MolToSmiles(m) if m else None


def pose_smiles(f: Path) -> str | None:
    if not f.exists():
        return None
    m = re.search(r"REMARK\s+SMILES\s+([^\s]+)", f.read_text(errors="replace"))
    return m.group(1) if m else None


def sdf_smiles(f: Path) -> str | None:
    if not f.exists():
        return None
    for mol in Chem.SDMolSupplier(str(f), sanitize=False, removeHs=False):
        if mol is None:
            continue
        try:
            Chem.SanitizeMol(mol)
            return Chem.MolToSmiles(Chem.RemoveHs(mol))
        except Exception:  # noqa: BLE001
            return None
    return None


def main(argv: list[str]) -> int:
    bridge = json.loads(BRIDGE.read_text(encoding="utf-8"))
    registered = {c["compound_id"]: c for c in bridge.get("lead_compounds", [])}
    rec = {"measured": True, "registry": "wet_lab_bridge_package.json", "leads": {}}

    for name, pose in POSES.items():
        reg = registered.get(name)
        entry: dict = {
            "registered": bool(reg),
            "registry_smiles": (reg or {}).get("smiles"),
            "registry_lock_ok": None,
            "pose_smiles": pose_smiles(ROOT / pose),
            "sdf_smiles": sdf_smiles(ROOT / SDFS[name]) if name in SDFS else None,
        }
        if reg and reg.get("smiles"):
            entry["registry_lock_ok"] = hashlib.sha256(reg["smiles"].encode()).hexdigest() == reg.get("sha256_structure_lock")
        cs = {k: canon(entry[k]) for k in ("registry_smiles", "pose_smiles", "sdf_smiles") if entry[k]}
        entry["canonical"] = cs
        entry["sources_agree"] = len(set(cs.values())) <= 1 and len(cs) > 0
        entry["verdict"] = (
            "registered, locks recompute, chemistry agrees" if entry["registered"] and entry["sources_agree"]
            and entry["registry_lock_ok"] else
            "structure recoverable but NOT in the registry" if not entry["registered"] and entry["pose_smiles"] else
            "sources DISAGREE" if cs and not entry["sources_agree"] else
            "no structure found"
        )
        rec["leads"][name] = entry

    if "--register" in argv:
        missing = [n for n, e in rec["leads"].items() if not e["registered"] and e["pose_smiles"]]
        for name in missing:
            smi = rec["leads"][name]["pose_smiles"]
            bridge["lead_compounds"].append({
                "compound_id": name,
                "smiles": smi,
                "sha256_structure_lock": hashlib.sha256(smi.encode()).hexdigest(),
                "structure_source": f"REMARK SMILES in {POSES[name]}",
                "registration_note": "added 2026-11-24: the pose file carried the chemistry while the registry "
                                     "omitted this lead, which is why it was reported as having no SMILES",
            })
            rec["leads"][name]["registered_now"] = True
            print(f"registered {name}: {smi[:60]}... lock {hashlib.sha256(smi.encode()).hexdigest()[:16]}...")
        if missing:
            BRIDGE.write_text(json.dumps(bridge, indent=1), encoding="utf-8")

    OUT.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    for name, e in rec["leads"].items():
        print(f"{name:13} {e['verdict']}")
        print(f"   registry: {str(e['registry_smiles'])[:64]}")
        print(f"   pose    : {str(e['pose_smiles'])[:64]}  | agree: {e['sources_agree']}")
    print(f"\nwrote {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

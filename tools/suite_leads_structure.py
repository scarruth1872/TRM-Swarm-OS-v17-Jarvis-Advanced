#!/usr/bin/env python3
"""The three TRM leads, from the structure files that actually exist (Python 3.12 + RDKit).

CORRECTION THIS SCRIPT EXISTS TO MAKE
-------------------------------------
The wet-lab audit (and my own G243 capability report) said TRM-ABX-220 has "no SMILES anywhere". That was
wrong in a way worth recording precisely: the ledgers carry no `smiles` FIELD, but the docked poses carry the
molecule as a `REMARK SMILES` line, and two leads also have 3D SDF files:

    TRM-ABX-220  REMARK in TRM-ABX-220_benchmark_docked.pdbqt   (no 3D SDF)
    TRM-ABX-219  REMARK in TRM-ABX-219.pdbqt  + TRM-ABX-219_3D.sdf
    TRM-ABX-082  REMARK in TRM-ABX-082.pdbqt  + TRM-ABX-082_3D.sdf

So the tasks were blocked by a *search that stopped at one field name*, not by missing chemistry. This script
extracts the structure from the artifact, cross-checks it against the 3D file where one exists, and computes
Task 9 (drug-likeness) for all three leads.

Evidence: data/suite_leads_descriptors.json
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "data" / "suite_leads_descriptors.json"

from rdkit import Chem, RDLogger  # noqa: E402
from rdkit.Chem import Crippen, Descriptors, Lipinski, rdMolDescriptors  # noqa: E402

RDLogger.DisableLog("rdApp.*")

LEADS = {
    "TRM-ABX-220": {"pose": "TRM-ABX-220_benchmark_docked.pdbqt", "sdf": None},
    "TRM-ABX-219": {"pose": "TRM-ABX-219.pdbqt", "sdf": "TRM-ABX-219_3D.sdf"},
    "TRM-ABX-082": {"pose": "TRM-ABX-082.pdbqt", "sdf": "TRM-ABX-082_3D.sdf"},
}


def smiles_from_pose(path: Path) -> str | None:
    m = re.search(r"REMARK\s+SMILES\s+([^\s]+)", path.read_text(errors="replace"))
    return m.group(1) if m else None


def smiles_from_sdf(path: Path) -> str | None:
    for mol in Chem.SDMolSupplier(str(path), sanitize=False, removeHs=False):
        if mol is None:
            continue
        try:
            Chem.SanitizeMol(mol)
            return Chem.MolToSmiles(Chem.RemoveHs(mol))
        except Exception:  # noqa: BLE001 - an unsanitizable molblock is a finding, reported below
            return None
    return None


def profile(smi: str) -> dict:
    m = Chem.MolFromSmiles(smi)
    mw, logp, mr = Descriptors.MolWt(m), Crippen.MolLogP(m), Crippen.MolMR(m)
    hbd, hba = Lipinski.NumHDonors(m), Lipinski.NumHAcceptors(m)
    tpsa, rot = rdMolDescriptors.CalcTPSA(m), Lipinski.NumRotatableBonds(m)
    heavy = m.GetNumHeavyAtoms()
    return {
        "smiles": Chem.MolToSmiles(m), "mw": round(mw, 2), "logp": round(logp, 2), "mr": round(mr, 2),
        "hbd": hbd, "hba": hba, "tpsa": round(tpsa, 2), "rotb": rot, "heavy_atoms": heavy,
        "lipinski_violations": sum([mw > 500, logp > 5, hbd > 5, hba > 10]),
        "veber_ok": rot <= 10 and tpsa <= 140,
        "ghose_ok": 160 <= mw <= 480 and -0.4 <= logp <= 5.6 and 40 <= mr <= 130 and 20 <= heavy <= 70,
        # the same 8 descriptors the QSAR model was trained on
        "model_features": [round(mw, 2), round(logp, 2), round(mr, 2), hbd, hba, round(tpsa, 2), rot, heavy],
    }


def main() -> int:
    rec = {"measured": True, "source_note": "SMILES taken from REMARK lines inside the lead pose files; "
           "3D SDF cross-checked where present", "leads": {}}
    for name, paths in LEADS.items():
        pose = ROOT / paths["pose"]
        entry: dict = {"pose_file": paths["pose"], "pose_exists": pose.exists()}
        if not pose.exists():
            entry["error"] = "pose file missing"
            rec["leads"][name] = entry
            continue
        smi = smiles_from_pose(pose)
        entry["smiles_from_pose"] = smi
        if paths["sdf"] and (ROOT / paths["sdf"]).exists():
            sdf_smi = smiles_from_sdf(ROOT / paths["sdf"])
            entry["smiles_from_sdf"] = sdf_smi
            entry["sdf_agrees"] = (sdf_smi == Chem.MolToSmiles(Chem.MolFromSmiles(smi))) if (sdf_smi and smi) else None
        if smi:
            try:
                entry.update(profile(smi))
            except Exception as exc:  # noqa: BLE001
                entry["error"] = f"unparseable SMILES: {type(exc).__name__}"
        rec["leads"][name] = entry

    OUT.write_text(json.dumps(rec, indent=1), encoding="utf-8")

    print(f"{'lead':13} {'MW':>7} {'logP':>6} {'HBD':>4} {'HBA':>4} {'TPSA':>6} {'nRot':>5} {'heavy':>6} Lip Veber Ghose  SDF")
    for name, e in rec["leads"].items():
        if "mw" not in e:
            print(f"{name:13} {e.get('error','no SMILES')}")
            continue
        sdf = {True: "AGREE", False: "DIFFER", None: "n/a"}[e.get("sdf_agrees")]
        print(f"{name:13} {e['mw']:7.2f} {e['logp']:6.2f} {e['hbd']:4} {e['hba']:4} {e['tpsa']:6.2f} "
              f"{e['rotb']:5} {e['heavy_atoms']:6}  {e['lipinski_violations']}   "
              f"{'ok' if e['veber_ok'] else 'FAIL':5} {'ok' if e['ghose_ok'] else 'FAIL':5}  {sdf}")
    print(f"\nwrote {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Build 3D PDBQT ligands for the ROC set (2 crystal-confirmed actives + decoys).

ACTIVES are built FROM SMILES, not from their crystal poses. Deliberate: the ROC
compares docking-PREDICTED poses for everything, so actives must go through the
identical pipeline as the decoys or the comparison is rigged.

Path: SMILES -> RDKit ETKDG 3D + MMFF -> SDF -> meeko's mk_prepare_ligand CLI
-> PDBQT. (meeko 0.8 changed its Python API; the CLI is the stable interface and
is what prepared the PHOENIX ligands too.)
"""
import os, subprocess, sys
from rdkit import Chem, RDLogger
from rdkit.Chem import AllChem

RDLogger.DisableLog("rdApp.*")
MK = r"C:\Users\shawn\AppData\Local\Programs\Python\Python312\Scripts\mk_prepare_ligand.exe"

ACTIVES = {
    "CHIR-090":    "O=C(NO)C(NC(=O)c3ccc(C#Cc1ccc(cc1)CN2CCOCC2)cc3)C(O)C",
    "NVS-LPXC-01": "C(C(NO)=O)(C(C)(C)N)NC(=O)c1ccc(OCC#CC)cc1",
}

def smiles_to_sdf(smiles, path, seed=42):
    m = Chem.MolFromSmiles(smiles)
    if m is None:
        return None
    m = Chem.AddHs(m)
    ps = AllChem.ETKDGv3(); ps.randomSeed = seed
    if AllChem.EmbedMolecule(m, ps) != 0:
        return None
    AllChem.MMFFOptimizeMolecule(m, maxIters=800)
    w = Chem.SDWriter(path); w.write(m); w.close()
    return m.GetNumHeavyAtoms()

def sdf_to_pdbqt(sdf, pdbqt):
    r = subprocess.run([MK, "-i", sdf, "-o", pdbqt], capture_output=True, text=True, timeout=300)
    return os.path.exists(pdbqt) and os.path.getsize(pdbqt) > 200

if __name__ == "__main__":
    decoy_file, outdir = sys.argv[1], sys.argv[2]
    sdfdir = os.path.join(outdir, "_sdf")
    os.makedirs(sdfdir, exist_ok=True)
    ok = fail = 0
    rows = [("ACTIVE", n, s) for n, s in ACTIVES.items()]
    for i, line in enumerate(open(decoy_file)):
        s = line.strip()
        if s:
            rows.append(("DECOY", f"decoy_{i:03d}", s))
    for kind, name, smi in rows:
        sdf = os.path.join(sdfdir, name + ".sdf")
        pdbqt = os.path.join(outdir, name + ".pdbqt")
        nh = smiles_to_sdf(smi, sdf)
        if nh is None:
            print(f"  [FAIL] {name}: no 3D embed"); fail += 1; continue
        if not sdf_to_pdbqt(sdf, pdbqt):
            print(f"  [FAIL] {name}: meeko CLI"); fail += 1; continue
        n = sum(1 for l in open(pdbqt) if l.startswith("ATOM"))
        if kind == "ACTIVE":
            print(f"  [OK  ] {name:16} {nh:3} heavy atoms -> {n:3} pdbqt atoms  (ACTIVE)")
        ok += 1
    print(f"\nprepared {ok} ligands ({fail} failed) -> {outdir}")

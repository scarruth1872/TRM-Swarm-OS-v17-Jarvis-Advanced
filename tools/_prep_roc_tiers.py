
"""SMILES -> 3D SDF -> PDBQT for the tiered ChEMBL ROC set."""
import os, subprocess, sys
from rdkit import Chem, RDLogger
from rdkit.Chem import AllChem
RDLogger.DisableLog("rdApp.*")
MK = r"C:\Users\shawn\AppData\Local\Programs\Python\Python312\Scripts\mk_prepare_ligand.exe"

smi_file, outdir = sys.argv[1], sys.argv[2]
sdfdir = os.path.join(outdir, "_sdf"); os.makedirs(sdfdir, exist_ok=True)
ok = fail = 0
for line in open(smi_file):
    line = line.rstrip("\n")
    if not line or "\t" not in line: continue
    smiles, tier, cid = line.split("\t")[:3]
    name = f"{tier}_{cid}"
    m = Chem.MolFromSmiles(smiles)
    if m is None:
        print(f"  [FAIL] {name}: bad SMILES"); fail += 1; continue
    m = Chem.AddHs(m)
    ps = AllChem.ETKDGv3(); ps.randomSeed = 42
    if AllChem.EmbedMolecule(m, ps) != 0:
        print(f"  [FAIL] {name}: embed failed"); fail += 1; continue
    AllChem.MMFFOptimizeMolecule(m, maxIters=800)
    sdf = os.path.join(sdfdir, name + ".sdf")
    w = Chem.SDWriter(sdf); w.write(m); w.close()
    pdbqt = os.path.join(outdir, name + ".pdbqt")
    subprocess.run([MK, "-i", sdf, "-o", pdbqt], capture_output=True, timeout=300)
    if not (os.path.exists(pdbqt) and os.path.getsize(pdbqt) > 200):
        print(f"  [FAIL] {name}: meeko CLI"); fail += 1; continue
    ok += 1
print(f"prepared {ok} ({fail} failed) -> {outdir}")

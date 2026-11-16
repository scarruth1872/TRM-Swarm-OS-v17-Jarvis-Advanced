
import sys, os, subprocess
from rdkit import Chem
from rdkit.Chem import AllChem
inp, outdir = sys.argv[1], sys.argv[2]
os.makedirs(outdir, exist_ok=True)
SD = r"C:\Users\shawn\AppData\Local\Programs\Python\Python312\Scripts\mk_prepare_ligand.exe"
ok = fail = 0
for line in open(inp, encoding="utf-8"):
    line = line.strip()
    if not line: continue
    cid, smi = line.split("\t")
    m = Chem.MolFromSmiles(smi)
    if m is None:
        print("BAD SMILES", cid); fail += 1; continue
    m = Chem.AddHs(m)
    ps = AllChem.ETKDGv3(); ps.randomSeed = 42
    if AllChem.EmbedMolecule(m, ps) != 0:
        print("EMBED FAIL", cid); fail += 1; continue
    try: AllChem.MMFFOptimizeMolecule(m, maxIters=400)
    except Exception: pass
    sdf = os.path.join(outdir, cid + ".sdf")
    Chem.MolToMolFile(m, sdf)
    r = subprocess.run([SD, "-i", sdf, "-o", os.path.join(outdir, cid + ".pdbqt")],
                       capture_output=True, text=True)
    if r.returncode != 0 or not os.path.exists(os.path.join(outdir, cid + ".pdbqt")):
        print("PDBQT FAIL", cid, (r.stderr or "")[:80]); fail += 1
    else:
        ok += 1
print(f"prepared ok={ok} fail={fail}")

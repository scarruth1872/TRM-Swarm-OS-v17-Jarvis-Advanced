
"""Select a diverse active/weak pair from the ChEMBL LpxC tiering, then emit SMILES."""
import json, sys
from rdkit import Chem, RDLogger
from rdkit.Chem import AllChem, DataStructs
RDLogger.DisableLog("rdApp.*")

src = json.load(open(sys.argv[1], encoding="utf-8"))
def diverse(pool, n, thr=0.55):
    out, fps = [], []
    for v in pool:
        m = Chem.MolFromSmiles(v["smiles"])
        if m is None: continue
        fp = AllChem.GetMorganFingerprintAsBitVect(m, 2, nBits=2048)
        if any(DataStructs.TanimotoSimilarity(fp, f) > thr for f in fps): continue
        fps.append(fp); out.append(v)
        if len(out) >= n: break
    return out

# actives already filtered/sorted by potency in the tiers file
A = diverse(src["actives"], 22)
W = diverse(src["weak"], 22, thr=0.5)
print(f"diverse actives: {len(A)}   diverse weak: {len(W)}")
for v in A[:6]: print(f"  ACT  {v['chembl_id']:14} pChEMBL={v['pchembl']:.2f} {v['smiles'][:52]}")
for v in W[:6]: print(f"  WEAK {v['chembl_id']:14} pChEMBL={v['pchembl']:.2f} {v['smiles'][:52]}")
out = {"actives": A, "weak": W}
json.dump(out, open(sys.argv[2], "w", encoding="utf-8"), indent=2)
# emit a SMILES file for the prep tool
with open(sys.argv[3], "w") as f:
    for v in A: f.write(v["smiles"] + "\tACT\t" + v["chembl_id"] + "\n")
    for v in W: f.write(v["smiles"] + "\tWEAK\t" + v["chembl_id"] + "\n")
print("wrote", sys.argv[3])

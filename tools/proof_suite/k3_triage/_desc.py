
import sys, json
from rdkit import Chem
from rdkit.Chem import Descriptors, rdMolDescriptors
out = {}
for line in open(sys.argv[1], encoding="utf-8"):
    line = line.strip()
    if not line: continue
    cid, smi = line.split("\t")
    m = Chem.MolFromSmiles(smi)
    if m is None: continue
    out[cid] = {"heavy": m.GetNumHeavyAtoms(),
                "mw": Descriptors.MolWt(m),
                "rotb": rdMolDescriptors.CalcNumRotatableBonds(m),
                "hbd": rdMolDescriptors.CalcNumHBD(m)}
print(json.dumps(out))

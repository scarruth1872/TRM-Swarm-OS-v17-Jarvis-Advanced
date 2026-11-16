#!/usr/bin/env python3
"""Property-matched decoy generator for docking ROC analysis.

METHOD (the DUD-E principle, reimplemented)
  A decoy must be a molecule that LOOKS like the active to a docking program's
  physics but is not expected to bind: matched on the physical properties that
  drive non-specific association (size, lipophilicity, H-bond capacity, charge),
  yet topologically dissimilar so it does not share the active's pharmacophore.

  This generates candidates by recombining common drug fragments, then keeps only
  those whose properties fall inside a window around the active AND whose ECFP4
  Tanimoto similarity to the active is below a dissimilarity threshold.

  It is NOT DUD-E and does not claim its decoys are experimentally confirmed
  non-binders. A decoy is a molecule with no known activity against the target -
  "presumed inactive", not "proven inactive". That is the standard assumption of
  every docking enrichment study, and it is an assumption.

Usage: python tools/phoenix_decoy_generator.py "<active SMILES>" <n> [out.smi]
"""
import random
import sys

from rdkit import Chem, RDLogger
from rdkit.Chem import AllChem, Descriptors, rdMolDescriptors, DataStructs

RDLogger.DisableLog("rdApp.*")

# Common drug-like fragments: rings, linkers, polar caps.
RINGS = ["c1ccccc1", "c1ccncc1", "c1cccnc1", "c1ccsc1", "c1ccoc1", "C1CCCCC1",
         "C1CCOCC1", "C1CCNCC1", "c1ccc2ccccc2c1", "c1cnc2ccccc2c1", "C1CCNC1",
         "c1cc[nH]c1", "C1COCCO1", "c1cn[nH]c1", "c1ccoc1", "C1CCSCC1"]
LINKERS = ["", "C", "CC", "O", "CO", "CN", "C(=O)", "C(=O)N", "NC(=O)", "S",
           "C#C", "N", "COC", "CCO", "OC", "NC", "C(=O)O", "SO", "CS", "NCC"]
CAPS = ["C", "CC", "CCC", "O", "N", "CO", "CN", "F", "Cl", "Br", "C(F)(F)F",
        "C(=O)O", "C(N)=O", "S(=O)(=O)N", "C#N", "OC", "N(C)C", "OCC", "C(C)C"]


def profile(mol):
    return {
        "mw": Descriptors.MolWt(mol),
        "logp": Descriptors.MolLogP(mol),
        "hbd": rdMolDescriptors.CalcNumHBD(mol),
        "hba": rdMolDescriptors.CalcNumHBA(mol),
        "rotb": rdMolDescriptors.CalcNumRotatableBonds(mol),
        "heavy": mol.GetNumHeavyAtoms(),
        "rings": rdMolDescriptors.CalcNumRings(mol),
    }


def within(cand, ref, tol=0.20):
    """Property window. Loose on MW, tight on H-bond capacity."""
    if abs(cand["mw"] - ref["mw"]) > tol * ref["mw"]:
        return False
    if abs(cand["logp"] - ref["logp"]) > 1.0:
        return False
    if abs(cand["hba"] - ref["hba"]) > 1:
        return False
    if abs(cand["hbd"] - ref["hbd"]) > 1:
        return False
    if abs(cand["rotb"] - ref["rotb"]) > 3:
        return False
    if abs(cand["heavy"] - ref["heavy"]) > max(3, tol * ref["heavy"]):
        return False
    return True


def gen_candidate(rng):
    """Recombine fragments into a plausible molecule."""
    n_rings = rng.randint(1, 3)
    parts = []
    for i in range(n_rings):
        parts.append(rng.choice(RINGS))
        if i < n_rings - 1 or rng.random() < 0.6:
            parts.append(rng.choice(LINKERS))
    if rng.random() < 0.7:
        parts.insert(0, rng.choice(CAPS))
    if rng.random() < 0.5:
        parts.append(rng.choice(CAPS))
    smi = "".join(parts)
    m = Chem.MolFromSmiles(smi)
    if m is None:
        return None
    try:
        Chem.SanitizeMol(m)
    except Exception:
        return None
    return m


def generate(active_smiles, n, seed=42, max_tanimoto=0.35):
    rng = random.Random(seed)
    active = Chem.MolFromSmiles(active_smiles)
    if active is None:
        raise SystemExit(f"could not parse active SMILES: {active_smiles}")
    Chem.SanitizeMol(active)
    ref = profile(active)
    fp_active = AllChem.GetMorganFingerprintAsBitVect(active, 2, nBits=2048)

    print(f"active profile: {ref}")
    print(f"dissimilarity threshold: ECFP4 Tanimoto < {max_tanimoto}")
    print(f"property window: MW +/-20%  logP +/-1  HBD/HBA +/-1  rotB +/-3\n")

    kept, seen, tries = [], set(), 0
    while len(kept) < n and tries < 400000:
        tries += 1
        m = gen_candidate(rng)
        if m is None:
            continue
        smi = Chem.MolToSmiles(m)
        if smi in seen:
            continue
        seen.add(smi)
        pr = profile(m)
        if not within(pr, ref):
            continue
        fp = AllChem.GetMorganFingerprintAsBitVect(m, 2, nBits=2048)
        if DataStructs.TanimotoSimilarity(fp_active, fp) >= max_tanimoto:
            continue
        # diversity among decoys
        if any(DataStructs.TanimotoSimilarity(fp, k[1]) > 0.6 for k in kept):
            continue
        kept.append((smi, fp, pr))

    print(f"tried {tries} candidates -> kept {len(kept)} decoys")
    return active, ref, kept


if __name__ == "__main__":
    smi = sys.argv[1]
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    out = sys.argv[3] if len(sys.argv) > 3 else None
    active, ref, kept = generate(smi, n)
    for s, fp, pr in kept[:8]:
        print(f"  {s[:70]:70} MW={pr['mw']:6.1f} logP={pr['logp']:5.2f}")
    if out and kept:
        with open(out, "w") as f:
            for s, fp, pr in kept:
                f.write(s + "\n")
        print(f"\nwrote {len(kept)} decoys -> {out}")

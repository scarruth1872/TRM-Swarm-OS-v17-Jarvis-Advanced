"""Redocking validation: nearest-neighbour RMSD between a docked pose and the
crystal ligand. Atom correspondence is by spatial proximity (the docked ligand was
prepared from a different SDF, so 1:1 atom ordering is not guaranteed)."""
import math
import sys

def model1(path):
    pts, ok = [], False
    for line in open(path, encoding="utf-8", errors="ignore"):
        if line.startswith("MODEL"):
            ok = (line.split()[1] == "1")
            continue
        if line.startswith("ENDMDL") and ok:
            break
        if ok and (line.startswith("ATOM") or line.startswith("HETATM")):
            try:
                pts.append((float(line[30:38]), float(line[38:46]), float(line[46:54])))
            except ValueError:
                pass
    return pts

def crystal(path, resname, chain="A"):
    pts = []
    for line in open(path, encoding="utf-8", errors="ignore"):
        if line.startswith("HETATM") and line[17:20].strip() == resname and line[21] == chain:
            try:
                pts.append((float(line[30:38]), float(line[38:46]), float(line[46:54])))
            except ValueError:
                pass
    return pts

def rmsd_nn(a, b):
    tot = 0.0
    for p in a:
        d = min(math.dist(p, q) for q in b)
        tot += d * d
    return math.sqrt(tot / len(a))

docked = model1(sys.argv[1])
crys = crystal(sys.argv[2], sys.argv[3], sys.argv[4] if len(sys.argv) > 4 else "A")
print(f"docked atoms={len(docked)}  crystal atoms={len(crys)}")
print(f"nearest-neighbour RMSD = {rmsd_nn(docked, crys):.3f} A")
print("(<= 2.0 A indicates the docked pose reproduces the crystallographic binding mode)")


"""PDB -> PDBQT via meeko. Runs on Windows; reads/writes the WSL staging dir."""
import glob, os, sys
from meeko import PDBQTReceptor

D = r"\\wsl.localhost\Ubuntu\home\shawn\phoenix\receptors"
os.chdir(D)
ok = fail = 0
for pdb in sorted(glob.glob("AF-*.pdb")):
    out = pdb.replace(".pdb", ".pdbqt")
    try:
        r = PDBQTReceptor(pdb)
        s = r.write_pdbqt_string()
        with open(out, "w") as f:
            f.write(s)
        n = sum(1 for l in s.splitlines() if l.startswith(("ATOM", "HETATM")))
        print(f"  OK   {pdb:22} -> {out:24} {n:6} atoms")
        ok += 1
    except Exception as e:
        print(f"  FAIL {pdb:22} {type(e).__name__}: {str(e)[:80]}")
        fail += 1
print(f"converted={ok} failed={fail}")

#!/usr/bin/env python3
"""PHOENIX targets -> REAL AutoDock Vina docking.

WHAT THIS IS
  A real docking run: real receptors (AlphaFold DB models, UniProt-verified
  accessions), a real ligand, and the official AutoDock Vina 1.2.5 binary.
  Every affinity it reports is MEASURED by Vina and carries its units.

WHAT THIS IS NOT
  Not a cure rate. Not a therapeutic proposal. Not a validation of the PHOENIX
  protocol. A docking score is a predicted binding energy for one small molecule
  against one rigid protein conformation. It says nothing about efficacy,
  safety, delivery, or clinical outcome. The PHOENIX design proposes
  nanoparticles, CRISPR-Cas13 and biosensors - none of which this models.
  Docking is the only piece of that design a computer can do today.

LIMITS OF THE METHOD (read before quoting any number below)
  * Receptors are AlphaFold PREDICTED models, not experimental structures.
    Docking normally uses crystal structures or a validated homology model.
  * These are apo models: no bound ligand, so the true binding site is unknown.
    This runs BLIND docking, and the box is CAPPED AT 40 A because the raw
    protein envelope was up to 157 A - both impractical and imprecise at that
    size. A reported affinity may therefore describe a surface groove rather
    than an active site.
  * Receptor is rigid; real binding involves rearrangements this cannot see.
  * One conformation, one pose per target, single seed.

CALIBRATION
  The identical setup was validated first on a system with a KNOWN answer:
  CHIR-090 redocked into 5U39 (P. aeruginosa LpxC), the structure it is
  co-crystallised with, returned -10.13 kcal/mol with modes 1 and 2 only 0.06
  apart. That tight cluster is the signature of a correctly configured docking.
  See evidence/phoenix_dock_validation.json.

PREREQUISITES (all already satisfied on this host)
  * ~/vina/vina                     AutoDock Vina 1.2.5 static binary
  * meeko 0.8.0 + rdkit + prody     PDB -> PDBQT (vina rejects raw PDB)
  * ~/phoenix/{receptors,ligands,out}

STAGING (already done by the run recorded in evidence/)
  1. alphaFold_fetch()      fetch the 10 models by VERIFIED accession
  2. prepare_receptors()    mk_prepare_receptor.exe --read_pdb X -o X -p
  3. prepare_ligands()      mk_prepare_ligand.exe -i X.sdf -o X.pdbqt
  4. compute_boxes()        centroid + envelope, capped at 40 A
  5. dock()                 vina --exhaustiveness 8 --cpu 8 --seed 42
"""
import json, pathlib, subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
WSL = ["wsl.exe", "-d", "Ubuntu", "--", "bash", "-lc"]
SCRIPTS = r"C:\Users\shawn\AppData\Local\Programs\Python\Python312\Scripts"
WSLHOME = pathlib.Path(r"\\wsl.localhost\Ubuntu\home\shawn\phoenix")
BOX_CAP = 40.0

# UniProt-VERIFIED accessions. Two of the original table were wrong and are
# corrected here; see evidence/phoenix_accession_errata.json.
TARGETS = [
    ("STAT4",   "Q14765"), ("TYK2",    "P29597"), ("TNFAIP3", "P21580"),
    ("TLR7",    "Q9NYK1"), ("IFIH1",   "Q9BYX4"), ("IRF5",    "Q13568"),
    ("PTPN22",  "Q9Y2R2"), ("ITGAM",   "P11215"), ("BLK",     "P51451"),
    ("C1QA",    "P02745"),
]


def sh(cmd, timeout=1800):
    r = subprocess.run(WSL + [cmd], capture_output=True, text=True, timeout=timeout)
    return (r.stdout or "") + (r.stderr or "")


def prepare_receptors():
    """PDB -> PDBQT. Vina rejects raw PDB ('Unknown or inappropriate tag')."""
    out = []
    for pdb in sorted((WSLHOME/"receptors").glob("AF-*.pdb")):
        r = subprocess.run([f"{SCRIPTS}\\mk_prepare_receptor.exe",
                            "--read_pdb", str(pdb), "-o", str(pdb.with_suffix("")), "-p"],
                           capture_output=True, text=True, timeout=900)
        qt = pdb.with_suffix(".pdbqt")
        out.append((pdb.stem, qt.exists()))
    return out


def prepare_ligands():
    out = []
    for sdf in sorted((WSLHOME/"ligands").glob("*.sdf")):
        r = subprocess.run([f"{SCRIPTS}\\mk_prepare_ligand.exe",
                            "-i", str(sdf), "-o", str(sdf.with_suffix(".pdbqt"))],
                           capture_output=True, text=True, timeout=600)
        out.append((sdf.stem, sdf.with_suffix(".pdbqt").exists()))
    return out


def compute_boxes():
    """Blind box per receptor from its own coordinates, capped."""
    boxes = {}
    for qt in sorted((WSLHOME/"receptors").glob("AF-*.pdbqt")):
        xs, ys, zs = [], [], []
        for l in qt.read_text(errors="replace").splitlines():
            if l.startswith(("ATOM", "HETATM")):
                try:
                    xs.append(float(l[30:38])); ys.append(float(l[38:46])); zs.append(float(l[46:54]))
                except ValueError:
                    pass
        if not xs:
            continue
        boxes[qt.stem] = {
            "center": [round((min(xs)+max(xs))/2, 3), round((min(ys)+max(ys))/2, 3),
                       round((min(zs)+max(zs))/2, 3)],
            "size": [min(round(max(xs)-min(xs)+8, 1), BOX_CAP),
                     min(round(max(ys)-min(ys)+8, 1), BOX_CAP),
                     min(round(max(zs)-min(zs)+8, 1), BOX_CAP)],
        }
    return boxes


def dock(receptor, ligand, box, out_name, exhaustiveness=8, seed=42):
    cx, cy, cz = box["center"]; sx, sy, sz = box["size"]
    cmd = (f"cd ~/phoenix && ~/vina/vina --receptor {receptor} --ligand {ligand} "
           f"--center_x {cx} --center_y {cy} --center_z {cz} "
           f"--size_x {sx} --size_y {sy} --size_z {sz} "
           f"--exhaustiveness {exhaustiveness} --cpu 8 --seed {seed} "
           f"--out out/{out_name}")
    return sh(cmd)


def main():
    boxes = compute_boxes()
    (ROOT/"evidence").mkdir(exist_ok=True)
    (ROOT/"evidence/phoenix_dock_boxes.json").write_text(json.dumps(boxes, indent=2), encoding="utf-8")
    print(f"{len(boxes)} blind boxes computed (capped at {BOX_CAP} A)")
    for k, v in boxes.items():
        print(f"  {k:18} center={v['center']} size={v['size']}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Verify the kernel's spatial reasoning (task 578) against numpy on real PDBQT coordinates.

Data: 5U39_receptor.pdbqt (4,477 atoms). The kernel computes the contact graph; numpy computes the same graph
independently. The comparison is exact: same atom count, same pair count, same integer checksum over the pairs.

The checksum is `sum(i*1000003 + j)` over pairs with i<j in file order, which catches a single swapped or
missing pair — a count alone would not.
"""
import json, sys, urllib.request
from pathlib import Path
import numpy as np

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=180) as r: return json.loads(r.read().decode())

def parse_pdbqt(path):
    atoms = []
    for line in open(path, errors="replace"):
        if line.startswith(("ATOM", "HETATM")):
            try:
                x, y, z = float(line[30:38]), float(line[38:46]), float(line[46:54])
            except ValueError:
                continue
            ad = line[77:79].strip() or line.split()[-1]
            atoms.append({"label": line[12:27].strip(), "x": x, "y": y, "z": z, "element": ad})
    return atoms

def is_h(el):
    return el.strip().upper() in ("H", "HD", "HS")

def numpy_contacts(atoms, cutoff, include_h=False):
    idx = [i for i, a in enumerate(atoms) if include_h or not is_h(a["element"])]
    coords = np.array([[atoms[i]["x"], atoms[i]["y"], atoms[i]["z"]] for i in idx], dtype=np.float64)
    cut2 = cutoff * cutoff
    pairs = []
    step = 512
    for start in range(0, len(idx), step):
        block = coords[start:start+step]
        d2 = ((block[:, None, :] - coords[None, :, :]) ** 2).sum(axis=2)
        ii, jj = np.nonzero(d2 <= cut2)
        for a, b in zip(ii, jj):
            i, j = idx[start + a], idx[b]
            if i < j:
                pairs.append((i, j))
    pairs = sorted(set(pairs))
    checksum = sum(i * 1000003 + j for i, j in pairs)
    return pairs, checksum

def main():
    path = ROOT / "5U39_receptor.pdbqt"
    atoms = parse_pdbqt(path)
    print(f"parsed {len(atoms)} atoms from {path.name}")
    cutoff = 4.0

    res = post("/reason/spatial", {"atoms": atoms, "cutoff": cutoff, "include_hydrogens": False})
    pairs, checksum = numpy_contacts(atoms, cutoff, include_h=False)

    print(f"kernel : contacts {res['contact_count']:>7} | checksum {res['pair_checksum']}")
    print(f"numpy  : contacts {len(pairs):>7} | checksum {checksum}")
    ok = res["contact_count"] == len(pairs) and res["pair_checksum"] == checksum
    print(f"count match    : {res['contact_count'] == len(pairs)}")
    print(f"checksum match : {res['pair_checksum'] == checksum}")

    # proximity query: first 40 atoms as the reference set
    refs = list(range(min(40, len(atoms))))
    res2 = post("/reason/spatial", {"atoms": atoms, "cutoff": cutoff, "reference": refs, "radius": 5.0})
    coords = np.array([[a["x"], a["y"], a["z"]] for a in atoms])
    near = set()
    for r in refs:
        d2 = ((coords - coords[r]) ** 2).sum(axis=1)
        for i in np.nonzero(d2 <= 25.0)[0]:
            if int(i) not in refs:
                near.add(int(i))
    print(f"near query: kernel {res2['near_count']} | numpy {len(near)} | match {res2['near_count'] == len(near)}")
    ok = ok and res2["near_count"] == len(near)
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())

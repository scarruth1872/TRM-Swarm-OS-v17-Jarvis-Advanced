"""pdbbind_serialize.py - turn PDBbind v2020-refined complex directories into text records.

Measured structure of the archives (not assumed): each complex is `<part>/<pdbid>/` containing at minimum
  <pdbid>_l.smi        ligand SMILES (the join key)
  <pdbid>_ligand.mol2  ligand with coordinates
  <pdbid>_pocket.pdb   binding-site atoms
  <pdbid>_protein.pdb  receptor
  <pdbid>_l.pdb        ligand PDB  (plus mae/ply variants)

**The tarballs carry no affinity values.** Labels come from a separate real source: the PGDTA
`pdbbind_refinedset_2019_split` CSVs (4,463 complexes, affinity 2.00-11.92). That source does NOT declare the
affinity scale, so records say `affinity <v> (scale not declared in source)` and never claim pKd.

Vintage mismatch is recorded, not hidden: labels are the 2019 refined set, structures are v2020-refined. A complex
is emitted only when BOTH a SMILES and a label exist; the unmatched counts are reported by the caller.
"""
import io
import json
import os
import tarfile

POCKET_SUFFIX = "_pocket.pdb"
SMILES_SUFFIX = "_l.smi"


def load_labels(label_json: str) -> dict:
    """{pdb_id_lower: {'affinity': float, 'uniprot': str}} from the PGDTA split CSVs."""
    rows = json.load(open(label_json, encoding="utf-8"))
    out = {}
    for r in rows:
        pid = (r.get("drug_id") or "").strip().lower()
        try:
            aff = float(r.get("affinity"))
        except (TypeError, ValueError):
            continue
        if pid:
            out[pid] = {"affinity": aff, "uniprot": (r.get("uniprot_id") or "").strip()}
    return out


def load_resolution(csv_path: str) -> dict:
    """{pdb_id_lower: resolution} from LP_PDBBind.csv (structure metadata only, no labels)."""
    import csv as _csv
    out = {}
    if not os.path.exists(csv_path):
        return out
    with open(csv_path, encoding="utf-8", errors="replace") as fh:
        rd = _csv.reader(fh)
        next(rd, None)
        for row in rd:
            if len(row) < 6:
                continue
            pid = (row[0] or "").strip().lower()
            if pid:
                out[pid] = (row[5] or "").strip()
    return out


def iter_complexes(tar_path: str):
    """Yield (pdb_id, smiles, pocket_atom_count) for each complex directory in a v2020-refined tarball.

    Streams: only the SMILES file and the pocket file are read into memory, one complex at a time.
    """
    cur_id = None
    cur_smi = None
    with tarfile.open(tar_path, "r:gz") as tf:
        for m in tf:
            if not m.isfile():
                continue
            parts = m.name.split("/")
            # <root>/<pdbid>/... or <root>/<pdbid>/<pdbid>_prot/...
            if len(parts) < 3:
                continue
            pid = parts[1].strip().lower()
            base = parts[-1]
            if pid != cur_id:
                if cur_id and cur_smi:
                    yield cur_id, cur_smi, 0
                cur_id, cur_smi = pid, None
                continue
            if base.endswith(SMILES_SUFFIX) and cur_smi is None:
                f = tf.extractfile(m)
                if f is not None:
                    txt = f.read().decode("utf-8", "ignore").strip()
                    if txt:
                        # first whitespace-delimited token is the SMILES
                        cur_smi = txt.split()[0]
        if cur_id and cur_smi:
            yield cur_id, cur_smi, 0


def record_text(pdb_id: str, smiles: str, labels: dict, resolutions: dict) -> str | None:
    lab = labels.get(pdb_id)
    if not lab:
        return None
    res = resolutions.get(pdb_id)
    parts = [f"PDBbind complex {pdb_id.upper()} ligand SMILES {smiles}",
             f"affinity {lab['affinity']:.2f} (scale not declared in source)",
             "affinity_source pdbbind_refinedset_2019"]
    if lab.get("uniprot"):
        parts.append(f"protein {lab['uniprot']}")
    if res:
        parts.append(f"resolution {res} A")
    return " ".join(parts)

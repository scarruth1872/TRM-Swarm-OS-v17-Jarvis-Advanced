"""bindingdb_serialize.py - one definition of how a BindingDB TSV row becomes text.

Imported by both `k3_ingest_real.py` (serial) and `ingest_parallel.py --mode bindingdb` (byte-ranged workers), so
the two paths cannot drift. Format facts were measured from the archive, not assumed:

  * 243 columns, tab separated, one row per ligand-target measurement.
  * Affinity columns are Ki (nM), IC50 (nM), Kd (nM), EC50 (nM); a blank means nothing was recorded for that
    pair and is OMITTED from the text rather than written as 0 - a fabricated affinity is a fabricated label.
  * Ligand SMILES is the join key; rows without one are skipped and counted.
"""
import io
import zipfile

AFFINITY_COLUMNS = (("Ki_nM", "Ki (nM)"), ("IC50_nM", "IC50 (nM)"),
                    ("Kd_nM", "Kd (nM)"), ("EC50_nM", "EC50 (nM)"))

ID_COLUMNS = (("uniprot", "UniProt (SwissProt) Primary ID of Target Chain"),
              ("pdb", "PDB ID(s) for Ligand-Target Complex"),
              ("chembl", "ChEMBL ID of Ligand"),
              ("doi", "Article DOI"),
              ("pmid", "PMID"))

REQUIRED = ["Ligand SMILES", "BindingDB MonomerID", "BindingDB Ligand Name", "Target Name",
            "Target Source Organism According to Curator or DataSource"]


def open_text(path: str):
    """Return a text stream for either the .zip archive or a plain .tsv."""
    if path.endswith(".zip"):
        z = zipfile.ZipFile(path)
        member = [n for n in z.namelist() if n.lower().endswith(".tsv")][0]
        return io.TextIOWrapper(z.open(member), encoding="utf-8", errors="replace")
    return open(path, encoding="utf-8", errors="replace")


def header_index(first_line: str):
    hdr = first_line.rstrip("\n").split("\t")
    ix = {c.strip(): i for i, c in enumerate(hdr)}

    # The 2026-10 export (640 columns) dropped the bare UniProt column and ships only numbered chain blocks
    # ("UniProt (SwissProt) Primary ID of Target Chain 47"). Resolve to the FIRST such block, and say so: silently
    # dropping the field would quietly remove every target identifier from the corpus.
    bare = "UniProt (SwissProt) Primary ID of Target Chain"
    if bare not in ix:
        import re as _re
        cands = [(i, c.strip()) for i, c in enumerate(hdr)
                 if _re.fullmatch(r"UniProt \(SwissProt\) Primary ID of Target Chain\s*\d*", c.strip())]
        if not cands:
            raise SystemExit("FAIL-CLOSED: no UniProt SwissProt primary-ID column in any form")
        cands.sort()
        ix[bare] = cands[0][0]
        print(f"  note: using column {cands[0][0]} ({cands[0][1]!r}) as {bare!r}", flush=True)

    missing = [w for w in REQUIRED + [c for _, c in AFFINITY_COLUMNS + ID_COLUMNS] if w not in ix]
    if missing:
        raise SystemExit(f"FAIL-CLOSED: BindingDB columns not found: {missing[:4]}")
    return ix


def row_to_record(fields, ix, seq: int):
    """Return (key, text) or None when the row carries no ligand SMILES."""
    if len(fields) < 12:
        return None

    def g(col):
        i = ix[col]
        return fields[i].strip() if i < len(fields) else ""

    smi = g("Ligand SMILES")
    if not smi:
        return None
    aff = " ".join(f"{lbl}={g(col)}" for lbl, col in AFFINITY_COLUMNS if g(col))
    parts = [f"BindingDB monomer {g('BindingDB MonomerID')} ligand {g('BindingDB Ligand Name')}",
             f"SMILES {smi}",
             f"target {g('Target Name')}",
             f"organism {g('Target Source Organism According to Curator or DataSource')}",
             f"affinity {aff if aff else 'none recorded'}"]
    for lbl, col in ID_COLUMNS:
        v = g(col)
        if v:
            parts.append(f"{lbl} {v}")
    return (f"bindingdb/{g('BindingDB MonomerID')}/{seq}", " ".join(parts))

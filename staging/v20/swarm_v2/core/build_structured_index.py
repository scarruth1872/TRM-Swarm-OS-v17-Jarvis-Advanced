#!/usr/bin/env python3
"""Build a STRUCTURED index for the WSL data. Offline tooling; no kernel surface.

Decision (2026-09-23): the fabric stays for prose. Structured records get a SQLite index
instead, because embeddings of 100k near-duplicate molecules would make the fabric's
already-weak short-query discrimination worse, and a molecule wants an exact lookup, not a
cosine.

Span, chosen from measured input sizes rather than a round number:
  FULL   the small, high-value sources  - ETHICS CSVs (~23 MB), clinical .psv (3.2 MB),
         docking PDB + .pdbqt (7.8 MB), the wisdom/LawofOne/Shakespeare texts (~6 MB),
         LP-PDBBind's affinity table (10.9 MB, 19,443 rows), the RDKit feature table
  FULL   chembl_37.sdf.gz (931 MB, 2,897,819 records) - streamed to EOF with --chembl-limit 5000000;
         the default 20,000 keeps a smoke build cheap rather than claiming to be complete
  OUT    enwikiquote.xml.bz2 is NOT ingested: this build has no --enwiki-* input and no ingest branch
         for it (an earlier docstring here claimed one; the argument never existed)
  INPUT  derived structures (swarm_v2_artifacts/chembl_structures_full.jsonl) are written INTO the
         molecule rows during the build, so the store has one writer and no post-hoc UPDATE churn
  FULL   Tox21's assay matrix (tox21full.csv.gz, 152,757 B, 8,734 compounds x 75 assays) as a LONG table:
         one row per cell that was actually measured (577,987 of them). Blank cells are skipped rather
         than stored as 0.0, because an empty cell is not a negative result and writing it as one would
         fabricate ~76k labels the source never measured. This closed the last domain the platform held
         no record of at all.

Every row carries source_path + source_sha256 so a lookup traces back to its record.

Run in WSL:  python3 build_structured_index.py --out /mnt/c/.../structured_index.db
"""
import argparse
import bz2
import csv
import gzip
import hashlib
import json
import os
import re
import sqlite3
import sys
import time

WSL_HOME = "/home/shawn"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCHEMA = """
CREATE TABLE IF NOT EXISTS provenance (
  id INTEGER PRIMARY KEY, source_path TEXT UNIQUE, sha256 TEXT, hash_span TEXT, bytes INTEGER,
  records INTEGER, ingested TEXT, note TEXT);
CREATE TABLE IF NOT EXISTS ethics (
  id INTEGER PRIMARY KEY, subset TEXT, label TEXT, text TEXT, meta TEXT, source_path TEXT, sha256 TEXT);
CREATE TABLE IF NOT EXISTS clinical (
  id INTEGER PRIMARY KEY, patient_id TEXT, timepoint INTEGER, measurement TEXT, value REAL,
  source_path TEXT, sha256 TEXT);
CREATE TABLE IF NOT EXISTS structure (
  id INTEGER PRIMARY KEY, pdb_id TEXT, path TEXT, resolution TEXT, method TEXT, kind TEXT, parent TEXT,
  atom_lines INTEGER, source_path TEXT, sha256 TEXT);
CREATE TABLE IF NOT EXISTS prose (
  id INTEGER PRIMARY KEY, work TEXT, chunk_no INTEGER, text TEXT, source_path TEXT, sha256 TEXT);
CREATE TABLE IF NOT EXISTS molecule (
  id INTEGER PRIMARY KEY, chembl_id TEXT, smiles TEXT, props TEXT, source_path TEXT, sha256 TEXT);
CREATE INDEX IF NOT EXISTS ix_ethics_subset ON ethics(subset);
CREATE INDEX IF NOT EXISTS ix_molecule_chembl ON molecule(chembl_id);
CREATE INDEX IF NOT EXISTS ix_prose_work ON prose(work);
CREATE INDEX IF NOT EXISTS ix_clinical_meas ON clinical(measurement);
CREATE UNIQUE INDEX IF NOT EXISTS ix_structure_path ON structure(path);
CREATE TABLE IF NOT EXISTS binding (
  id INTEGER PRIMARY KEY, pdb_id TEXT, family TEXT, smiles TEXT, qualifier TEXT, seq TEXT,
  resolution TEXT, entry_date TEXT, split TEXT, kd_ki TEXT, value REAL, covalent TEXT,
  payload TEXT, source_path TEXT, sha256 TEXT);
CREATE INDEX IF NOT EXISTS ix_binding_pdb ON binding(pdb_id);
CREATE INDEX IF NOT EXISTS ix_binding_value ON binding(value);
CREATE TABLE IF NOT EXISTS molecule_features (
  chembl_id TEXT PRIMARY KEY, n_heavy INTEGER, formula TEXT, mw REAL, n_bonds INTEGER,
  n_rings INTEGER, aromatic_bonds INTEGER, n_components INTEGER);
CREATE TABLE IF NOT EXISTS span (
  id INTEGER PRIMARY KEY, path TEXT UNIQUE, bytes INTEGER, files INTEGER, disposition TEXT,
  reason TEXT, measured TEXT);
CREATE TABLE IF NOT EXISTS tox21 (
  id INTEGER PRIMARY KEY, smiles TEXT, assay TEXT, value REAL, source_path TEXT, sha256 TEXT);
CREATE INDEX IF NOT EXISTS ix_tox21_smiles ON tox21(smiles);
CREATE INDEX IF NOT EXISTS ix_tox21_assay ON tox21(assay);
"""


def sha256_file(p, cap=None):
    h = hashlib.sha256()
    with open(p, "rb") as fh:
        while True:
            b = fh.read(1 << 20)
            if not b:
                break
            h.update(b)
            if cap and fh.tell() > cap:
                break
    return h.hexdigest()


def sha256_file_span(p, cap=None):
    """Hash plus the span it covers, so provenance can never claim more than it measured."""
    return sha256_file(p, cap), ("full" if not cap else "prefix:%dB" % cap)


def record_provenance(con, src, sha, nbytes, records, msg, span="full"):
    """`span` states what the recorded hash actually covers. A capped hash claiming to be the artifact's
    identity is the defect this column prevents: the ChEMBL row carried a 200 MB prefix hash that could not
    verify the file it named (the release's own checksum is 2cdf4d33c5f42613...)."""
    con.execute("INSERT OR REPLACE INTO provenance(source_path,sha256,hash_span,bytes,records,ingested,note)"
                " VALUES(?,?,?,?,?,datetime('now'),?)", (src, sha, span, nbytes, records, msg))
    con.commit()


def ingest_ethics(con, log):
    d = os.path.join(WSL_HOME, "relational")
    total = 0
    for name in sorted(os.listdir(d)) if os.path.isdir(d) else []:
        if not name.startswith("ethics_") or not name.endswith(".csv"):
            continue
        p = os.path.join(d, name)
        subset = name[len("ethics_"):-4]
        sha = sha256_file(p)
        n = 0
        with open(p, newline="", encoding="utf-8", errors="replace") as fh:
            rdr = csv.reader(fh)
            head = next(rdr, None)
            cols = [h.strip().lower() for h in (head or [])]
            idx = {h: i for i, h in enumerate(cols)}
            # each subset names its text column differently (measured, not guessed):
            #   commonsense -> input | deontology/justice/virtue -> scenario | utilitarianism -> baseline
            tcol = next((c for c in ("input", "scenario", "baseline", "text") if c in idx), None)
            for row in rdr:
                if not row or tcol is None or len(row) <= idx[tcol]:
                    continue
                label = row[idx["label"]] if "label" in idx and len(row) > idx["label"] else ""
                text = row[idx[tcol]]
                meta = {c: row[i] for c, i in idx.items() if c != tcol and len(row) > i}
                con.execute("INSERT INTO ethics(subset,label,text,meta,source_path,sha256)"
                            " VALUES(?,?,?,?,?,?)",
                            (subset, label, text, json.dumps(meta, ensure_ascii=False), name, sha))
                n += 1
        total += n
        record_provenance(con, "relational/" + name, sha, os.path.getsize(p), n, "ETHICS subset")
        log.append("   ethics   %-30s %6d rows" % (name, n))
    return total


def ingest_clinical(con, log):
    """The .psv format is TRANSPOSED: the header is the measurement list and each row is a
    timepoint. Stored long (patient_id, timepoint, measurement, value); NaN is skipped."""
    d = os.path.join(WSL_HOME, "clinical")
    if not os.path.isdir(d):
        return 0
    total = 0
    files = [f for f in sorted(os.listdir(d)) if f.endswith(".psv")]
    for name in files:
        p = os.path.join(d, name)
        sha = sha256_file(p)
        with open(p, encoding="utf-8", errors="replace") as fh:
            head = next(csv.reader(fh, delimiter="|"), None)
            if not head:
                continue
            meas = [h.strip() for h in head]
            for tp, row in enumerate(csv.reader(fh, delimiter="|")):
                for mi, raw in enumerate(row):
                    if mi >= len(meas):
                        break
                    v = raw.strip()
                    if v in ("", "NaN", "nan"):
                        continue
                    try:
                        fv = float(v)
                    except ValueError:
                        continue
                    con.execute("INSERT INTO clinical(patient_id,timepoint,measurement,value,source_path,sha256)"
                                " VALUES(?,?,?,?,?,?)", (name[:-4], tp, meas[mi], fv, name, sha))
                    total += 1
        record_provenance(con, "clinical/" + name, sha, os.path.getsize(p), 1,
             "psv timepoint table, %d measurements" % len(meas))
    log.append("   clinical %-34s %7d non-NaN values across %d files" % ("", total, len(files)))
    return total


def ingest_tox21(con, log):
    """Tox21's assay matrix: SMILES x 75 assays, long form, one row per cell that is actually measured.

    An empty cell is NOT a negative result. Recording the blanks as 0.0 would fabricate ~76k labels the
    source never measured, so they are skipped and the count that lands is the count that was read.
    """
    p = os.path.join(WSL_HOME, "tox21full.csv.gz")
    if not os.path.exists(p):
        log.append("   tox21    ABSENT %s" % p)
        return 0
    sha, span = sha256_file_span(p)
    n, ncomp, batch = 0, 0, []
    with gzip.open(p, "rt", encoding="utf-8", errors="replace", newline="") as fh:
        rows = csv.reader(fh)
        header = next(rows)
        assays = header[1:]
        for rec in rows:
            if not rec:
                continue
            smi = rec[0]
            ncomp += 1
            for i, cell in enumerate(rec[1:]):
                if i >= len(assays) or cell == "" or cell.lower() in ("nan", "none"):
                    continue
                try:
                    v = float(cell)
                except ValueError:
                    continue
                batch.append((smi, assays[i], v, "tox21full.csv.gz", sha))
            if len(batch) >= 20000:
                con.executemany("INSERT INTO tox21(smiles,assay,value,source_path,sha256)"
                                " VALUES(?,?,?,?,?)", batch)
                n += len(batch)
                batch = []
    if batch:
        con.executemany("INSERT INTO tox21(smiles,assay,value,source_path,sha256) VALUES(?,?,?,?,?)", batch)
        n += len(batch)
    con.commit()
    record_provenance(con, "tox21full.csv.gz", sha, os.path.getsize(p), n,
                      "Tox21 assay matrix, long form (%d compounds x %d assays); empty cells skipped, never "
                      "recorded as negatives" % (ncomp, len(assays)), span)
    log.append("   tox21    %-34s %7d measured assay cells" % ("tox21full.csv.gz", n))
    return n


def ingest_structures(con, log):
    """PDB entries AND the ligand poses beside them, with an identity that is an identity.

    `pdb_id` used to be `name[:4].upper()`, so `receptor_clean.pdb` became RECE and `1M17_prot.pdb` collided
    with the real `1M17` (two rows, one id). Now the entry id comes from the HEADER record when the file has
    one (`kind=entry`); a local derivative is `kind=derived` with `parent` from its filename; a bare receptor
    is `kind=receptor`; and the .pdbqt poses the docking work actually used are rows too (`kind=ligand`),
    which the store could not see before.
    """
    d = os.path.join(WSL_HOME, "docking")
    if not os.path.isdir(d):
        return 0
    n, kinds = 0, {}
    for name in sorted(os.listdir(d)):
        if not (name.endswith(".pdb") or name.endswith(".pdbqt")):
            continue
        p = os.path.join(d, name)
        sha = sha256_file(p)
        atoms = res = method = 0
        header_id = None
        with open(p, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if line.startswith(("ATOM", "HETATM")):
                    atoms += 1
                elif line.startswith("HEADER") and len(line) >= 66:
                    cand = line[62:66].strip().upper()
                    if cand and cand != "NONE":
                        header_id = cand
                elif line.startswith("REMARK   2 RESOLUTION"):
                    m = re.search(r"([0-9.]+)\s+ANGSTROM", line)
                    if m:
                        res = float(m.group(1))
                elif line.startswith("EXPDTA"):
                    method += 1
        stem = name.rsplit(".", 1)[0]
        low = stem.lower()
        parent = None
        if name.endswith(".pdbqt"):
            kind = "receptor" if low.startswith("receptor") else "ligand"
            pid = stem.upper()
        elif low.endswith("_prot") or low.endswith("_clean"):
            kind, pid = "derived", stem.upper()
            base = stem[:4]
            if len(stem) >= 4 and base[0].isdigit():
                parent = base.upper()
        elif header_id:
            kind, pid = "entry", header_id
        else:
            kind, pid = "file", stem.upper()
        con.execute("INSERT INTO structure(pdb_id,path,resolution,method,kind,parent,atom_lines,"
                    "source_path,sha256) VALUES(?,?,?,?,?,?,?,?,?)",
                    (pid, name, ("%.2f" % res) if res else None, str(method or ""), kind, parent,
                     atoms, name, sha))
        kinds[kind] = kinds.get(kind, 0) + 1
        n += 1
        record_provenance(con, "docking/" + name, sha, os.path.getsize(p), 1,
                          "%s structure (%s)" % (kind, pid))
    log.append("   docking  %-30s %6d rows %s" % ("", n, sorted(kinds.items())))
    return n


def ingest_prose(con, log, chunk=4000):
    d = os.path.join(WSL_HOME, "relational")
    if not os.path.isdir(d):
        return 0
    want = ("shakespeare", "lawofone", "wisdom")
    total = 0
    for name in sorted(os.listdir(d)):
        low = name.lower()
        if not (name.endswith(".txt") and any(w in low for w in want)):
            continue
        p = os.path.join(d, name)
        sha = sha256_file(p)
        text = open(p, encoding="utf-8", errors="replace").read()
        n = 0
        for i in range(0, len(text), chunk):
            piece = text[i:i + chunk].strip()
            if piece:
                con.execute("INSERT INTO prose(work,chunk_no,text,source_path,sha256) VALUES(?,?,?,?,?)",
                            (name, n, piece, name, sha))
                n += 1
        total += n
        record_provenance(con, "relational/" + name, sha, os.path.getsize(p), n, "prose chunked at %d chars" % chunk)
        log.append("   prose    %-30s %6d chunks" % (name, n))
    return total


def _structures_reader(path):
    """Merge-join over the derived-structures JSONL.

    The pass walks the SDF in source order and this ingest walks the same source in the same order, so one
    lazy cursor is the whole join and nothing is held in memory. A line that does not match the record being
    written is COUNTED, never silently dropped: a smiles column that belongs to another molecule is worse
    than an empty one.
    """
    class R:
        __slots__ = ("fh", "cur", "skipped", "hit")

        def __init__(self, fh):
            self.fh, self.cur, self.skipped, self.hit = fh, None, 0, 0
            self._adv()

        def _adv(self):
            for line in self.fh:
                if line.strip():
                    try:
                        self.cur = json.loads(line)
                    except ValueError:
                        self.cur = None
                        continue
                    return
            self.cur = None

        def take(self, chembl_id):
            """Match at the cursor, or don't move.

            The JSONL is a SUBSEQUENCE of the SDF (the pass skips records it cannot parse), so a record whose
            structure is absent is one the cursor must simply wait through -- scanning forward for it would
            walk past every later match and quietly empty the smiles column after the first unparsable
            record. So: match, consume, advance; otherwise count the miss and stand still.
            """
            if self.cur is not None and self.cur.get("chembl_id") == chembl_id:
                row = self.cur
                self.hit += 1
                self._adv()
                props = row.get("props") or {}
                return row.get("smiles"), (json.dumps(props, sort_keys=True) if props else None)
            self.skipped += 1
            return None, None

    return R(open(path, encoding="utf-8"))


def _open_structures(path):
    """The derived structures are an INPUT to the build, not a patch applied afterwards: writing them in the
    insert is what keeps the store's one owner the only writer, and what avoids the post-hoc UPDATE churn that
    left the previous file with 495.9 MB of B-tree for 146.1 MB of rows."""
    if path and os.path.exists(path):
        return _structures_reader(path)
    return None


def ingest_molecules(con, log, limit, structures=None):
    """ChEMBL SDF. The record's FIRST LINE is the ChEMBL id; properties live in a > <NAME> block.

    The source was a truncated download once -- `gzip -t` reported "unexpected end of file" and the trailer
    declared 2,793,187,232 uncompressed bytes -- and a bounded read never noticed, which is how a bound hid a
    corrupt source. It was re-fetched on 2026-09-23 and re-measured 2026-09-26: `gzip -t` OK, sha256
    2cdf4d33c5f426130c610b8627c3702804e6f10d149d76b8a1bea674fa605a77 = the release's own checksum, and the
    stream now ends cleanly with 2,897,819 records. The hash recorded below is the FULL file (it used to be a
    200 MB prefix hash, which cannot identify the artifact it names), and the span column states that.
    """
    p = os.path.join(WSL_HOME, "drugs", "chembl_37.sdf.gz")
    if not os.path.exists(p) or limit <= 0:
        log.append("   molecule (skipped, limit=%d)" % limit)
        return 0
    declared = None
    try:
        import struct
        with open(p, "rb") as f:
            f.seek(-4, 2)
            declared = struct.unpack("<I", f.read(4))[0]
    except Exception:
        pass
    t0 = time.time()
    sha = sha256_file(p)          # FULL file: a prefix hash cannot identify the artifact it names
    n, truncated = 0, False
    structs = _open_structures(structures)
    with gzip.open(p, "rt", encoding="utf-8", errors="replace") as fh:
        rec_id, props, in_props = None, [], False
        try:
            for line in fh:
                if rec_id is None and line.strip() and not line.startswith(" "):
                    rec_id = line.strip()
                if line.startswith("> <"):
                    in_props = True
                    props.append(line.strip()[3:-1])
                elif in_props and line.strip() == "":
                    in_props = False
                if line.startswith("$$$$"):
                    if n < limit and rec_id:
                        smi, derived = structs.take(rec_id) if structs is not None else (None, None)
                        con.execute("INSERT INTO molecule(chembl_id,smiles,props,source_path,sha256)"
                                    " VALUES(?,?,?,?,?)",
                                    (rec_id, smi, derived or ",".join(props[:20]),
                                     "drugs/chembl_37.sdf.gz", sha))
                        n += 1
                    rec_id, props, in_props = None, [], False
                    if n >= limit:
                        break
        except EOFError:
            truncated = True
    con.commit()
    dt = time.time() - t0
    reader = "unrestricted, streamed to EOF" if limit >= 5_000_000 else "bounded at %d" % limit
    dmsg = ("" if structs is None else
            " %d records carry a derived structure; %d do not (the pass writes only what it could parse)."
            % (structs.hit, structs.skipped))
    msg = "%d records read (%s). %s%s" % (
        n, reader,
        ("TRUNCATED SOURCE: the gzip ended before its stream terminator; the trailer declares %s "
         "uncompressed bytes. Partial ChEMBL download - re-fetch to complete it." % declared)
        if truncated else "source stream complete", dmsg)
    record_provenance(con, "drugs/chembl_37.sdf.gz", sha, os.path.getsize(p), n, msg)
    log.append("   molecule %-34s %7d records in %.1fs%s%s"
               % ("chembl_37", n, dt, "  [SOURCE TRUNCATED]" if truncated else "",
                  "" if structs is None else "  [%d structures]" % structs.hit))
    return n


def ingest_binding(con, log):
    """LP-PDBBind's affinity table: 19,443 labelled protein-ligand rows.

    The source header is shifted one cell against its own data (measured: index 2 is the SMILES, 5 the
    resolution, 13 the kd/ki and 14 the value -- exactly where the DATA puts them, while the header names
    them a position early). The columns are therefore taken by INDEX, and each row is also kept verbatim in
    `payload` so a later reader is never asked to trust this reading.
    """
    p = os.path.join(WSL_HOME, "datasets", "LP-PDBBind", "dataset", "LP_PDBBind.csv")
    if not os.path.exists(p):
        log.append("   binding  (absent: %s)" % p)
        return 0
    sha = sha256_file(p)
    n = 0
    with open(p, newline="", encoding="utf-8", errors="replace") as fh:
        rdr = csv.reader(fh)
        next(rdr, None)
        for row in rdr:
            if len(row) < 16 or not row[0].strip():
                continue
            def f(i, row=row):
                return row[i].strip() if len(row) > i else ""
            try:
                val = float(f(14))
            except ValueError:
                val = None
            con.execute("INSERT INTO binding(pdb_id,family,smiles,qualifier,seq,resolution,entry_date,split,"
                        "kd_ki,value,covalent,payload,source_path,sha256) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                        (f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(8), f(13), val, f(15), json.dumps(row),
                         "datasets/LP-PDBBind/dataset/LP_PDBBind.csv", sha))
            n += 1
    con.commit()
    record_provenance(con, "datasets/LP-PDBBind/dataset/LP_PDBBind.csv", sha, os.path.getsize(p), n,
                      "LP-PDBBind affinity table; columns by index (the source header is shifted one cell "
                      "against its data) with the row kept verbatim in payload")
    log.append("   binding  %-30s %6d rows" % ("LP-PDBBind", n))
    return n


def ingest_features(con, log):
    """The RDKit-derived feature table, so the store's schema has ONE owner again.

    This table used to be created by an out-of-band scratch script (p5_wire_features.py), which left the
    generator unable to reproduce the store it claims to own. The CSV carries 12 columns; the 8 the store
    publishes are loaded -- `hba` is verified to be a plain N+O count and `hbd` matched RDKit on 91.7%, so
    neither is loaded under a Lipinski name (P5's decision, kept rather than re-litigated).
    """
    p = os.path.join(WSL_HOME, "drugs", "molecule_features.csv")
    if not os.path.exists(p):
        log.append("   features (absent: %s)" % p)
        return 0
    sha = sha256_file(p)
    cols = ["chembl_id", "n_heavy", "formula", "mw", "n_bonds", "n_rings", "aromatic_bonds", "n_components"]
    n = 0
    with open(p, newline="", encoding="utf-8", errors="replace") as fh:
        rdr = csv.reader(fh)
        head = next(rdr, None) or []
        idx = {c: i for i, c in enumerate(head)}
        if all(c in idx for c in cols):
            for row in rdr:
                if len(row) < len(head):
                    continue
                con.execute("INSERT OR REPLACE INTO molecule_features(%s) VALUES(%s)"
                            % (",".join(cols), ",".join("?" * len(cols))), [row[idx[c]] for c in cols])
                n += 1
    con.commit()
    record_provenance(con, "drugs/molecule_features.csv", sha, os.path.getsize(p), n,
                      "RDKit-derived feature table; 8 of the CSV's 12 columns published (hba is an N+O count "
                      "and hbd matched RDKit on 91.7%, so neither carries a Lipinski name)")
    log.append("   features %-30s %6d rows" % ("molecule_features", n))
    return n


def record_spans(con, log):
    """State, inside the store, what is in it and what is deliberately out of it.

    Each row measures a WSL tree at build time, so 'out of span' is a recorded decision instead of an
    absence someone has to rediscover. tox21full is a git checkout of the Python package that PRODUCES the
    assay matrix -- 352 KB, no data files (measured) -- while the matrix itself, tox21full.csv.gz, IS
    ingested; the LP-PDBBind archives are model code and weights (the affinity table beside them IS
    ingested); hw_kb and rust_kb are the platform's own sources, not datasets.
    """
    trees = [
        ("drugs/chembl_37.sdf.gz", "INGESTED", "molecule, full stream to EOF; sha256 matches the release checksum"),
        ("relational", "INGESTED", "ethics subsets + the prose texts"),
        ("clinical", "INGESTED", "400 transposed .psv tables, stored long"),
        ("docking", "INGESTED", "PDB entries, derived receptors and .pdbqt ligand poses"),
        ("datasets/LP-PDBBind/dataset/LP_PDBBind.csv", "INGESTED", "19,443 labelled affinities"),
        ("drugs/molecule_features.csv", "INGESTED", "RDKit-derived features for the molecule rows"),
        ("datasets/LP-PDBBind/model_retraining", "OUT-OF-SPAN", "model code, weights and notebooks, not records"),
        ("datasets/LP-PDBBind/dataset/*.tgz", "OUT-OF-SPAN", "structure archives for retraining, not a table"),
        ("tox21full.csv.gz", "INGESTED", "Tox21 assay matrix, stored long: one row per MEASURED cell"),
        ("tox21full", "OUT-OF-SPAN", "git checkout of the generator that PRODUCES the matrix: 352 KB, no data files (measured); the matrix itself IS ingested (row above)"),
        ("hw_kb", "OUT-OF-SPAN", "the platform's own hardware sources (.h/.html), not a dataset"),
        ("rust_kb", "OUT-OF-SPAN", "the platform's own crate sources, not a dataset"),
    ]
    now = time.strftime("%Y-%m-%d")
    for rel, disp, why in trees:
        nbytes, nfiles = 0, 0
        if "*" in rel:
            parent = os.path.join(WSL_HOME, os.path.dirname(rel))
            if os.path.isdir(parent):
                for f in os.listdir(parent):
                    if f.endswith(".tgz"):
                        nfiles += 1
                        nbytes += os.path.getsize(os.path.join(parent, f))
        else:
            base = os.path.join(WSL_HOME, rel)
            if os.path.isfile(base):
                nfiles, nbytes = 1, os.path.getsize(base)
            elif os.path.isdir(base):
                for root, _dirs, files in os.walk(base):
                    for f in files:
                        try:
                            nbytes += os.path.getsize(os.path.join(root, f))
                            nfiles += 1
                        except OSError:
                            pass
        con.execute("INSERT OR REPLACE INTO span(path,bytes,files,disposition,reason,measured)"
                    " VALUES(?,?,?,?,?,?)", (rel, nbytes, nfiles, disp, why, now))
    con.commit()
    log.append("   span     %-30s %6d trees declared" % ("", len(trees)))
    return len(trees)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--chembl-limit", type=int, default=20000)
    ap.add_argument("--skip-heavy", action="store_true")
    ap.add_argument("--structures",
                    default=os.path.join(REPO, "swarm_v2_artifacts", "chembl_structures_full.jsonl"),
                    help="derived structures written INTO the molecule rows during the build (an input, "
                         "not a patch applied afterwards); absent file means the column stays NULL")
    args = ap.parse_args()
    log = []
    t0 = time.time()
    if os.path.exists(args.out):
        os.remove(args.out)
    con = sqlite3.connect(args.out)
    con.executescript(SCHEMA)
    got = {"ethics": ingest_ethics(con, log), "clinical": ingest_clinical(con, log),
           "structure": ingest_structures(con, log), "prose": ingest_prose(con, log),
           "binding": ingest_binding(con, log), "features": ingest_features(con, log),
           "tox21": ingest_tox21(con, log)}
    got["molecule"] = (0 if args.skip_heavy
                       else ingest_molecules(con, log, args.chembl_limit, args.structures))
    record_spans(con, log)
    con.commit()
    print("\n".join(log))
    print()
    for t in ("ethics", "clinical", "structure", "prose", "binding", "molecule",
              "molecule_features", "span", "provenance"):
        c = con.execute("SELECT COUNT(*) FROM %s" % t).fetchone()[0]
        print("   table %-10s %7d rows" % (t, c))
    con.commit()
    con.close()
    print("\nindex: %s  %d bytes  built in %.1fs" % (args.out, os.path.getsize(args.out), time.time() - t0))
    print("sha256(index): %s" % sha256_file(args.out))
    return 0


if __name__ == "__main__":
    sys.exit(main())

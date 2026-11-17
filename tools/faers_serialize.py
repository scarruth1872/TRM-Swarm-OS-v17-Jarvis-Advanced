"""faers_serialize.py - FAERS quarterly ASCII files -> one text record per case report.

Source: FDA quarterly export `https://fis.fda.gov/content/Exports/faers_ascii_<YYYY>Q<n>.zip` (2026Q3 measured at
62,624,514 B - the "14.5 GB" figure elsewhere in the tree is *all* years, which is why only the current quarter is
ingested).

Layout follows the published FAERS ASCII schema, read from the extracted files rather than assumed:
  DEMO  primaryid, caseid, caseversion, i_f_code, event_dt, ..., age, age_cod, sex, e_sub, wt, ..., rept_dt, ...
  DRUG  primaryid, caseid, drug_seq, role_cod, drugname, prod_ai, ...
  REAC  primaryid, caseid, pt            (MedDRA preferred term - the adverse event)
  INDI  primaryid, caseid, indi_pt       (indication)
  OUTC  primaryid, caseid, outc_cod      (outcome code)
  THER  primaryid, caseid, start_dt, end_dt, dur, dur_cod

Rules:
  * DEMO is the spine: one record per `primaryid`, so the record count is exactly the number of case-version rows.
  * Child tables are joined by primaryid; every list keeps its true size (`+N more`) so truncation cannot read as
    completeness.
  * Missing values stay missing - a blank sex or age is omitted, never rendered as "unknown".
"""
import io
import os
import zipfile

DEMO_COLS = ["primaryid", "caseid", "caseversion", "i_f_code", "event_dt", "mfr_dt", "init_fda_dt", "fda_dt",
             "rept_cod", "auth_num", "mfr_num", "mfr_sndr", "lit_ref", "age", "age_cod", "age_grp", "sex",
             "e_sub", "wt", "wt_cod", "rept_dt", "to_mfr", "occp_cod", "reporter_country", "occr_country"]

MAX_LIST = 8


def _find_member(z: zipfile.ZipFile, prefix: str):
    for n in z.namelist():
        if os.path.basename(n).upper().startswith(prefix) and n.lower().endswith(".txt"):
            return n
    return None


def _rows(z, prefix):
    """Yield (header, reader) for a table: header names resolved once, then field lists.

    Column POSITIONS are not assumed. The 2026Q3 INDI table is `primaryid, caseid, indi_drug_seq, indi_pt` -
    four columns - so a hard-coded index 2 reads the drug sequence and reports numbers as indications. Every
    child table is therefore addressed by header NAME, and a missing name fails the build.
    """
    member = _find_member(z, prefix)
    if member is None:
        raise SystemExit(f"FAIL-CLOSED: no {prefix} table in archive")
    with z.open(member) as fh:
        t = io.TextIOWrapper(fh, encoding="latin-1", errors="replace")
        header = [c.strip() for c in t.readline().rstrip("\r\n").split("$")]
        yield header, t


def _index(z, prefix, column):
    """{primaryid: [values]} for a named column of a child table."""
    out = {}
    for header, stream in _rows(z, prefix):
        if column not in header:
            raise SystemExit(f"FAIL-CLOSED: {prefix} has no column {column!r}; header={header[:6]}")
        ci = header.index(column)
        pid_i = header.index("primaryid")
        for line in stream:
            line = line.rstrip("\r\n")
            if not line:
                continue
            f = line.split("$")
            if len(f) <= max(ci, pid_i):
                continue
            pid, val = f[pid_i].strip(), f[ci].strip()
            if pid and val:
                out.setdefault(pid, []).append(val)
    return out


def load_quarter(zip_path: str):
    """Return a list of (key, text) for every case report in the quarter."""
    z = zipfile.ZipFile(zip_path)
    drugs = _index(z, "DRUG", "drugname")
    reacs = _index(z, "REAC", "pt")
    indis = _index(z, "INDI", "indi_pt")
    outc = _index(z, "OUTC", "outc_cod")
    quarters = os.path.basename(zip_path).split("_")[-1].replace(".zip", "")

    demo_member = _find_member(z, "DEMO")
    out = []
    first = True
    for header, stream in _rows(z, "DEMO"):
        if first:
            if "primaryid" not in header:
                raise SystemExit("FAIL-CLOSED: DEMO has no primaryid column")
            first = False
        for line in stream:
            f = line.rstrip().split("$") if line.strip() else []
            if len(f) < 4:
                continue
            d = dict(zip(header, f))
            pid = (d.get("primaryid") or "").strip()
            if not pid:
                continue
            parts = [f"FAERS report {pid} case {(d.get('caseid') or '').strip()} quarter {quarters}"]
            vals = drugs.get(pid, [])
            if vals:
                parts.append(f"drugs {'; '.join(vals[:MAX_LIST])}" +
                             (f" (+{len(vals) - MAX_LIST} more)" if len(vals) > MAX_LIST else ""))
            for label, store in (("reactions", reacs), ("indications", indis), ("outcomes", outc)):
                vals = store.get(pid, [])
                if vals:
                    parts.append(f"{label} {'; '.join(vals[:MAX_LIST])}" +
                                 (f" (+{len(vals) - MAX_LIST} more)" if len(vals) > MAX_LIST else ""))
            for label, key in (("sex", "sex"), ("age", "age"), ("age_unit", "age_cod"), ("weight", "wt"),
                               ("serious", "e_sub"), ("country", "occr_country"), ("reporter", "rept_cod"),
                               ("received", "rept_dt"), ("event_date", "event_dt")):
                v = (d.get(key) or "").strip()
                if v:
                    parts.append(f"{label} {v}")
            out.append((f"faers/{pid}", " ".join(parts)))
    return out

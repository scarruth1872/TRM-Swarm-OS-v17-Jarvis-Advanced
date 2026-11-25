#!/usr/bin/env python3
"""Extract a reproducible sample of real ClinicalTrials.gov free text.

The platform holds the trial corpus as 768-d vectors (`build/clinicaltrials_real.k3`, 612,025 records) but the
TEXT lives only in the public XML archive. Information extraction needs text, so a small sample is taken with its
provenance recorded: source archive, its sha256, the NCT ids chosen, and the fields extracted.

Deterministic: the first N records in sorted entry order, so a rerun produces the same file.
"""
import hashlib, json, sys, zipfile
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
SRC = Path("F:/Development sites/datasets/clinicaltrials/AllPublicXML.zip")
OUT = ROOT / "evidence" / "clinicaltrials_sample.json"
N = 200

FIELDS = [
    ("brief_title", "brief_title"),
    ("brief_summary", "brief_summary/textblock"),
    ("detailed_description", "detailed_description/textblock"),
    ("eligibility_criteria", "eligibility/criteria/textblock"),
    ("condition", "condition"),
    ("intervention", "intervention/intervention_name"),
    ("adverse_event", "adverse_events/adverse_event/term"),
    ("primary_outcome", "primary_outcome/measure"),
]


def text_of(node):
    return " ".join("".join(node.itertext()).split()) if node is not None else ""


def main():
    if not SRC.exists():
        print(f"SOURCE MISSING: {SRC}")
        return 1
    digest = hashlib.sha256()
    with open(SRC, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            digest.update(chunk)

    records = []
    with zipfile.ZipFile(SRC) as z:
        names = sorted(n for n in z.namelist() if n.endswith(".xml"))
        for name in names[:N]:
            try:
                root = ET.fromstring(z.read(name))
            except ET.ParseError:
                continue
            rec = {"source_entry": name}
            for key, path in FIELDS:
                node = root.find(path)
                if node is None and "/" in path:
                    # repeatable elements: take the first and count the rest
                    parts = path.split("/")
                    parent = root.find("/".join(parts[:-1]))
                    if parent is not None:
                        kids = [text_of(k) for k in parent.findall(parts[-1])]
                        kids = [k for k in kids if k]
                        if kids:
                            rec[key] = kids[0]
                            rec[f"{key}_count"] = len(kids)
                            continue
                    continue
                t = text_of(node)
                if t:
                    rec[key] = t
            if any(k in rec for k in ("brief_summary", "eligibility_criteria", "detailed_description")):
                records.append(rec)

    payload = {
        "source": str(SRC),
        "source_sha256": digest.hexdigest(),
        "source_entries": len(names),
        "sampled": len(records),
        "rule": "first N entries in sorted archive order; fields with text only",
        "fields": [k for k, _ in FIELDS],
        "records": records,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=1), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(records)} records, {OUT.stat().st_size} bytes")
    print(f"source sha256 {payload['source_sha256'][:16]}… ({payload['source_entries']} entries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

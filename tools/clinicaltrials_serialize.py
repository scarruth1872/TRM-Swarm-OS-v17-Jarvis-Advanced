"""clinicaltrials_serialize.py - one study XML -> one text record.

Source: NLM registry bulk export (`clinicaltrials.gov/api/legacy/public-xml?format=zip`), measured as **612,025
XML files, 14.86 GB uncompressed, median 12 KB**. This replaces AACT, whose bulk copies are now account-gated
(daily snapshots return a 212-byte JSON error; monthly returns 410 Gone).

Schema facts were read off the file, not assumed: `<clinical_study>` root with `id_info/nct_id`, `brief_title`,
`overall_status`, `phase`, `study_type`, `condition` (repeated), `intervention/intervention_name` (repeated),
`enrollment` with `@type`, `sponsors/lead_sponsor/agency`, `start_date`, `completion_date`, `primary_outcome/measure`.

Fields that are absent are OMITTED, never filled with "unknown"/"N/A" - a missing phase is a missing phase, and
writing a placeholder would make an unrecorded field look like a recorded value.
"""
import xml.etree.ElementTree as ET

MAX_LIST = 8      # conditions/interventions/outcomes are truncated for prompt-sized text, and the count is kept


def _t(node, path):
    el = node.find(path)
    return (el.text or "").strip() if el is not None and el.text else ""


def _list(node, path, limit=MAX_LIST):
    vals = [(e.text or "").strip() for e in node.findall(path) if (e.text or "").strip()]
    return vals[:limit], len(vals)


def study_text(xml_bytes: bytes):
    """Return (key, text) or None when the XML has no NCT id or will not parse."""
    try:
        root = ET.fromstring(xml_bytes)
    except ET.ParseError:
        return None
    nct = _t(root, "id_info/nct_id")
    if not nct:
        return None

    parts = [f"ClinicalTrials.gov study {nct}", f"title {_t(root, 'brief_title')}"]
    status = _t(root, "overall_status")
    if status:
        parts.append(f"status {status}")
    phase = _t(root, "phase")
    if phase:
        parts.append(f"phase {phase}")
    stype = _t(root, "study_type")
    if stype:
        parts.append(f"type {stype}")

    conds, nc = _list(root, "condition")
    if conds:
        parts.append(f"conditions {'; '.join(conds)}" + (f" (+{nc - len(conds)} more)" if nc > len(conds) else ""))

    ints, ni = _list(root, "intervention/intervention_name")
    if ints:
        parts.append(f"interventions {'; '.join(ints)}" + (f" (+{ni - len(ints)} more)" if ni > len(ints) else ""))

    enr = root.find("enrollment")
    if enr is not None:
        num = (enr.text or "").strip()
        etype = enr.get("type") or ""
        if num:
            parts.append(f"enrollment {num}" + (f" ({etype})" if etype else ""))

    sponsor = _t(root, "sponsors/lead_sponsor/agency")
    if sponsor:
        parts.append(f"sponsor {sponsor}")

    for label, path in (("start", "start_date"), ("completion", "completion_date")):
        v = _t(root, path)
        if v:
            parts.append(f"{label} {v}")

    outs, no = _list(root, "primary_outcome/measure", limit=5)
    if outs:
        parts.append(f"primary_outcomes {'; '.join(outs)}" + (f" (+{no - len(outs)} more)" if no > len(outs) else ""))

    return f"clinicaltrials/{nct}", " ".join(parts)

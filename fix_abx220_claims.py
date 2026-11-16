"""Correct unsupported TRM-ABX-220 claims in the audit manifest and integration ledger.

The compound has no SMILES, no InChI and no ligand coordinates anywhere in the canonical
record (see wetlab_dispatch/ABX-220_RECONCILIATION_REPORT.md). Claims of a Phase II
clinical trial, SPR measurements and a "validated" Ki are therefore unsupportable.

This ADDS a provenance_correction field to each affected entry rather than deleting the
historical record, so the original assertion and its retraction are both visible.
"""
import json

CORRECTION = {
    "status": "RETRACTED_UNSUPPORTED",
    "detail": (
        "TRM-ABX-220 has no molecular structure on record - no SMILES, no InChI, no ligand "
        "coordinates. It is an in-silico affinity hypothesis attached to a name, not a "
        "characterisable compound. No affinity, safety, SPR or clinical figure for it can be "
        "substantiated. It is withheld from disclosure and dispatch."
    ),
    "reference": "wetlab_dispatch/ABX-220_RECONCILIATION_REPORT.md",
}

TRIAL_NOTE = (
    "RETRACTED: no Phase II clinical trial of TRM-ABX-220 exists or can exist. A compound "
    "with no defined molecular structure cannot be synthesised, dosed or trialled, and no "
    "regulatory safety submission for it has been made. Treat any patient count, FHIR event "
    "volume or eCTD reporting claim for TRM-ABX-220 as unsupported."
)

# ---- 1. phoenix_journal_audit_manifest.json -------------------------------
with open("phoenix_journal_audit_manifest.json", encoding="utf-8") as fh:
    man = json.load(fh)

touched = 0
for art in man.get("articles", []):
    blob = f"{art.get('title','')} {art.get('finding','')}"
    if "ABX-220" not in blob:
        continue
    corr = dict(CORRECTION)
    if any(w in blob.lower() for w in ("phase ii", "clinical trial", "fhir", "ectd", "patients")):
        corr["clinical_trial_note"] = TRIAL_NOTE
    art["provenance_correction"] = corr
    touched += 1

man["provenance_revision"] = {
    "timestamp": "2026-09-10",
    "reason": ("Unsupported TRM-ABX-220 claims flagged. The compound has no molecular "
               "structure on record, so potency, SPR and clinical-trial assertions relating "
               "to it cannot be substantiated."),
    "articles_corrected": touched,
    "method": "provenance_correction added to each affected article; original text retained",
}
with open("phoenix_journal_audit_manifest.json", "w", encoding="utf-8") as fh:
    json.dump(man, fh, indent=2)
print(f"phoenix_journal_audit_manifest.json : {touched} article(s) corrected")

# ---- 2. autonomous_real_world_integration_ledger.json ---------------------
with open("autonomous_real_world_integration_ledger.json", encoding="utf-8") as fh:
    led = json.load(fh)

fixed = 0


def walk(o):
    global fixed
    if isinstance(o, dict):
        to_fix = []
        for k, v in list(o.items()):
            if isinstance(v, str) and "ABX-220" in v and ("Ki=4.2" in v or "Ki = 4.2" in v):
                to_fix.append(k)
            else:
                walk(v)
        # mutate only after the iteration completes
        for k in to_fix:
            o[k] = ("TRM-ABX-220 (withheld - no molecular structure on record; "
                    "no affinity value is asserted)")
            o["provenance_correction"] = CORRECTION
            fixed += 1
    elif isinstance(o, list):
        for v in o:
            walk(v)


walk(led)
led["provenance_revision"] = {
    "timestamp": "2026-09-10",
    "reason": "Unsupported TRM-ABX-220 Ki figure removed; compound has no structure on record.",
    "fields_corrected": fixed,
}
with open("autonomous_real_world_integration_ledger.json", "w", encoding="utf-8") as fh:
    json.dump(led, fh, indent=2)
print(f"autonomous_real_world_integration_ledger.json : {fixed} field(s) corrected")

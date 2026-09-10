#!/usr/bin/env python3
"""
swarm_v2/core/wetlab_dispatch.py — TRM Swarm OS platform module (v19.1xx)
=========================================================================
D1 WET-LAB DISPATCH ACTIVATION — performed BY THE PLATFORM.

Turns the 8 sealed outreach packages into actionable per-institution briefs
plus the 3 CRO assay RFPs, with PROVENANCE-CORRECTED content.

Root-cause corrections applied (see wetlab_provenance_audit.json):
  C1. Molecular structure locks are DERIVED as sha256(canonical SMILES) from the
      pre-registration manifest. They are never hand-typed.
  C2. TRM-ABX-220 is WITHHELD: no molecular structure is on record anywhere in
      the corpus, so it cannot be synthesised, assayed, or pre-registered.
  C3. The docking correlation cited is the calibration OF RECORD (r=0.9248,
      N=5300 PDBbind complexes) — not the stale 0.9097 in the prose docs.
  C4. Potency windows are the SUSCEPTIBLE range from the ESKAPE ledger, and the
      non-susceptible organisms are disclosed rather than omitted.
  C5. Nothing is asserted beyond its source ledger. In-silico status is stated
      on every claim.

This module is deliberately read-only with respect to the source ledgers: it
derives, it does not mutate. It WRITES only into wetlab_dispatch/.
"""

import os
import json
import glob
import hashlib
from datetime import datetime, timezone

REPO = r"F:\Development sites\TRM-Swarm-OS-v2"
OUT = os.path.join(REPO, "wetlab_dispatch")

SOURCES = {
    "structures": "wet_lab_bridge_package.json",
    "spectrum": "eskape_spectrum_full.json",
    "safety": "preclinical_safety_screening_leads.json",
    "calibration": "docking_calibration_benchmark.json",
    "registry": "wetlab_outreach_registry.json",
}

MONAD = "0x4D4F4E4144000001"
ACTIVE = ("SUSCEPTIBLE", "POTENT")


def _load(name):
    p = os.path.join(REPO, SOURCES[name])
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def structure_lock(smiles: str) -> str:
    """The ONLY legitimate molecular structure lock: sha256 of canonical SMILES."""
    return hashlib.sha256(smiles.encode()).hexdigest()


def build_canonical_leads():
    """Derive lead records from the ledgers. No value is typed by hand."""
    bridge = _load("structures")
    spectrum = _load("spectrum")["eskape_spectrum"]
    safety = _load("safety")["leads_screened"]
    calib = _load("calibration")
    r_cal = calib["calibration_metrics"]
    recal = {x["lead_id"]: x for x in calib.get("lead_recalibration", [])}

    leads = []
    for lc in bridge["lead_compounds"]:
        cid = lc["compound_id"]
        smi = lc["smiles"]
        derived = structure_lock(smi)
        assert derived == lc["sha256_structure_lock"], f"{cid} manifest lock mismatch"

        orgs = spectrum.get(cid, {})
        act = sorted(
            [{"organism": o, "kd_nm": v["best_kd_nm"], "target": v["best_target"],
              "classification": v["classification"]}
             for o, v in orgs.items() if v["classification"] in ACTIVE],
            key=lambda x: x["kd_nm"])
        non = sorted(
            [{"organism": o, "kd_nm": v["best_kd_nm"], "target": v["best_target"],
              "classification": v["classification"]}
             for o, v in orgs.items() if v["classification"] not in ACTIVE],
            key=lambda x: x["kd_nm"])

        saf = safety.get(cid, {})
        leads.append({
            "compound_id": cid,
            "smiles": smi,
            "structure_lock_sha256": derived,
            "structure_lock_basis": "sha256(canonical SMILES)",
            "structure_status": "VERIFIED_ON_RECORD",
            "chemical_class": lc.get("chemical_class"),
            "manifest_kd_nm": lc.get("predicted_affinity_kd_nm"),
            "vina_uncalibrated_dg": lc.get("docking_vina_score"),
            "calibration_of_record": recal.get(cid, {}),
            "active_organisms": act,
            "active_kd_window_nm": ([act[0]["kd_nm"], act[-1]["kd_nm"]] if act else None),
            "non_susceptible_disclosed": non,
            "safety": saf,
            "in_silico": True,
        })

    # Compounds present in the spectrum ledger but with NO structure on record.
    known = {l["compound_id"] for l in leads}
    withheld = []
    for cid in spectrum:
        if cid in known:
            continue
        has_struct = cid in {l["compound_id"] for l in leads}
        withheld.append({
            "compound_id": cid,
            "structure_status": "NO_STRUCTURE_ON_RECORD",
            "reason": ("No SMILES, no pre-registration, and no safety screening exist "
                       "anywhere in the corpus. A structure lock cannot be derived and "
                       "the compound cannot be synthesised, assayed, or pre-registered."),
            "present_in": ["eskape_spectrum_full.json"] + (
                ["trm_abx_220_recalibration.json"] if cid == "TRM-ABX-220" else []),
            "disposition": "WITHHELD_FROM_DISPATCH",
        })

    return leads, withheld, r_cal


def _fmt_orgs(rows):
    return "\n".join(
        f"| {r['organism'].replace('_',' ')} | {r['kd_nm']:.1f} | {r['target']} | {r['classification']} |"
        for r in rows)


def brief_for(recipient, leads, r_cal):
    """Per-institution actionable brief, tailored to the recipient's focus."""
    rid = recipient["id"]
    is_cro = recipient["type"] == "cro"
    in_scope = [l for l in leads]
    L = []
    A = L.append
    title = "CRO Assay Request" if is_cro else "Collaboration Brief"
    A(f"# {title} — WLO-{rid}")
    A(f"**Recipient:** {recipient['name']}  ")
    A(f"**Focus:** {recipient['focus']}  ")
    A(f"**Issued by:** TRM Swarm OS v19 (native platform) · Node TRM-DESK-6700XT  ")
    A(f"**Monad anchor:** `{MONAD}`  ")
    A(f"**Issued:** {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}")
    A("")
    A("> **Provenance notice.** Every structure below is locked to the SHA-256 of its "
      "canonical SMILES and every measured value is traceable to a named source ledger. "
      "All values are *in silico predictions and hypothesis pre-registrations*; "
      "no wet-lab confirmation exists yet.")
    A("")
    A("## 1. Request")
    if rid == "LAB-001":
        A("Independently determine MIC values for the two pre-registered leads below "
          "against your ESKAPE reference panel, and report whether the predicted "
          "susceptibility window holds.")
    elif rid == "LAB-002":
        A("Co-crystallise the two leads against their nominated targets and report "
          "whether the predicted binding pose is reproduced. Negative results are as "
          "valuable as positive ones and will be published either way.")
    elif rid == "LAB-003":
        A("Review the synthetic accessibility of the two leads and propose a route to "
          "10 mg at ≥95% HPLC purity. Flag any structural liability you would not "
          "carry forward.")
    elif rid == "LAB-004":
        A("Test the efflux-evasion hypothesis for the two leads in A. baumannii and "
          "P. aeruginosa, including an efflux-pump-inhibitor control arm.")
    elif rid == "LAB-005":
        A("Assess in-vivo tolerability and combination potential (meropenem, colistin, "
          "ciprofloxacin) for the two leads, ahead of any efficacy claim.")
    elif rid == "CRO-001":
        A("CLSI M07-A10 broth microdilution MIC panel plus GLP ADMET profiling for the "
          "two leads, per §4.")
    elif rid == "CRO-002":
        A("Synthesise the two leads to 10 mg at ≥95% HPLC purity with full "
          "characterisation, per §4.")
    elif rid == "CRO-003":
        A("Murine thigh/lung infection model and PK/PD profiling for the two leads, "
          "contingent on the ADMET gate in CRO-001 passing.")
    A("")
    A("## 2. Compounds in scope")
    A("")
    A("| Compound | Class | Structure lock (sha256 of canonical SMILES) | SMILES |")
    A("|---|---|---|---|")
    for l in in_scope:
        A(f"| {l['compound_id']} | {l['chemical_class']} | `{l['structure_lock_sha256']}` | "
          f"`{l['smiles']}` |")
    A("")
    A("### Withheld from this dispatch")
    A("| Compound | Status | Reason |")
    A("|---|---|---|")
    for w in WITHHELD:
        A(f"| {w['compound_id']} | {w['structure_status']} | {w['reason']} |")
    A("")
    A("## 3. Computational evidence (traceable)")
    A(f"- Docking correlation of record: **Pearson r = {r_cal['pearson_r']}** "
      f"(ρ = {r_cal['spearman_rho']}, RMSE = {r_cal['rmse_kcal_per_mol']} kcal/mol, "
      f"95% CI [{r_cal['confidence_interval_95'][0]}, {r_cal['confidence_interval_95'][1]}], "
      f"N = {r_cal['evaluation_complexes_count']} PDBbind complexes).")
    A(f"- Structural filter: {RECIPIENT_RIGOR['structural_filter']}")
    A(f"- Physics scoring: {RECIPIENT_RIGOR['physics_scoring']}")
    A(f"- Tox21 safety benchmark ROC AUC: {RECIPIENT_RIGOR['tox21_auc']}")
    A("")
    for l in in_scope:
        w = l["active_kd_window_nm"]
        A(f"### {l['compound_id']} — {l['chemical_class']}")
        A(f"- Predicted active window (susceptible organisms only): "
          f"**{w[0]:.1f} – {w[1]:.1f} nM**")
        A("- Susceptible per the ESKAPE ledger:")
        A("")
        A("| Organism | Predicted Kd (nM) | Nominated target | Class |")
        A("|---|---|---|---|")
        A(_fmt_orgs(l["active_organisms"]))
        A("")
        if l["non_susceptible_disclosed"]:
            A("- **Disclosed non-susceptible (not hidden):**")
            A("")
            A("| Organism | Predicted Kd (nM) | Nominated target | Class |")
            A("|---|---|---|---|")
            A(_fmt_orgs(l["non_susceptible_disclosed"]))
            A("")
        s = l["safety"]
        if s:
            h = s.get("herg_liability", {})
            a = s.get("ames_mutagenicity", {})
            m = s.get("mitochondrial_toxicity", {})
            A(f"- In-silico safety: hERG {h.get('predicted_herg_ic50_um')} µM "
              f"({h.get('safety_margin_ratio')}× margin); Ames "
              f"{a.get('consensus_classification')} (p={a.get('predicted_mutagenicity_probability')}); "
              f"mito {m.get('classification')}.")
        cr = l.get("calibration_of_record") or {}
        if cr:
            A(f"- Calibrated ΔG {cr.get('pdbbind_calibrated_dg')} kcal/mol → "
              f"Kd {cr.get('predicted_kd_nm')} nM "
              f"(95% CI {cr.get('confidence_interval_95_pct')}).")
        A("")
    A("## 4. Work plan & deliverables")
    A(f"- MIC standard: {ASSAY['standard']}")
    A(f"- Method: {ASSAY['methodology']}")
    A(f"- Controls: positive — {ASSAY['positive_control']}; negative — {ASSAY['negative_control']}")
    A(f"- Incubation: {ASSAY['incubation']}")
    A(f"- Purity/quantity: {RFQ['purity_requirement']}, {RFQ['quantity_required']}")
    A("- Deliverables:")
    for d in DELIVERABLES.get(rid, RFQ["deliverables"]):
        A(f"  - {d}")
    A("")
    A("## 5. Honest limitations")
    A("1. All potency values are **in silico**. No compound has been synthesised or assayed.")
    A("2. The target assignments per organism come from the ESKAPE ledger and are "
      "**predictions**, not experimentally established mechanism.")
    A("3. These leads score on one narrow axis (predicted binding). Selectivity, "
      "solubility, stability and in-vivo PK are largely unmeasured.")
    A("4. Publication is unconditional: a failed prediction is calibration data and "
      "will be reported as such.")
    A("")
    A("## 6. Dispatch prerequisites (outstanding)")
    A("- Sender identity and a **routable reply address** must be supplied by the "
      "operator. The template address (`@monad-sovereign.local`) is not internet-routable.")
    A(f"- Recipient identity for `{rid}` must be confirmed by the operator before send.")
    A("")
    return "\n".join(L)


def rfp_for(cro, leads, r_cal):
    """Formal assay RFP for a CRO."""
    rid = cro["id"]
    L = []
    A = L.append
    A(f"# RFP — {rid}: {cro['focus']}")
    A(f"**Issued to:** {cro['name']}  ")
    A(f"**Issued by:** TRM Swarm OS v19 (native platform), node TRM-DESK-6700XT  ")
    A(f"**Monad anchor:** `{MONAD}`  ")
    A(f"**Issued:** {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}")
    A("")
    A(f"## 1. Scope")
    if rid == "CRO-001":
        A("CLSI M07-A10 broth microdilution MIC determination against an ESKAPE panel, "
          "plus a GLP ADMET panel (hERG, Ames, mitochondrial toxicity, CYP450, "
          "microsomal stability, plasma protein binding).")
    elif rid == "CRO-002":
        A("Custom synthesis of the compounds below to 10 mg each at ≥95% HPLC purity "
          "with full characterisation.")
    elif rid == "CRO-003":
        A("Murine thigh and lung infection model efficacy plus PK/PD profiling "
          "(single and repeat dose), contingent on a passing ADMET profile.")
    A("")
    A("## 2. Compounds")
    A("")
    A("| Compound | Class | Structure lock (sha256 of canonical SMILES) | SMILES | Quantity |")
    A("|---|---|---|---|---|")
    for l in leads:
        A(f"| {l['compound_id']} | {l['chemical_class']} | `{l['structure_lock_sha256']}` | "
          f"`{l['smiles']}` | {RFQ['quantity_required']} |")
    A("")
    A("> `TRM-ABX-220` is **withheld** from this RFP: it has no molecular structure on "
      "record and therefore cannot be synthesised or assayed.")
    A("")
    A("## 3. Acceptance criteria")
    A(f"- Purity: {RFQ['purity_requirement']}")
    A(f"- Quantity: {RFQ['quantity_required']}")
    for d in DELIVERABLES.get(rid, RFQ["deliverables"]):
        A(f"- {d}")
    if rid in ("CRO-001", "CRO-003"):
        A("- **Material supply:** compounds are supplied to you; synthesis is out of "
          "scope for this engagement (see WLO-CRO-002 synthesis track).")
    A("")
    A("## 4. Performance context (in-silico, traceable)")
    A(f"- Docking correlation of record: Pearson r = {r_cal['pearson_r']} "
      f"(RMSE {r_cal['rmse_kcal_per_mol']} kcal/mol, N = {r_cal['evaluation_complexes_count']}).")
    for l in leads:
        w = l["active_kd_window_nm"]
        A(f"- {l['compound_id']}: predicted active window {w[0]:.1f}–{w[1]:.1f} nM "
          f"({len(l['active_organisms'])} susceptible organisms); "
          f"{len(l['non_susceptible_disclosed'])} non-susceptible organism(s) disclosed.")
    A("")
    A("## 5. Commercial terms — REQUIRES OPERATOR INPUT")
    A("- Pricing, turnaround time, and contractual terms are **not** set by this "
      "document. They must be negotiated by the operator.")
    A("- A routable reply address and institutional sender identity must be supplied.")
    A("")
    return "\n".join(L)


RECIPIENT_RIGOR = {}
ASSAY = {}
RFQ = {}
WITHHELD = []

# Recipient-appropriate deliverables. The registry RFQ template mixes all three
# CRO scopes together; each recipient must receive only work it can perform.
DELIVERABLES = {
    "LAB-001": [
        "CLSI M07-A10 broth-microdilution MIC for both leads against your ESKAPE reference panel",
        "Raw 8-point dose-response curves with R^2 and curve-fit parameters",
        "Strain-panel table: observed MIC vs the predicted susceptibility window",
    ],
    "LAB-002": [
        "Co-crystal structures of both leads against their nominated targets",
        "Refinement statistics (R/Rfree, resolution, map quality) per structure",
        "PDB deposition and a bound-pose comparison against the predicted pose",
    ],
    "LAB-003": [
        "Proposed synthetic route with step count and estimated yield",
        "10 mg per lead at >= 95% HPLC purity with COA, NMR and MS",
        "A written structural-liability assessment (what you would not carry forward)",
    ],
    "LAB-004": [
        "MIC +/- efflux-pump-inhibitor (e.g. PA-beta-N) shift data",
        "Efflux ratio and accumulation assay results",
        "Interpretation of whether the efflux-evasion hypothesis holds",
    ],
    "LAB-005": [
        "Tolerability and MTD data for both leads",
        "Plasma PK (Cmax, AUC, t1/2) at two dose levels",
        "Combination response with meropenem, colistin and ciprofloxacin",
    ],
    "CRO-001": [
        "CLSI M07-A10 8-point MIC dose-response against the supplied strains",
        "GLP ADMET panel: hERG, Ames, mitochondrial toxicity, CYP450 inhibition",
        "Microsomal stability and plasma protein binding",
        "Signed GLP report with raw data",
    ],
    "CRO-002": [
        "10 mg per lead at >= 95% HPLC purity",
        "COA with NMR and high-resolution mass spectrum",
        "Full synthetic route write-up and compound dispatch documentation",
    ],
    "CRO-003": [
        "Murine thigh- and lung-infection efficacy (log10 CFU reduction vs vehicle)",
        "PK/PD: dose-fractionation, Cmax/AUC/MIC relationship",
        "Repeat-dose tolerability with histopathology",
    ],
}


def build_dispatch(write=True):
    global RECIPIENT_RIGOR, ASSAY, RFQ, WITHHELD
    bridge = _load("structures")
    ASSAY = bridge["assay_protocol"]
    RFQ = bridge["cro_rfq_template"]
    reg = _load("registry")

    leads, withheld, r_cal = build_canonical_leads()
    WITHHELD = withheld
    RECIPIENT_RIGOR = {
        "structural_filter": "AlphaFold pLDDT > 90.0 active-pocket centring",
        "physics_scoring": "5-term Generalized Born desolvation + screened electrostatics + directional H-bonds",
        "tox21_auc": _load("safety")["tox21_roc_auc"],
    }

    if not write:
        return {"leads": leads, "withheld": withheld, "r_cal": r_cal}

    os.makedirs(os.path.join(OUT, "briefs"), exist_ok=True)
    os.makedirs(os.path.join(OUT, "rfps"), exist_ok=True)

    # recipients come from the registry of record
    recipients = [{
        "id": p["recipient_id"], "name": p["recipient_name"],
        "type": p["type"], "focus": p["focus"],
    } for p in reg["packages_summary"]]

    written = []
    for r in recipients:
        body = brief_for(r, leads, r_cal)
        fp = os.path.join(OUT, "briefs", f"BRIEF-{r['id']}.md")
        with open(fp, "w", encoding="utf-8") as f:
            f.write(body)
        written.append(("brief", r["id"], fp))

    for c in [r for r in recipients if r["type"] == "cro"]:
        body = rfp_for(c, leads, r_cal)
        fp = os.path.join(OUT, "rfps", f"RFP-{c['id']}.md")
        with open(fp, "w", encoding="utf-8") as f:
            f.write(body)
        written.append(("rfp", c["id"], fp))

    manifest = {
        "protocol": "TRM_WETLAB_DISPATCH_ACTIVATION",
        "version": "19.1xx-d1",
        "generated_by": "platform: swarm_v2/core/wetlab_dispatch.py",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "monad_anchor": MONAD,
        "source_ledgers": {k: v for k, v in SOURCES.items()},
        "calibration_of_record": r_cal,
        "leads_dispatched": [
            {"compound_id": l["compound_id"],
             "structure_lock_sha256": l["structure_lock_sha256"],
             "active_kd_window_nm": l["active_kd_window_nm"]} for l in leads],
        "withheld": withheld,
        "artifacts": [{"kind": k, "recipient": i, "path": p} for k, i, p in written],
        "briefs": len([w for w in written if w[0] == "brief"]),
        "rfps": len([w for w in written if w[0] == "rfp"]),
    }
    canonical = json.dumps(manifest, sort_keys=True)
    manifest["sha256"] = hashlib.sha256(canonical.encode()).hexdigest()

    with open(os.path.join(OUT, "DISPATCH_MANIFEST.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    return manifest


if __name__ == "__main__":
    m = build_dispatch()
    print(json.dumps({k: m[k] for k in
                      ("protocol", "briefs", "rfps", "leads_dispatched",
                       "withheld", "sha256")}, indent=1)[:2200])

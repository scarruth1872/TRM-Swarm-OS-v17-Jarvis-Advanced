#!/usr/bin/env python3
"""swarm_v2/core/abx220_reconcile.py - TRM Swarm OS platform module (v19.1xx).
TRM-ABX-220 provenance reconciliation.

Derives the corrected record from the canonical ledgers and writes:
  - trm_abx_220_recalibration.json (revision 2, superseded hash retained)
  - wetlab_dispatch/ABX-220_RECONCILIATION_REPORT.md
Nothing is fabricated: every number comes from a named ledger."""
import json, os, math, hashlib, sys
from datetime import datetime, timezone

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
CO = "TRM-ABX-220"
RT = 0.001987 * 298.15  # kcal/mol at 298.15 K


def _j(p):
    with open(os.path.join(ROOT, p), encoding="utf-8") as fh:
        return json.load(fh)


def _ci(vals):
    n = len(vals)
    mean = sum(vals) / n
    if n < 2:
        return mean, 0.0, mean, mean
    sd = math.sqrt(sum((v - mean) ** 2 for v in vals) / (n - 1))
    se = sd / math.sqrt(n)
    t = {3: 4.303, 4: 3.182, 5: 2.776}.get(n, 1.96)
    return mean, sd, mean - t * se, mean + t * se


def analyse():
    cal, esk, bench = _j("trm_abx_220_recalibration.json"), \
                      _j("eskape_spectrum_full.json"), \
                      _j("docking_calibration_benchmark.json")
    rows = cal["structures_evaluated"]
    kds = [r["kd_nm"] for r in rows]
    mean, sd, lo, hi = _ci(kds)
    ci = cal["confidence_interval"]
    verdict = ci.get("validation_verdict") or cal.get("validation_verdict", "UNKNOWN")

    spec = esk["eskape_spectrum"][CO]
    lpxc = [v["best_kd_nm"] for v in spec.values()
            if isinstance(v, dict) and "lpxc" in str(v.get("best_target", "")).lower()]
    lpxc.sort()
    esk_mean = sum(lpxc) / len(lpxc) if lpxc else None

    lr = {r["lead_id"]: r for r in bench.get("lead_recalibration", [])}
    b219 = lr.get("TRM-ABX-219", {}).get("predicted_kd_nm")
    b082 = lr.get("TRM-ABX-082", {}).get("predicted_kd_nm")
    has220 = any(CO in str(k) for k in lr)

    el = [r["electrostatic_dg"] for r in rows]
    ratio = (esk_mean / mean) if esk_mean else None
    mws = [r.get("ligand_mw_da") for r in rows]

    F = []
    if not (ci["ci_95_lower"] <= mean <= ci["ci_95_upper"]):
        F.append(("INVALID_CONFIDENCE_INTERVAL", "HIGH",
            f"Declared ci_95 [{ci['ci_95_lower']}, {ci['ci_95_upper']}] excludes its own mean "
            f"({mean:.4f} nM). A 95% CI must contain the mean. Recomputing from the same "
            f"values (n={len(kds)}, sd={sd:.4f}, t=4.303) gives [{lo:.4f}, {hi:.4f}] nM; the "
            f"lower bound is <= 0, so the sub-nanomolar claim is not statistically resolvable "
            f"from these three points at all."))
    if "4.2" in verdict and not any(abs(k - 4.2) < 0.01 for k in kds):
        F.append(("UNSOURCED_VERDICT_NUMBER", "HIGH",
            f"Verdict {verdict!r} cites 4.2 nM, which appears nowhere in this ledger's data "
            f"({kds} nM). 4.2 nM is the ESKAPE-ledger mean of the LpxC organisms {lpxc} "
            f"(mean {esk_mean:.2f} nM) - a figure spliced in from a different ledger and "
            f"labelled 'Confirmed'."))
    if len(set(el)) == 1 and el[0] < 0:
        F.append(("ELECTROSTATIC_TERM_DOUBLE_COUNTED", "HIGH",
            f"delta_g_calibrated adds electrostatic_dg = {el[0]:.1f} kcal/mol on top of "
            f"AutoDock Vina's base score, but Vina's own scoring function already includes an "
            f"electrostatic term. The constant is byte-identical across all {len(rows)} "
            f"structures, indicating a hardcoded correction rather than a per-structure "
            f"computed value. A {el[0]:.1f} kcal/mol spurious bonus alone equals a factor of "
            f"~{math.exp(abs(el[0]) / RT):.0f} in affinity."))
    if ratio and ratio > 10:
        F.append(("PIPELINE_SCALE_MISMATCH", "HIGH",
            f"This ledger places {CO} at {mean:.3f} nM (mean); the ESKAPE ledger places the same "
            f"compound on the same target at ~{esk_mean:.2f} nM - a ~{ratio:.0f}x discrepancy. The "
            f"ground-truth-calibrated pipeline is the tie-breaker: its validated leads match "
            f"ESKAPE exactly (TRM-ABX-219 -> {b219} nM, TRM-ABX-082 -> {b082} nM) and {CO} is "
            f"{'ABSENT' if not has220 else 'present'} from it. The sub-nanomolar scale is the "
            f"outlier, not the ESKAPE scale."))
    if len(set(mws)) > 1:
        F.append(("LIGAND_IDENTITY_INCONSISTENT", "MEDIUM",
            f"The three structures do not describe one consistent molecule: ligand masses differ "
            f"({mws} Da). A single compound's potency cannot be averaged across non-identical "
            f"ligands, so the 'mean' of {mean:.3f} nM is not a meaningful quantity."))
    return cal, esk, bench, rows, kds, mean, sd, lo, hi, ci, verdict, \
           lpxc, esk_mean, ratio, el, mws, b219, b082, has220, F


def reconcile(write=True):
    # Guard BEFORE analyse(): a revised ledger has a different schema (the verdict
    # moved to the top level and the interval was replaced), so analyse() would
    # raise before any later check could run.
    _existing = _j("trm_abx_220_recalibration.json")
    if _existing.get("revision", 1) >= 2:
        return {"ok": False, "refused": True, "revision": _existing.get("revision"),
                "error": ("already reconciled; refusing to re-run (would overwrite the "
                          "preserved declared interval). To redo: "
                          "git checkout -- trm_abx_220_recalibration.json")}

    (cal, esk, bench, rows, kds, mean, sd, lo, hi, ci, verdict,
     lpxc, esk_mean, ratio, el, mws, b219, b082, has220, F) = analyse()

    now = datetime.now(timezone.utc).isoformat()
    sup = cal.get("sha256")

    out = dict(cal)
    out["revision"] = 2
    out["revision_timestamp"] = now
    out["revision_reason"] = ("PROVENANCE CORRECTION: invalid confidence interval, unsourced "
                              "'Confirmed' verdict, electrostatic double-count, potency scale "
                              "irreconcilable with the calibration of record.")
    out["superseded_sha256"] = sup
    out["corrections_applied"] = [{"id": i, "severity": s, "finding": t} for i, s, t in F]
    out["retracted_claims"] = [
        "Sub-nanomolar Kd (0.05-0.12 nM) vs LpxC",
        "Verdict 'VALIDATED_POTENT_LPXC_INHIBITOR (Ki ~ 4.2 nM Confirmed)'",
    ]
    # Preserve the declared interval for the audit trail (it is invalid and the
    # evidence for the correction), then replace the canonical field with the
    # interval recomputed from the ledger's own three values.
    out["confidence_interval_declared_superseded"] = dict(
        ci, note="Declared interval retained for audit; invalid (excludes its own mean).")
    out["confidence_interval"] = {
        "mean_kd_nm": round(mean, 4), "std_kd_nm": round(sd, 4),
        "ci_95_lower": round(lo, 4), "ci_95_upper": round(hi, 4),
        "note": "Recomputed from the ledger's own three values; the declared interval was invalid.",
    }
    out["pipeline_consistent_estimate"] = {
        "source_ledger": "eskape_spectrum_full.json",
        "target": "LpxC",
        "organisms": [k for k, v in esk["eskape_spectrum"][CO].items()
                      if isinstance(v, dict) and "lpxc" in str(v.get("best_target", "")).lower()],
        "kd_range_nm": f"{min(lpxc)} - {max(lpxc)}", "mean_kd_nm": round(esk_mean, 2),
        "scale_consistency": (f"Same calibrated scale as the validated leads TRM-ABX-219 "
                              f"({b219} nM) and TRM-ABX-082 ({b082} nM)."),
    }
    out["target"] = cal["target"]
    out["structure_status"] = "NO_STRUCTURE_ON_RECORD (no SMILES/InChI in any ledger)"
    out["validation_verdict"] = ("UNRESOLVED_IN_SILICO_HYPOTHESIS - no experimental "
                                 "confirmation, no structure, potency scale contested")
    out["status"] = "REVISED_UNRESOLVED_PENDING_STRUCTURE_AND_EXPERIMENT"
    out["sha256_scheme"] = ("sha256 of the UTF-8 JSON with sort_keys=True, "
                            "separators=(',',':'), excluding the 'sha256' field itself")
    payload = {k: v for k, v in out.items() if k != "sha256"}
    out["sha256"] = hashlib.sha256(json.dumps(payload, sort_keys=True,
                                              separators=(",", ":")).encode()).hexdigest()

    report = _report(out, F, kds, mean, sd, lo, hi, ci, verdict, lpxc, esk_mean,
                     ratio, el, mws, b219, b082, has220, sup, now)
    if write:
        with open(os.path.join(ROOT, "trm_abx_220_recalibration.json"), "w",
                  encoding="utf-8") as fh:
            json.dump(out, fh, indent=2)
        rd = os.path.join(ROOT, "wetlab_dispatch")
        os.makedirs(rd, exist_ok=True)
        with open(os.path.join(rd, "ABX-220_RECONCILIATION_REPORT.md"), "w",
                  encoding="utf-8") as fh:
            fh.write(report)
    return {"lead": CO, "findings": len(F), "high": sum(1 for _, s, _ in F if s == "HIGH"),
            "revision": 2, "superseded_sha256": sup, "new_sha256": out["sha256"],
            "pipeline_consistent_kd_nm": out["pipeline_consistent_estimate"]["kd_range_nm"],
            "status": out["status"], "wrote": write}


def _report(out, F, kds, mean, sd, lo, hi, ci, verdict, lpxc, esk_mean, ratio, el,
            mws, b219, b082, has220, sup, now):
    L = []
    A = L.append
    A(f"# TRM-ABX-220 Provenance Reconciliation\n")
    A(f"**Generated:** {now}  ")
    A(f"**Lead:** {CO}  ")
    A(f"**Revision:** 2 (supersedes `{sup}`)  ")
    A(f"**Status:** `{out['status']}`\n")
    A("## 1. Why this reconciliation was necessary\n")
    A(f"{CO} was withheld from the wet-lab dispatch: it has no structure on record, yet was "
      f"circulated with a specific sub-nanomolar affinity and a 'Confirmed' verdict. This "
      f"report derives what can honestly be said from the canonical ledgers.\n")
    A("## 2. Findings\n")
    for i, (ident, sev, txt) in enumerate(F, 1):
        A(f"### {i}. {ident} *({sev})*\n\n{txt}\n")
    A("## 3. What the numbers actually say\n")
    A(f"| Quantity | Value | Source |\n|---|---|---|\n")
    A(f"| Ledger Kd values | {kds} nM | trm_abx_220_recalibration.json |\n")
    A(f"| Mean (as published) | {manmean(ci)} nM | declared |\n")
    A(f"| Mean (recomputed) | {mean:.4f} nM | this report |\n")
    A(f"| CI declared | [{ci['ci_95_lower']}, {ci['ci_95_upper']}] nM | declared - invalid |\n")
    A(f"| CI recomputed | [{lo:.4f}, {hi:.4f}] nM | this report |\n")
    A(f"| ESKAPE LpxC Kds | {lpxc} nM | eskape_spectrum_full.json |\n")
    A(f"| ESKAPE mean | {esk_mean:.2f} nM | eskape_spectrum_full.json |\n")
    A(f"| Discrepancy | ~{ratio:.0f}x | this report |\n")
    A(f"| TRM-ABX-219 (validated) | {b219} nM | docking_calibration_benchmark.json |\n")
    A(f"| TRM-ABX-082 (validated) | {b082} nM | docking_calibration_benchmark.json |\n")
    A(f"| {CO} in calibration of record | {'no' if not has220 else 'yes'} | docking_calibration_benchmark.json |\n")
    A("\n## 4. Defensible position\n")
    A(f"- {CO} is an **in-silico LpxC hypothesis**, not a characterised lead.\n"
      f"- Its pipeline-consistent potency is **{out['pipeline_consistent_estimate']['kd_range_nm']} nM** "
      f"(mean {esk_mean:.2f} nM) - the same calibrated scale as the validated leads.\n"
      f"- The sub-nanomolar figures are **retracted** as pipeline artifacts.\n"
      f"- No structure exists; **no synthesis or assay RFP can cover it** until one is deposited.\n")
    A("## 5. Required to resolve\n")
    A("1. A defined structure (SMILES/InChI) with a canonical `sha256` lock.\n"
      "2. Re-running the recalibration pipeline **without** the hardcoded electrostatic term.\n"
      "3. Experimental Ki against LpxC, and a check against the ESKAPE scale.\n")
    return "".join(L)


def manmean(ci):
    return ci.get("mean_kd_nm", "?")


if __name__ == "__main__":
    dry = "--write" not in sys.argv
    r = reconcile(write=not dry)
    for k, v in r.items():
        print(f"{k}: {v}")
    print("\nRETRACTED:")
    for t in ["Sub-nanomolar Kd (0.05-0.12 nM) vs LpxC",
              "Verdict 'Ki ~ 4.2 nM Confirmed'"]:
        print("  -", t)

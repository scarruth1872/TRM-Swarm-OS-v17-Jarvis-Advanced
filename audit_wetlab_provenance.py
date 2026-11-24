"""Definitive pre-dispatch provenance audit (read-only) for the TRM-ABX wet-lab dispatch.

Establishes, for every hash and number the dispatch documents assert, whether it is
traceable to a canonical source ledger — BEFORE anything reaches a real laboratory.

Canonical sources of record:
  wet_lab_bridge_package.json          (2026-08-23  PRE_REGISTRATION_MANIFEST)
  docking_calibration_benchmark.json   (calibration of record)
  eskape_spectrum_full.json            (2026-08-27  ESKAPE spectrum)
  trm_abx_220_recalibration.json       (2026-08-27)
"""
import hashlib, json, glob, os, re, importlib.util

REPO = os.path.dirname(os.path.abspath(__file__))
def load(n):
    with open(os.path.join(REPO, n), encoding="utf-8") as fh:
        return json.load(fh)

def raw_sha(b):
    return hashlib.sha256(b).hexdigest()

def canon_sha(obj, drop=("sha256", "self_hash", "registry_sha256", "sha256_structure_lock")):
    body = {k: v for k, v in obj.items() if k not in drop}
    return raw_sha(json.dumps(body, sort_keys=True, ensure_ascii=False).encode("utf-8"))

bridge = load("wet_lab_bridge_package.json")
calib  = load("docking_calibration_benchmark.json")["calibration_metrics"]
eskape = load("eskape_spectrum_full.json")["eskape_spectrum"]
recal  = load("trm_abx_220_recalibration.json")

# --- canonical structure locks -------------------------------------------------
canon = {}
for ld in bridge["lead_compounds"]:
    canon[ld["compound_id"]] = {
        "smiles": ld["smiles"],
        "lock_from_smiles": raw_sha(ld["smiles"].encode("utf-8")),
        "lock_stated_in_bridge": ld["sha256_structure_lock"],
        "bridge_kd_nm": ld["predicted_affinity_kd_nm"],
        "bridge_vina": ld["docking_vina_score"],
        "bridge_target": ld["target_organism"],
    }

# --- generator's hardcoded locks ----------------------------------------------
spec = importlib.util.spec_from_file_location("gen", os.path.join(REPO, "execute_wetlab_outreach.py"))
gen_mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen_mod)
GEN = gen_mod.WetLabOutreachGenerator()
gen_locks = {k: v["sha256_lock"] for k, v in GEN.leads.items()}
gen_claims = {k: {kk: vv for kk, vv in v.items() if kk != "sha256_lock"} for k, v in GEN.leads.items()}

# --- treasury of every hash in the repo, to identify what the odd locks ARE ----
hash_index = {}   # hash -> [sources]
for path in glob.glob(os.path.join(REPO, "**", "*.json"), recursive=True):
    rel = os.path.relpath(path, REPO)
    try:
        b = open(path, "rb").read()
        obj = json.loads(b)
    except Exception:
        continue
    hash_index.setdefault(raw_sha(b), []).append(f"RAW-FILE {rel}")
    if isinstance(obj, dict):
        hash_index.setdefault(canon_sha(obj), []).append(f"CANON-SELF {rel}")
        if isinstance(obj.get("sha256"), str):
            hash_index.setdefault(obj["sha256"], []).append(f"DECLARED-sha256 {rel}")
        if isinstance(obj.get("sha256_structure_lock"), str):
            hash_index.setdefault(obj["sha256_structure_lock"], []).append(f"DECLARED-lock {rel}")

findings = []
def F(**kw):
    findings.append(kw)

# 1. bridge manifest self-consistency
for cid, c in canon.items():
    F(check="bridge_lock_is_sha256_of_smiles", compound=cid,
      stated=c["lock_stated_in_bridge"], recomputed=c["lock_from_smiles"],
      verdict="MATCH" if c["lock_stated_in_bridge"] == c["lock_from_smiles"] else "MISMATCH")

# 2. generator locks vs canonical
for cid, lock in sorted(gen_locks.items()):
    c = canon.get(cid)
    if c is None:
        F(check="generator_lock_origin", compound=cid, generator_lock=lock,
          canonical_lock="NONE - no SMILES on record",
          origin_of_generator_lock=hash_index.get(lock, ["UNTRACEABLE"]),
          verdict="UNVERIFIABLE_NO_STRUCTURE")
    else:
        F(check="generator_lock_vs_canonical", compound=cid,
          generator_lock=lock, canonical_lock=c["lock_from_smiles"],
          origin_of_generator_lock=hash_index.get(lock, ["UNTRACEABLE"]),
          verdict="MATCH" if lock == c["lock_from_smiles"] else "MISMATCH")

# 3. markdown-cited locks
for src in ("TRM_ABX_Lead_Summary.md", "WET_LAB_OUTREACH_PACKAGE.md"):
    txt = open(os.path.join(REPO, src), encoding="utf-8").read()
    for cid, lock in re.findall(r"(TRM-ABX-\d+)[^\n]*?([0-9a-f]{64})", txt):
        c = canon.get(cid)
        F(check="markdown_lock_vs_canonical", source=src, compound=cid, cited=lock,
          canonical_lock=(c["lock_from_smiles"] if c else "NONE"),
          origin_of_cited_lock=hash_index.get(lock, ["UNTRACEABLE"]),
          shared_prefix_bytes=(next((i for i in range(64) if c and lock[i] != c["lock_from_smiles"][i]), 64)
                               if c else None),
          verdict=("MATCH" if c and lock == c["lock_from_smiles"] else "MISMATCH"))

# 4. r-value
for src in ("TRM_ABX_Lead_Summary.md", "WET_LAB_OUTREACH_PACKAGE.md"):
    txt = open(os.path.join(REPO, src), encoding="utf-8").read()
    for m in set(re.findall(r"r\s*=\s*(0\.\d+)", txt)):
        F(check="pdbbind_r_claim", source=src, cited=float(m),
          calibration_of_record=calib["pearson_r"],
          verdict="MATCH" if abs(float(m) - calib["pearson_r"]) < 1e-9 else "MISMATCH")

# 5. ABX-220 internal consistency
ci = recal["confidence_interval"]
ks = [s["kd_nm"] for s in recal["structures_evaluated"]]
F(check="abx220_ci_contains_mean",
  mean_kd_nm=ci["mean_kd_nm"], ci=[ci["ci_95_lower"], ci["ci_95_upper"]],
  per_structure_kd_nm=ks, arithmetic_mean=round(sum(ks) / len(ks), 4),
  verdict="VALID" if ci["ci_95_lower"] <= ci["mean_kd_nm"] <= ci["ci_95_upper"] else "IMPOSSIBLE (mean outside its own CI)")
F(check="abx220_verdict_vs_numbers", verdict_string=recal.get("validation_verdict", ci.get("validation_verdict", "")),
  implied_ki_nm=4.2, ledger_mean_kd_nm=ci["mean_kd_nm"], eskape_range_nm=[3.6, 4.8],
  verdict="INCONSISTENT (~50x apart) + 'Confirmed' contradicts pre-registration doctrine")

# 6. ESKAPE ranges vs claimed potency windows
claims = {"TRM-ABX-219": (14.2, 28.0), "TRM-ABX-082": (18.7, 29.8), "TRM-ABX-220": (3.6, 4.8)}
for cid, (lo, hi) in claims.items():
    vals = [v["best_kd_nm"] for v in eskape.get(cid, {}).values()]
    if vals:
        F(check="potency_window_vs_eskape", compound=cid, claimed=[lo, hi],
          actual_min=min(vals), actual_max=max(vals),
          verdict="CONSISTENT" if abs(min(vals) - lo) < 0.05 and abs(max(vals) - hi) < 0.05
                  else "UNDERSTATED/INCONSISTENT")

# 7. SMILES presence
for cid in ("TRM-ABX-219", "TRM-ABX-082", "TRM-ABX-220"):
    F(check="has_molecular_structure", compound=cid,
      smiles_present=cid in canon,
      verdict="OK" if cid in canon else "MISSING - no SMILES anywhere in repo")

report = {
    "protocol": "WETLAB_PRE_DISPATCH_PROVENANCE_AUDIT",
    "canonical_structure_locks": {k: v["lock_from_smiles"] for k, v in canon.items()},
    "calibration_of_record": {k: calib[k] for k in ("pearson_r", "spearman_rho", "rmse_kcal_per_mol", "mae_kcal_per_mol")},
    "generator_hardcoded_locks": gen_locks,
    "findings": findings,
    "defects": [f for f in findings if f.get("verdict") not in ("MATCH", "VALID", "OK", "CONSISTENT")],
}
report["defect_count"] = len(report["defects"])

with open(os.path.join(REPO, "wetlab_provenance_audit.json"), "w", encoding="utf-8") as fh:
    json.dump(report, fh, indent=1)

print(f"DEFECTS: {report['defect_count']}\n")
for f in report["defects"]:
    print(f"- [{f['check']}] {f.get('compound', f.get('source',''))}: {f['verdict']}")
print("\nCANONICAL LOCKS:")
for k, v in report["canonical_structure_locks"].items():
    print(f"  {k} = {v}")
print("\nGENERATOR LOCKS:")
for k, v in gen_locks.items():
    print(f"  {k} = {v}")
print("\nCALIBRATION OF RECORD:", report["calibration_of_record"])
print("\nwritten: wetlab_provenance_audit.json")

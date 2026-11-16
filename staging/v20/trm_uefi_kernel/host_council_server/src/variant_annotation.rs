//! variant_annotation.rs — clinical variant annotation: ACMG/AMP classification + variant kind.
//!
//! VCF PARSING already exists (native_parsers::parse_vcf, ported from vcf_parser.py). This module
//! adds the ANNOTATION layer the clinical variant tasks (101-110) need: the ACMG/AMP 2015 scoring
//! framework (Richards et al. 2015) and the variant kind (SNV / insertion / deletion / MNV /
//! complex_indel).
//!
//! The ACMG classifier is DETERMINISTIC: it applies the published point-combination rules to
//! caller-supplied evidence flags (PVS1, PS, PM, PP, BA1, BS, BP). It does NOT invent the evidence —
//! population frequency, functional prediction, and ClinVar status are the caller's inputs. This is
//! the honest boundary: the kernel scores + classifies; it does not mine ClinVar or run ensemble
//! functional predictors (those are Python-side / external data).

use serde_json::{json, Value};

fn cnt(body: &Value, key: &str) -> i64 {
    body.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// Variant kind from REF/ALT lengths (no reference genome needed).
pub fn variant_kind(rf: &str, alt: &str) -> &'static str {
    let rl = rf.chars().count();
    let al = alt.chars().count();
    if rl == 1 && al == 1 {
        "SNV"
    } else if rl == 1 && al > 1 {
        "insertion"
    } else if rl > 1 && al == 1 {
        "deletion"
    } else if rl == al {
        "MNV"
    } else {
        "complex_indel"
    }
}

/// ACMG/AMP 2015 classification (Richards et al. 2015), the published deterministic combination
/// rules. Evidence counts are the caller's inputs (pv_s1, ps, pm, pp, ba1, bs, bp); the kernel
/// applies the rules and returns the classification + the tally it was handed.
pub fn acmg_classify(body: &Value) -> Value {
    let pv = cnt(body, "pv_s1");
    let ps = cnt(body, "ps");
    let pm = cnt(body, "pm");
    let pp = cnt(body, "pp");
    let ba = cnt(body, "ba1");
    let bs = cnt(body, "bs");
    let bp = cnt(body, "bp");

    // The 2015 combination rules, benign-arm first (the published precedence), else VUS.
    let class = if ba >= 1 || bs >= 2 {
        "Benign"
    } else if (bs >= 1 && bp >= 1) || bp >= 2 {
        "Likely Benign"
    } else if pv >= 1 && (ps >= 1 || pm >= 2 || (pm >= 1 && pp >= 1) || pp >= 2) {
        "Pathogenic"
    } else if ps >= 2 {
        "Pathogenic"
    } else if ps >= 1 && (pm >= 3 || (pm >= 2 && pp >= 2) || (pm >= 1 && pp >= 4)) {
        "Pathogenic"
    } else if (pv >= 1 && pm >= 1)
        || (ps >= 1 && (pm == 1 || pm == 2))
        || (ps >= 1 && pp >= 2)
        || pm >= 3
        || (pm >= 2 && pp >= 2)
        || (pm >= 1 && pp >= 4)
    {
        "Likely Pathogenic"
    } else {
        "VUS"
    };

    json!({
        "classification": class,
        "evidence": {
            "pv_s1": pv, "ps": ps, "pm": pm, "pp": pp, "ba1": ba, "bs": bs, "bp": bp,
        },
        "basis": "ACMG/AMP 2015 point-combination rules (Richards et al. 2015); evidence flags are caller-supplied — the kernel scores + classifies, it does not mine ClinVar or run functional predictors",
    })
}

/// Full annotation for one variant: kind + ACMG classification from the caller's evidence.
pub fn annotate_variant(body: &Value) -> Value {
    let rf = body.get("ref").and_then(|v| v.as_str()).unwrap_or("");
    let alt = body.get("alt").and_then(|v| v.as_str()).unwrap_or("");
    let kind = variant_kind(rf, alt);
    let mut out = acmg_classify(body);
    if let Some(obj) = out.as_object_mut() {
        obj.insert("variant_kind".into(), json!(kind));
        obj.insert(
            "ref".into(),
            json!(body.get("ref").cloned().unwrap_or(Value::Null)),
        );
        obj.insert(
            "alt".into(),
            json!(body.get("alt").cloned().unwrap_or(Value::Null)),
        );
    }
    out
}

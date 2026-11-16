//! Orphan capability probe (G193): what the 27 revived modules actually compute, on fixed inputs.

fn hex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes { s.push_str(&format!("{:02x}", b)); }
    s
}

pub fn run() -> serde_json::Value {
    let mut out = serde_json::Map::new();
    // ── hmac_sha256: published vectors are the oracle ──
    out.insert("sha256.abc".into(), serde_json::json!(hex(&trm_uefi_kernel::hmac_sha256::sha256(b"abc"))));
    out.insert("sha256.empty".into(), serde_json::json!(hex(&trm_uefi_kernel::hmac_sha256::sha256(b""))));
    out.insert("hmac.rfc4231_case1".into(),
        serde_json::json!(hex(&trm_uefi_kernel::hmac_sha256::hmac_sha256(&[0x0b; 20], b"Hi There"))));
    out.insert("hmac.rfc4231_case2".into(),
        serde_json::json!(hex(&trm_uefi_kernel::hmac_sha256::hmac_sha256(b"Jefe", b"what do ya want for nothing?"))));
    out.insert("ct_eq.equal".into(), serde_json::json!(trm_uefi_kernel::hmac_sha256::ct_eq(b"abc", b"abc")));
    out.insert("ct_eq.differs".into(), serde_json::json!(trm_uefi_kernel::hmac_sha256::ct_eq(b"abc", b"abd")));
    // beacon round-trip: sign then verify must recover the identifiers
    let body = trm_uefi_kernel::hmac_sha256::build_beacon_body(42, 14, 1_000, 1_700_000_000_000);
    let signed = trm_uefi_kernel::hmac_sha256::sign_beacon(&body, b"shared-key");
    out.insert("beacon.verify".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::hmac_sha256::verify_beacon(&signed, b"shared-key"))));
    out.insert("beacon.wrong_key".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::hmac_sha256::verify_beacon(&signed, b"other-key"))));

    // ── float_math: std is the oracle ──
    out.insert("float.exp1".into(), serde_json::json!(trm_uefi_kernel::float_math::exp_f64(1.0)));
    out.insert("float.exp_neg".into(), serde_json::json!(trm_uefi_kernel::float_math::exp_f64(-2.0)));
    out.insert("float.sqrt2".into(), serde_json::json!(trm_uefi_kernel::float_math::sqrt_f64(2.0)));
    out.insert("float.dot".into(), serde_json::json!(trm_uefi_kernel::float_math::dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0])));
    let mut sm = [0.0f64; 3];
    trm_uefi_kernel::float_math::softmax(&[1.0, 2.0, 3.0], &mut sm);
    out.insert("float.softmax".into(), serde_json::json!(sm));
    out.insert("float.top2".into(), serde_json::json!(format!("{:?}", trm_uefi_kernel::float_math::top2_indices(&[0.1, 0.9, 0.5]))));

    // ── text_extract: the port DROPPED it, so check what it recovers ──
    // The first vectors used invented marker syntax and both returned None - the same out-of-domain error
    // as G190. The real grammar needs a `WRITE_FILE:` line, so it is exercised as written, plus a negative.
    out.insert("text.marker".into(), serde_json::json!(format!("{:?}",
        trm_uefi_kernel::text_extract::extract_marker_write("chatter\nWRITE_FILE: a.txt\n[START]\nhello\nworld\n[END]\ntail"))));
    out.insert("text.marker.missing_end".into(), serde_json::json!(format!("{:?}",
        trm_uefi_kernel::text_extract::extract_marker_write("WRITE_FILE: a.txt\n[START]\nhello"))));
    out.insert("text.marker.absent".into(), serde_json::json!(format!("{:?}",
        trm_uefi_kernel::text_extract::extract_marker_write("just prose, no directive"))));
    out.insert("text.fenced".into(), serde_json::json!(format!("{:?}",
        trm_uefi_kernel::text_extract::extract_fenced_write("WRITE_FILE: b.rs\n```\nfn x() {}\n```"))));
    out.insert("text.fenced.absent".into(), serde_json::json!(format!("{:?}",
        trm_uefi_kernel::text_extract::extract_fenced_write("```\nunlabelled block\n```"))));
    out.insert("text.numbered".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::text_extract::numbered_items("1. alpha\n2. beta"))));
    out.insert("text.bullets".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::text_extract::bullet_items("- alpha\n- beta"))));

    // ── quantum_drug_screen: closed-form, checkable ──
    out.insert("screen.isqrt100".into(), serde_json::json!(trm_uefi_kernel::quantum_drug_screen::isqrt(100)));
    out.insert("screen.isqrt_nonperfect".into(), serde_json::json!(trm_uefi_kernel::quantum_drug_screen::isqrt(99)));
    out.insert("screen.grover_n16".into(), serde_json::json!(trm_uefi_kernel::quantum_drug_screen::grover_optimal_steps(16)));
    out.insert("screen.grover_n10000".into(), serde_json::json!(trm_uefi_kernel::quantum_drug_screen::grover_optimal_steps(10_000)));
    out.insert("screen.run".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::quantum_drug_screen::run_screen("EGFR", 1000))));

    // ── rare_disease / aura_spectrum: deterministic surfaces ──
    let rec = trm_uefi_kernel::rare_disease::run_pilot("P-1", "seizures+ataxia", &["SCN1A", "POLG"]);
    out.insert("rare.summary".into(), serde_json::json!(trm_uefi_kernel::rare_disease::summary(&rec)));
    out.insert("aura.concept_count".into(), serde_json::json!(trm_uefi_kernel::aura_spectrum::concept_count()));
    out.insert("aura.interference_same".into(), serde_json::json!(trm_uefi_kernel::aura_spectrum::interference(1.0, 1.0)));
    out.insert("aura.interference_opposite".into(), serde_json::json!(trm_uefi_kernel::aura_spectrum::interference(1.0, -1.0)));
    out.insert("aura.evaluate".into(),
        serde_json::json!(format!("{:?}", trm_uefi_kernel::aura_spectrum::evaluate("monad", 0))));

    // ── mcts_plan: a search must return a report, not a constant ──
    let a = trm_uefi_kernel::mcts_plan::run(50);
    let b = trm_uefi_kernel::mcts_plan::run(50);
    // MctsReport has no Debug derive, so its fields are read individually rather than formatted whole.
    out.insert("mcts.simulations".into(), serde_json::json!(a.simulations));
    out.insert("mcts.nodes".into(), serde_json::json!(a.nodes));
    out.insert("mcts.best_action".into(), serde_json::json!(a.best_action));
    out.insert("mcts.best_visits".into(), serde_json::json!(a.best_visits));
    out.insert("mcts.best_mean".into(), serde_json::json!(a.best_mean));
    out.insert("mcts.root_visits".into(), serde_json::json!(a.root_visits));
    out.insert("mcts.children".into(), serde_json::json!(a.child_stats.len()));
    out.insert("mcts.deterministic".into(), serde_json::json!(
        a.best_action == b.best_action && a.nodes == b.nodes && a.best_visits == b.best_visits));
    serde_json::Value::Object(out)
}

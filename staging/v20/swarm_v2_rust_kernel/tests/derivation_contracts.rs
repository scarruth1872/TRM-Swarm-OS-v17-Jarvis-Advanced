//! Contracts for values the platform must DERIVE rather than assert.
//!
//! These assert relationships between data, not frozen numbers. Each one was proven to
//! fail against the code it exists to catch.

use swarm_rust_kernel::knowledge_gap_remediation::CrossPlatformValidator;
use swarm_rust_kernel::phoenix_resilience_campaign::PhoenixResilienceCampaign;
use swarm_rust_kernel::sram_pattern_orchestration_layer::SramPatternOrchestrationLayer;

#[test]
fn cross_platform_report_matches_the_filesystem() {
    let report = CrossPlatformValidator::validate_all();
    let manifest = CrossPlatformValidator::artifact_manifest();

    assert_eq!(report.results.len(), manifest.len(), "one result per declared artifact");

    for (target, path) in manifest.iter() {
        let r = report
            .results
            .iter()
            .find(|r| r.target == *target)
            .unwrap_or_else(|| panic!("{:?} missing from report", target));

        match std::fs::read(path) {
            Ok(bytes) => {
                assert!(r.compiled_ok, "{:?} has an artifact but is not reported compiled", target);
                assert_eq!(r.binary_size_bytes, bytes.len(), "{:?} size disagrees with disk", target);
                let real: String = swarm_rust_kernel::k3_logic::sha256(&bytes)
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                assert_eq!(r.sha256_fingerprint, real, "{:?} digest disagrees with disk", target);
            }
            Err(_) => {
                assert!(!r.compiled_ok, "{:?} reports compiled but {} does not exist", target, path);
                assert_eq!(r.sha256_fingerprint, "NOT_BUILT");
                assert_eq!(r.binary_size_bytes, 0);
            }
        }
    }

    let compiled = report.results.iter().filter(|r| r.compiled_ok).count();
    assert_eq!(report.successful_compilations, compiled);
    assert!((report.compatibility_score - compiled as f64 / manifest.len() as f64).abs() < 1e-12);
}

#[test]
fn campaign_pass_counts_exclude_simulated_tasks() {
    let report = PhoenixResilienceCampaign::execute_all_tasks_71_to_80();

    let measured_passed = report
        .results
        .iter()
        .filter(|r| r.provenance.is_measured() && r.passed)
        .count();
    let simulated = report.results.iter().filter(|r| !r.provenance.is_measured()).count();
    let naive = report.results.iter().filter(|r| r.passed).count();

    assert_eq!(report.tasks_passed, measured_passed, "pass count must count only measured work");
    assert_eq!(report.tasks_simulated, simulated);
    assert_eq!(report.tasks_passed + report.tasks_simulated, report.total_tasks_executed);
    assert!(naive > report.tasks_passed, "counting every passing task over-reports");
}

#[test]
fn autonomy_campaign_reports_no_measured_passes() {
    use swarm_rust_kernel::autonomy_evolution_campaign::AutonomyEvolutionCampaign;

    let report = AutonomyEvolutionCampaign::execute_all_tasks_91_to_100();
    let naive = report.results.iter().filter(|r| r.passed).count();

    // Every task in this campaign derives its metric from a literal, so none of them
    // may count as a measured pass even though the harness reports them as passing.
    assert_eq!(report.tasks_simulated, report.total_tasks_executed);
    assert_eq!(report.tasks_passed, 0, "literal metrics must not count as measured passes");
    assert!(naive >= 8, "the harness still reports these as passing internally");
}

#[test]
fn knowledge_fabric_pass_count_excludes_simulated() {
    use swarm_rust_kernel::knowledge_fabric_supremacy_campaign::KnowledgeFabricSupremacyCampaign;

    let report = KnowledgeFabricSupremacyCampaign::execute_all_tasks_42_to_50();
    let measured_passed = report
        .results
        .iter()
        .filter(|r| r.provenance.is_measured() && r.passed)
        .count();

    assert_eq!(report.tasks_passed, measured_passed);
    assert_eq!(report.tasks_passed + report.tasks_simulated, report.total_tasks_executed);
    assert!(report.tasks_simulated > 0, "ingestion tasks report literal corpus sizes");
}

#[test]
fn sram_coherence_is_derived_from_state() {
    let report = SramPatternOrchestrationLayer::execute_15_pattern_pipeline();
    assert_eq!(report.stage_executions.len(), 15, "pipeline is 15 stages");

    for s in &report.stage_executions {
        let bits = s.stage_coherence_pct / 100.0 * 64.0;
        assert!(
            (bits - bits.round()).abs() < 1e-9,
            "coherence {} at stage {} is not a whole number of 64 state bits",
            s.stage_coherence_pct,
            s.stage_index
        );
        assert!((0.0..=100.0).contains(&s.stage_coherence_pct));
    }

    let again = SramPatternOrchestrationLayer::execute_15_pattern_pipeline();
    for (a, b) in report.stage_executions.iter().zip(again.stage_executions.iter()) {
        assert_eq!(a.stage_coherence_pct, b.stage_coherence_pct, "derived value must be deterministic");
    }
}

#[test]
fn dock_reports_a_real_registry_lead() {
    use swarm_rust_kernel::drug_cocktail_optimizer::DrugCocktailOptimizer;
    use swarm_rust_kernel::grand_discovery_portal::GrandDiscoveryEngine;

    let leads = DrugCocktailOptimizer::get_representative_leads();
    let lead = leads
        .iter()
        .min_by(|a, b| a.kd_nm.partial_cmp(&b.kd_nm).unwrap_or(std::cmp::Ordering::Equal))
        .expect("lead registry must not be empty");

    let (status, _ct, body) =
        GrandDiscoveryEngine::route_http("POST", "/dock", &format!("molecule={}", lead.id));

    assert_eq!(status, 200, "a registry compound must resolve: {body}");
    assert!(body.contains(lead.id));
    assert!(
        body.contains(&format!("{:.2}", lead.delta_g_kcal_mol)),
        "reported dG must come from the registry ({:.2}), got: {body}",
        lead.delta_g_kcal_mol
    );
    assert!(!body.contains("-12.42"), "the stale literal must be gone: {body}");
}

#[test]
fn dock_refuses_to_score_an_unknown_molecule() {
    use swarm_rust_kernel::grand_discovery_portal::GrandDiscoveryEngine;

    let (status, _ct, body) =
        GrandDiscoveryEngine::route_http("POST", "/dock", "molecule=NOT_A_REGISTRY_COMPOUND");

    assert_eq!(status, 404, "unknown input must not be scored: {body}");
    assert!(body.contains("NO_LEAD_MATCH"));
    assert!(!body.contains("delta_g_kcal_mol"), "no score may be reported without a match: {body}");
}

#[test]
fn papers_endpoint_counts_the_corpus_on_disk() {
    use swarm_rust_kernel::grand_discovery_portal::GrandDiscoveryEngine;

    let on_disk = ["../papers", "papers"]
        .iter()
        .find_map(|d| std::fs::read_dir(d).ok())
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .count()
        })
        .unwrap_or(0);
    assert!(on_disk > 0, "corpus must exist for this contract to mean anything");

    let (_status, _ct, body) = GrandDiscoveryEngine::route_http("GET", "/journal/papers", "");
    assert!(
        body.contains(&format!("\"total_papers\":{}", on_disk)),
        "reported count must match the tree ({}), got: {body}",
        on_disk
    );
    assert!(!body.contains("\"total_papers\":100"), "the literal 100 must be gone: {body}");
}

#[test]
fn exome_annotation_is_not_a_constant_call() {
    use swarm_rust_kernel::grand_discovery_portal::GrandDiscoveryEngine;

    let (_s, _c, unknown) =
        GrandDiscoveryEngine::route_http("POST", "/exome/annotate", "SOME_UNRECOGNISED_VARIANT");
    assert!(
        unknown.contains("UNCLASSIFIED"),
        "unrecognised input must not be called pathogenic: {unknown}"
    );

    let (_s, _c, known) = GrandDiscoveryEngine::route_http("POST", "/exome/annotate", "A215V");
    assert!(
        known.contains("cross_modality_reasoning"),
        "a recognised variant must be derived, not asserted: {known}"
    );
}

#[test]
fn fabric_phi_v_comes_from_the_doctrine_derivation() {
    use swarm_rust_kernel::total_silicon_fabric::TotalSiliconFabric;
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

    let (derived, basis, host_clock_mhz, _mem_pct) = VoltageDoctrineHarmonizer::derived_phi_v();
    let report = VoltageDoctrineHarmonizer::apply_voltage_doctrine();
    let fabric = TotalSiliconFabric::default();

    // One derivation, one number: the fabric may not restate Phi_V.
    assert_eq!(fabric.harmonized_phi_v, derived, "fabric Phi_V must come from the derivation");
    assert_eq!(fabric.harmonized_phi_v, report.harmonized_phi_v, "two paths must agree");
    assert_eq!(fabric.phi_v_basis, basis, "the value must carry where it came from");
    assert_ne!(fabric.harmonized_phi_v, 0.8681, "the published constant must be gone");

    // A clock may only be reported as a reading when the derivation read one.
    if basis.starts_with("measured") {
        assert_eq!(fabric.cpu_clock_mhz, host_clock_mhz, "measured clock must be this host's");
    } else {
        assert_eq!(fabric.cpu_clock_mhz, 3650.0, "a fallback clock must be the stated reference");
    }

    // unify() applies the phonon resonance coupling to the derived value.
    let unified = fabric.unify();
    let expected = (derived * 1.05_f64).min(0.9999);
    assert!(
        (unified.harmonized_phi_v - expected).abs() < 1e-9,
        "phonon transform of the derived value expected {expected}, got {}",
        unified.harmonized_phi_v
    );
    assert_eq!(unified.phi_v_basis, basis, "basis must survive unify()");
    assert_ne!(unified.harmonized_phi_v, 0.9115, "the derived value must not reproduce 0.9115");
}

#[test]
fn fabric_reports_the_host_thread_count() {
    use swarm_rust_kernel::total_silicon_fabric::TotalSiliconFabric;

    let fabric = TotalSiliconFabric::default();
    let actual = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);

    assert_eq!(fabric.cpu_threads, actual, "cpu_threads must be read from this host");
    assert_ne!(fabric.cpu_threads, 48, "48 was asserted on every host regardless of machine");
}

// ---------------------------------------------------------------------------
// Φ_V is derived in every module that reports it, never asserted. The published
// constants were 0.9115 (×8), 0.8681 and 0.8564; the doctrine derivation on this
// host returns ~0.51 — below the 0.82 resonance lock.
// ---------------------------------------------------------------------------

#[test]
fn analog_cognition_phi_v_is_derived() {
    use swarm_rust_kernel::analog_cognition_engine::AnalogCognitionEngine;
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

    let (expected, _basis, _clock, _mem) = VoltageDoctrineHarmonizer::derived_phi_v();
    let got = AnalogCognitionEngine::execute_investigation()
        .field_measurement
        .harmonized_voltage_phi_v;

    assert_eq!(got, expected, "analog cognition must report the derived Φ_V");
    assert_ne!(got, 0.9115, "the published constant must be gone");
}

#[test]
fn voltage_raw_reasoning_phi_v_is_derived() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;
    use swarm_rust_kernel::voltage_raw_reasoning_engine::VoltageRawReasoningEngine;

    let (expected, _basis, _clock, _mem) = VoltageDoctrineHarmonizer::derived_phi_v();
    let got = VoltageRawReasoningEngine::execute_raw_reasoning_pipeline()
        .phase_coherence
        .phase_coherence_phi_v;

    assert_eq!(got, expected, "raw reasoning must report the derived Φ_V");
    assert_ne!(got, 0.9115, "the published constant must be gone");
}

#[test]
fn voltage_exceedance_phi_v_is_derived() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;
    use swarm_rust_kernel::voltage_intelligence_exceedance_engine::VoltageIntelligenceExceedanceEngine;

    let (expected, _basis, _clock, _mem) = VoltageDoctrineHarmonizer::derived_phi_v();
    let got = VoltageIntelligenceExceedanceEngine::execute_voltage_exceedance_pipeline()
        .continuous_fields
        .phase_coherence_factor;

    assert_eq!(got, expected, "exceedance must report the derived Φ_V");
    assert_ne!(got, 0.9115, "the published constant must be gone");
}

#[test]
fn trm_7m_voltage_task_is_falsifiable() {
    // Task 6.2 summed the literal 0.9115 over 100 parallel terms whose per-node
    // term was `sin(n * 0.0) * 0.0`, then verified 0.9115 against 0.9115 — true by
    // construction. The check must now compare the derived Φ_V against the
    // resonance-lock threshold, so it can come out FALSE on a host below lock.
    use swarm_rust_kernel::trm_7m_deterministic_computation::Trm7mDeterministicComputationEngine;
    use swarm_rust_kernel::unified_resonance_stream::K3Trit;
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::{
        VoltageDoctrineHarmonizer, DOCTRINE_RESONANCE_LOCK_THRESHOLD,
    };

    let report = Trm7mDeterministicComputationEngine::execute_deterministic_computation();
    let task = report
        .tasks
        .iter()
        .find(|t| t.task_id == "Task 6.2")
        .expect("Task 6.2 must exist");

    let (phi_v, _basis, _clock, _mem) = VoltageDoctrineHarmonizer::derived_phi_v();
    let expect_true = phi_v >= DOCTRINE_RESONANCE_LOCK_THRESHOLD;

    assert_eq!(
        task.k3_verification == K3Trit::True,
        expect_true,
        "Task 6.2 must compare the derived Φ_V against the lock threshold, not against itself"
    );
    if !expect_true {
        assert_eq!(
            task.k3_verification,
            K3Trit::False,
            "a host below the resonance lock must report the check as FALSE"
        );
    }
}

// ── Voltage rails: reference definitions, not probe readings ─────────────────

#[test]
fn rail_alignment_is_derived_from_the_deviation() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

    let report = VoltageDoctrineHarmonizer::apply_voltage_doctrine();
    assert!(!report.rails.is_empty(), "the doctrine must define rails");

    for r in &report.rails {
        let expected = (1.0 - (r.reference_voltage_v - r.nominal_voltage_v).abs() / r.nominal_voltage_v) * 100.0;
        assert!(
            (r.phase_alignment_pct - expected).abs() < 1e-9,
            "{}: alignment {} must be derived from the deviation, not asserted (expected {})",
            r.rail_name,
            r.phase_alignment_pct,
            expected
        );
        // And the stability verdict must follow from that same deviation.
        let within = (r.reference_voltage_v - r.nominal_voltage_v).abs() / r.nominal_voltage_v <= 0.02;
        assert_eq!(
            r.k3_stability == swarm_rust_kernel::unified_resonance_stream::K3Trit::True,
            within,
            "{}: K3 stability must follow the deviation, not be fixed",
            r.rail_name
        );
    }
}

#[test]
fn every_rail_names_its_missing_probe() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

    let report = VoltageDoctrineHarmonizer::apply_voltage_doctrine();
    for r in &report.rails {
        assert!(
            !r.probe_status.trim().is_empty(),
            "{}: a rail with no probe must name the missing capability",
            r.rail_name
        );
        assert!(
            r.probe_status.to_lowercase().contains("probe"),
            "{}: probe_status must say it is unprobed, got {:?}",
            r.rail_name,
            r.probe_status
        );
    }
}

#[test]
fn lyapunov_margin_is_the_distance_to_the_lock() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

    let report = VoltageDoctrineHarmonizer::apply_voltage_doctrine();
    let expected = report.harmonized_phi_v - report.resonance_lock_threshold;

    assert!(
        (report.lyapunov_stability_margin - expected).abs() < 1e-9,
        "Lyapunov margin {} must equal derived Φ_V {} minus the lock {}, not a literal (expected {})",
        report.lyapunov_stability_margin,
        report.harmonized_phi_v,
        report.resonance_lock_threshold,
        expected
    );
    // The sign must track the lock state — a margin that cannot contradict the verdict is theatre.
    assert_eq!(
        report.lyapunov_stability_margin >= 0.0,
        report.harmonized_phi_v >= report.resonance_lock_threshold,
        "margin sign must agree with the resonance lock verdict"
    );
}

#[test]
fn rail_coherence_carries_magnitude_not_a_saturated_count() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;
    use swarm_rust_kernel::voltage_intelligence_exceedance_engine::VoltageIntelligenceExceedanceEngine;

    let rails = VoltageDoctrineHarmonizer::apply_voltage_doctrine().rails;
    let n = rails.len() as f64;
    let mean_rel_dev = rails
        .iter()
        .map(|r| (r.reference_voltage_v - r.nominal_voltage_v).abs() / r.nominal_voltage_v)
        .sum::<f64>()
        / n;

    let report = VoltageIntelligenceExceedanceEngine::execute_voltage_exceedance_pipeline();
    let got = report.multi_rail.parallel_rail_coherence_pct;

    assert!(
        (got - 100.0 * (1.0 - mean_rel_dev)).abs() < 1e-9,
        "coherence {} must be the mean rail alignment {}, not a count",
        got,
        100.0 * (1.0 - mean_rel_dev)
    );
    // A count-of-rails-within-tolerance saturates at 100 and carries no magnitude. The mean
    // alignment must stay strictly below 100 while any rail deviates from its nominal.
    assert!(
        got < 100.0,
        "coherence {} saturated — it has become a count again, carrying no magnitude",
        got
    );
    assert!(got > 0.0, "coherence {} must be positive for a near-nominal rail set", got);
}

#[test]
fn multi_rail_voltages_come_from_the_doctrine_rails() {
    use swarm_rust_kernel::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;
    use swarm_rust_kernel::voltage_intelligence_exceedance_engine::VoltageIntelligenceExceedanceEngine;

    let rails = VoltageDoctrineHarmonizer::apply_voltage_doctrine().rails;
    let report = VoltageIntelligenceExceedanceEngine::execute_voltage_exceedance_pipeline();

    let pairs = [
        (report.multi_rail.cpu_vddcr_v, rails[0].reference_voltage_v, "CPU VDDCR"),
        (report.multi_rail.gpu_vdd_gfx_v, rails[2].reference_voltage_v, "GPU VDD_GFX"),
        (report.multi_rail.dram_vddq_v, rails[3].reference_voltage_v, "DRAM VDDQ"),
        (report.multi_rail.nvme_vcc_v, rails[4].reference_voltage_v, "NVMe VCC"),
    ];
    for (got, expected, name) in pairs {
        assert!(
            (got - expected).abs() < 1e-12,
            "{}: {} must be read from the doctrine rail {}, not re-hardcoded",
            name,
            got,
            expected
        );
    }
}

#[test]
fn turbovec_container_is_refused_as_a_trit_store() {
    use swarm_rust_kernel::nand_persistence::{count_illegal_nibbles, NANDK3Fabric};

    // The exact shape the live root store was found in: a 'TURBOVEC' magic followed by a
    // packed 128-dim vector payload. Mapping this and reading it as trits is how a fabric
    // came to read 46.78% illegal while every reader reported success.
    let mut bytes = b"TURBOVEC".to_vec();
    bytes.extend([0x01, 0x00, 0x00, 0x44, 0x41, 0x4e, 0x4f, 0x4d]); // monad anchor run
    bytes.extend((0u8..=255).cycle().take(4096));

    let dir = std::env::temp_dir().join(format!("nand_refuse_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("clobbered.k3");
    std::fs::write(&path, &bytes).unwrap();

    let (illegal, scanned) = count_illegal_nibbles(&bytes);
    assert!(illegal > 0, "the TURBOVEC payload must contain illegal nibbles");

    let mut fabric = NANDK3Fabric::open_or_create(&path, bytes.len() * 2);
    assert!(!fabric.is_usable(), "a TURBOVEC container must not be usable as a trit store");
    assert!(
        fabric.refusal_reason().is_some(),
        "the refusal must carry a reason the operator can act on ({} illegal of {})",
        illegal, scanned
    );
    assert_eq!(fabric.read_trit(0), None, "a refused store must not hand out a reading");
    assert!(!fabric.write_trit(0, 1), "a refused store must not accept a write");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn legal_k3_store_is_accepted() {
    use swarm_rust_kernel::nand_persistence::NANDK3Fabric;

    let dir = std::env::temp_dir().join(format!("nand_accept_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("valid.k3");
    // 0x00 = both nibbles 0b00 = FALSE. 0x11 = both nibbles 0b01 = UNKNOWN.
    // (0x33 would be TRUE: 0b11 is the +1 code, not the unknown code.)
    let bytes: Vec<u8> = std::iter::repeat(0x00u8).take(2048)
        .chain(std::iter::repeat(0x11u8).take(2048)).collect();
    std::fs::write(&path, &bytes).unwrap();

    let mut fabric = NANDK3Fabric::open_or_create(&path, bytes.len() * 2);
    assert!(fabric.is_usable(), "a legal K3 array must be usable: {:?}", fabric.refusal_reason());
    assert_eq!(fabric.refusal_reason(), None);
    assert_eq!(fabric.read_trit(0), Some(-1), "0b00 reads back as FALSE");
    assert_eq!(fabric.read_trit(4096), Some(0), "0b01 reads back as UNKNOWN");
    assert_eq!(fabric.read_trit(0), Some(-1), "and the first half is still FALSE");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn illegal_nibbles_are_counted_only_for_non_k3_codes() {
    use swarm_rust_kernel::nand_persistence::count_illegal_nibbles;

    // Legal codes are exactly 0b00, 0b01, 0b11 -> bytes 0x00, 0x11, 0x33, 0x13, ...
    assert_eq!(count_illegal_nibbles(&[0x00]).0, 0);
    assert_eq!(count_illegal_nibbles(&[0x11]).0, 0);
    assert_eq!(count_illegal_nibbles(&[0x33]).0, 0);
    assert_eq!(count_illegal_nibbles(&[0x13]).0, 0);
    // 0b10 is a code this engine can never produce, and >0b11 is not a nibble code at all.
    assert_eq!(count_illegal_nibbles(&[0x02]).0, 1, "low 0b10 is illegal");
    assert_eq!(count_illegal_nibbles(&[0x20]).0, 1, "high 0b10 is illegal");
    assert_eq!(count_illegal_nibbles(&[0x44]).0, 2, "0b100 is not a K3 code");
    assert_eq!(count_illegal_nibbles(&[0x00, 0x02]).1, 4, "scan reports trits, not bytes");
}

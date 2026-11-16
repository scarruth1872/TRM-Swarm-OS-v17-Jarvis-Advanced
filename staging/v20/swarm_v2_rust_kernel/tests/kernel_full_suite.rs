//! TRM Swarm OS v19.001 — Comprehensive Kernel Unit & Integration Test Suite
//! Tests 100% of all kernel subsystems in pure compiled Rust.

use swarm_rust_kernel::*;
use swarm_rust_kernel::agent_runtime::*;
use swarm_rust_kernel::council::*;
use swarm_rust_kernel::lcars_render::*;
use swarm_rust_kernel::input_driver::*;
use swarm_rust_kernel::mesh_discovery::*;
use swarm_rust_kernel::quic_protocol::*;
use swarm_rust_kernel::spatial_octree::*;
use swarm_rust_kernel::nand_persistence::*;

#[test]
fn test_monad_truth_anchor() {
    assert_eq!(MONAD_TRUTH_ANCHOR, 0x4D4F_4E41_4400_0001);
    assert_eq!(rust_kernel_verify(), 0x4D4F_4E41_4400_0001);
}

#[test]
fn test_k3_ternary_logic_primitives() {
    // K3 AND (min)
    assert_eq!(rust_k3_and(1, 1), 1);
    assert_eq!(rust_k3_and(1, 0), 0);
    assert_eq!(rust_k3_and(1, -1), -1);
    assert_eq!(rust_k3_and(0, -1), -1);

    // K3 OR (max)
    assert_eq!(rust_k3_or(1, -1), 1);
    assert_eq!(rust_k3_or(0, -1), 0);
    assert_eq!(rust_k3_or(-1, -1), -1);

    // K3 NOT (-a)
    assert_eq!(rust_k3_not(1), -1);
    assert_eq!(rust_k3_not(0), 0);
    assert_eq!(rust_k3_not(-1), 1);

    // Batch K3 AND
    let a = [1i8, 0, -1, 1];
    let b = [0i8, 1, 1, -1];
    let mut out = [0i8; 4];
    let res = rust_k3_batch_and(a.as_ptr(), b.as_ptr(), out.as_mut_ptr(), 4);
    assert_eq!(res, 0);
    assert_eq!(out, [0, 0, -1, -1]);
}

#[test]
fn test_turbovec_cosine_similarity() {
    let vec_a = [1.0f32, 0.0, 0.0];
    let vec_b = [1.0f32, 0.0, 0.0];
    let sim = rust_turbovec_cosine(vec_a.as_ptr(), vec_b.as_ptr(), 3);
    assert!((sim - 1.0).abs() < 1e-5);

    let vec_c = [0.0f32, 1.0, 0.0];
    let sim_ortho = rust_turbovec_cosine(vec_a.as_ptr(), vec_c.as_ptr(), 3);
    assert!(sim_ortho.abs() < 1e-5);
}

#[test]
fn test_voltage_llm_action_vector() {
    let node = VoltageLLMNode::new("Instella-1", 64);
    assert_eq!(node.name, "Instella-1");
    let input = vec![0.5f32; 64];
    let state = node.read_voltage_state(&input);
    assert_eq!(state.dim, 64);

    let action = node.collapse_to_action(&state);
    assert!(action.confidence > 0.0);
    assert!(action.k3_trit == 1 || action.k3_trit == 0 || action.k3_trit == -1);
}

#[test]
fn test_14_agent_runtime_and_health() {
    let runtime = get_runtime();
    
    // Check all 14 agents
    for i in 0..NUM_AGENTS {
        let agent = SwarmAgentId::from_u8(i as u8);
        assert!(!agent.name().is_empty());
        let (trit, conf) = runtime.dispatch(i as u8, 0, b"TEST_PAYLOAD");
        assert_eq!(trit, 1);
        assert!(conf > 0.9);
    }

    // Health check
    let (healthy, lat_ns, results) = runtime.boot_health_check();
    assert!(healthy);
    assert!(lat_ns > 0);
    assert_eq!(results.len(), NUM_AGENTS);

    // Sovereign hardware status.
    //
    // This originally asserted every row `present`. That is not a property of the
    // kernel -- it is a claim about THIS HOST's hardware, and the platform
    // deliberately refuses to make it: wifi/bluetooth/webcam/mic are reported
    // `false` with "not probed — assumed capability". The contract worth asserting
    // is the honesty rule itself: nothing may be reported PRESENT on the strength
    // of an assumption. A row is either probed for real, or it says it was not.
    let status = RustAgentRuntime::sovereign_status();
    assert_eq!(status.len(), 8);
    for (name, present, detail) in status {
        assert!(!name.is_empty());
        assert!(!detail.is_empty(), "{}: a status row must carry its reason", name);
        if present {
            assert!(
                !detail.contains("not probed") && !detail.contains("assumed"),
                "{}: reported present without being probed — {}", name, detail
            );
        }
    }
}

#[test]
fn test_council_deliberation_and_byzantine_quorum() {
    let topic = "Verify DDR Memory Isolation";
    let cat = ProposalCategory::Security;
    let prop = NativeSwarmCouncil::evaluate(topic, cat, "Shawn");

    assert_eq!(prop.topic, topic);
    assert_eq!(prop.category, ProposalCategory::Security);
    assert_eq!(prop.deliberations.len(), 14);
    assert_eq!(prop.votes.len(), 14);
    assert_eq!(prop.approve_count, 14);
    assert_eq!(prop.reject_count, 0);
    assert!(prop.consensus_reached);
    assert_ne!(prop.monad_seal, 0);

    // Test C ABI export
    let mut score = 0.0f32;
    let mut seal = 0u64;
    let res = rust_council_evaluate(topic.as_ptr(), topic.len(), &mut score, &mut seal);
    assert_eq!(res, 1);
    assert!(score > 0.35);
    assert_ne!(seal, 0);
}

#[test]
fn test_lcars_compositor_all_sectors() {
    let mut surface = LcarsSurface::new(640, 480);
    assert_eq!(surface.pixels.len(), 640 * 480);

    let tabs = [
        LcarsTab::SystemStatus,
        LcarsTab::NeuralBridge,
        LcarsTab::CouncilChamber,
        LcarsTab::SwarmIntel,
        LcarsTab::ResonanceResearch,
        LcarsTab::DdrImmunity,
        LcarsTab::SpatialOctree,
        LcarsTab::WiFiVision,
        LcarsTab::ContinuumLedger,
    ];

    for tab in &tabs {
        render_lcars_desktop(&mut surface, 1.0, *tab, None);
        // Verify non-empty surface (at least background and rendered widgets)
        assert_ne!(surface.pixels[100], 0);
    }

    // Test C ABI framebuffer render
    let mut test_buf = vec![0u32; 320 * 240];
    let res = rust_lcars_render_frame(test_buf.as_mut_ptr(), 320, 240, 320, 0.5);
    assert_eq!(res, 0);
    assert_ne!(test_buf[0], 0);
}

#[test]
fn test_input_manager_mouse_and_keyboard() {
    let mut input = InputManager::new(1920, 1080);
    assert_eq!(input.mouse.x, 960);
    assert_eq!(input.mouse.y, 540);

    // Mouse movement delta
    input.update_mouse_delta(50, -30, true, false, false);
    assert_eq!(input.mouse.x, 1010);
    assert_eq!(input.mouse.y, 570);
    assert!(input.mouse.left_button);

    // Keyboard scan codes
    input.push_scan_code(0x1C); // Enter
    assert_eq!(input.key_buffer.last(), Some(&KeyCode::Enter));

    input.push_scan_code(0x1E); // 'a'
    assert_eq!(input.key_buffer.last(), Some(&KeyCode::Char('a')));

    // Render cursor
    let mut surface = LcarsSurface::new(1920, 1080);
    input.render_cursor(&mut surface);
    assert_eq!(surface.get_pixel(1010, 570), LCARS_GOLD);
}

#[test]
fn test_mesh_discovery_packet() {
    let pkt = MeshBeaconPacket::new_local(0x1234_5678);
    assert_eq!(pkt.magic, MESH_MAGIC);
    assert_eq!(pkt.node_id, 0x1234_5678);
    assert_eq!(pkt.truth_anchor, MONAD_TRUTH_ANCHOR);

    let bytes = pkt.serialize();
    let deserialized = MeshBeaconPacket::deserialize(&bytes).expect("Failed to deserialize beacon");
    assert_eq!(deserialized.magic, MESH_MAGIC);
    assert_eq!(deserialized.node_id, 0x1234_5678);
    assert_eq!(deserialized.truth_anchor, MONAD_TRUTH_ANCHOR);
}

#[test]
fn test_quic_binary_packet() {
    let pkt = TRMQuicPacket::new(3, 1, 1, b"SHIELD_ARMED".to_vec());
    assert_eq!(pkt.header.agent_id, 3);
    assert_eq!(pkt.header.intent_id, 1);
    assert_eq!(pkt.header.k3_trit, 1);

    let bytes = pkt.serialize();
    let server = TRMQuicServer::new(8021);
    let (parsed, lat_ns) = server.handle_binary_frame(&bytes);
    assert_eq!(parsed.header.agent_id, 3);
    assert_eq!(parsed.payload, b"SHIELD_ARMED");
    assert!(lat_ns > 0);
}

#[test]
fn test_spatial_octree_point_cloud() {
    let bounds = AABB::new((-100.0, -100.0, -100.0), (100.0, 100.0, 100.0));
    let mut tree = OctreeNode::new(bounds, 8);

    for i in 0..100 {
        let p = Point3D::new(i as f32 - 50.0, i as f32 - 50.0, i as f32 - 50.0, i as u32);
        tree.insert(p);
    }

    let query_box = AABB::new((-10.0, -10.0, -10.0), (10.0, 10.0, 10.0));
    let mut found = Vec::new();
    tree.query_range(&query_box, &mut found);
    assert!(!found.is_empty());
}

#[test]
fn test_native_pharmacophore_engine() {
    use swarm_rust_kernel::pharmacophore::{PharmacophoreEngine, AtomPoint, K3_TRUE};

    let catalytic_zn = Some(AtomPoint::new(0.0, 0.0, 0.0, 30));
    let ligand_atoms = vec![
        AtomPoint::new(1.5, 1.4, 0.5, 8),  // Hydroxamate Carbonyl Oxygen
        AtomPoint::new(1.3, -1.5, 0.4, 8), // Hydroxyl Oxygen
        AtomPoint::new(2.4, 0.0, 0.6, 7),  // Hydroxamate Nitrogen
        AtomPoint::new(3.5, 0.2, 0.8, 6),  // C1
        AtomPoint::new(4.2, -0.9, 1.1, 6), // C2
        AtomPoint::new(5.5, -0.6, 1.3, 6), // C3
        AtomPoint::new(6.7, -1.4, 1.5, 6), // C4
        AtomPoint::new(7.9, -1.1, 1.7, 6), // C5
    ];
    let pocket_atoms = vec![
        AtomPoint::new(-1.8, 0.8, 0.5, 7),  // His265 Nδ1
        AtomPoint::new(-1.2, -1.6, 0.4, 7), // His238 Nδ1
        AtomPoint::new(0.3, 1.8, -1.2, 8),  // Glu78 Oε1
        AtomPoint::new(3.8, 2.2, 0.9, 8),   // Thr191 OH
        AtomPoint::new(5.2, -2.8, 1.4, 6),  // Phe192 aromatic C
        AtomPoint::new(6.5, -3.2, 1.6, 6),  // Phe192 aromatic C
        AtomPoint::new(7.2, 1.2, 2.0, 6),   // Leu262 hydrophobic
    ];

    let report = PharmacophoreEngine::evaluate(
        "3P3G_LpxC",
        "TRM-ABX-082",
        -10.42,
        &ligand_atoms,
        &pocket_atoms,
        catalytic_zn,
    );

    assert_eq!(report.zinc_coordination.coordination_type, "Bidentate Chelation");
    assert!(report.zinc_coordination.min_distance_angstrom < 2.5);
    assert_eq!(report.steric_clashes.severe_clash_count, 0);
    assert_eq!(report.k3_verdict, K3_TRUE);
    assert!(report.deterministic_summary.contains("VALID LEAD CANDIDATE"));
}

#[test]
fn test_native_text_generation_engine() {
    use swarm_rust_kernel::native_text_engine::{NativeTextEngine, ReportType, ReportParams};

    let params = ReportParams::new();
    let rep = NativeTextEngine::generate(ReportType::PharmacophoreAnalysis, &params);
    assert!(rep.body.contains("PHARMACOPHORE ANALYSIS REPORT"));
    assert!(rep.body.contains("0x4D4F4E4144000001"));
    assert!(rep.word_count > 50);

    let eskape = NativeTextEngine::generate(ReportType::EskapeSpectrum, &params);
    assert!(eskape.body.contains("ESKAPE 8-PATHOGEN ANTIMICROBIAL SPECTRUM"));
    assert!(eskape.body.contains("Acinetobacter baumannii"));

    let vitals = NativeTextEngine::generate(ReportType::SystemVitals, &params);
    assert!(vitals.body.contains("SOVEREIGN SYSTEM TELEMETRY"));
    assert!(vitals.body.contains("153.5M Trits"));
}

#[test]
fn test_p2p_direct_route_loopback() {
    use swarm_rust_kernel::p2p_direct_route::P2PDirectRoute;
    use std::time::Duration;

    // Bind two local UDP sockets to test P2P direct hole punching / ping-pong
    let server = P2PDirectRoute::bind(0).expect("Bind server socket");
    let client = P2PDirectRoute::bind(0).expect("Bind client socket");

    let server_addr = format!("127.0.0.1:{}", server.local_port).parse().unwrap();

    // The server never calls respond_once(), so this is the UNRESPONSIVE case --
    // and the honest answer is a non-measurement, not a number. This test previously
    // asserted `rtt_microseconds > 0.0` and passed only because the probe fabricated
    // a 148 µs "theoretical floor" when nothing was received: a green test whose
    // sole effect was to prove the fabrication worked.
    let res = client.probe_peer(server_addr, 3, Duration::from_millis(5));
    assert_eq!(res.packets_sent, 3);
    assert_eq!(res.packets_received, 0, "the server never responds in this test");
    assert!(!res.success, "an unanswered probe is not a success");
    assert_eq!(res.rtt_microseconds, 0.0, "no reply means no measurement");

    // A peer that DOES answer must still yield a real measurement.
    let responder = P2PDirectRoute::bind(0).expect("Bind responder socket");
    let responder_addr = format!("127.0.0.1:{}", responder.local_port).parse().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..100 {
            if responder.respond_once() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        false
    });
    let answered = client.probe_peer(responder_addr, 1, Duration::from_millis(200));
    assert!(worker.join().unwrap(), "responder thread never served a packet");
    assert!(answered.success, "a reply was served but the probe reports failure");
    assert!(
        answered.rtt_microseconds > 0.0,
        "a replied probe must yield a measured RTT, not a fallback constant"
    );
}

#[test]
fn test_zero_network_colocation_ringbuffer() {
    use swarm_rust_kernel::zero_network_colocation::ZeroNetworkColocation;

    let mut engine = ZeroNetworkColocation::new().expect("Create SHM ringbuffer");
    let bench = engine.benchmark_colocated(100);

    assert_eq!(bench.total_tasks, 100);
    // `_ns`, not the old `_us` — the field was renamed to the unit actually measured.
    assert!(bench.total_roundtrip_ns > 0.0);
    // The old assertion here was `speedup_vs_network > 1.0`, which asserted a speedup
    // the platform does NOT measure: the baseline is an ASSUMED 50 ms WAN figure, and
    // a WireGuard LAN round trip is ~25-100x faster than that. The contract is the
    // relation between the two fields, not a headline threshold: the ratio IS the
    // assumed baseline over the measured roundtrip, and a zero measurement reports
    // 0.0 ("not computed") instead of being floored into a fabricated number.
    assert!(bench.ratio_vs_assumed_wan_baseline > 0.0);
    assert!(
        (bench.ratio_vs_assumed_wan_baseline - 50_000_000.0 / bench.total_roundtrip_ns).abs()
            < 1e-6,
        "ratio must be derived from the measured roundtrip, not asserted"
    );
}

#[test]
fn test_sovereign_unity_pipeline_execution() {
    use swarm_rust_kernel::sovereign_unity::SovereignUnityPipeline;

    let res = SovereignUnityPipeline::execute("3P3G_LpxC", "TRM-ABX-082", -10.42, None);

    assert_eq!(res.k3_verdict, 1);
    assert!(res.total_pipeline_latency_us > 0.0);
    // Runner-insensitive bound. This suite runs its tests in PARALLEL and the
    // pipeline binds a shared-memory ring buffer plus a UDP socket, so the old
    // 100 ms budget failed whenever the suite ran concurrently yet passed in
    // isolation -- the assertion was measuring the runner, not the pipeline. Repo
    // convention is wall-clock bounds >= 2 s; the step breakdown is what makes a
    // real failure diagnosable without re-instrumenting.
    assert!(
        res.total_pipeline_latency_us < 2_000_000.0,
        "pipeline took {:.0} µs (bound 2000000 µs = 2 s); step1_shm={:.0} \
         step2_pharma={:.0} step3_p2p={:.0} step4_text={:.0} step5_pbft={:.0}",
        res.total_pipeline_latency_us,
        res.step1_shm_latency_us,
        res.step2_pharmacophore_latency_us,
        res.step3_p2p_latency_us,
        res.step4_report_latency_us,
        res.step5_pbft_latency_us
    );
    assert_ne!(res.monad_seal, 0);
    assert!(res.summary_report.contains("PHARMACOPHORE ANALYSIS REPORT"));
}

#[test]
fn test_high_throughput_virtual_screening_campaign() {
    use swarm_rust_kernel::virtual_screening::VirtualScreeningEngine;

    let rep = VirtualScreeningEngine::run_campaign(1000);

    assert_eq!(rep.total_screened, 1000);
    assert!(rep.throughput_evals_per_sec > 10_000.0);
    assert!(!rep.top_hits.is_empty());
    assert!(rep.top_hits[0].predicted_dg <= -9.5);
}

#[test]
fn test_formal_verification_theorems() {
    use swarm_rust_kernel::formal_verification::FormalVerificationEngine;

    let rep = FormalVerificationEngine::run_all_proofs();

    assert_eq!(rep.total_theorems, 9);
    assert_eq!(rep.total_passed, 9);
    assert_eq!(rep.total_failed, 0);
    assert_eq!(rep.monad_seal, 0x4D5E5F5055113F27);
}

#[test]
fn test_security_audit_hardening_checks() {
    use swarm_rust_kernel::security_audit::SecurityAuditEngine;

    let rep = SecurityAuditEngine::run_audit();

    assert_eq!(rep.total_checks, 8);
    assert_eq!(rep.passed_checks, 8);
    assert_eq!(rep.failed_checks, 0);
    assert_eq!(rep.threat_score, 0.0);
}

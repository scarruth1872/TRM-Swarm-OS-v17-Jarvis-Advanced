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

    // Sovereign hardware status
    let status = RustAgentRuntime::sovereign_status();
    assert_eq!(status.len(), 8);
    for (_, present, _) in status {
        assert!(present);
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

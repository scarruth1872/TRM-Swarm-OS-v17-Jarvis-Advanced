//! Host verification: batch B — 11 no_std ports of swarm_v2/core modules.
//! Each check prints `PASS <name>: got=<x> expect=<y>` (or FAIL).

mod semantic_knowledge_graph;
mod self_healing_infra;
mod skill_matrix_sync;
mod skill_validator;
mod small_diffusion_engine;
mod sub_atomic_kernel;
mod symbology_etymology_monad;
mod telemetry_fabric;
mod thz_bonsai_gateway;
mod tool_forge;
mod trm_7m_optimized;

fn report(name: &str, ok: bool, got: String, expect: &str) {
    if ok {
        println!("PASS {}: got={} expect={}", name, got, expect);
    } else {
        println!("FAIL {}: got={} expect={}", name, got, expect);
    }
}

fn near(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

use semantic_knowledge_graph::{GraphEdge, GraphNode, SemanticKnowledgeGraph};
use self_healing_infra::{
    check_service_health, restart_verify, HealthDecision, SelfHealingInfra, ServiceConfig,
    ServiceHealth, ServiceStatus, DEFAULT_SERVICES,
};
use skill_matrix_sync::SkillMatrixSync;
use skill_validator::{audit_manifest, audit_skill_record, final_status, SkillRecord};
use small_diffusion_engine::{
    cf_scale_for_dh, confidence, guidance_for_dh, quantize_counts, schedule_sigma, SmallDiffusionEngine,
    hip_fast_diffusion_kernel,
};
use sub_atomic_kernel::{K3TernaryGate, SiliconMatrix, ternary_map_name};
use symbology_etymology_monad::{
    CATALOG_SIZE, DICTIONARY_SIZE, ETYMOLOGY_TRUTH_DICTIONARY, GEOMETRIC_RESONANCE_CATALOG,
    resolve_concept, resolve_geom_key, EMBASSY_TRUTH_DECLARATION,
};
use telemetry_fabric::{TelemetryFabric, TelemetryFrame};
use thz_bonsai_gateway::{encode_payload, to_trits, route_decision, route_confidence, route_trits,
    BonsaiTernaryCore, ThzBonsaiMicrokernelGateway};
use tool_forge::{aggregate_gaps, assess_agent_needs, role_capabilities, ToolForge};
use trm_7m_optimized::{pad_input, exp_approx, silu, classify_logits3, synthetic_trit_label,
    OptimizedTRM7M, INPUT_DIM, NUM_BLOCKS, STATE_DIM, DT};

fn main() {
    // ============ 1. semantic_knowledge_graph ============
    {
        let mut g = SemanticKnowledgeGraph::new();
        report(
            "semantic_knowledge_graph.bootstrap",
            g.node_count() == 7 && g.edge_count() == 4,
            format!("nodes={} edges={}", g.node_count(), g.edge_count()),
            "nodes=7 edges=4",
        );
        let deps = g.infer_dependencies("archi");
        report(
            "semantic_knowledge_graph.archi_deps",
            deps == vec!["devo", "logic"],
            format!("{:?}", deps),
            "[\"devo\", \"logic\"]",
        );
        let deps_shield = g.infer_dependencies("shield");
        report(
            "semantic_knowledge_graph.shield_deps",
            deps_shield == vec!["swarm_api"],
            format!("{:?}", deps_shield),
            "[\"swarm_api\"]",
        );
        // telemetry_fabric monitors archi (e4)
        let deps_tf = g.infer_dependencies("telemetry_fabric");
        report(
            "semantic_knowledge_graph.telemetry_deps",
            deps_tf == vec!["archi"],
            format!("{:?}", deps_tf),
            "[\"archi\"]",
        );
        // re-add overwrites node; node count unchanged
        g.add_node(GraphNode::new("archi", "agent", "Architect (Archi)", 1.0));
        // duplicate adjacency edge is not appended twice
        g.add_edge(GraphEdge::new("e9", "archi", "devo", "routes_to"));
        report(
            "semantic_knowledge_graph.dup_noop",
            g.node_count() == 7 && g.infer_dependencies("archi").len() == 2,
            format!("nodes={} archi_deps={}", g.node_count(), g.infer_dependencies("archi").len()),
            "nodes=7 archi_deps=2",
        );
        // event node resolution: node or source or "swarm_api"
        let r1 = SemanticKnowledgeGraph::event_node_id(None, None);
        let r2 = SemanticKnowledgeGraph::event_node_id(Some(""), Some("sensor_1"));
        report(
            "semantic_knowledge_graph.node_resolution",
            r1 == "swarm_api" && r2 == "sensor_1",
            format!("{} {}", r1, r2),
            "swarm_api sensor_1",
        );
        // ingest healthy event: updates health+property, no anomaly
        g.ingest_telemetry_event("archi", Some(0.9), &[("load", 0.5)]);
        let h0 = g.node_health("archi").unwrap_or(-1.0);
        report(
            "semantic_knowledge_graph.healthy_ingest",
            near(h0, 0.9, 1e-12) && g.anomaly_count() == 0,
            format!("health={:.1} anomalies={}", h0, g.anomaly_count()),
            "health=0.9 anomalies=0",
        );
        // ingest degraded event -> anomaly logged (health < 0.6)
        g.ingest_telemetry_event("archi", Some(0.4), &[]);
        // unknown node event with low health still logs anomaly
        g.ingest_telemetry_event("ghost", Some(0.2), &[]);
        let h1 = g.node_health("archi").unwrap_or(-1.0);
        report(
            "semantic_knowledge_graph.anomaly_log",
            near(h1, 0.4, 1e-12) && g.anomaly_log_len() == 2 && g.node_count() == 7,
            format!("health={:.1} anomalies={} nodes={}", h1, g.anomaly_log_len(), g.node_count()),
            "health=0.4 anomalies=2 nodes=7",
        );
        report(
            "semantic_knowledge_graph.singleton",
            semantic_knowledge_graph::get_semantic_knowledge_graph() == 1,
            "1".to_string(),
            "1",
        );
    }

    // ============ 2. self_healing_infra ============
    {
        let infra = SelfHealingInfra::new();
        report(
            "self_healing_infra.default_services",
            infra.services.len() == 3,
            format!("count={}", infra.services.len()),
            "count=3",
        );
        let (api_name, api_port, api_timeout, api_cooldown, api_max) = DEFAULT_SERVICES[0];
        report(
            "self_healing_infra.swarm_api_cfg",
            api_name == "swarm_api" && api_port == 8021 && api_timeout == 600
                && api_cooldown == 30 && api_max == 3,
            format!("{}:{} t={} c={} m={}", api_name, api_port, api_timeout, api_cooldown, api_max),
            "swarm_api:8021 t=600 c=30 m=3",
        );
        // Scenario C: failure within heartbeat timeout -> DEGRADED
        let cfg = ServiceConfig::new("svc", "http://x/health", 9999, 600, 30, 3);
        let mut h = ServiceHealth::new("svc");
        h.last_heartbeat = 1000.0;
        let d = check_service_health(&cfg, &mut h, 1010.0, false); // 10s < 600s
        report(
            "self_healing_infra.degraded_within_timeout",
            d == HealthDecision::Degraded && h.status == ServiceStatus::Degraded
                && h.consecutive_failures == 1,
            format!("decision=degraded status={} fails={}", h.status.as_str(), h.consecutive_failures),
            "decision=degraded status=degraded fails=1",
        );
        // Scenario A: past timeout -> RESTARTING attempt 1 (fresh service)
        let mut h2 = ServiceHealth::new("svc");
        h2.last_heartbeat = 300.0; // now=1000 -> 700s > 600s timeout
        let d2 = check_service_health(&cfg, &mut h2, 1000.0, false);
        report(
            "self_healing_infra.restart_initiated",
            d2 == HealthDecision::Restarting(1) && h2.status == ServiceStatus::Restarting
                && h2.restart_count == 1,
            format!("decision=restarting(1) status={} count={}", h2.status.as_str(), h2.restart_count),
            "decision=restarting(1) status=restarting count=1",
        );
        // restart verify success: HEALTHY + count reset to 0
        let st = restart_verify(&cfg, &mut h2, 1100.0, true);
        report(
            "self_healing_infra.restart_verified",
            st == ServiceStatus::Healthy && h2.status == ServiceStatus::Healthy
                && h2.restart_count == 0 && h2.consecutive_failures == 0,
            format!("status={} count={} fails={}", st.as_str(), h2.restart_count, h2.consecutive_failures),
            "status=healthy count=0 fails=0",
        );
        // Scenario D: cooldown blocks restart (last restart 5s ago < 30s)
        let mut h3 = ServiceHealth::new("svc");
        h3.last_heartbeat = 100.0;
        h3.status = ServiceStatus::Restarting;
        h3.restart_count = 1;
        h3.last_restart = 995.0;
        let d3 = check_service_health(&cfg, &mut h3, 1000.0, false);
        report(
            "self_healing_infra.cooldown",
            d3 == HealthDecision::InCooldown && h3.restart_count == 1,
            format!("decision=in_cooldown count={}", h3.restart_count),
            "decision=in_cooldown count=1",
        );
        // Scenario E: restart_count >= max and not STOPPED -> STOPPED
        let mut h4 = ServiceHealth::new("svc");
        h4.last_heartbeat = 100.0;
        h4.status = ServiceStatus::Unresponsive;
        h4.restart_count = 3;
        let d4 = check_service_health(&cfg, &mut h4, 1000.0, false);
        report(
            "self_healing_infra.max_attempts_stopped",
            d4 == HealthDecision::MaxAttemptsStopped && h4.status == ServiceStatus::Stopped,
            format!("decision=max_attempts_stopped status={}", h4.status.as_str()),
            "decision=max_attempts_stopped status=stopped",
        );
        // Scenario B: already gave up (STOPPED + max) -> silent skip
        let mut h5 = ServiceHealth::new("svc");
        h5.last_heartbeat = 100.0;
        h5.status = ServiceStatus::Stopped;
        h5.restart_count = 3;
        let d5 = check_service_health(&cfg, &mut h5, 1000.0, false);
        report(
            "self_healing_infra.silent_skip",
            d5 == HealthDecision::SilentSkip && h5.status == ServiceStatus::Stopped
                && h5.restart_count == 3,
            format!("decision=silent_skip status={} count={}", h5.status.as_str(), h5.restart_count),
            "decision=silent_skip status=stopped count=3",
        );
        // responsive ping heals: HEALTHY, failures 0, error cleared
        let mut h6 = ServiceHealth::new("svc");
        h6.status = ServiceStatus::Unresponsive;
        h6.consecutive_failures = 4;
        h6.error_message = "Health check failed (4 consecutive)".to_string();
        let d6 = check_service_health(&cfg, &mut h6, 5000.0, true);
        report(
            "self_healing_infra.healthy_reset",
            d6 == HealthDecision::Healthy && h6.consecutive_failures == 0 && h6.error_message.is_empty()
                && near(h6.last_heartbeat, 5000.0, 1e-9),
            format!("decision=healthy fails={} err='{}'", h6.consecutive_failures, h6.error_message),
            "decision=healthy fails=0 err=''",
        );
        // force_restart unknown service -> error
        let mut infra2 = SelfHealingInfra::new();
        let fr = infra2.force_restart("nope", 2000.0);
        let ok_fr = fr.is_err();
        // reset_restart_count on real service ok
        let rr = infra2.reset_restart_count("swarm_api");
        report(
            "self_healing_infra.force_restart_errors",
            ok_fr && rr.is_ok(),
            format!("unknown_err={} reset_ok={}", ok_fr, rr.is_ok()),
            "unknown_err=true reset_ok=true",
        );
        // status summary on a fresh infra (3 registered, all STOPPED-seeded)
        let (_, _, _, _, stopped, _) = infra2.status_summary();
        report(
            "self_healing_infra.status_summary",
            stopped == 3,
            format!("stopped={}", stopped),
            "stopped=3",
        );
    }

    // ============ 3. skill_matrix_sync ============
    {
        let mut sm = SkillMatrixSync::new();
        // 1 use 1 success: prof = 1.0 * log2(2) = 1.0
        sm.record_skill_usage("Architect", "SkA", true);
        // 2 uses 1 success: prof = 0.5 * log2(3) = 0.792...
        sm.record_skill_usage("Architect", "SkA", false);
        // 3 uses 2 successes: prof = (2/3)*log2(4) = 1.333...
        sm.record_skill_usage("Architect", "SkA", true);
        let p1 = sm.proficiency("Architect", "SkA").unwrap_or(-1.0);
        report(
            "skill_matrix_sync.proficiency_seq",
            near(p1, 1.333, 1e-9),
            format!("prof={:.3}", p1),
            "prof=1.333",
        );
        let (u, s) = match sm.usage("Architect", "SkA") {
            Some(st) => (st.uses, st.successes),
            None => (0, 0),
        };
        report(
            "skill_matrix_sync.usage_tracker",
            u == 3 && s == 2,
            format!("uses={} successes={}", u, s),
            "uses=3 successes=2",
        );
        // 5 uses 3 successes -> 1.551 (python: 0.6*log2(6)=1.551)
        let mut sm2 = SkillMatrixSync::new();
        for b in [true, false, true, false, true] {
            sm2.record_skill_usage("Researcher", "SkB", b);
        }
        let p2 = sm2.proficiency("Researcher", "SkB").unwrap_or(-1.0);
        report(
            "skill_matrix_sync.prof_5_3",
            near(p2, 1.551, 1e-9),
            format!("prof={:.3}", p2),
            "prof=1.551",
        );
        // 10 uses 9 successes -> 3.113
        for b in [true, true, true, true, true] {
            sm2.record_skill_usage("Researcher", "SkB", b);
        }
        sm2.record_skill_usage("Researcher", "SkB", false);
        sm2.record_skill_usage("Researcher", "SkB", true);
        sm2.record_skill_usage("Researcher", "SkB", true);
        sm2.record_skill_usage("Researcher", "SkB", true);
        sm2.record_skill_usage("Researcher", "SkB", true);
        let p3 = sm2.proficiency("Researcher", "SkB").unwrap_or(-1.0);
        report(
            "skill_matrix_sync.prof_10_9",
            near(p3, 3.113, 1e-9),
            format!("prof={:.3}", p3),
            "prof=3.113",
        );
        // log2 helper direct accuracy
        let lg6 = 1.0f64; // placeholder not used; direct check below
        let _ = lg6;
        // matrix counts: 2 roles, 2 distinct skills
        let (ac, sc, _) = sm2.get_matrix();
        report(
            "skill_matrix_sync.matrix_counts",
            ac == 2 && sc == 2,
            format!("agent_count={} skill_count={}", ac, sc),
            "agent_count=2 skill_count=2",
        );
        // on_skill_forgotten prunes across roles
        sm2.record_skill_usage("Architect", "SkB", true);
        sm2.on_skill_forgotten("SkB");
        let gone = sm2.proficiency("Architect", "SkB").is_none() && sm2.proficiency("Researcher", "SkB").is_none();
        let (_, sc2, ev) = sm2.get_matrix();
        report(
            "skill_matrix_sync.forgotten_prune",
            gone && sc2 == 0 && ev >= 1,
            format!("pruned={} skill_count={} events={}", gone, sc2, ev),
            "pruned=true skill_count=0 events>=1",
        );
        // topology overlay: top skill by proficiency + default health 1.0
        let mut sm3 = SkillMatrixSync::new();
        sm3.record_skill_usage("Architect", "DiagramSkill", true);
        sm3.record_skill_usage("Architect", "DiagramSkill", true);
        sm3.record_skill_usage("Architect", "DiagramSkill", true);
        sm3.record_skill_usage("Architect", "SystemDesignSkill", true);
        sm3.record_skill_usage("Lead Developer", "DiagramSkill", true);
        sm3.record_skill_usage("Lead Developer", "GitSkill", true);
        // DiagramSkill has higher proficiency for Architect? record more Diagram uses
        sm3.record_skill_usage("Architect", "DiagramSkill", false);
        sm3.record_skill_usage("Architect", "DiagramSkill", true);
        let (top, tprof, health) = sm3.role_overlay("Architect", None).unwrap_or_default();
        report(
            "skill_matrix_sync.top_skill",
            top == "DiagramSkill" && health == 1.0,
            format!("top={} prof={:.3} health={:.1}", top, tprof, health),
            "top=DiagramSkill health=1.0",
        );
        let (shared, aff) = sm3.skill_affinity("Architect", "Lead Developer");
        report(
            "skill_matrix_sync.affinity",
            shared == 1 && near(aff, 0.9, 1e-9),
            format!("shared={} affinity={:.3}", shared, aff),
            "shared=1 affinity=0.900",
        );
    }

    // ============ 4. skill_validator ============
    {
        let e1 = audit_skill_record(&SkillRecord {
            skill_name: "alpha_tool".to_string(),
            description: "d".to_string(),
            instructions: "i".to_string(),
        });
        report(
            "skill_validator.valid_record",
            e1.is_empty(),
            format!("errors={:?}", e1),
            "errors=[]",
        );
        let e2 = audit_skill_record(&SkillRecord {
            skill_name: "bad name!".to_string(),
            description: "d".to_string(),
            instructions: "i".to_string(),
        });
        report(
            "skill_validator.invalid_name",
            e2 == vec!["Invalid characters in name: bad name!".to_string()],
            format!("errors={:?}", e2),
            "errors=[Invalid characters in name: bad name!]",
        );
        let e3 = audit_skill_record(&SkillRecord {
            skill_name: "".to_string(),
            description: "".to_string(),
            instructions: "".to_string(),
        });
        report(
            "skill_validator.missing_fields",
            e3 == vec![
                "Missing skill_name".to_string(),
                "Missing description field".to_string(),
                "Missing instructions usage guidelines".to_string(),
            ],
            format!("errors={:?}", e3),
            "errors=[Missing skill_name, Missing description field, Missing instructions usage guidelines]",
        );
        // empty name does not also raise "Invalid characters" (python elif)
        let e4 = audit_skill_record(&SkillRecord {
            skill_name: "".to_string(),
            description: "d".to_string(),
            instructions: "i".to_string(),
        });
        report(
            "skill_validator.empty_name_no_invalid",
            e4 == vec!["Missing skill_name".to_string()],
            format!("errors={:?}", e4),
            "errors=[Missing skill_name]",
        );
        // unicode name rejected by [a-zA-Z0-9_]+
        let e5 = audit_skill_record(&SkillRecord {
            skill_name: "café".to_string(),
            description: "d".to_string(),
            instructions: "i".to_string(),
        });
        report(
            "skill_validator.unicode_rejected",
            e5 == vec!["Invalid characters in name: café".to_string()],
            format!("errors={:?}", e5),
            "errors=[Invalid characters in name: café]",
        );
        // status decision matrix
        let s1 = final_status(false, 0);
        let s2 = final_status(true, 0);
        let s3 = final_status(false, 1);
        let s4 = final_status(true, 2);
        report(
            "skill_validator.status_logic",
            s1 == "PASS" && s2 == "WARNING" && s3 == "FAIL" && s4 == "WARNING",
            format!("{} {} {} {}", s1, s2, s3, s4),
            "PASS WARNING FAIL WARNING",
        );
        // manifest audit: mixed -> 2 kept, 2 pruned, healed, WARNING
        let recs = vec![
            SkillRecord { skill_name: "good_1".to_string(), description: "d".to_string(), instructions: "i".to_string() },
            SkillRecord { skill_name: "bad name!".to_string(), description: "d".to_string(), instructions: "i".to_string() },
            SkillRecord { skill_name: "good_2".to_string(), description: "d".to_string(), instructions: "i".to_string() },
            SkillRecord { skill_name: "tool_x".to_string(), description: "d".to_string(), instructions: "".to_string() },
        ];
        let ar = audit_manifest(&recs);
        report(
            "skill_validator.audit_mixed",
            ar.status == "WARNING" && ar.valid_count == 2 && ar.pruned_count == 2 && ar.healed && ar.errors.len() == 2,
            format!("status={} valid={} pruned={} healed={} errors={}",
                ar.status, ar.valid_count, ar.pruned_count, ar.healed, ar.errors.len()),
            "status=WARNING valid=2 pruned=2 healed=true errors=2",
        );
        report(
            "skill_validator.drift_msg1",
            ar.errors[0] == "Skill 'bad name!' contract drift: Invalid characters in name: bad name!",
            format!("msg={}", ar.errors[0]),
            "Skill 'bad name!' contract drift: Invalid characters in name: bad name!",
        );
        report(
            "skill_validator.drift_msg2",
            ar.errors[1] == "Skill 'tool_x' contract drift: Missing instructions usage guidelines",
            format!("msg={}", ar.errors[1]),
            "Skill 'tool_x' contract drift: Missing instructions usage guidelines",
        );
        // all-valid manifest -> PASS, not healed
        let all_ok = vec![SkillRecord { skill_name: "a".to_string(), description: "d".to_string(), instructions: "i".to_string() }];
        let ar2 = audit_manifest(&all_ok);
        report(
            "skill_validator.audit_clean",
            ar2.status == "PASS" && !ar2.healed && ar2.valid_count == 1,
            format!("status={} healed={} valid={}", ar2.status, ar2.healed, ar2.valid_count),
            "status=PASS healed=false valid=1",
        );
        // validator object audit log cap (python 50)
        let mut v = skill_validator::SkillValidator::new("dir");
        for _ in 0..55 {
            v.verify_manifest(&all_ok);
        }
        report(
            "skill_validator.audit_log_cap",
            v.audit_log_len() == 50,
            format!("len={}", v.audit_log_len()),
            "len=50",
        );
    }

    // ============ 5. small_diffusion_engine ============
    {
        report(
            "small_diffusion_engine.guidance",
            near(guidance_for_dh(1.2), 0.1, 1e-12) && near(guidance_for_dh(1.9), -0.2, 1e-12),
            format!("dh1.2={} dh1.9={}", guidance_for_dh(1.2), guidance_for_dh(1.9)),
            "dh1.2=0.1 dh1.9=-0.2",
        );
        report(
            "small_diffusion_engine.cf_scale",
            near(cf_scale_for_dh(1.2), 1.5, 1e-12) && near(cf_scale_for_dh(1.6), 0.6, 1e-12)
                && near(cf_scale_for_dh(1.9), 0.5, 1e-12),
            format!("1.2={} 1.6={} 1.9={}", cf_scale_for_dh(1.2), cf_scale_for_dh(1.6), cf_scale_for_dh(1.9)),
            "1.2=1.5 1.6=0.6 1.9=0.5",
        );
        report(
            "small_diffusion_engine.schedule",
            near(schedule_sigma(0), 0.8, 1e-12) && near(schedule_sigma(3), 0.05, 1e-12)
                && near(schedule_sigma(5), 0.05, 1e-12),
            format!("s0={} s3={} s5={}", schedule_sigma(0), schedule_sigma(3), schedule_sigma(5)),
            "s0=0.8 s3=0.05 s5=0.05",
        );
        let (p, f, a) = quantize_counts(&[0.5, 0.1, -0.3, 0.25]);
        report(
            "small_diffusion_engine.quantize",
            p == 2 && f == 1 && a == 1,
            format!("pass={} fault={} ambig={}", p, f, a),
            "pass=2 fault=1 ambig=1",
        );
        let c = confidence(&[0.5, 0.1, -0.3, 0.25]);
        report(
            "small_diffusion_engine.confidence",
            near(c, 0.2875, 1e-12),
            format!("conf={}", c),
            "conf=0.2875",
        );
        let (p1, f1, a1) = hip_fast_diffusion_kernel(&[0.5, 0.2, -0.4, -0.2, 0.0], 1.2);
        report(
            "small_diffusion_engine.hip_dh1_2",
            p1 == 1 && f1 == 0 && a1 == 4,
            format!("pass={} fault={} ambig={}", p1, f1, a1),
            "pass=1 fault=0 ambig=4",
        );
        let (p2, f2, a2) = hip_fast_diffusion_kernel(&[0.5, 0.2, -0.4, -0.2, 0.0], 1.9);
        report(
            "small_diffusion_engine.hip_dh1_9",
            p2 == 0 && f2 == 2 && a2 == 3,
            format!("pass={} fault={} ambig={}", p2, f2, a2),
            "pass=0 fault=2 ambig=3",
        );
        // denoise state machine: zero latent never exceeds early-exit gate
        let mut eng = SmallDiffusionEngine::new();
        let out = eng.denoise(&[0.0; 6], 1.2, None);
        report(
            "small_diffusion_engine.denoise_zeros",
            out.steps_used == 4 && !out.early_exit && out.num_ambiguous == 6 && out.num_pass == 0,
            format!("steps={} early={} ambig={}", out.steps_used, out.early_exit, out.num_ambiguous),
            "steps=4 early=false ambig=6",
        );
        // strong latent converges at step 1 (mean |z| 0.5 > 0.15)
        let out2 = eng.denoise(&[0.5; 6], 1.2, None);
        report(
            "small_diffusion_engine.denoise_early_exit",
            out2.steps_used == 1 && out2.early_exit && out2.num_pass == 6,
            format!("steps={} early={} pass={}", out2.steps_used, out2.early_exit, out2.num_pass),
            "steps=1 early=true pass=6",
        );
        // stats: avg latency round-1 + early exit rate one-decimal %
        eng.record_denoise(250.0);
        let (avg_us, rate, calls, _) = eng.get_stats();
        report(
            "small_diffusion_engine.stats",
            near(avg_us, 125.0, 1e-9) && rate == "50.0%" && calls == 2,
            format!("avg={} rate={} calls={}", avg_us, rate, calls),
            "avg=125 rate=50.0% calls=2",
        );
    }

    // ============ 6. sub_atomic_kernel ============
    {
        let si = SiliconMatrix::new();
        report(
            "sub_atomic_kernel.silicon_defaults",
            near(si.v_core, 1.25, 1e-12) && near(si.phi_v, 0.6089, 1e-12)
                && near(si.delta_v, 0.0006089, 1e-12) && near(si.t_cycle, 0.25, 1e-12)
                && near(si.clock_cycles_per_step, 2.4356, 1e-9),
            format!("phi_v={} delta_v={} t_cycle={} ccps={}", si.phi_v, si.delta_v, si.t_cycle, si.clock_cycles_per_step),
            "phi_v=0.6089 delta_v=0.0006089 t_cycle=0.25 ccps=2.4356",
        );
        let mut si2 = SiliconMatrix::new();
        let phi = si2.recalibrate(Some(1.4), Some(4.6));
        report(
            "sub_atomic_kernel.recalibrate",
            near(phi, 1.44256, 1e-9) && near(si2.delta_v, 0.00144256, 1e-9)
                && near(si2.t_cycle, 1.0 / 4.6, 1e-9),
            format!("phi_v={} delta_v={} t_cycle={}", si2.phi_v, si2.delta_v, si2.t_cycle),
            "phi_v=1.44256 delta_v=0.00144256 t_cycle=0.217391",
        );
        // K3 collapse on mixed wavefunction
        let mut gate = K3TernaryGate::new(0.15);
        let wave = [2.0, -1.0, 0.0, 0.5, -0.2, 3.0];
        let (trits, counts) = gate.collapse(&wave);
        let dominant = gate.dominant(&counts);
        report(
            "sub_atomic_kernel.collapse_mixed",
            trits == vec![1, -1, 0, 1, -1, 1] && counts.proceed == 3 && counts.analyze == 1
                && counts.block == 2 && dominant == "PROCEED",
            format!("trits={:?} p={} a={} b={} dom={}", trits, counts.proceed, counts.analyze, counts.block, dominant),
            "trits=[1,-1,0,1,-1,1] p=3 a=1 b=2 dom=PROCEED",
        );
        // superposition below epsilon stays ANALYZE
        let (_, c2) = gate.collapse(&[0.05; 10]);
        report(
            "sub_atomic_kernel.collapse_superposition",
            c2.analyze == 10 && c2.proceed == 0 && c2.block == 0,
            format!("p={} a={} b={}", c2.proceed, c2.analyze, c2.block),
            "p=0 a=10 b=0",
        );
        // running collapse counts accumulate across calls
        report(
            "sub_atomic_kernel.cumulative_counts",
            gate.collapse_count.proceed == 3 && gate.collapse_count.analyze == 11
                && gate.collapse_count.block == 2,
            format!("p={} a={} b={}", gate.collapse_count.proceed, gate.collapse_count.analyze, gate.collapse_count.block),
            "p=3 a=11 b=2",
        );
        // energy discharge: +1 -> signal, 0 -> 0.5*signal, -1 -> 0
        let ed = energy_discharge(&[1, 0, -1], 1.0);
        report(
            "sub_atomic_kernel.energy_discharge",
            near(ed[0], 1.0, 1e-12) && near(ed[1], 0.5, 1e-12) && near(ed[2], 0.0, 1e-12),
            format!("{:?}", ed),
            "[1.0, 0.5, 0.0]",
        );
        report(
            "sub_atomic_kernel.ternary_map",
            ternary_map_name(-1) == "BLOCK" && ternary_map_name(0) == "ANALYZE"
                && ternary_map_name(1) == "PROCEED",
            format!("{} {} {}", ternary_map_name(-1), ternary_map_name(0), ternary_map_name(1)),
            "BLOCK ANALYZE PROCEED",
        );
    }

    // ============ 7. symbology_etymology_monad ============
    {
        report(
            "symbology_etymology_monad.catalog_size",
            CATALOG_SIZE == 6 && DICTIONARY_SIZE == 5,
            format!("geom={} dict={}", CATALOG_SIZE, DICTIONARY_SIZE),
            "geom=6 dict=5",
        );
        report(
            "symbology_etymology_monad.geom_keys",
            GEOMETRIC_RESONANCE_CATALOG.iter().all(|g| !g.key.is_empty()) && g_freq("MONAD") == 432.0
                && g_freq("MERKABAH") == 963.0,
            format!("monad_hz={} merkabah_hz={}", g_freq("MONAD"), g_freq("MERKABAH")),
            "monad_hz=432 merkabah_hz=963",
        );
        report(
            "symbology_etymology_monad.etymology_keys",
            ETYMOLOGY_TRUTH_DICTIONARY.iter().any(|e| e.key == "MONAD")
                && ETYMOLOGY_TRUTH_DICTIONARY.iter().any(|e| e.key == "SWARM"),
            format!("count={}", ETYMOLOGY_TRUTH_DICTIONARY.len()),
            "count=5",
        );
        // keyword cascade (python branch order)
        let cases = [
            ("monad", "MONAD"),
            ("all is one", "MONAD"),
            ("dual nature", "VESICA_PISCIS"),
            ("synthesis of truth", "VESICA_PISCIS"),
            ("energy flow", "TORUS"),
            ("telemetry vortex", "TORUS"),
            ("agent mesh", "FLOWER_OF_LIFE"),
            ("tree of life", "FLOWER_OF_LIFE"),
            ("light transit", "MERKABAH"),
            ("quantum decoherence", "TETRAHEDRON"),
        ];
        let mut ok = true;
        let mut dbg = String::new();
        for (w, expect) in cases.iter() {
            let g = resolve_geom_key(&w.to_uppercase());
            dbg = format!("{} -> {}", w, g);
            if g != *expect {
                ok = false;
                break;
            }
        }
        report(
            "symbology_etymology_monad.geom_cascade",
            ok,
            dbg,
            "all 10 keywords map per python cascade",
        );
        // full resolution: etymology fallback + geometry + femto energy
        let r = resolve_concept("truth");
        report(
            "symbology_etymology_monad.resolve_unknown_word",
            r.geom_key == "VESICA_PISCIS" && near(r.frequency_hz, 528.0, 1e-9)
                && r.etymology_root == "etymon(TRUTH)"
                && near(r.femto_scan_energy, 67.584, 1e-9),
            format!("geom={} hz={} root={} energy={}", r.geom_key, r.frequency_hz, r.etymology_root, r.femto_scan_energy),
            "geom=VESICA_PISCIS hz=528 root=etymon(TRUTH) energy=67.584",
        );
        let r2 = resolve_concept("monad");
        report(
            "symbology_etymology_monad.resolve_dict_word",
            r2.geom_key == "MONAD" && near(r2.frequency_hz, 432.0, 1e-9)
                && !r2.etymology_root.starts_with("etymon(")
                && near(r2.femto_scan_energy, 55.296, 1e-9),
            format!("geom={} hz={} root={} energy={}", r2.geom_key, r2.frequency_hz, r2.etymology_root, r2.femto_scan_energy),
            "geom=MONAD hz=432 dict-root energy=55.296",
        );
        report(
            "symbology_etymology_monad.declaration",
            r2.embassy_truth_declaration == EMBASSY_TRUTH_DECLARATION,
            r2.embassy_truth_declaration.to_string(),
            "ALL IS ONE (1 = ∞)",
        );
    }

    // ============ 8. telemetry_fabric ============
    {
        let mut fab = TelemetryFabric::with_max_buffer(3);
        let n1 = fab.subscribe("metrics");
        let n2 = fab.subscribe("metrics");
        fab.subscribe("security_events");
        report(
            "telemetry_fabric.subscribe",
            n1 == 1 && n2 == 2,
            format!("first={} second={}", n1, n2),
            "first=1 second=2",
        );
        let mk = |id: &str, chan: &str| {
            TelemetryFrame::new(id, "node_a", chan, vec![("health_score".to_string(), 0.8)], 1.0)
        };
        let d1 = fab.publish(mk("f1", "metrics"));
        let d2 = fab.publish(mk("f2", "metrics"));
        fab.publish(mk("f3", "security_events"));
        let d4 = fab.publish(mk("f4", "metrics"));
        let d0 = fab.publish(mk("f5", "ipc_stream"));
        let (total, buffered, channels) = fab.get_fabric_metrics();
        let oldest = fab.oldest_frame_id().unwrap_or_default();
        report(
            "telemetry_fabric.publish_metrics",
            d1 == 2 && d2 == 2 && d0 == 0 && total == 5,
            format!("d1={} d2={} unsub={} total={}", d1, d2, d0, total),
            "d1=2 d2=2 unsub=0 total=5",
        );
        report(
            "telemetry_fabric.buffer_cap",
            buffered == 3 && oldest == "f3",
            format!("buffered={} oldest={}", buffered, oldest),
            "buffered=3 oldest=f3",
        );
        report(
            "telemetry_fabric.active_channels",
            channels == 2 && d4 == 2,
            format!("channels={} d4={}", channels, d4),
            "channels=2 d4=2",
        );
    }

    // ============ 9. thz_bonsai_gateway ============
    {
        let v = encode_payload(b"hi");
        report(
            "thz_bonsai_gateway.encode",
            v.len() == 128 && near(v[0], 104.0 / 255.0, 1e-12) && near(v[1], 105.0 / 255.0, 1e-12)
                && v[2] == 0.0 && v[127] == 0.0,
            format!("len={} v0={:.6} v1={:.6} v2={}", v.len(), v[0], v[1], v[2]),
            "len=128 v0=0.407843 v1=0.411765 v2=0",
        );
        let tr = to_trits(&[0.05, -0.05, 0.0, 0.5]);
        report(
            "thz_bonsai_gateway.to_trits",
            tr == vec![1, -1, 0, 1],
            format!("{:?}", tr),
            "[1, -1, 0, 1]",
        );
        let (dom, conf) = route_trits(&[1, -1, 0, 1]);
        report(
            "thz_bonsai_gateway.route_proceed",
            dom == "PROCEED" && near(conf, 0.75, 1e-12),
            format!("dom={} conf={}", dom, conf),
            "dom=PROCEED conf=0.75",
        );
        let (dom2, conf2) = route_trits(&[1, -1, -1]);
        report(
            "thz_bonsai_gateway.route_block",
            dom2 == "BLOCK" && near(conf2, 1.0, 1e-12),
            format!("dom={} conf={}", dom2, conf2),
            "dom=BLOCK conf=1",
        );
        let (dom3, conf3) = route_trits(&[0, 0, 0]);
        report(
            "thz_bonsai_gateway.route_analyze",
            dom3 == "ANALYZE" && conf3 == 0.0,
            format!("dom={} conf={}", dom3, conf3),
            "dom=ANALYZE conf=0",
        );
        let (sum, _) = (1i64, 0.0);
        let _ = sum;
        report(
            "thz_bonsai_gateway.route_decision_sign",
            route_decision(3) == "PROCEED" && route_decision(-3) == "BLOCK" && route_decision(0) == "ANALYZE",
            format!("{} {} {}", route_decision(3), route_decision(-3), route_decision(0)),
            "PROCEED BLOCK ANALYZE",
        );
        let mut core = BonsaiTernaryCore::new(128, 32);
        let st = core.process(&[0.05, -0.05, 0.0]);
        core.record_latency_ns(1000);
        core.process(&[0.0, 0.0, 0.0]);
        let (calls, avg) = core.get_stats();
        report(
            "thz_bonsai_gateway.stats",
            st.trits == vec![1, -1, 0] && calls == 2 && near(avg, 0.5, 1e-9),
            format!("trits={:?} calls={} avg_us={}", st.trits, calls, avg),
            "trits=[1,-1,0] calls=2 avg_us=0.5",
        );
        let gw = ThzBonsaiMicrokernelGateway::new();
        let msg = gw.decode_message("PROCEED", 0.75, 12.3);
        report(
            "thz_bonsai_gateway.decode",
            msg == "[Bonsai 27B • THz Microkernel • 12.3μs] PROCEED — Ternary K3 routing complete. 75% confidence.",
            msg,
            "[Bonsai 27B • THz Microkernel • 12.3μs] PROCEED — Ternary K3 routing complete. 75% confidence.",
        );
    }

    // ============ 10. tool_forge ============
    {
        report(
            "tool_forge.role_capabilities",
            role_capabilities("Architect").len() == 3 && role_capabilities("DevOps Engineer").len() == 4
                && role_capabilities("unknown").is_empty(),
            format!("archi={} devops={} unknown={}",
                role_capabilities("Architect").len(), role_capabilities("DevOps Engineer").len(),
                role_capabilities("unknown").len()),
            "archi=3 devops=4 unknown=0",
        );
        let gaps = assess_agent_needs("Architect", &["DiagramSkill"]);
        report(
            "tool_forge.assess_gaps",
            gaps == vec!["SystemDesignSkill", "ConstraintSolver"],
            format!("{:?}", gaps),
            "[\"SystemDesignSkill\", \"ConstraintSolver\"]",
        );
        let gaps_none = assess_agent_needs("Architect", &["DiagramSkill", "SystemDesignSkill", "ConstraintSolver"]);
        report(
            "tool_forge.assess_no_gaps",
            gaps_none.is_empty(),
            format!("{:?}", gaps_none),
            "[]",
        );
        let all_gaps = vec![
            ("Architect".to_string(), vec!["SystemDesignSkill".to_string(), "ConstraintSolver".to_string()]),
            ("Lead Developer".to_string(), vec!["SystemDesignSkill".to_string(), "GitSkill".to_string()]),
        ];
        let agg = aggregate_gaps(&all_gaps);
        let ok_agg = agg.len() == 3 && agg[0].tool_name == "SystemDesignSkill" && agg[0].score == 2
            && agg[0].needed_by == vec!["Architect".to_string(), "Lead Developer".to_string()]
            && agg[1].tool_name == "ConstraintSolver" && agg[1].score == 1
            && agg[2].tool_name == "GitSkill" && agg[2].score == 1;
        report(
            "tool_forge.aggregate_gaps",
            ok_agg,
            format!("[{}({},{:?}), {}({}), {}({})]", agg[0].tool_name, agg[0].score, agg[0].needed_by,
                agg[1].tool_name, agg[1].score, agg[2].tool_name, agg[2].score),
            "[SystemDesignSkill(2,[Architect, Lead Developer]), ConstraintSolver(1), GitSkill(1)]",
        );
        // SystemMetricsSkill shared by 3 roles (DevOps, Swarm Manager, Data Analyst)
        let mut g3 = Vec::new();
        for role in ["DevOps Engineer", "Swarm Manager", "Data Analyst"] {
            g3.push((role.to_string(), vec!["SystemMetricsSkill".to_string()]));
        }
        let agg3 = aggregate_gaps(&g3);
        report(
            "tool_forge.shared_skill_score",
            agg3.len() == 1 && agg3[0].score == 3 && agg3[0].needed_by.len() == 3,
            format!("score={} roles={}", agg3[0].score, agg3[0].needed_by.len()),
            "score=3 roles=3",
        );
        let mut tf = ToolForge::new();
        let (c1, lead1) = tf.run_forge_pipeline("CoolTool", false);
        let (c2, lead2) = tf.run_forge_pipeline("CoolTool", true);
        report(
            "tool_forge.pipeline_gate",
            !c1 && !lead1 && c2 && lead2 && tf.build_log_len() == 2,
            format!("no_lead=({},{}) with_lead=({},{}) log={}", c1, lead1, c2, lead2, tf.build_log_len()),
            "no_lead=(false,false) with_lead=(true,true) log=2",
        );
    }

    // ============ 11. trm_7m_optimized ============
    {
        report(
            "trm_7m_optimized.constants",
            STATE_DIM == 256 && NUM_BLOCKS == 3 && INPUT_DIM == 128 && near(DT, 1e-12, 1e-30),
            format!("state={} blocks={} input={} dt={}", STATE_DIM, NUM_BLOCKS, INPUT_DIM, DT),
            "state=256 blocks=3 input=128 dt=1e-12",
        );
        let p = pad_input(&[1.0, 2.0]);
        let long = pad_input(&[0.5; 200]);
        report(
            "trm_7m_optimized.pad_input",
            p.len() == 128 && near(p[0], 1.0, 1e-12) && near(p[1], 2.0, 1e-12) && p[127] == 0.0
                && long.len() == 128 && near(long[127], 0.5, 1e-12),
            format!("len={} p0={} p1={} tail={} long_tail={}", p.len(), p[0], p[1], p[127], long[127]),
            "len=128 p0=1 p1=2 tail=0 long_tail=0.5",
        );
        report(
            "trm_7m_optimized.exp",
            near(exp_approx(0.0), 1.0, 1e-12) && near(exp_approx(1.0), 2.718281828459045, 1e-9)
                && near(exp_approx(-1.0), 0.36787944117144233, 1e-9),
            format!("e0={} e1={} em1={}", exp_approx(0.0), exp_approx(1.0), exp_approx(-1.0)),
            "e0=1 e1=2.718281828 e-1=0.367879441",
        );
        report(
            "trm_7m_optimized.silu",
            near(silu(0.0), 0.0, 1e-12) && near(silu(1.0), 0.7310585786300049, 1e-9),
            format!("s0={} s1={}", silu(0.0), silu(1.0)),
            "s0=0 s1=0.731058579",
        );
        let (lab, conf) = classify_logits3(&[1.0, 2.0, 0.5]);
        report(
            "trm_7m_optimized.classify",
            lab == 0 && near(conf, 0.6285317192117624, 1e-6),
            format!("label={} conf={:.9}", lab, conf),
            "label=0 conf=0.628531719",
        );
        let (lab0, conf0) = classify_logits3(&[0.0, 0.0, 0.0]);
        report(
            "trm_7m_optimized.classify_zero",
            lab0 == 1 && near(conf0, 1.0 / 3.0, 1e-9),
            format!("label={} conf={:.6}", lab0, conf0),
            "label=1 conf=0.333333",
        );
        let (labt, conft) = classify_logits3(&[2.0, 2.0, -5.0]);
        report(
            "trm_7m_optimized.classify_tie_first",
            labt == 1 && near(conft, 0.499886, 1e-5),
            format!("label={} conf={:.6}", labt, conft),
            "label=1 conf=0.499886 (np.argmax first-on-tie)",
        );
        report(
            "trm_7m_optimized.synthetic_label",
            synthetic_trit_label(&[0.3, 0.2, 0.1, 0.5]) == 1
                && synthetic_trit_label(&[-0.5, -0.1, -0.2, -0.4]) == -1
                && synthetic_trit_label(&[0.3, 0.2, 0.1, -0.2]) == 0,
            format!("{} {} {}",
                synthetic_trit_label(&[0.3, 0.2, 0.1, 0.5]),
                synthetic_trit_label(&[-0.5, -0.1, -0.2, -0.4]),
                synthetic_trit_label(&[0.3, 0.2, 0.1, -0.2])),
            "1 -1 0",
        );
        let mut m = OptimizedTRM7M::new();
        m.record_forward(1500.0);
        m.record_forward(2500.0);
        let (avg_ns, avg_us, calls) = m.get_stats();
        report(
            "trm_7m_optimized.stats",
            calls == 2 && near(avg_ns, 2000.0, 1e-9) && near(avg_us, 2.0, 1e-9),
            format!("calls={} avg_ns={} avg_us={}", calls, avg_ns, avg_us),
            "calls=2 avg_ns=2000 avg_us=2",
        );
        m.record_forward(3.5);
        let (avg_ns3, avg_us3, calls3) = m.get_stats();
        report(
            "trm_7m_optimized.stats_rounding",
            calls3 == 3 && near(avg_ns3, 667.8, 1e-9) && near(avg_us3, 0.668, 1e-9),
            format!("calls={} avg_ns={} avg_us={}", calls3, avg_ns3, avg_us3),
            "calls=3 avg_ns=667.8 avg_us=0.668",
        );
    }

    // ============ summary ============
    let _ = CollapseCounts::default();
    println!();
    println!("BATCH B: 11 modules exercised (deterministic cores verified; weight/IO bodies MANUAL)");
}

/// helper: geometry entry frequency by key.
fn g_freq(key: &str) -> f64 {
    GEOMETRIC_RESONANCE_CATALOG
        .iter()
        .find(|g| g.key == key)
        .map(|g| g.frequency_hz)
        .unwrap_or(-1.0)
}

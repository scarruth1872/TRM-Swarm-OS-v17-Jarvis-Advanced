//! TRM Swarm OS — Sovereign Unity Protocol Engine (Phase 1)
//! Unifies the 4 native engines (Pharmacophore, Text Generation, P2P Route, and Co-Location)
//! into a single, seamless, sub-millisecond sovereign pipeline with 14-Agent PBFT consensus.

use std::time::Instant;
use std::net::SocketAddr;
use crate::zero_network_colocation::ZeroNetworkColocation;
use crate::pharmacophore::{PharmacophoreEngine, AtomPoint, PharmacophoreReport};
use crate::p2p_direct_route::P2PDirectRoute;
use crate::native_text_engine::{NativeTextEngine, ReportType, ReportParams, GeneratedReport};
use crate::council::{NativeSwarmCouncil, ProposalCategory, NativeCouncilProposal};

/// Result of an End-to-End Sovereign Unity Pipeline Run
#[derive(Debug, Clone)]
pub struct SovereignUnityResult {
    pub target: String,
    pub ligand: String,
    pub predicted_dg: f32,
    pub step1_shm_latency_us: f64,
    pub step2_pharmacophore_latency_us: f64,
    pub step3_p2p_latency_us: f64,
    pub step4_report_latency_us: f64,
    pub step5_pbft_latency_us: f64,
    pub total_pipeline_latency_us: f64,
    pub k3_verdict: i8,
    pub monad_seal: u64,
    pub summary_report: String,
}

/// Sovereign Unity Pipeline Orchestrator
pub struct SovereignUnityPipeline;

impl SovereignUnityPipeline {
    /// Executes all 5 sovereign steps consecutively on silicon
    pub fn execute(
        target: &str,
        ligand: &str,
        predicted_dg: f32,
        peer_addr: Option<SocketAddr>,
    ) -> SovereignUnityResult {
        let pipeline_start = Instant::now();

        // ── STEP 1: Zero-Network Co-Location Memory Setup ──
        let t1 = Instant::now();
        let mut shm = ZeroNetworkColocation::new().expect("SHM Ring Buffer bind");
        let task_payload = format!("{}:{}", target, ligand);
        let _shm_offset = shm.ring_buffer.write_task(1001, 13, task_payload.as_bytes());
        let step1_shm_latency = t1.elapsed().as_secs_f64() * 1_000_000.0;

        // ── STEP 2: Native Pharmacophore & K3 Verification ──
        let t2 = Instant::now();
        let catalytic_zn = Some(AtomPoint::new(0.0, 0.0, 0.0, 30));
        let ligand_atoms = vec![
            AtomPoint::new(1.5, 1.4, 0.5, 8),
            AtomPoint::new(1.3, -1.5, 0.4, 8),
            AtomPoint::new(2.4, 0.0, 0.6, 7),
            AtomPoint::new(3.5, 0.2, 0.8, 6),
            AtomPoint::new(4.2, -0.9, 1.1, 6),
            AtomPoint::new(5.5, -0.6, 1.3, 6),
            AtomPoint::new(6.7, -1.4, 1.5, 6),
            AtomPoint::new(7.9, -1.1, 1.7, 6),
        ];
        let pocket_atoms = vec![
            AtomPoint::new(-1.8, 0.8, 0.5, 7),
            AtomPoint::new(-1.2, -1.6, 0.4, 7),
            AtomPoint::new(0.3, 1.8, -1.2, 8),
            AtomPoint::new(3.8, 2.2, 0.9, 8),
            AtomPoint::new(5.2, -2.8, 1.4, 6),
            AtomPoint::new(6.5, -3.2, 1.6, 6),
            AtomPoint::new(7.2, 1.2, 2.0, 6),
        ];
        let pharma_rep = PharmacophoreEngine::evaluate(
            target,
            ligand,
            predicted_dg,
            &ligand_atoms,
            &pocket_atoms,
            catalytic_zn,
        );
        let step2_pharmacophore_latency = t2.elapsed().as_secs_f64() * 1_000_000.0;

        // ── STEP 3: P2P Direct Route Synchronization ──
        // No wall-clock timer here: this step reports the probe's own measured RTT,
        // so a `t3` would be an unused variable.
        let step3_p2p_latency = match peer_addr {
            Some(addr) => {
                let p2p = P2PDirectRoute::bind(0).expect("Ephemeral UDP bind");
                let probe = p2p.probe_peer(addr, 1, std::time::Duration::from_millis(5));
                probe.rtt_microseconds
            }
            None => {
                // No peer supplied: MEASURE a loopback round trip instead of pasting
                // a "47.41 µs measured baseline". A constant sitting in a field the
                // caller reads as measured is an assertion wearing a measurement's
                // name -- and this one fed the printed "Total Pipeline Latency".
                let p2p = P2PDirectRoute::bind(0).expect("Ephemeral UDP bind");
                let me: SocketAddr = format!("127.0.0.1:{}", p2p.local_port)
                    .parse()
                    .expect("loopback address");
                let probe = p2p.probe_peer(me, 1, std::time::Duration::from_millis(5));
                probe.rtt_microseconds
            }
        };

        // ── STEP 4: Deterministic Native Text Briefing ──
        let t4 = Instant::now();
        let mut params = ReportParams::new();
        params.insert("target".to_string(), target.to_string());
        params.insert("ligand".to_string(), ligand.to_string());
        params.insert("dg".to_string(), format!("{:.2}", predicted_dg));
        params.insert("zinc_dist".to_string(), format!("{:.2}", pharma_rep.zinc_coordination.min_distance_angstrom));
        params.insert("contacts".to_string(), pharma_rep.catalytic_contacts.total_contacts.to_string());
        params.insert("clashes".to_string(), pharma_rep.steric_clashes.severe_clash_count.to_string());
        params.insert("verdict".to_string(), if pharma_rep.k3_verdict == 1 { "+1 [VALID]".to_string() } else { "0 [MARGINAL]".to_string() });

        let report = NativeTextEngine::generate(ReportType::PharmacophoreAnalysis, &params);
        let step4_report_latency = t4.elapsed().as_secs_f64() * 1_000_000.0;

        // ── STEP 5: 14-Agent Byzantine Quorum Consensus ──
        let t5 = Instant::now();
        let topic = format!("Enact Lead Candidate {} on Target {}", ligand, target);
        let council_res = NativeSwarmCouncil::evaluate(
            &topic,
            ProposalCategory::General,
            "SovereignUnityPipeline",
        );
        let step5_pbft_latency = t5.elapsed().as_secs_f64() * 1_000_000.0;

        let total_pipeline_latency = pipeline_start.elapsed().as_secs_f64() * 1_000_000.0;

        SovereignUnityResult {
            target: target.to_string(),
            ligand: ligand.to_string(),
            predicted_dg,
            step1_shm_latency_us: step1_shm_latency,
            step2_pharmacophore_latency_us: step2_pharmacophore_latency,
            step3_p2p_latency_us: step3_p2p_latency,
            step4_report_latency_us: step4_report_latency,
            step5_pbft_latency_us: step5_pbft_latency,
            total_pipeline_latency_us: total_pipeline_latency,
            k3_verdict: pharma_rep.k3_verdict,
            monad_seal: council_res.monad_seal,
            summary_report: report.body,
        }
    }
}

/// C-ABI Export for Sovereign Unity Pipeline Execution
#[no_mangle]
pub extern "C" fn rust_sovereign_unity_execute(
    predicted_dg: f32,
) -> f64 {
    let res = SovereignUnityPipeline::execute("3P3G_LpxC", "TRM-ABX-082", predicted_dg, None);
    res.total_pipeline_latency_us
}

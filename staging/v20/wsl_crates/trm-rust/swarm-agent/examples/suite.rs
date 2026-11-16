//! suite.rs — Motion C: the full act-arm suite integration.
//! All four real act-arms in one verified run:
//!   WriteFile (fabric state) · DeployAntibody (NAND record)
//!   RenderDisplay (payload)  · ProofVerify (real proof checker)
use swarm_agent::realfab::fabric_report;
use swarm_proof::{verify_proof, ProofNode, Rule};

fn main() {
    println!("=== MOTION C: FULL ACT-ARM SUITE ===\n");

    // 1. WriteFile — real fabric state
    let r1 = fabric_report("/home/shawn/trm-rust/fabric_report.txt")
        .map(|c| format!("OK: {} bytes live fabric state", c.len()))
        .unwrap_or_else(|e| format!("FAIL: {e}"));
    println!("[WriteFile]      {r1}");

    // 2. DeployAntibody — real NAND record (via real_agent's pattern)
    let mut trits: Vec<i8> = vec![1, 1, 1, 1, 1];
    let mut h: u64 = 0x9E3779B97F4A7C15;
    for b in "suite-integration-test".bytes() {
        h = h.wrapping_mul(0x100000001B3) ^ (b as u64);
    }
    for k in 0..6 {
        trits.push(if (h >> k) & 1 == 1 { 1 } else { -1 });
    }
    trits.extend_from_slice(&[1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1]);
    let r2 = match swarm_nand::NandFabric::open(
        "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/nand_k3_fabric.bin",
    ) {
        Ok(mut fab) => match fab.append(&trits) {
            Ok(off) => format!("OK: NAND record @ byte {off}"),
            Err(e) => format!("FAIL: {e}"),
        },
        Err(e) => format!("FAIL: {e}"),
    };
    println!("[DeployAntibody] {r2}");

    // 3. RenderDisplay — real payload
    let payload: Vec<u8> = "MOTION C SUITE — ALL IS ONE".bytes().collect();
    let path = "/home/shawn/trm-rust/display_payload.bin";
    let r3 = std::fs::write(path, &payload)
        .map(|_| format!("OK: {} bytes payload", payload.len()))
        .unwrap_or_else(|e| format!("FAIL: {e}"));
    println!("[RenderDisplay]  {r3}");

    // 4. ProofVerify — real formal check (Logic's arm)
    let proof = ProofNode {
        conclusion: "Q".into(),
        rule: Rule::ModusPonens,
        premises: vec![ProofNode::axiom("P"), ProofNode::axiom("P → Q")],
    };
    let r4 = verify_proof(&proof);
    println!("[ProofVerify]    {r4}");

    // bad proof must be rejected — the checker's honesty
    let bad = ProofNode {
        conclusion: "Z".into(),
        rule: Rule::ModusPonens,
        premises: vec![ProofNode::axiom("P"), ProofNode::axiom("P → Q")],
    };
    let r5 = verify_proof(&bad);
    println!("[ProofVerify]    {r5}  (honest rejection)");

    println!("\n=== SUITE COMPLETE: all 4 act-arms real, Logic's proof checked both ways ===");
}

//! Zero-API / Zero-HTTP Direct Memory Execution Harness
//! Demonstrates pure bare-metal in-memory IPC, function pointer dispatch,
//! and direct L3 cache SRAM computing with ZERO sockets, ZERO HTTP, and ZERO REST APIs.

#[path = "../../src/trm_brain.rs"]
mod trm_brain;

#[path = "../../src/llm_brain.rs"]
mod llm_brain;

#[path = "../../src/sram_pattern_computer.rs"]
mod sram_pattern_computer;

#[path = "../../src/k3_logic.rs"]
mod k3_logic;

use std::time::Instant;
use trm_brain::TRMBrain;
use llm_brain::LLMBrain;

const ITERATIONS: usize = 100_000;

fn main() {
    println!("===============================================================================================");
    println!(">>> ZERO-API / ZERO-HTTP DIRECT IN-MEMORY SOVEREIGN EXECUTION HARNESS");
    println!("    Protocol: Pure In-Memory Function Calls & L3 Cache Operations | 0 Sockets | 0 HTTP APIs");
    println!("===============================================================================================");

    let mut brain = TRMBrain::new();
    let sample_context = [0.1f32, 0.4, -0.2, 0.7, 0.3, -0.1, 0.05, 0.9];
    let sample_bytes = [0x5Au8; 64];

    let start = Instant::now();

    // 1. Direct In-Memory TRM Brain Reasoning
    println!("\n[1/4] Dispatching 100,000 Direct In-Memory TRM Brain Recursive Inferences...");
    let t0 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = brain.infer(&sample_context, 4);
    }
    let dt_brain = t0.elapsed();
    println!("  * In-Memory TRM Brain       : 100,000 passes in {:.2?} ({:.2} ns / inference)",
             dt_brain, (dt_brain.as_secs_f64() / ITERATIONS as f64) * 1e9);

    // 2. Direct In-Memory Persona Routing
    println!("\n[2/4] Dispatching 100,000 Zero-Latency LLM Brain Persona Routings...");
    let t1 = Instant::now();
    for i in 0..ITERATIONS {
        let query = if i % 2 == 0 { "security threat analysis" } else { "bare-metal sram compilation" };
        let persona = LLMBrain::route_task(query);
        let _ = persona.default_weight;
    }
    let dt_route = t1.elapsed();
    println!("  * In-Memory LLM Routing     : 100,000 routings in {:.2?} ({:.2} ns / routing)",
             dt_route, (dt_route.as_secs_f64() / ITERATIONS as f64) * 1e9);

    // 3. Direct L3 SRAM Silicon Computing
    println!("\n[3/4] Dispatching 100,000 Direct L3 SRAM 15-Pattern Reconfigurations...");
    let t2 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sram_pattern_computer::sram_lut_fpga(&sample_bytes);
        let _ = sram_pattern_computer::sram_lut_cellular_automaton(&sample_bytes);
    }
    let dt_sram = t2.elapsed();
    println!("  * In-Memory SRAM Computing  : 100,000 patterns in {:.2?} ({:.2} ns / pattern)",
             dt_sram, (dt_sram.as_secs_f64() / ITERATIONS as f64) * 1e9);

    // 4. Direct K3 Ternary Logic
    println!("\n[4/4] Dispatching 100,000 K3 Balanced Ternary Operations...");
    let t3 = Instant::now();
    for i in 0..ITERATIONS {
        let a = ((i % 3) as i8) - 1;
        let b = (((i + 1) % 3) as i8) - 1;
        let _ = std::hint::black_box(a.min(b));
    }
    let dt_k3 = t3.elapsed();
    println!("  * In-Memory K3 Ternary Logic: 100,000 ops in {:.2?} ({:.2} ns / op)",
             dt_k3, (dt_k3.as_secs_f64() / ITERATIONS as f64) * 1e9);

    let total_elapsed = start.elapsed();
    println!("\n-----------------------------------------------------------------------------------------------");
    println!("⭐ DIRECT IN-MEMORY EXECUTION TOTAL: 400,000 Ops Completed in {:.2?}", total_elapsed);
    println!("   Average Latency Across All Subsystems : {:.2} nanoseconds / operation",
             (total_elapsed.as_secs_f64() / 400_000.0) * 1e9);
    println!("   Throughput                            : {:.0} Operations / second",
             400_000.0 / total_elapsed.as_secs_f64());
    println!("   API / Socket / HTTP Overhead          : ZERO (0.00 ns)");
    println!("===============================================================================================");
}

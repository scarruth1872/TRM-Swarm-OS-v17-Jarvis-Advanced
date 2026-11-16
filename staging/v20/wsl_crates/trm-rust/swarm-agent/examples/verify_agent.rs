//! verify_agent.rs — run the agent loop on the REAL GPU voltage field.
//! Uses the actual hidden_state.bin (from llama.cpp CUDA forward) as the
//! observation, collapses through the same K3 semantics, and executes.
use std::fs::File;
use std::io::Read;
use swarm_agent::{agent_loop, collapse_to_verdict, Verdict};

fn main() {
    // OBSERVE: read the real voltage field from the GPU forward
    let mut f = File::open(swarm_paths::resolve("/home/shawn/trm-rust/hidden_state.bin")).unwrap();
    let mut nbuf = [0u8; 4];
    f.read_exact(&mut nbuf).unwrap();
    let n = i32::from_le_bytes(nbuf) as usize;
    let mut field = vec![0f32; n];
    unsafe {
        let ptr = field.as_mut_ptr() as *mut u8;
        f.read_exact(std::slice::from_raw_parts_mut(ptr, n * 4)).unwrap();
    }
    println!("OBSERVE: loaded {n}-dim voltage field, |v|={:.4}",
             field.iter().map(|v| v*v).sum::<f32>().sqrt());

    // DECIDE on the real field
    let v = collapse_to_verdict(&field, 0.35);
    println!("DECIDE: K3 verdict = {v:?}");
    let acts = field.iter().filter(|v| v.abs() >= 0.35 && **v > 0.0).count();
    let blocks = field.iter().filter(|v| v.abs() >= 0.35 && **v < 0.0).count();
    let unks = field.iter().filter(|v| v.abs() < 0.35).count();
    println!("       TRUE={acts} FALSE={blocks} UNKNOWN={unks}");

    // ACT: run the loop on tasks with the real field
    for task in [
        "write the voltage node report file",
        "deploy antibody against the exploit",
        "render the jarvis display",
    ] {
        let result = agent_loop(task, &field, 0.35);
        println!("ACT: [{task}] → {result}");
    }
    let _ = Verdict::Act;
}
